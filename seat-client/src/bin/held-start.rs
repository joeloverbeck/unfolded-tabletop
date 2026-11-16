//! Record one held start of a game's agent seat as evidence for a pull request.
//! `HELP`, which `held-start --help` prints, describes the command, the files it
//! writes, the record and its refusals.

use anyhow::{Context, Result, bail};
use std::{
    ffi::OsString,
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
    time::{Duration, Instant},
};
use unfolded_seat_client as seat_client;
use unfolded_seat_protocol::{Request, Status};

/// How long the seat may take to remove its socket after `quit` or a refusal.
const SOCKET_REMOVAL: Duration = Duration::from_secs(10);
/// How long one response may take once the seat has been admitted.
const RESPONSE: Duration = Duration::from_secs(180);
const USAGE: &str = "usage: held-start [--clean] [--request <json>]... <out-dir> <case> -- <command> [<argument>...]";
/// What `--help` prints after the usage: everything a caller needs to read a record.
const HELP: &str = "\
Record one held start of a game's agent seat as evidence for a pull request. Run it
from the game's checkout, so that the record names that checkout's revision.

It appends `--socket <out-dir>/<case>.sock` to the command, with <out-dir> made
absolute, and runs the command with no standard input. A socket path holds at most
107 bytes, so keep <out-dir> short. It writes, beside each other in <out-dir>:

  <case>.stdout, <case>.stderr  the launching command's standard output and error
  <case>.seat.log               the held seat's log
  <case>.record                 the record, which is also printed

The record names the command, the working directory, that directory's Git revision
and whether its tracked files are unchanged; the launching command's exit status, its
standard-output line count, and the first line's status and any error; when that
status is `ready`, what `observe` and `quit` answered; each --request as it was sent,
its status, and for a capture its file and whether it was written; whether the socket
was removed within ten seconds; and the log's last two lines.

Each --request is one seat request, sent in the given order on the same connection
after `observe` and before `quit`, and only when the start is `ready`; otherwise the
record lists it as `not sent`. If the session ends early, a request with no answer is
`unanswered`, and an `exchange:` line says why. With no --request, `observe` and
`quit` share one line of the record. A `capture` names a plain file name, which is
written as <out-dir>/<case>.<name>.

Before anything is launched it refuses: a line that is no seat request, a `quit` (it
sends its own), a capture path that is not a plain file name, a name it writes itself
(record, stdout, stderr, seat.log, sock, sock.log), two captures to one file, a capture
whose file exists, and a case already recorded in <out-dir>, so a later run cannot
replace an earlier one. With --clean it also refuses a working directory that is not
a Git working tree, or whose tracked files differ from its revision.

Record a \"before\" case with --clean, from the game's `main`, before changing the
game, and compare the captures with `compare-captures`.

Exit status: 0 once the record is written, whatever the start did; 2 when no record
could be made.
";
/// The files `held-start` itself writes for a case, which no capture may name.
const OWN_FILES: [&str; 6] = ["record", "stdout", "stderr", "seat.log", "sock", "sock.log"];

/// One `--request` as it is sent, and the file a `capture` writes.
struct Extra {
    request: Request,
    capture: Option<PathBuf>,
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if let [only] = &arguments[..]
        && (only == "--help" || only == "-h")
    {
        print!("{USAGE}\n\n{HELP}");
        return ExitCode::SUCCESS;
    }
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

fn run(mut arguments: &[String]) -> Result<String> {
    let mut lines = Vec::new();
    let mut clean = false;
    loop {
        match arguments {
            [flag, rest @ ..] if flag == "--clean" => {
                clean = true;
                arguments = rest;
            }
            [flag, line, rest @ ..] if flag == "--request" => {
                lines.push(line);
                arguments = rest;
            }
            _ => break,
        }
    }
    let [out, case, separator, program, rest @ ..] = arguments else {
        bail!(USAGE);
    };
    if separator != "--" {
        bail!(USAGE);
    }
    if case.is_empty() || case.contains('/') {
        bail!("the case {case:?} must be a plain file name");
    }
    // A game may refuse a relative socket path, as Arkham does, so every path is absolute.
    let out = std::path::absolute(out)
        .with_context(|| format!("cannot find the absolute path of {out}"))?;
    fs::create_dir_all(&out).with_context(|| format!("cannot create {}", out.display()))?;
    let file = |suffix: &str| out.join(format!("{case}{suffix}"));
    let record_path = file(".record");
    if record_path.exists() {
        bail!(
            "{} already exists: record this run under another case or directory",
            record_path.display()
        );
    }
    let extras = extras(&lines, case, &out)?;
    let directory = std::env::current_dir().context("cannot read the working directory")?;
    let (revision, tree) = revision(&directory);
    if clean && tree != Tree::Clean {
        bail!("--clean: {revision}");
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
    line(&mut record, "working directory", &directory.display());
    line(&mut record, "revision", &revision);

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
    let ready = first_status == Status::Ready.name();
    record.push_str(&converse(&socket, &extras, ready));
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

/// Whether a working tree's tracked files match its revision.
#[derive(PartialEq)]
enum Tree {
    Clean,
    Changed,
    Absent,
}

/// The working directory's Git revision and whether its tree is clean, or why there
/// is none.
fn revision(directory: &Path) -> (String, Tree) {
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
            let (words, tree) = if changed == 0 {
                ("clean".to_owned(), Tree::Clean)
            } else {
                (format!("{changed} changed tracked files"), Tree::Changed)
            };
            (format!("{} ({words})", head.trim()), tree)
        }
        _ => ("none: not a Git working tree".to_owned(), Tree::Absent),
    }
}

/// Read each `--request` line before anything is launched: a seat request other than
/// `quit`, whose capture names a plain file name that is none of `held-start`'s own
/// files, no earlier capture's and not yet written. Its capture path becomes
/// `<out>/<case>.<name>`.
fn extras(lines: &[&String], case: &str, out: &Path) -> Result<Vec<Extra>> {
    let mut extras: Vec<Extra> = Vec::with_capacity(lines.len());
    for line in lines {
        let request: Request = serde_json::from_str(line)
            .with_context(|| format!("--request {line} is not a seat request"))?;
        let (request, capture) = match request {
            Request::Quit {} => bail!("--request {line}: held-start sends quit itself"),
            Request::Capture { path } => {
                let name = path.to_string_lossy();
                if path.file_name() != Some(path.as_os_str()) {
                    bail!(
                        "--request {line}: a capture's path must be a plain file name, \
                         written as <out-dir>/<case>.<name>"
                    );
                }
                if OWN_FILES.contains(&name.as_ref()) {
                    bail!("--request {line}: held-start writes {case}.{name} itself");
                }
                let mut file = OsString::from(format!("{case}."));
                file.push(&path);
                let target = out.join(file);
                if target.exists() {
                    bail!(
                        "{} already exists: capture under another name, case or directory",
                        target.display()
                    );
                }
                if extras
                    .iter()
                    .any(|extra| extra.capture.as_ref() == Some(&target))
                {
                    bail!("--request {line}: an earlier request captures to the same file");
                }
                (
                    Request::Capture {
                        path: target.clone(),
                    },
                    Some(target),
                )
            }
            request => (request, None),
        };
        extras.push(Extra { request, capture });
    }
    Ok(extras)
}

/// When the start was `ready`, send `observe`, each extra request and `quit` on one
/// connection, and say what each answered, or why none did. Otherwise say that the
/// extra requests were not sent. With no extra request, one line holds both statuses.
fn converse(socket: &Path, extras: &[Extra], ready: bool) -> String {
    let answered = ready.then(|| {
        let requests: Vec<String> = std::iter::once(Request::Observe {})
            .chain(extras.iter().map(|extra| extra.request.clone()))
            .chain([Request::Quit {}])
            .map(|request| request.to_string())
            .collect();
        exchange(socket, &requests)
    });
    let mut text = String::new();
    if extras.is_empty() {
        if let Some((statuses, error)) = answered {
            let _ = write!(text, "observe, quit: {}", statuses.join(", "));
            match error {
                Some(error) if statuses.is_empty() => text = format!("observe, quit: {error}"),
                Some(error) => {
                    let _ = write!(text, " (then {error})");
                }
                None => {}
            }
            text.push('\n');
        }
        return text;
    }
    let (statuses, error) = answered.unwrap_or_default();
    let status = |index: usize| {
        if !ready {
            "not sent"
        } else {
            statuses.get(index).map_or("unanswered", String::as_str)
        }
    };
    if ready {
        let _ = writeln!(text, "observe: {}", status(0));
    }
    for (index, extra) in extras.iter().enumerate() {
        let _ = writeln!(text, "request {}: {}", index + 1, extra.request);
        let _ = writeln!(text, "  status: {}", status(index + 1));
        if let Some(capture) = &extra.capture {
            let written = if capture.exists() {
                "written"
            } else {
                "absent"
            };
            let _ = writeln!(text, "  capture: {} ({written})", capture.display());
        }
    }
    if ready {
        let _ = writeln!(text, "quit: {}", status(extras.len() + 1));
    }
    if let Some(error) = error {
        let _ = writeln!(text, "exchange: {error}");
    }
    text
}

/// Send `requests` on one connection: the status each answer carried, in order, and
/// why the exchange ended early, if it did: an error, or a session that ended with
/// requests unanswered.
fn exchange(socket: &Path, requests: &[String]) -> (Vec<String>, Option<String>) {
    let connection = match seat_client::Connection::open(socket) {
        Ok(connection) => connection,
        Err(error) => return (Vec::new(), Some(format!("cannot connect: {error}"))),
    };
    let deadline = Instant::now() + RESPONSE;
    let mut answers = Vec::new();
    let exchanged = connection.send(requests, &mut answers, || {
        if Instant::now() >= deadline {
            bail!("no response within {} seconds", RESPONSE.as_secs());
        }
        Ok(())
    });
    let statuses = String::from_utf8_lossy(&answers)
        .lines()
        .map(|answer| {
            serde_json::from_str::<serde_json::Value>(answer)
                .ok()
                .and_then(|answer| answer["status"].as_str().map(str::to_owned))
                .unwrap_or_else(|| "malformed".to_owned())
        })
        .collect();
    let ended = match exchanged {
        Ok(seat_client::Outcome::Short { answered }) => Some(format!(
            "the seat answered {answered} of {} requests before its session ended",
            requests.len()
        )),
        Ok(_) => None,
        Err(error) => Some(format!("{error:#}")),
    };
    (statuses, ended)
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
