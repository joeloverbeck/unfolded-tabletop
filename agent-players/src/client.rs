//! One agent player's request to a running game: [`request`] sends one line and returns
//! the one line that answers it, as a command does that ends with each request.
//!
//! The game's command keeps what it says and how it exits: it spells the request, prints
//! the [`Answer`] with [`Answer::write_to`], maps [`Answer::status`] to an exit status,
//! words each [`Unanswered`] cause, and bounds the wait in its `check`, which runs about
//! five times a second while the game says nothing.

use serde_json::Value;
use std::{
    error::Error,
    fmt,
    io::{self, Write},
    path::Path,
};
use unfolded_seat_client::{AskError, Connection, Unreached};

/// Send `request`, one line given without its line ending, to the game listening on
/// `socket`, end the connection's input and read one answer. The game may hold it as long
/// as it likes; an error of `check` ends the wait and comes back as it is.
pub fn request<E>(
    socket: &Path,
    request: &str,
    check: impl FnMut() -> Result<(), E>,
) -> Result<Answer, Unanswered<E>> {
    let connection = Connection::open(socket).map_err(|unreached| match unreached {
        Unreached::NotListening(error) => Unanswered::NotListening(error),
        Unreached::Failed(error) => Unanswered::Unreached(error),
    })?;
    let line = connection
        .ask(request, check)
        .map_err(|failed| match failed {
            AskError::Stopped(error) => Unanswered::Stopped(error),
            AskError::Unsent(error) => Unanswered::Unsent(error),
            AskError::Unreadable(error) => Unanswered::Unreadable(error),
        })?;
    line.map(|line| Answer { line }).ok_or(Unanswered::Closed)
}

/// The one line that answered a request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    line: Vec<u8>,
}

impl Answer {
    /// Write the line as it arrived, with the line ending it may lack, and flush.
    pub fn write_to(&self, output: &mut impl Write) -> io::Result<()> {
        output.write_all(&self.line)?;
        if !self.line.ends_with(b"\n") {
            output.write_all(b"\n")?;
        }
        output.flush()
    }

    /// The answer's `status`, when the line is JSON with a string there.
    pub fn status(&self) -> Option<String> {
        let answer: Value = serde_json::from_slice(&self.line).ok()?;
        Some(answer["status"].as_str()?.to_owned())
    }
}

/// Why a request has no answer. `E` is the caller's check's own error.
#[derive(Debug)]
#[non_exhaustive]
pub enum Unanswered<E> {
    /// No game listens there: there is no socket at the path, or nothing answers on the
    /// one there.
    NotListening(io::Error),
    /// The socket could not be reached for another reason.
    Unreached(io::Error),
    /// The request could not be sent.
    Unsent(io::Error),
    /// The caller's check ended the wait, with its error.
    Stopped(E),
    /// Reading the answer failed.
    Unreadable(io::Error),
    /// The game closed the connection without answering.
    Closed,
}

impl<E: fmt::Display> fmt::Display for Unanswered<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotListening(error) => write!(f, "no game listens there: {error}"),
            Self::Unreached(error) => write!(f, "cannot reach the game: {error}"),
            Self::Unsent(error) => write!(f, "cannot send the request: {error}"),
            Self::Stopped(error) => error.fmt(f),
            Self::Unreadable(error) => write!(f, "cannot read the answer: {error}"),
            Self::Closed => f.write_str("the game closed the connection without answering"),
        }
    }
}

/// Each cause's text is this error's, so only the cause's own source is passed on. The
/// caller's own error is its own: as a source it is not named.
impl<E: fmt::Debug + fmt::Display> Error for Unanswered<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NotListening(error)
            | Self::Unreached(error)
            | Self::Unsent(error)
            | Self::Unreadable(error) => error.source(),
            Self::Stopped(_) | Self::Closed => None,
        }
    }
}
