//! Record one check invocation so its result survives a lost tool session.

mod inputs;
pub mod resources;
mod source;
pub mod summary;

use crate::process::{Completion, OwnedChild};
use anyhow::{Context, Result};
use clap::Args;
use serde::Serialize;
use std::{
    fs::{self, File},
    io::Write,
    os::unix::process::ExitStatusExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Args)]
pub struct CheckArgs {
    /// Fresh directory for check.log and run.json; its parent must exist.
    #[arg(long)]
    output_dir: PathBuf,
    /// Capture host pressure/memory and the child's visible cgroup once per second.
    #[arg(long)]
    resources: bool,
    /// Snapshot a regular file (at most 16 MiB) as child stdin; otherwise stdin is null.
    /// Piped input and shell heredocs are never inherited.
    #[arg(long)]
    stdin_file: Option<PathBuf>,
    /// Require a fresh, nonempty regular output file after success (repeatable).
    /// Relative paths use the command's cwd. This does not validate file contents.
    #[arg(long = "expect-file")]
    expected_files: Vec<PathBuf>,
    /// Noninteractive command in the current directory/environment; stdin defaults to null.
    #[arg(last = true, required = true, num_args = 1.., value_name = "COMMAND")]
    command: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    Running,
    Finished,
    Interrupted,
    Incomplete,
    Invalidated,
}

#[derive(Serialize)]
struct Record {
    command: Vec<String>,
    cwd: PathBuf,
    started_unix_ms: u128,
    finished_unix_ms: Option<u128>,
    status: Status,
    pid: Option<u32>,
    exit_code: Option<i32>,
    signal: Option<i32>,
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resources: Option<resources::Report>,
    source: source::Report,
    stdin: inputs::Input,
    expected_files: Vec<inputs::ExpectedFile>,
}

fn now() -> Result<u128> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())
}

fn save(directory: &Path, record: &Record) -> Result<()> {
    let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
    serde_json::to_writer_pretty(&mut temporary, record)?;
    writeln!(temporary)?;
    temporary.as_file().sync_all()?;
    temporary.persist(directory.join("run.json"))?;
    Ok(())
}

pub fn run(args: CheckArgs) -> Result<u8> {
    let mut record = Record {
        command: args.command,
        cwd: std::env::current_dir()?,
        started_unix_ms: now()?,
        finished_unix_ms: None,
        status: Status::Running,
        pid: None,
        exit_code: None,
        signal: None,
        error: None,
        resources: None,
        source: source::Report::default(),
        stdin: inputs::Input::default(),
        expected_files: args
            .expected_files
            .iter()
            .map(|path| inputs::ExpectedFile::new(path))
            .collect::<Result<_>>()?,
    };
    fs::create_dir(&args.output_dir).with_context(|| {
        format!(
            "Create fresh check directory {} (parent must exist)",
            args.output_dir.display()
        )
    })?;
    let directory = fs::canonicalize(&args.output_dir)?;
    let mut resources = args
        .resources
        .then(|| resources::Capture::start(&directory));
    record.resources = resources.as_ref().map(|capture| capture.report.clone());
    save(&directory, &record)?;
    // Opening this once and cloning the handle preserves both streams in one file.
    let result: Result<u8> = (|| {
        record.source = source::Report::start(&record.cwd, &directory)?;
        for file in &record.expected_files {
            file.require_absent()?;
        }
        let (input, stdin) = inputs::prepare(args.stdin_file.as_deref(), &directory)?;
        record.stdin = input;
        save(&directory, &record)?;
        let code = capture(&directory, &mut record, &mut resources, stdin)?;
        record.source.finish(&directory)?;
        if record.source.changed() {
            record.status = Status::Invalidated;
            record.error =
                Some("Source changed during the check; rerun on a stable candidate".into());
            return Ok(if code == 0 { 1 } else { code });
        }
        if code == 0 {
            for file in &mut record.expected_files {
                file.verify()?;
            }
        }
        Ok(code)
    })();
    if let Some(mut capture) = resources {
        capture.finish();
        record.resources = Some(capture.report);
    }
    record.finished_unix_ms = Some(now()?);
    if let Err(error) = &result {
        record.status = Status::Incomplete;
        record.error = Some(format!("{error:#}"));
    }
    save(&directory, &record).context("Persist check completion")?;
    let code = result.unwrap_or_else(|_| {
        if crate::process::interrupted() {
            130
        } else {
            1
        }
    });
    let mut heading = format!(
        "Check output: {}\nCheck result: {}\n",
        directory.join("check.log").display(),
        directory.join("run.json").display()
    );
    if let Some(error) = &record.error {
        heading.push_str(&format!("Check diagnostic: {error}\n"));
    }
    if let Some(error) = record
        .resources
        .as_ref()
        .and_then(|report| report.error.as_ref())
    {
        heading.push_str(&format!("Resource capture incomplete: {error}\n"));
    }
    // Paths and recorder errors share the same output budget as log excerpts.
    if heading.len() > 5000 {
        heading.truncate(heading.floor_char_boundary(5000));
        heading.push_str(" … [metadata truncated; inspect output directory]\n");
    }
    let text = if code == 0 {
        heading
    } else {
        heading.push_str("Diagnostic excerpts only; consult run.json for status.\n");
        fs::File::open(directory.join("check.log"))
            .map_err(anyhow::Error::from)
            .and_then(|file| summary::summarize(std::io::BufReader::new(file), &heading, 6000))
            .unwrap_or_else(|_| format!("{heading}Diagnostic excerpts unavailable; inspect the retained record and log.\n"))
    };
    // A diagnostic-output failure must not replace the recorded command result.
    let _ = std::io::stderr().write_all(text.as_bytes());
    Ok(code)
}

fn capture(
    directory: &Path,
    record: &mut Record,
    resources: &mut Option<resources::Capture>,
    stdin: Stdio,
) -> Result<u8> {
    let log = File::create_new(directory.join("check.log"))?;
    let mut command = Command::new(&record.command[0]);
    command
        .args(&record.command[1..])
        .stdin(stdin)
        .stdout(log.try_clone()?)
        .stderr(log.try_clone()?);
    let mut child = OwnedChild::spawn(&mut command)
        .with_context(|| format!("Start check executable {:?}", record.command[0]))?;
    record.pid = Some(child.id());
    save(directory, record)?;
    let completion = loop {
        if let Some(resources) = resources {
            resources.sample(Some(child.id()));
        }
        match child.wait(Duration::from_secs(1))? {
            Completion::TimedOut => continue,
            completion => break completion,
        }
    };
    child.cleanup()?;
    log.sync_all().context("Flush complete check output")?;
    match completion {
        Completion::Exited(status) => {
            record.exit_code = status.code();
            record.signal = status.signal();
            if let Some(code) = record.exit_code {
                record.status = Status::Finished;
                u8::try_from(code).context("Check exit code is outside 0..=255")
            } else {
                record.status = Status::Interrupted;
                Ok((128 + record.signal.unwrap_or(1)).min(255) as u8)
            }
        }
        Completion::Interrupted => {
            record.status = Status::Interrupted;
            Ok(130)
        }
        Completion::TimedOut => unreachable!("the wait loop continues while the check runs"),
    }
}
