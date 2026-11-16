//! Record one held start of a game's agent seat as evidence:
//!
//! ```text
//! held-start <out-dir> <case> -- <command> [<argument>...]
//! ```
//!
//! It runs the command with `--socket <out-dir>/<case>.sock` appended, as an agent's
//! shell would, and keeps what a reviewer needs to check a claim about that start:
//! `<case>.stdout` and `<case>.stderr`, the seat's log as `<case>.seat.log`, and
//! `<case>.record`. The record names the command, the working directory and its Git
//! revision, the launching command's exit status, its stdout line count and first
//! status, and whether the socket was removed. When the start prints `ready`, the
//! record also holds what `observe` and `quit` answered. A case already recorded in
//! the directory is refused, so a later run cannot replace an earlier one.

use anyhow::{Context, Result, bail};
use std::{
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
    time::{Duration, Instant},
};
use unfolded_seat_client as seat_client;

/// How long the seat may take to remove its socket after `quit` or a refusal.
const SOCKET_REMOVAL: Duration = Duration::from_secs(10);
/// How long one response may take once the seat has been admitted.
const RESPONSE: Duration = Duration::from_secs(180);

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match run(&arguments) {
        Ok(record) => {
            print!("{record}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("held-start: {error:#}");
            ExitCode::from(2)
        }
    }
}

fn run(arguments: &[String]) -> Result<String> {
    let [out, case, separator, program, rest @ ..] = arguments else {
        bail!("usage: held-start <out-dir> <case> -- <command> [<argument>...]");
    };
    if separator != "--" {
        bail!("usage: held-start <out-dir> <case> -- <command> [<argument>...]");
    }
    if case.is_empty() || case.contains('/') {
        bail!("the case {case:?} must be a plain file name");
    }
    let out = Path::new(out);
    fs::create_dir_all(out).with_context(|| format!("cannot create {}", out.display()))?;
    let file = |suffix: &str| out.join(format!("{case}{suffix}"));
    let record_path = file(".record");
    if record_path.exists() {
        bail!(
            "{} already exists: record this run under another case or directory",
            record_path.display()
        );
    }
    let socket = file(".sock");
    let mut record = String::new();
    let line = |record: &mut String, key: &str, value: &dyn std::fmt::Display| {
        let _ = writeln!(record, "{key}: {value}");
    };
    line(
        &mut record,
        "command",
        &format!(
            "{} --socket {}",
            std::iter::once(program)
                .chain(rest)
                .map(|word| shown(word))
                .collect::<Vec<_>>()
                .join(" "),
            shown(&socket.display().to_string())
        ),
    );
    let directory = std::env::current_dir().context("cannot read the working directory")?;
    line(&mut record, "working directory", &directory.display());
    line(&mut record, "revision", &revision(&directory));

    let status = Command::new(program)
        .args(rest)
        .arg("--socket")
        .arg(&socket)
        .stdin(Stdio::null())
        .stdout(fs::File::create(file(".stdout"))?)
        .stderr(fs::File::create(file(".stderr"))?)
        .status()
        .with_context(|| format!("cannot start {program}"))?;
    line(
        &mut record,
        "launcher exit",
        &status
            .code()
            .map_or_else(|| status.to_string(), |code| code.to_string()),
    );
    let stdout = fs::read_to_string(file(".stdout"))?;
    line(
        &mut record,
        "launcher stdout lines",
        &stdout.lines().count(),
    );
    let first = stdout
        .lines()
        .next()
        .and_then(|first| serde_json::from_str::<serde_json::Value>(first).ok());
    let first_status = first
        .as_ref()
        .and_then(|first| first["status"].as_str())
        .unwrap_or("none");
    line(&mut record, "first status", &first_status);
    if let Some(error) = first.as_ref().and_then(|first| first["error"].as_str()) {
        line(&mut record, "first error", &error);
    }
    if first_status == "ready" {
        line(&mut record, "observe, quit", &converse(&socket));
    }
    let deadline = Instant::now() + SOCKET_REMOVAL;
    while socket.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    let removed = if socket.exists() {
        "present"
    } else {
        "removed"
    };
    line(&mut record, "socket after", &removed);
    record.push_str(&seat_log(&socket, &file(".seat.log")));
    fs::write(&record_path, &record)
        .with_context(|| format!("cannot write {}", record_path.display()))?;
    Ok(record)
}

/// The working directory's Git revision and whether its tree is clean, or why there
/// is none.
fn revision(directory: &Path) -> String {
    let git = |arguments: &[&str]| {
        Command::new("git")
            .arg("-C")
            .arg(directory)
            .args(arguments)
            .stderr(Stdio::null())
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
    };
    match (
        git(&["rev-parse", "HEAD"]),
        git(&["status", "--porcelain", "--untracked-files=no"]),
    ) {
        (Some(head), Some(changes)) => {
            let changed = changes.lines().count();
            let tree = if changed == 0 {
                "clean".to_owned()
            } else {
                format!("{changed} changed tracked files")
            };
            format!("{} ({tree})", head.trim())
        }
        _ => "none: not a Git working tree".to_owned(),
    }
}

/// Send `observe` and `quit` on one connection: the statuses that came back, or why
/// none did.
fn converse(socket: &Path) -> String {
    let requests = [
        r#"{"op":"observe"}"#.to_owned(),
        r#"{"op":"quit"}"#.to_owned(),
    ];
    let stream = match seat_client::connect(socket) {
        Ok(stream) => stream,
        Err(error) => return format!("cannot connect: {error}"),
    };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
    let deadline = Instant::now() + RESPONSE;
    let mut answers = Vec::new();
    let exchanged = seat_client::exchange(&stream, &requests, &mut answers, || {
        if Instant::now() >= deadline {
            bail!("no response within {} seconds", RESPONSE.as_secs());
        }
        Ok(())
    });
    let statuses: Vec<String> = String::from_utf8_lossy(&answers)
        .lines()
        .map(|answer| {
            serde_json::from_str::<serde_json::Value>(answer)
                .ok()
                .and_then(|answer| answer["status"].as_str().map(str::to_owned))
                .unwrap_or_else(|| "malformed".to_owned())
        })
        .collect();
    match exchanged {
        Ok(_) => statuses.join(", "),
        Err(error) => format!("{} (then {error:#})", statuses.join(", ")),
    }
}

/// Copy the seat's log, which the launcher writes at the socket's path with `.log`
/// added, and quote its last two lines.
fn seat_log(socket: &Path, copy: &PathBuf) -> String {
    let mut log = socket.as_os_str().to_owned();
    log.push(".log");
    match fs::read_to_string(&log) {
        Ok(text) => {
            let _ = fs::write(copy, &text);
            let last: Vec<&str> = text.lines().rev().take(2).collect();
            let mut quoted = "seat log, last lines:\n".to_owned();
            for line in last.into_iter().rev() {
                let _ = writeln!(quoted, "  {}", plain(line));
            }
            quoted
        }
        Err(_) => "seat log: none\n".to_owned(),
    }
}

/// `line` without the terminal's colour sequences, which Bevy's log writes.
fn plain(line: &str) -> String {
    let mut text = String::with_capacity(line.len());
    let mut characters = line.chars();
    while let Some(character) = characters.next() {
        if character == '\x1b' {
            if characters.next() == Some('[') {
                for code in characters.by_ref() {
                    if ('@'..='~').contains(&code) {
                        break;
                    }
                }
            }
        } else {
            text.push(character);
        }
    }
    text
}

/// A command word as a shell reads it back: quoted when it holds a space or a quote.
fn shown(word: &str) -> String {
    if word.is_empty()
        || word.contains(|character: char| character.is_whitespace() || "'\"\\".contains(character))
    {
        format!("{word:?}")
    } else {
        word.to_owned()
    }
}
