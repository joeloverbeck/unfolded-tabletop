//! Seam: a game's agent players over its real socket, with the game's frames played by
//! the test. Expectations come from Cuba Libre #199: several connections, one answer line
//! for each request line, requests held without blocking a frame, a wake of the frame
//! loop, a clean close, a dead client noticed, and a half-closed one served.
#![cfg(feature = "seat")]

use std::{
    io::{BufRead, BufReader, Write},
    net::Shutdown,
    os::unix::{
        fs::{FileTypeExt, PermissionsExt},
        net::{UnixListener, UnixStream},
    },
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use unfolded_tabletop::{
    agent_transport::{Incoming, Protocol, Transport, Unopened, Waker, window_waker},
    seat::socket::Unclaimable,
};

/// A game whose request `brief` may stay unanswered for a second.
struct Game;

impl Protocol for Game {
    fn players(&self) -> &str {
        "Test players"
    }

    fn closed(&self) -> String {
        "closed".into()
    }

    fn unanswered(&self, request: &str) -> Option<(Duration, String)> {
        request
            .starts_with("brief")
            .then(|| (Duration::from_secs(1), format!("nothing for {request}")))
    }
}

struct Open {
    transport: Transport,
    socket: PathBuf,
    wakes: Arc<AtomicUsize>,
    _directory: tempfile::TempDir,
}

fn open() -> Open {
    let directory = tempfile::tempdir().unwrap();
    let socket = directory.path().join("players.sock");
    let wakes = Arc::new(AtomicUsize::new(0));
    let counted = wakes.clone();
    let waker: Waker = Arc::new(move || {
        counted.fetch_add(1, Ordering::SeqCst);
    });
    let transport = Transport::open(&socket, Game, Some(waker)).unwrap();
    Open {
        transport,
        socket,
        wakes,
        _directory: directory,
    }
}

impl Open {
    /// Wait until the frame loop has been woken this often in all. A connection wakes it
    /// just after its request can be taken.
    fn woken(&self, times: usize) {
        until("the wakes", || self.wakes.load(Ordering::SeqCst) >= times);
        assert_eq!(self.wakes.load(Ordering::SeqCst), times);
    }

    /// The next request, taken as a frame would take it.
    fn next(&self) -> Incoming {
        let mut taken = self.take(1);
        taken.remove(0)
    }

    /// The next `count` requests.
    fn take(&self, count: usize) -> Vec<Incoming> {
        let mut taken = Vec::new();
        until("the requests arrive", || {
            taken.extend(self.transport.take());
            taken.len() >= count
        });
        assert_eq!(taken.len(), count);
        taken
    }
}

/// Wait at most ten seconds until `reached`.
fn until(what: &str, mut reached: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !reached() {
        assert!(Instant::now() < deadline, "It never came: {what}");
        thread::sleep(Duration::from_millis(2));
    }
}

struct Client(BufReader<UnixStream>);

impl Client {
    fn connect(socket: &Path) -> Self {
        let stream = UnixStream::connect(socket).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        Self(BufReader::new(stream))
    }

    fn send(&self, request: &str) {
        writeln!(self.0.get_ref(), "{request}").unwrap();
    }

    fn half_close(&self) {
        self.0.get_ref().shutdown(Shutdown::Write).unwrap();
    }

    /// The next answer line; empty once the connection has ended.
    fn answer(&mut self) -> String {
        let mut line = String::new();
        self.0.read_line(&mut line).unwrap();
        line
    }
}

#[test]
fn connections_are_served_at_once_and_a_held_request_holds_only_its_own() {
    let open = open();
    let mut waiting = Client::connect(&open.socket);
    waiting.send("wait");
    let held = open.next();
    assert_eq!(held.line, "wait");
    open.woken(1);
    assert!(!open.transport.waiting(), "it was taken");

    let mut other = Client::connect(&open.socket);
    other.send("look");
    until("the second request arrives", || open.transport.waiting());
    let look = open.next();
    assert_eq!(look.line, "look");
    open.woken(2);
    look.reply.send("seen".into());
    assert_eq!(other.answer(), "seen\n");

    held.reply.send("your decision".into());
    assert_eq!(waiting.answer(), "your decision\n");
    assert!(
        open.transport.take().is_empty(),
        "taking blocks on no request"
    );
}

#[test]
fn a_connection_sends_request_after_request_and_reads_an_answer_for_each_in_order() {
    let open = open();
    let mut client = Client::connect(&open.socket);
    client.send("one\n \ntwo");
    client.half_close();
    let one = open.next();
    assert_eq!(one.line, "one");
    assert!(
        !one.reply.hung_up(),
        "ending its input is no hang-up: it still reads"
    );
    thread::sleep(Duration::from_millis(500));
    assert!(
        open.transport.take().is_empty(),
        "the second waits for the first's answer"
    );
    one.reply.send("first".into());
    assert_eq!(client.answer(), "first\n");
    let two = open.next();
    assert_eq!(two.line, "two");
    two.reply.send("second".into());
    assert_eq!(client.answer(), "second\n");
    assert_eq!(client.answer(), "", "its input has ended");
    open.woken(2); // One for each request, and no hang-up.
}

#[test]
fn a_client_that_closes_while_its_request_is_held_is_noticed_and_wakes_the_frame_loop() {
    let open = open();
    let dying = Client::connect(&open.socket);
    dying.send("wait");
    // As the one-request client leaves it: its input ended, the answer awaited.
    dying.half_close();
    let mut living = Client::connect(&open.socket);
    living.send("wait");
    living.half_close();
    let mut held = open.take(2);
    open.woken(2);
    drop(dying);
    open.woken(3);
    let gone: Vec<usize> = (0..2).filter(|at| held[*at].reply.hung_up()).collect();
    assert_eq!(gone.len(), 1, "only the client that closed has hung up");
    assert!(
        open.transport.take().is_empty(),
        "a hang-up is no request of its own"
    );

    // The game may answer what it still holds; only the living client hears it.
    held.remove(gone[0]).reply.send("to nobody".into());
    held.remove(0).reply.send("your decision".into());
    assert_eq!(living.answer(), "your decision\n");
}

#[test]
fn a_hang_up_is_known_to_the_reply_at_once() {
    let open = open();
    let dying = Client::connect(&open.socket);
    dying.send("wait");
    let held = open.next();
    assert!(!held.reply.hung_up());
    drop(dying);
    assert!(held.reply.hung_up());
}

#[test]
fn what_the_transport_says_itself_is_in_the_games_words() {
    let open = open();
    let mut client = Client::connect(&open.socket);

    // A request the game allows only so long is answered for it, and the connection
    // goes on; the game's late answer goes nowhere.
    client.send("brief look");
    let late = open.next();
    assert_eq!(client.answer(), "nothing for brief look\n");
    late.reply.send("too late".into());
    client.send("brief again");
    open.next().reply.send("in time".into());
    assert_eq!(client.answer(), "in time\n");

    // A request the game lets go of ends its connection.
    client.send("wait");
    drop(open.next());
    assert_eq!(client.answer(), "closed\n");
    assert_eq!(client.answer(), "");
}

#[test]
fn closing_writes_the_answers_already_sent_then_ends_every_connection_and_the_path() {
    let open = open();
    let mut answered = Client::connect(&open.socket);
    answered.send("wait");
    answered.half_close();
    let held = open.next();
    let mut idle = Client::connect(&open.socket);
    idle.send("look");
    let look = open.next();
    look.reply.send("seen".into());
    assert_eq!(idle.answer(), "seen\n");

    held.reply.send("the table closed".into());
    let Open {
        transport, socket, ..
    } = open;
    let started = Instant::now();
    drop(transport);
    let took = started.elapsed();
    assert!(
        took < Duration::from_secs(5),
        "a silent client holds it only a moment: {took:?}"
    );
    assert!(!socket.exists(), "the path goes with it");
    assert_eq!(answered.answer(), "the table closed\n");
    assert_eq!(answered.answer(), "");
    assert_eq!(idle.answer(), "", "a connection left open is cut");
}

#[test]
fn only_its_owner_reaches_the_socket_a_dead_one_is_replaced_and_an_answering_one_is_left() {
    let directory = tempfile::tempdir().unwrap();
    let socket = directory.path().join("players.sock");
    drop(UnixListener::bind(&socket).unwrap());
    assert!(socket.exists(), "a crashed game leaves its socket");
    let first = Transport::open(&socket, Game, None).unwrap();
    let file = std::fs::symlink_metadata(&socket).unwrap();
    assert!(file.file_type().is_socket());
    assert_eq!(file.permissions().mode() & 0o777, 0o600);

    let refused = Transport::open(&socket, Game, None).err().unwrap();
    let Unopened::Unclaimed(unclaimed) = &refused else {
        panic!("{refused}");
    };
    assert!(matches!(unclaimed.cause(), Unclaimable::Answered));
    assert_eq!(unclaimed.path(), socket);
    assert_eq!(refused.to_string(), unclaimed.to_string());

    // The one that answers goes on: without a waker, requests wait for a frame.
    let mut client = Client::connect(&socket);
    client.send("look");
    until("the request arrives", || first.waiting());
    for look in first.take() {
        look.reply.send(format!("seen {}", look.line));
    }
    assert_eq!(client.answer(), "seen look\n");
}

#[test]
fn a_table_without_a_window_has_no_window_to_wake() {
    assert!(window_waker(&bevy::prelude::World::new()).is_none());
}
