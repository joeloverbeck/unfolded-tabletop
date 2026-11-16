//! The `held-start` recorder against a stand-in game. Run as `game <case> --socket
//! <path>`, this executable is that game: a start that prints `ready` and leaves a
//! seat of its own serving the socket, or one that refuses. Run by `cargo test`, it
//! records those starts through the real `held-start` binary. A started game
//! prints on standard output, so this executable has no test harness of its own.

use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixListener,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
    time::{Duration, Instant},
};

const READY: &str = r#"{"ok":true,"mode":"seat","status":"ready"}"#;
const REFUSED: &str =
    r#"{"ok":false,"mode":"seat","status":"refused","error":"The stand-in table refuses."}"#;

fn log_of(socket: &Path) -> PathBuf {
    let mut log = socket.as_os_str().to_owned();
    log.push(".log");
    log.into()
}

/// The stand-in game's command.
fn game(arguments: &[String]) -> ExitCode {
    let [case, flag, socket] = arguments else {
        panic!("usage: game <case> --socket <path>");
    };
    assert_eq!(flag, "--socket");
    let socket = Path::new(socket);
    // As Arkham's seat does, the stand-in refuses a socket path that is not absolute.
    if !socket.is_absolute() {
        println!(
            r#"{{"ok":false,"mode":"seat","status":"refused","error":"--socket needs an absolute path"}}"#
        );
        return ExitCode::FAILURE;
    }
    match case.as_str() {
        "ready" => {
            let log = std::fs::File::create(log_of(socket)).unwrap();
            // The seat outlives this launching command, as a launched seat does.
            #[expect(clippy::zombie_processes, reason = "the seat outlives its launcher")]
            Command::new(std::env::current_exe().unwrap())
                .args(["serve", socket.to_str().unwrap()])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(log)
                .spawn()
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(10);
            while !socket.exists() {
                assert!(
                    Instant::now() < deadline,
                    "the stand-in seat never listened"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
            println!("{READY}");
            ExitCode::SUCCESS
        }
        "refused" => {
            std::fs::write(log_of(socket), "starting\nThe stand-in table refuses.\n").unwrap();
            println!("{REFUSED}");
            ExitCode::FAILURE
        }
        _ => panic!("no case {case}"),
    }
}

/// The stand-in seat: one connection, `observed` for `observe` and `bye` for `quit`.
fn serve(socket: &Path) -> ExitCode {
    let listener = UnixListener::bind(socket).unwrap();
    eprintln!("listening");
    let (mut stream, _) = listener.accept().unwrap();
    for line in BufReader::new(stream.try_clone().unwrap()).lines() {
        let request: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        let status = match request["op"].as_str() {
            Some("observe") => "observed",
            Some("quit") => "bye",
            _ => "refused",
        };
        writeln!(stream, r#"{{"ok":true,"mode":"seat","status":"{status}"}}"#).unwrap();
        if status == "bye" {
            break;
        }
    }
    std::fs::remove_file(socket).unwrap();
    eprintln!("The agent seat's session ended.");
    ExitCode::SUCCESS
}

/// A directory of this run's own, short enough for a socket path.
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// `held-start <out> <case> -- <this executable> game <game case>`, run in `out`.
fn record(out: &Path, case: &str, game_case: &str) -> std::process::Output {
    record_from(out, out, case, game_case)
}

/// The same, run in `directory` and naming `out` as given, which may be relative to it.
fn record_from(directory: &Path, out: &Path, case: &str, game_case: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_held-start"))
        .args([out.to_str().unwrap(), case, "--"])
        .arg(std::env::current_exe().unwrap())
        .args(["game", game_case])
        .current_dir(directory)
        // So that a Git repository above the temporary directory is not found.
        .env("GIT_CEILING_DIRECTORIES", directory.parent().unwrap())
        .output()
        .unwrap()
}

fn read(path: PathBuf) -> String {
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn a_ready_start_records_its_exchange_and_the_removed_socket(scratch: &Path) {
    let recorded = record(scratch, "up", "ready");
    assert!(recorded.status.success(), "{recorded:?}");
    let record = read(scratch.join("up.record"));
    let socket = scratch.join("up.sock");
    let expected = "launcher exit: 0\nlauncher stdout lines: 1\nfirst status: ready\nobserve, quit: observed, bye\nsocket after: removed\nseat log, last lines:\n  listening\n  The agent seat's session ended.\n";
    assert!(record.ends_with(expected), "{record}");
    assert!(
        record.contains(&format!("game ready --socket {}\n", socket.display())),
        "{record}"
    );
    assert!(
        record.contains("revision: none: not a Git working tree\n"),
        "{record}"
    );
    assert_eq!(read(scratch.join("up.stdout")), format!("{READY}\n"));
    assert_eq!(
        read(scratch.join("up.seat.log")),
        "listening\nThe agent seat's session ended.\n"
    );
    assert_eq!(String::from_utf8_lossy(&recorded.stdout), record);
}

fn a_refused_start_records_its_error_and_sends_nothing(scratch: &Path) {
    let recorded = record(scratch, "down", "refused");
    assert!(recorded.status.success(), "{recorded:?}");
    let record = read(scratch.join("down.record"));
    let expected = "launcher exit: 1\nlauncher stdout lines: 1\nfirst status: refused\nfirst error: The stand-in table refuses.\nsocket after: removed\nseat log, last lines:\n  starting\n  The stand-in table refuses.\n";
    assert!(record.ends_with(expected), "{record}");
}

fn a_case_already_recorded_is_refused_and_kept(scratch: &Path) {
    let before = read(scratch.join("down.record"));
    let again = record(scratch, "down", "ready");
    assert_eq!(again.status.code(), Some(2), "{again:?}");
    let error = String::from_utf8_lossy(&again.stderr);
    assert!(error.contains("down.record already exists"), "{error}");
    assert_eq!(read(scratch.join("down.record")), before);
    assert!(!scratch.join("down.sock").exists(), "nothing was started");
}

fn a_relative_out_directory_still_hands_the_game_an_absolute_socket(scratch: &Path) {
    let recorded = record_from(scratch, Path::new("relative"), "up", "ready");
    assert!(recorded.status.success(), "{recorded:?}");
    let record = read(scratch.join("relative/up.record"));
    let socket = scratch.join("relative/up.sock");
    assert!(
        record.contains(&format!("game ready --socket {}\n", socket.display())),
        "{record}"
    );
    assert!(record.contains("first status: ready\n"), "{record}");
    assert!(
        record.contains("observe, quit: observed, bye\n"),
        "{record}"
    );
}

/// One test: its name and what it runs.
type Test = (&'static str, fn(&Path));

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.first().map(String::as_str) {
        Some("game") => return game(&arguments[1..]),
        Some("serve") => return serve(Path::new(&arguments[1])),
        _ => {}
    }
    // In order: the third reads the record the second wrote.
    let tests: [Test; 4] = [
        (
            "a_ready_start_records_its_exchange_and_the_removed_socket",
            a_ready_start_records_its_exchange_and_the_removed_socket,
        ),
        (
            "a_refused_start_records_its_error_and_sends_nothing",
            a_refused_start_records_its_error_and_sends_nothing,
        ),
        (
            "a_case_already_recorded_is_refused_and_kept",
            a_case_already_recorded_is_refused_and_kept,
        ),
        (
            "a_relative_out_directory_still_hands_the_game_an_absolute_socket",
            a_relative_out_directory_still_hands_the_game_an_absolute_socket,
        ),
    ];
    if arguments.iter().any(|argument| argument == "--list") {
        for (name, _) in tests {
            println!("{name}: test");
        }
        return ExitCode::SUCCESS;
    }
    let path = std::env::temp_dir().join(format!("ut-held-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    let scratch = Scratch(path);
    for (name, test) in tests {
        test(&scratch.0);
        println!("test {name} ... ok");
    }
    println!("test result: ok. {} passed", tests.len());
    ExitCode::SUCCESS
}
