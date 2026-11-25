//! Keeping a held seat's table running while others play at it. A windowless table runs
//! frames only inside the seat's requests, so an agent player's request to a table the
//! seat hosts is taken only when one comes. [`keep_running`] sends them between the
//! seat's own decisions: it reads the table once, then settles it again and again, and
//! answers nothing.
//!
//! The game says when the table needs the agent at its seat, what is worth printing and
//! what the endings mean, in the `see` it is given every answer through.

use crate::{Connection, Ended, Reply, RequestError, Unreached};
use serde_json::Value;
use std::{
    error::Error,
    fmt, io,
    path::Path,
    time::{Duration, Instant},
};
use unfolded_seat_protocol::{Request, Status};

/// The pause between settles while the game says the table rests: what an agent player
/// asked waits at most this long for the settle that serves it.
const INTERVAL: Duration = Duration::from_millis(500);
/// The steps of one settle: two seconds of table time, so that one under way when the
/// time is up ends soon after.
const STEPS: u32 = 120;

/// What to do after an answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Next<S> {
    /// Return, with the game's own reason.
    Stop(S),
    /// The table rests: settle it again after a pause of half a second.
    Rest,
    /// The table is still moving: settle it again at once.
    Go,
}

/// Why the table is no longer kept running.
#[derive(Debug)]
pub enum Kept<S> {
    /// The game stopped it, with its reason.
    Stopped(S),
    /// The time was up.
    TimedOut,
    /// The seat is gone, or was never there.
    Gone(Gone),
}

/// How the seat was found to be gone.
#[derive(Debug)]
pub enum Gone {
    /// No seat listens on the socket.
    NotListening(io::Error),
    /// Its session ended at a request.
    Ended(Ended),
}

impl fmt::Display for Gone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotListening(error) => error.fmt(f),
            Self::Ended(ended) => ended.fmt(f),
        }
    }
}

/// Why keeping the table running failed. `E` is the caller's own error.
#[derive(Debug)]
#[non_exhaustive]
pub enum Unkept<E> {
    /// The caller's `pause`, or its `see`.
    Stopped(E),
    /// The socket could not be reached, for another reason than that no seat listens.
    Unreached(io::Error),
    /// A request has no reply; the caller's `check` may have ended it.
    Request(RequestError<E>),
    /// The seat answered neither an observation nor a settle.
    Unexpected {
        /// The request, as it was sent.
        request: String,
        /// The answer's line.
        line: String,
    },
}

impl<E: fmt::Display> fmt::Display for Unkept<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stopped(error) => error.fmt(f),
            Self::Unreached(error) => write!(f, "cannot reach the seat: {error}"),
            Self::Request(error) => error.fmt(f),
            Self::Unexpected { request, line } => {
                write!(f, "the seat answered {request} with: {line}")
            }
        }
    }
}

/// The caller's own error is its own: as a source it is not named.
impl<E: fmt::Debug + fmt::Display> Error for Unkept<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Unreached(error) => error.source(),
            Self::Request(error) => error.source(),
            Self::Stopped(_) | Self::Unexpected { .. } => None,
        }
    }
}

/// Keep the table of the seat held on `socket` running until `see` stops it, `until` has
/// passed when an answer has been seen and any pause after it is over, or the seat goes
/// away. The first request only observes, so what the first settle changes shows; every
/// later one settles 120 steps. Each is a connection of its own, so the seat serves other
/// commands between them. Every answer goes through `see`.
///
/// `check` runs about five times a second while a request waits for its answer and
/// nothing arrives, and at no other time: a seat that answers sooner never has it run.
/// Its error ends the request, so a deadline past `until` belongs in it for a seat that
/// stopped answering.
///
/// `pause` waits out the time it is given, half a second or what is left until `until`,
/// after the answer of a resting table. Its error, such as an interrupt, ends the loop.
/// The seat has answered by then, so a pause that ends late is no seat that stopped: the
/// time is up, and that is all.
pub fn keep_running<S, E>(
    socket: &Path,
    until: Instant,
    mut check: impl FnMut() -> Result<(), E>,
    mut pause: impl FnMut(Duration) -> Result<(), E>,
    mut see: impl FnMut(&Value) -> Result<Next<S>, E>,
) -> Result<Kept<S>, Unkept<E>> {
    let mut request = Request::Observe {};
    loop {
        let answer = match ask(socket, &request, &mut check)? {
            Ok(answer) => answer,
            Err(gone) => return Ok(Kept::Gone(gone)),
        };
        request = Request::Settle { steps: Some(STEPS) };
        match see(&answer).map_err(Unkept::Stopped)? {
            Next::Stop(reason) => return Ok(Kept::Stopped(reason)),
            Next::Rest => {
                let left = until.saturating_duration_since(Instant::now());
                pause(INTERVAL.min(left)).map_err(Unkept::Stopped)?;
            }
            Next::Go => {}
        }
        if Instant::now() >= until {
            return Ok(Kept::TimedOut);
        }
    }
}

/// One request on a connection of its own, answered by the seat or, inside, showing it
/// is gone.
fn ask<E>(
    socket: &Path,
    request: &Request,
    check: &mut impl FnMut() -> Result<(), E>,
) -> Result<Result<Value, Gone>, Unkept<E>> {
    let connection = match Connection::open(socket) {
        Ok(connection) => connection,
        Err(Unreached::NotListening(error)) => return Ok(Err(Gone::NotListening(error))),
        Err(Unreached::Failed(error)) => return Err(Unkept::Unreached(error)),
    };
    match connection
        .request(request, check)
        .map_err(Unkept::Request)?
    {
        Reply::Answer(answer) => match answer.status {
            Status::Observed | Status::Settled | Status::Unsettled => Ok(Ok(answer.response)),
            _ => Err(Unkept::Unexpected {
                request: request.to_string(),
                line: answer.line,
            }),
        },
        // A seat that ends with this request unread resets the connection.
        Reply::Ended(ended) => Ok(Err(Gone::Ended(ended))),
    }
}
