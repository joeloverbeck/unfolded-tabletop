//! Seam: waking a session over the socket, from its folder to the table's record of who
//! waits for what. A stand-in table serves a socket with `Seats`. A stand-in for Codex
//! reads a folder `Folders` wrote for the limit of one poll and the one program its rules
//! allow, runs that program with requests of this test's own, one process at a time, and
//! counts what would call Codex's model: each time a command ends. Expectations come from
//! Cuba Libre #202. Nothing here runs Codex. The words were measured once, in one
//! session under the approval policy `on-request` with a reviewing model (Cuba Libre #198);
//! the written folder sets `never` and `user`, and no session runs in such a folder until
//! Cuba Libre #205 and Arkham #297. The rule was measured in #198's folders.

use serde_json::{Value, json};
use std::{
    convert::Infallible,
    fs,
    io::{self, BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use unfolded_agent_players::{
    client,
    folder::{CODEX_WAIT, Folders, install},
    seats::{Ended, Event, Game, Joining, Pending, Seats, Submitted, Told, Waiting},
};

const RED: &str = "red";
const BLUE: &str = "blue";

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if let [socket, request] = &arguments[..]
        && request.starts_with('{')
    {
        return play(Path::new(socket), request);
    }
    for (name, test) in [
        ("one_wake_for_each_event_and_none_besides", events as fn()),
        ("a_wait_does_not_tell_what_an_answer_told", told),
        ("a_wait_ends_with_the_session_its_seat_or_the_table", ends),
    ] {
        test();
        println!("test {name} ... ok");
    }
    ExitCode::SUCCESS
}

/// The client program: one request, one answer.
fn play(socket: &Path, request: &str) -> ExitCode {
    match client::request(socket, request, || Ok::<(), Infallible>(())) {
        Ok(answer) => {
            answer.write_to(&mut io::stdout().lock()).unwrap();
            ExitCode::SUCCESS
        }
        Err(_) => ExitCode::from(2),
    }
}

/// A position each choice moves on by one. `turns` are who decides at each revision.
struct Position {
    revision: u64,
    turns: Vec<&'static str>,
}

impl Position {
    fn turn(&self) -> &'static str {
        self.turns[usize::try_from(self.revision).unwrap()]
    }
}

impl Game for Position {
    type Seat = &'static str;
    type Choice = String;

    fn revision(&self) -> u64 {
        self.revision
    }

    fn settled(&self) -> bool {
        true
    }

    fn check(&self, seat: &&'static str, _: &String) -> Result<(), String> {
        (self.turn() == *seat)
            .then_some(())
            .ok_or_else(|| "It is not your turn.".into())
    }

    fn commit(&mut self, _: &&'static str, _: &String) -> Result<(), String> {
        self.revision += 1;
        Ok(())
    }
}

/// What the human, or another session, does at the stand-in table.
enum Act {
    /// The human's own choice.
    Choose,
    TakeBack,
    Leave(String),
    /// The human ends play and the table says so.
    Close,
    /// The process ends with nothing said.
    Die,
}

struct Table {
    socket: PathBuf,
    folder: PathBuf,
    client: PathBuf,
    acts: Sender<Act>,
    serving: JoinHandle<()>,
    directory: tempfile::TempDir,
}

/// A table with red handed over and its player's folder written, whose `turns` are as
/// given.
fn table(turns: &[&'static str]) -> Table {
    let directory = tempfile::tempdir().unwrap();
    let socket = directory.path().join("players.sock");
    let folder = directory.path().join("player");
    let client = directory.path().join("client/play");
    install(&std::env::current_exe().unwrap(), &client).unwrap();
    let program = client.display();
    Folders {
        claude_code: &directory.path().join("claude-code"),
        codex: &folder,
        client: &client,
        instructions: &format!("How to run `{program}`, exactly. {CODEX_WAIT}"),
    }
    .write()
    .unwrap();
    let listener = UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    let (acts, received) = mpsc::channel();
    let game = Position {
        revision: 0,
        turns: turns.to_vec(),
    };
    let serving = thread::spawn(move || serve(&listener, game, &received));
    Table {
        socket,
        folder,
        client,
        acts,
        serving,
        directory,
    }
}

type Waits = Seats<&'static str, String, UnixStream>;

fn serve(listener: &UnixListener, mut game: Position, acts: &Receiver<Act>) {
    let mut seats: Waits = Seats::new([RED, BLUE]);
    seats.hand_over(&RED);
    loop {
        match acts.try_recv() {
            Ok(Act::Choose) => game.revision += 1,
            Ok(Act::TakeBack) => end(seats.take_back(&RED, "taken back")),
            Ok(Act::Leave(id)) => end(seats.leave(&id)),
            Ok(Act::Close) => {
                for ended in seats.close("the table closed") {
                    end(Some(ended));
                }
                return;
            }
            Ok(Act::Die) | Err(mpsc::TryRecvError::Disconnected) => return,
            Err(mpsc::TryRecvError::Empty) => {}
        }
        if let Ok((stream, _)) = listener.accept() {
            stream.set_nonblocking(false).unwrap();
            let mut line = String::new();
            BufReader::new(&stream).read_line(&mut line).unwrap();
            request(
                &mut seats,
                &mut game,
                &serde_json::from_str(&line).unwrap(),
                stream,
            );
        }
        let decision = (game.turn() == RED).then_some(game.revision);
        let ids: Vec<String> = seats.sessions().map(str::to_owned).collect();
        for id in ids {
            let due = decision.map(Event::<()>::Decision);
            if let Some((reply, told)) = seats.tell(&id, due, Instant::now()) {
                assert!(matches!(told, Told::Decision { .. }), "{told:?}");
                say(&reply, "decision", &game);
            }
        }
        thread::sleep(Duration::from_millis(2));
    }
}

fn request(seats: &mut Waits, game: &mut Position, asked: &Value, reply: UnixStream) {
    let now = Instant::now();
    if asked["op"] == "join" {
        let id = seats.claim(&RED, Joining::default(), now).unwrap();
        writeln!(&reply, "{}", json!({"status": "joined", "player": id})).unwrap();
        return;
    }
    let id = asked["player"].as_str().unwrap();
    if asked["op"] == "choose" {
        let revision = asked["revision"].as_u64().unwrap();
        let submitted = seats.submit(id, revision, "march".into(), game);
        assert_eq!(submitted, Submitted::Applied);
        if game.turn() == RED {
            seats.told_in_answer(id, game.revision);
            say(&reply, "applied", game);
            return;
        }
    }
    let pending = if game.turn() == RED {
        Pending::Decision(game.revision)
    } else {
        Pending::Nothing
    };
    match seats.wait(id, reply, None, pending, now) {
        Waiting::Held { .. } => {}
        Waiting::Refused(reply) => say(&reply, "refused", game),
    }
}

fn say(reply: &UnixStream, status: &str, game: &Position) {
    let mine = game.turn() == RED;
    writeln!(
        &*reply,
        "{}",
        json!({"status": status, "revision": game.revision, "yours": mine})
    )
    .unwrap();
}

fn end(ended: Option<Ended<String, UnixStream>>) {
    if let Some(reply) = ended.and_then(|ended| ended.wait) {
        writeln!(&reply, "{}", json!({"status": "closed"})).unwrap();
    }
}

impl Table {
    /// Run the installed client program once and read its answer.
    fn ask(&self, request: &Value) -> Value {
        let output = run(&self.client, &self.socket, request);
        serde_json::from_str(&output).unwrap()
    }

    fn join(&self) -> String {
        let joined = self.ask(&json!({"op": "join"}));
        joined["player"].as_str().unwrap().to_owned()
    }

    fn act(&self, act: Act) {
        self.acts.send(act).unwrap();
    }
}

/// What the program prints; nothing when the table ended without an answer.
fn run(client: &Path, socket: &Path, request: &Value) -> String {
    let output = Command::new(client)
        .arg(socket)
        .arg(request.to_string())
        .output()
        .unwrap();
    String::from_utf8(output.stdout).unwrap()
}

/// A stand-in for a Codex session, in a turn of its own.
struct Codex {
    /// What each ended command printed: one model call each.
    wakes: Arc<Mutex<Vec<Value>>>,
    turn: JoinHandle<()>,
}

impl Codex {
    /// Codex reads its folder's instructions and settings, waits, and acts on each wake:
    /// one choice and the next wait in one command, or the end of its turn.
    fn start(table: &Table, id: &str) -> Self {
        let read = |name: &str| fs::read_to_string(table.folder.join(name)).unwrap();
        // It is told how to wait, its settings let one poll last that long, and the command
        // is one its rules allow without an approval.
        assert!(read("AGENTS.md").contains(CODEX_WAIT));
        let settings = read(".codex/config.toml");
        let (_, limit) = settings
            .split_once("background_terminal_max_timeout = ")
            .unwrap();
        let limit: u64 = limit.lines().next().unwrap().parse().unwrap();
        assert!(limit >= 2 * 60 * 60 * 1000, "Two hours");
        assert!(CODEX_WAIT.contains(&format!("a yield time of {limit} ms")));
        let rules = read(".codex/rules/player.rules");
        let (_, pattern) = rules.split_once("pattern = [\"").unwrap();
        let permitted = PathBuf::from(pattern.split_once("\"]").unwrap().0);

        let wakes = Arc::new(Mutex::new(Vec::new()));
        let (woken, socket, id) = (wakes.clone(), table.socket.clone(), id.to_owned());
        let turn = thread::spawn(move || {
            let mut request = json!({"op": "wait", "player": id});
            loop {
                let printed = run(&permitted, &socket, &request);
                let wake: Value = serde_json::from_str(&printed)
                    .unwrap_or_else(|_| json!({"status": "no answer"}));
                woken.lock().unwrap().push(wake.clone());
                if wake["yours"] != true || wake["status"] == "refused" {
                    return;
                }
                request = json!({
                    "op": "choose", "player": id, "revision": wake["revision"],
                });
            }
        });
        Self { wakes, turn }
    }

    /// The status and revision of each wake so far.
    fn woken(&self) -> Vec<(String, Option<u64>)> {
        let wakes = self.wakes.lock().unwrap();
        wakes
            .iter()
            .map(|wake| {
                let status = wake["status"].as_str().unwrap().to_owned();
                (status, wake["revision"].as_u64())
            })
            .collect()
    }

    /// Wait until it has been woken `count` times, and then for nothing more.
    fn settles_at(&self, count: usize) {
        let limit = Instant::now() + Duration::from_secs(20);
        while self.woken().len() < count {
            assert!(Instant::now() < limit, "{:?}", self.woken());
            thread::sleep(Duration::from_millis(5));
        }
        thread::sleep(Duration::from_millis(300));
        assert_eq!(self.woken().len(), count, "{:?}", self.woken());
    }
}

/// One wake for each decision that becomes the session's, none while the human plays, and
/// none for a decision that the answer to its own choice told it.
fn events() {
    let table = table(&[BLUE, BLUE, RED, RED, BLUE, RED, BLUE]);
    let id = table.join();
    let codex = Codex::start(&table, &id);
    codex.settles_at(0);
    table.act(Act::Choose);
    codex.settles_at(0);
    table.act(Act::Choose);
    // Its decision at 2; its choice there leaves it one at 3, which that answer tells; and
    // its choice at 3 waits for what comes next.
    codex.settles_at(2);
    let told = |status: &str, revision| (status.to_owned(), Some(revision));
    assert_eq!(codex.woken(), [told("decision", 2), told("applied", 3)]);
    table.act(Act::Choose);
    codex.settles_at(3);
    assert_eq!(codex.woken()[2], told("decision", 5));
    assert!(!codex.turn.is_finished(), "It waits again");
    table.act(Act::Close);
    codex.settles_at(4);
    assert_eq!(codex.woken()[3], ("closed".to_owned(), None));
    codex.turn.join().unwrap();
    table.serving.join().unwrap();
    table.directory.close().unwrap();
}

/// A session that waits although the answer to its choice told it its decision is answered
/// so at once, and is not woken by that decision.
fn told() {
    let table = table(&[RED, RED]);
    let id = table.join();
    let wait = json!({"op": "wait", "player": id});
    assert_eq!(table.ask(&wait)["status"], "decision");
    let chosen = table.ask(&json!({"op": "choose", "player": id, "revision": 0}));
    assert_eq!(chosen["status"], "applied");
    assert_eq!(chosen["yours"], true);
    assert_eq!(table.ask(&wait)["status"], "refused");
    table.act(Act::Close);
    table.serving.join().unwrap();
}

/// The wait ends, in one wake, when its session leaves, its seat is taken back, the table
/// closes or the table dies; its client program has ended, and no other is started.
fn ends() {
    type End = fn(&str) -> Act;
    let ends: [(End, &str); 4] = [
        (|id| Act::Leave(id.to_owned()), "closed"),
        (|_| Act::TakeBack, "closed"),
        (|_| Act::Close, "closed"),
        (|_| Act::Die, "no answer"),
    ];
    for (end, said) in ends {
        let table = table(&[BLUE]);
        let id = table.join();
        let codex = Codex::start(&table, &id);
        codex.settles_at(0);
        table.act(end(&id));
        codex.settles_at(1);
        assert_eq!(codex.woken(), [(said.to_owned(), None)]);
        // Its turn is over, with its client program, which it waited for.
        codex.turn.join().unwrap();
        drop(table.acts);
        table.serving.join().unwrap();
        table.directory.close().unwrap();
    }
}
