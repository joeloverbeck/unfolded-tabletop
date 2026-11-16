//! A game's start of its agent seat through [`Start`], end to end: this executable
//! is the game. Run with `game <case> [--socket <path>]` it is the game's own
//! command, and its table belongs to no game and needs no graphics adapter. Run
//! by `cargo test` with no such arguments, it starts itself as that command in
//! each case, as an agent's shell would, and reads what the command prints.
//!
//! A held start goes through the real launcher: the launching command starts this
//! executable again, in its own session, as the seat. So this executable has no
//! test harness of its own. Each game command it runs is a process of its own; the
//! cases run one after another in this one. As a harness would, it runs only the
//! cases whose names contain a filter given on its command line, and `--list` lists
//! them.
#![cfg(feature = "seat")]

use bevy::{asset::AssetPlugin, prelude::*};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Output, Stdio},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use unfolded_tabletop::{
    capture,
    seat::{
        Admission, Game, Hit, Name, Observed,
        start::{ADMISSION, IDLE, Start, Table},
    },
};

/// The variable that marks the seat this game's launcher started.
const MARKER: &str = "UNFOLDED_TABLETOP_START_TEST_SEAT";
/// The table's refusal of play, when it refuses.
const REFUSAL: &str = "The table of no game refuses play.";
/// Why the table cannot be built, when it cannot.
const UNBUILT: &str = "The table of no game cannot be built here.";
/// What still blocks the table's admission, which the game's words read from it.
const BLOCKER: &str = "its pieces are still loading";

/// What the game's words for an unfinished admission read from the table.
#[derive(Resource)]
struct Blocker(&'static str);

/// The game's own admission of its table.
#[derive(Resource)]
struct Admitting(Admission);

/// The camera the seat's image is rendered by.
#[derive(Component)]
struct Seen;

/// The game's side of the seat: an empty table that admits as it was built to.
struct Empty;

impl Game for Empty {
    fn observe(&self, _: &mut World) -> Observed {
        Observed {
            observation: json!({"table": "empty"}),
            table: json!({"settled": true}),
        }
    }

    fn revision(&self, _: &mut World) -> u64 {
        0
    }

    fn resolve(&self, _: &mut World, name: &Name) -> Result<Vec<Vec2>, String> {
        Err(format!("The empty table shows no {name:?}."))
    }

    fn identify(&self, _: &mut World, _: Entity) -> Hit {
        Hit {
            name: None,
            description: Value::Null,
        }
    }

    fn unsettled(&self, _: &mut World) -> Vec<String> {
        Vec::new()
    }

    fn admission(&self, world: &World) -> Admission {
        world.resource::<Admitting>().0.clone()
    }
}

/// The game's start: its marker, a short admission limit, an idle end that cannot
/// outlast a failed test for long, and its words for an unfinished admission.
const START: Start = Start::new(MARKER, unfinished)
    .admission(Duration::from_millis(300))
    .idle(Duration::from_secs(60));

fn unfinished(world: &World, limit: Duration) -> String {
    format!(
        "The empty table was not admitted within {} milliseconds: {}.",
        limit.as_millis(),
        world.resource::<Blocker>().0
    )
}

/// How the table is built in each case.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Case {
    /// Admitted at once.
    Ready,
    /// The game cannot build its table.
    Unbuilt,
    /// The table has no camera for the seat's image, so the seat cannot open.
    Unopened,
    /// The game refuses play.
    Refused,
    /// The game is still checking when the limit passes.
    Unfinished,
}

impl Case {
    const ALL: [Self; 5] = [
        Self::Ready,
        Self::Unbuilt,
        Self::Unopened,
        Self::Refused,
        Self::Unfinished,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Unbuilt => "unbuilt",
            Self::Unopened => "unopened",
            Self::Refused => "refused",
            Self::Unfinished => "unfinished",
        }
    }

    fn named(name: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|case| case.name() == name)
            .unwrap_or_else(|| panic!("no case {name}"))
    }

    /// The table as the game builds it, with its windowless surface installed.
    fn build(self) -> Result<Table<Empty>, String> {
        if self == Self::Unbuilt {
            return Err(UNBUILT.into());
        }
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Image>()
            .insert_resource(Blocker(BLOCKER))
            .insert_resource(Admitting(match self {
                Self::Refused => Admission::Refused(REFUSAL.into()),
                Self::Unfinished => Admission::Checking,
                _ => Admission::Admitted,
            }));
        capture::install::<Seen>(&mut app, UVec2::new(64, 40));
        if self != Self::Unopened {
            app.world_mut().spawn(Seen);
        }
        Ok(Table::new(app, Empty))
    }

    /// The `refused` error the case's start answers with, or none for a start that is
    /// ready.
    fn refusal(self) -> Option<String> {
        match self {
            Self::Ready => None,
            Self::Unbuilt => Some(UNBUILT.into()),
            Self::Unopened => Some("The table spawned no camera for the seat's image.".into()),
            Self::Refused => Some(REFUSAL.into()),
            Self::Unfinished => Some(format!(
                "The empty table was not admitted within 300 milliseconds: {BLOCKER}."
            )),
        }
    }
}

/// The game's own command: `game <case> [--socket <path>]`.
fn game(arguments: &[String]) -> ExitCode {
    let (case, socket) = match arguments {
        [case] => (Case::named(case), None),
        [case, flag, path] if flag == "--socket" => (Case::named(case), Some(Path::new(path))),
        _ => panic!("usage: game <case> [--socket <path>]"),
    };
    match START.run(socket, || case.build()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

/// A directory of this run's own, removed when it ends. Its path stays short, since a
/// socket path holds at most 107 bytes.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("ut-start-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn join(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The game's command, as an agent's shell starts it.
fn command(arguments: &[&str]) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.arg("game").args(arguments).env_remove(MARKER);
    command
}

/// What a command printed on standard output, one response per line.
fn responses(output: &Output) -> Vec<Value> {
    String::from_utf8(output.stdout.clone())
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap_or_else(|_| panic!("not JSON: {line}")))
        .collect()
}

fn refused(error: &str) -> Value {
    json!({"ok": false, "mode": "seat", "status": "refused", "error": error})
}

/// One connection to a held session: each request, then its response.
fn converse(socket: &Path, requests: &[Value]) -> Vec<Value> {
    let mut stream = UnixStream::connect(socket).unwrap();
    for request in requests {
        writeln!(stream, "{request}").unwrap();
    }
    stream.shutdown(std::net::Shutdown::Write).unwrap();
    BufReader::new(stream)
        .lines()
        .map(|line| serde_json::from_str(&line.unwrap()).unwrap())
        .collect()
}

/// Wait until `done` holds, for at most ten seconds.
fn eventually(mut done: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !done() {
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    true
}

fn a_held_start_prints_the_launched_seats_ready_and_leaves_it_serving(scratch: &Scratch) {
    let socket = scratch.join("ready.sock");
    let started = command(&["ready", "--socket", socket.to_str().unwrap()])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(started.status.success(), "{started:?}");
    let printed = responses(&started);
    assert_eq!(printed.len(), 1, "{printed:?}");
    assert_eq!(printed[0]["status"], "ready", "{printed:?}");
    assert_eq!(printed[0]["observation"], json!({"table": "empty"}));

    let answered = converse(&socket, &[json!({"op": "observe"}), json!({"op": "quit"})]);
    let statuses: Vec<_> = answered.iter().map(|line| &line["status"]).collect();
    assert_eq!(statuses, ["observed", "bye"], "{answered:?}");
    assert!(
        eventually(|| !socket.exists()),
        "the session left its socket"
    );
    let log = scratch.join("ready.sock.log");
    let ended = || {
        std::fs::read_to_string(&log)
            .unwrap()
            .contains("The agent seat's session ended.")
    };
    assert!(
        eventually(ended),
        "{}",
        std::fs::read_to_string(&log).unwrap()
    );
}

fn batch_form_answers_ready_first_and_then_its_input(_: &Scratch) {
    let mut started = command(&["ready"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(started.stdin.take().unwrap(), "{}", json!({"op": "quit"})).unwrap();
    let ended = started.wait_with_output().unwrap();
    assert!(ended.status.success(), "{ended:?}");
    let statuses: Vec<_> = responses(&ended)
        .into_iter()
        .map(|line| line["status"].clone())
        .collect();
    assert_eq!(statuses, ["ready", "bye"]);
}

/// Each failure before `ready` is one `refused` line and an unsuccessful exit, in batch
/// form and from the command that starts a held seat, which relays the seat's.
fn every_failure_before_ready_is_one_refused_line_in_both_forms(scratch: &Scratch) {
    for case in Case::ALL {
        let Some(error) = case.refusal() else {
            continue;
        };
        let batch = command(&[case.name()])
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert!(!batch.status.success(), "{case:?}: {batch:?}");
        assert_eq!(responses(&batch), [refused(&error)], "batch form, {case:?}");

        let socket = scratch.join(&format!("{}.sock", case.name()));
        let held = command(&[case.name(), "--socket", socket.to_str().unwrap()])
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert!(!held.status.success(), "{case:?}: {held:?}");
        assert_eq!(responses(&held), [refused(&error)], "held, {case:?}");
        assert!(!socket.exists(), "{case:?} left its socket");
        let log =
            std::fs::read_to_string(scratch.join(&format!("{}.sock.log", case.name()))).unwrap();
        assert!(
            log.contains(&error),
            "{case:?}: the seat's log has its error: {log}"
        );
    }
}

/// A socket that cannot be claimed refuses the start before any seat is launched.
fn a_socket_that_cannot_be_claimed_is_one_refused_line(scratch: &Scratch) {
    let path = scratch.join("taken");
    std::fs::write(&path, "kept").unwrap();
    let held = command(&["ready", "--socket", path.to_str().unwrap()])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!held.status.success(), "{held:?}");
    let error = format!(
        "Cannot listen on {}: the path exists and is not a socket; it was left unchanged",
        path.display()
    );
    assert_eq!(responses(&held), [refused(&error)]);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "kept");
    assert!(!scratch.join("taken.log").exists(), "no seat was launched");
}

/// A process marked as a launched seat that was handed no listener refuses.
fn a_marked_seat_handed_no_listener_is_one_refused_line(scratch: &Scratch) {
    let path = scratch.join("unhanded.sock");
    let seat = command(&["ready", "--socket", path.to_str().unwrap()])
        .env(MARKER, "1")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!seat.status.success(), "{seat:?}");
    let printed = responses(&seat);
    assert_eq!(printed.len(), 1, "{printed:?}");
    assert_eq!(printed[0]["status"], "refused");
    let error = printed[0]["error"].as_str().unwrap();
    let handed = format!(
        "The seat was not handed a listening socket for {} (",
        path.display()
    );
    assert!(error.starts_with(&handed), "{error}");
    assert!(
        error.ends_with(&format!("); start it without {MARKER}.")),
        "{error}"
    );
}

/// The defaults are what the games used before they moved here: Arkham, Netrunner and
/// Cuba Libre admitted within 120 seconds, and all four idled for 30 minutes.
fn the_defaults_are_the_games_own(_: &Scratch) {
    assert_eq!(ADMISSION, Duration::from_secs(120));
    assert_eq!(IDLE, Duration::from_secs(1_800));
}

/// Something whose drop is recorded, in order with the others.
struct Ends(&'static str, Arc<Mutex<Vec<&'static str>>>);

impl Drop for Ends {
    fn drop(&mut self) {
        self.1.lock().unwrap().push(self.0);
    }
}

/// Held by the app, so it drops with the app.
#[derive(Resource)]
struct InApp(#[expect(dead_code, reason = "held for its drop")] Ends);

/// The case's table, keeping something, and the record of what has dropped.
fn kept_table(case: Case) -> (Table<Empty, Ends>, Arc<Mutex<Vec<&'static str>>>) {
    let ended = Arc::new(Mutex::new(Vec::new()));
    let mut table = case.build().unwrap();
    table.app.insert_resource(InApp(Ends("app", ended.clone())));
    (table.keeping(Ends("kept", ended.clone())), ended)
}

/// What a table keeps outlives its app, and a test seats an agent with no process of
/// its own and no graphics adapter.
fn a_table_seated_in_process_keeps_what_outlives_its_app(_: &Scratch) {
    let (table, ended) = kept_table(Case::Ready);
    let mut seated = START.admit(table).unwrap_or_else(|error| panic!("{error}"));
    let ready = seated.seat.ready(&mut seated.app);
    assert_eq!(ready["status"], "ready");
    drop(seated);
    assert_eq!(*ended.lock().unwrap(), ["app", "kept"]);
}

/// A table that is not admitted still drops what it keeps after its app.
fn a_refused_table_drops_what_it_keeps_after_its_app(_: &Scratch) {
    let (table, ended) = kept_table(Case::Refused);
    assert_eq!(START.admit(table).err().as_deref(), Some(REFUSAL));
    assert_eq!(*ended.lock().unwrap(), ["app", "kept"]);
}

/// One test: its name and what it runs.
type Test = (&'static str, fn(&Scratch));

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.first().is_some_and(|first| first == "game") {
        return game(&arguments[1..]);
    }
    let tests: [Test; 8] = [
        (
            "a_held_start_prints_the_launched_seats_ready_and_leaves_it_serving",
            a_held_start_prints_the_launched_seats_ready_and_leaves_it_serving,
        ),
        (
            "batch_form_answers_ready_first_and_then_its_input",
            batch_form_answers_ready_first_and_then_its_input,
        ),
        (
            "every_failure_before_ready_is_one_refused_line_in_both_forms",
            every_failure_before_ready_is_one_refused_line_in_both_forms,
        ),
        (
            "a_socket_that_cannot_be_claimed_is_one_refused_line",
            a_socket_that_cannot_be_claimed_is_one_refused_line,
        ),
        (
            "a_marked_seat_handed_no_listener_is_one_refused_line",
            a_marked_seat_handed_no_listener_is_one_refused_line,
        ),
        (
            "the_defaults_are_the_games_own",
            the_defaults_are_the_games_own,
        ),
        (
            "a_table_seated_in_process_keeps_what_outlives_its_app",
            a_table_seated_in_process_keeps_what_outlives_its_app,
        ),
        (
            "a_refused_table_drops_what_it_keeps_after_its_app",
            a_refused_table_drops_what_it_keeps_after_its_app,
        ),
    ];
    let filters: Vec<&str> = arguments
        .iter()
        .filter(|argument| !argument.starts_with('-'))
        .map(String::as_str)
        .collect();
    let chosen: Vec<&Test> = tests
        .iter()
        .filter(|(name, _)| filters.is_empty() || filters.iter().any(|f| name.contains(f)))
        .collect();
    if arguments.iter().any(|argument| argument == "--list") {
        for (name, _) in &chosen {
            println!("{name}: test");
        }
        return ExitCode::SUCCESS;
    }
    let scratch = Scratch::new();
    for (name, test) in &chosen {
        test(&scratch);
        println!("test {name} ... ok");
    }
    println!("test result: ok. {} passed", chosen.len());
    ExitCode::SUCCESS
}
