//! The game's side of its agent players: the local socket on which external agents play
//! at a running table. This is not the [agent seat](crate::seat): an agent player sends
//! requests of the game's own protocol to a table that someone else is running.
//!
//! A [`Transport`] claims a path with [`Listener::claim`], so only its owner reaches it,
//! a dead socket is replaced, and one that answers is left alone and reported. It serves
//! any number of connections, each on a thread of its own. A connection sends request
//! lines, any number of them one after another, and reads one answer line for each. A
//! blank line is no request.
//!
//! A request crosses to the frame loop as an [`Incoming`], which [`Transport::take`]
//! returns, and the transport's [`Waker`] runs so that a frame comes. The game answers
//! through [`Reply::send`] on that frame or on any later one, and meanwhile nothing
//! blocks: a held request holds only its own connection's thread.
//!
//! **A dead client.** A client that closes its connection while a request of its is held
//! has hung up. [`Reply::hung_up`] is true from then on. The transport looks for it about
//! five times a second; when it finds it, it ends the connection and runs the waker.
//! What that ends is the game's decision: a game that goes on holding the request may
//! still answer it, and the answer goes nowhere. A client that only ended its input, as
//! `unfolded-agent-players` does after each request, has not hung up.
//!
//! **Closing.** Dropping the transport stops accepting and removes the path, then waits
//! up to a second for the connections to write the answers already sent and end, and cuts
//! those still open. A request that is untaken or still held by then is cut with its
//! connection: its client reads an end and no line. So take and answer what has arrived
//! before dropping the transport, or use [`Transport::close`] to answer untaken
//! requests without another frame. In either case, send the game's replies to held
//! requests first.
//!
//! Not here: the requests, who may send them, what they are answered, where the socket
//! is and what the player is told. The transport reads no request. The answers it sends
//! itself are the game's, through [`Protocol`].

use crate::seat::socket::{Listener, Unclaimed};
use bevy::{prelude::World, winit::EventLoopProxyWrapper};
use rustix::event::{PollFd, PollFlags, Timespec, poll};
use std::{
    io::{self, BufRead, BufReader, ErrorKind, Write},
    net::Shutdown,
    os::unix::net::UnixStream,
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{self, RecvTimeoutError},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

/// How often a held request's thread looks whether its client has hung up.
const WATCH: Duration = Duration::from_millis(200);

/// Makes a frame come: the transport runs it from a connection's thread when a request
/// arrives and when the client of a held one hangs up.
pub type Waker = Arc<dyn Fn() + Send + Sync>;

/// A [`Waker`] for a table in a winit window, where a reactive frame loop otherwise
/// sleeps until input. `None` without winit's event loop, as at the agent seat, whose
/// frames run only in its requests. What no test here holds is that it wakes a sleeping
/// window: a request to a game's windowed table does.
pub fn window_waker(world: &World) -> Option<Waker> {
    let proxy = (**world.get_resource::<EventLoopProxyWrapper>()?).clone();
    Some(Arc::new(move || {
        let _ = proxy.send_event(bevy::winit::WinitUserEvent::WakeUp);
    }))
}

/// The game's words where the transport speaks for it: the answers it must send itself,
/// each a whole line without its line ending, and what its log calls the players.
pub trait Protocol: Send + Sync + 'static {
    /// What begins the transport's own log line when it cannot accept a connection, such
    /// as `Agent partners`.
    fn players(&self) -> &str;

    /// The answer to a request whose [`Reply`] was dropped unsent while its connection
    /// was open, as when the game drops it or [`Transport::close`] lets it go. The
    /// connection ends after this answer.
    fn closed(&self) -> String;

    /// How long `request` may stay unanswered, and its answer then, after which the
    /// connection goes on and the game's own answer to it goes nowhere. Unless a game says
    /// otherwise, every request waits as long as the game holds it; so does one given
    /// longer than the clock can count.
    fn unanswered(&self, request: &str) -> Option<(Duration, String)> {
        let _ = request;
        None
    }
}

/// One request line and where its answer goes.
#[derive(Debug)]
pub struct Incoming {
    /// The request, without its line ending.
    pub line: String,
    pub reply: Reply,
}

/// Where one request's answer goes. Dropped unsent, the request is answered with
/// [`Protocol::closed`], unless [`Protocol::unanswered`] has answered it, its client has
/// hung up or the transport has cut its connection.
#[derive(Debug)]
pub struct Reply {
    answer: mpsc::Sender<String>,
    stream: Arc<UnixStream>,
}

impl Reply {
    /// Answer the request with one line, given without its line ending. The answer to a
    /// client that has hung up goes nowhere.
    pub fn send(self, answer: String) {
        let _ = self.answer.send(answer);
    }

    /// Whether the client closed its connection, or the transport ended it, so no answer
    /// can arrive. A game that ends such a request does so on a frame, so a windowless
    /// table with one it has not ended yet is not settled, as with
    /// [`Transport::waiting`].
    pub fn hung_up(&self) -> bool {
        ended(&self.stream)
    }
}

/// Whether `stream` is closed at either end. A client that ended only its input still
/// reads.
fn ended(stream: &UnixStream) -> bool {
    let mut polled = [PollFd::new(stream, PollFlags::empty())];
    let at_once = Timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    poll(&mut polled, Some(&at_once)).is_ok()
        && polled[0]
            .revents()
            .intersects(PollFlags::HUP | PollFlags::ERR)
}

/// Why no transport was opened. Nothing is left at the path that was not there before.
#[derive(Debug)]
#[non_exhaustive]
pub enum Unopened {
    /// The path could not be claimed.
    Unclaimed(Unclaimed),
    /// The claimed socket could not be set to accept without blocking.
    Unaccepting(io::Error),
    /// The thread that accepts connections could not be started.
    Unstarted(io::Error),
}

impl std::fmt::Display for Unopened {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unclaimed(unclaimed) => unclaimed.fmt(f),
            Self::Unaccepting(error) => write!(f, "cannot accept connections: {error}"),
            Self::Unstarted(error) => write!(f, "cannot start accepting connections: {error}"),
        }
    }
}

/// Each cause's text is this error's, so only the cause's own source is passed on.
impl std::error::Error for Unopened {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Unclaimed(unclaimed) => unclaimed.source(),
            Self::Unaccepting(error) | Self::Unstarted(error) => error.source(),
        }
    }
}

/// A game's socket for agent players while it is open. Dropping it closes it.
pub struct Transport {
    stop: Arc<AtomicBool>,
    accepting: Option<JoinHandle<()>>,
    connections: Connections,
    queued: Arc<AtomicUsize>,
    inbox: Option<Mutex<mpsc::Receiver<Incoming>>>,
}

/// The connections being served, by number, kept to cut them when the transport closes.
type Connections = Arc<Mutex<Vec<(u64, Arc<UnixStream>)>>>;

/// What every connection's thread needs of the transport.
#[derive(Clone)]
struct Shared {
    connections: Connections,
    outbox: mpsc::Sender<Incoming>,
    queued: Arc<AtomicUsize>,
    protocol: Arc<dyn Protocol>,
    waker: Option<Waker>,
}

impl Transport {
    /// Claim `path` and serve agent players on it. Without a waker, requests are taken on
    /// the frames that come anyway.
    pub fn open(
        path: &Path,
        protocol: impl Protocol,
        waker: Option<Waker>,
    ) -> Result<Self, Unopened> {
        let claimed = Listener::claim(path).map_err(Unopened::Unclaimed)?;
        claimed
            .unix_listener()
            .set_nonblocking(true)
            .map_err(Unopened::Unaccepting)?;
        let (outbox, inbox) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let shared = Shared {
            connections: Connections::default(),
            outbox,
            queued: Arc::new(AtomicUsize::new(0)),
            protocol: Arc::new(protocol),
            waker,
        };
        let (connections, queued) = (shared.connections.clone(), shared.queued.clone());
        let accepting = {
            let stop = stop.clone();
            thread::Builder::new()
                .name("agent players".into())
                // The claim lives as long as this thread and is dropped with it.
                .spawn(move || accept(&claimed, &stop, &shared))
                .map_err(Unopened::Unstarted)?
        };
        Ok(Self {
            stop,
            accepting: Some(accepting),
            connections,
            queued,
            inbox: Some(Mutex::new(inbox)),
        })
    }

    /// Whether a request has arrived that [`take`](Self::take) has not returned. A
    /// windowless table with one is not settled.
    pub fn waiting(&self) -> bool {
        self.queued.load(Ordering::Acquire) > 0
    }

    /// Every request that arrived since the last call, in arrival order.
    pub fn take(&self) -> Vec<Incoming> {
        let inbox = self
            .inbox
            .as_ref()
            .expect("the transport is still open")
            .lock()
            .expect("no thread panics holding it");
        let incoming: Vec<_> = inbox.try_iter().collect();
        self.queued.fetch_sub(incoming.len(), Ordering::AcqRel);
        incoming
    }

    /// Stop admitting requests to the frame queue and let go of those still in it.
    /// Their connection threads write [`Protocol::closed`] and end. A request already
    /// answered by its timeout gets only that answer. A request the game took remains
    /// the game's to answer before calling this.
    ///
    /// Stop accepting connections, remove the socket path and end socket reads so an
    /// idle connection can finish. Give in-flight answers the same bounded write
    /// grace as on drop.
    pub fn close(mut self) {
        // Disconnect the queue before ending socket reads. Dropping the queued
        // replies wakes their workers through the usual unsent-reply path.
        drop(self.inbox.take());
    }
}

/// Accept connections until `stop`.
fn accept(claimed: &Listener, stop: &AtomicBool, shared: &Shared) {
    let mut number = 0;
    while !stop.load(Ordering::Acquire) {
        match claimed.unix_listener().accept() {
            Ok((stream, _)) => {
                number += 1;
                serve(number, stream, shared);
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(error) => {
                eprintln!(
                    "{}: a connection failed: {error}",
                    shared.protocol.players()
                );
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

/// Serve one accepted connection on a thread of its own.
fn serve(number: u64, stream: UnixStream, shared: &Shared) {
    if stream.set_nonblocking(false).is_err() {
        return;
    }
    let stream = Arc::new(stream);
    open_connections(&shared.connections).push((number, stream.clone()));
    let served = shared.clone();
    let spawned = thread::Builder::new()
        .name("agent player".into())
        .spawn(move || {
            converse(&stream, &served);
            // Every answer is written: end the connection for its client too.
            let _ = stream.shutdown(Shutdown::Both);
            open_connections(&served.connections).retain(|(kept, _)| *kept != number);
        });
    if spawned.is_err() {
        open_connections(&shared.connections).retain(|(kept, _)| *kept != number);
    }
}

fn open_connections(
    connections: &Connections,
) -> std::sync::MutexGuard<'_, Vec<(u64, Arc<UnixStream>)>> {
    connections.lock().expect("no thread panics holding it")
}

/// One connection: each request line goes to the frame loop, and its answer comes back as
/// one line.
fn converse(stream: &Arc<UnixStream>, shared: &Shared) {
    let mut writer = &**stream;
    for line in BufReader::new(&**stream).lines() {
        let Ok(line) = line else {
            return;
        };
        if line.trim().is_empty() {
            continue;
        }
        let limit = shared.protocol.unanswered(&line);
        let (answer, answered) = mpsc::channel();
        let reply = Reply {
            answer,
            stream: stream.clone(),
        };
        shared.queued.fetch_add(1, Ordering::AcqRel);
        if shared.outbox.send(Incoming { line, reply }).is_err() {
            shared.queued.fetch_sub(1, Ordering::AcqRel);
            return;
        }
        wake(shared);
        let (answer, last) = match held(stream, &answered, limit) {
            Held::Answered(answer) => (answer, false),
            Held::LetGo => (shared.protocol.closed(), true),
            Held::HungUp => {
                wake(shared);
                return;
            }
        };
        if writeln!(writer, "{answer}")
            .and_then(|()| writer.flush())
            .is_err()
            || last
        {
            return;
        }
    }
}

fn wake(shared: &Shared) {
    if let Some(waker) = &shared.waker {
        waker();
    }
}

/// How a held request ended.
enum Held {
    /// With this answer: the game's, or the protocol's for a request left unanswered too
    /// long.
    Answered(String),
    /// The game let go of it unanswered.
    LetGo,
    /// Its client hung up first.
    HungUp,
}

fn held(
    stream: &UnixStream,
    answered: &mpsc::Receiver<String>,
    limit: Option<(Duration, String)>,
) -> Held {
    let limit =
        limit.and_then(|(within, answer)| Some((Instant::now().checked_add(within)?, answer)));
    loop {
        let watch = match &limit {
            Some((end, _)) => end.saturating_duration_since(Instant::now()).min(WATCH),
            None => WATCH,
        };
        match answered.recv_timeout(watch) {
            Ok(answer) => return Held::Answered(answer),
            Err(RecvTimeoutError::Disconnected) => return Held::LetGo,
            Err(RecvTimeoutError::Timeout) => {}
        }
        if ended(stream) {
            return Held::HungUp;
        }
        if let Some((end, answer)) = &limit
            && Instant::now() >= *end
        {
            return Held::Answered(answer.clone());
        }
    }
}

impl Drop for Transport {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(accepting) = self.accepting.take() {
            let _ = accepting.join();
        }
        if self.inbox.is_none() {
            for (_, stream) in open_connections(&self.connections).iter() {
                let _ = stream.shutdown(Shutdown::Read);
            }
        }
        // Connections write the answers already sent to them, then end when their client
        // closes its side; a connection still open after a second is cut.
        let deadline = Instant::now() + Duration::from_secs(1);
        while !open_connections(&self.connections).is_empty() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
        for (_, stream) in open_connections(&self.connections).drain(..) {
            let _ = stream.shutdown(Shutdown::Both);
        }
    }
}
