//! Unix socket client for the held agent seat, extracted from Arkham's tools.
//! No renderer is required. A [`Connection`] answers the questions its callers ask:
//! "send these requests and tell me how it went" ([`Connection::send`]), "send one
//! request and give me its answer" ([`Connection::request`]) and, for a protocol that is
//! not the seat's, "send one line and give me the line back" ([`Connection::ask`]). Each
//! wakes regularly while nothing arrives, to run the caller's check of its own deadline
//! or interrupt. Consumers own launch diagnostics, signal handling,
//! deadlines, wording and exit statuses: a caller's check returns its own error, which
//! comes back as it is, and every other failure is a cause to match ([`Unsent`],
//! [`RequestError`], [`AskError`], [`Unrequested`], [`Unreached`]). Response lines have no size limit, and
//! nothing bounds a write; a final line without a newline is accepted as it arrived.
//! The protocol's statuses and requests are `unfolded-seat-protocol`'s.
//!
//! [`requests`] splits arguments or standard-input lines into request lines for
//! [`Connection::send`]. [`hosting`] keeps a held seat's table running while others play
//! at it.
#![cfg(unix)]

pub mod hosting;

use serde_json::Value;
use std::{
    error::Error,
    fmt,
    io::{self, BufRead, BufReader, ErrorKind, Write},
    net::Shutdown,
    os::unix::net::UnixStream,
    path::Path,
    thread,
    time::Duration,
};
use unfolded_seat_protocol::{Request, Status};

/// How often a call waiting on the seat wakes to run its caller's check.
const WAKE: Duration = Duration::from_millis(200);

/// One connection to a held seat. Each call sends its requests, ends the connection's
/// input and reads the answers, so a connection serves one call.
pub struct Connection {
    stream: UnixStream,
}

/// Why no connection was made.
#[derive(Debug)]
pub enum Unreached {
    /// No seat listens there: there is no socket at the path, or nothing answers on
    /// the one there, as a seat that crashed or quit leaves it.
    NotListening(io::Error),
    /// Any other failure, such as a path too long for a socket or one through a file.
    Failed(io::Error),
}

impl fmt::Display for Unreached {
    /// The operating system's words.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotListening(error) | Self::Failed(error) => error.fmt(formatter),
        }
    }
}

impl Error for Unreached {}

/// How a whole exchange went.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Every request was answered, and no answer failed.
    Answered,
    /// Every request was answered, and at least one answer was refused, missed or
    /// skipped, or carried no status: it was not JSON, had none or had one that is no
    /// string. A string status the protocol does not name is no failure.
    Failed,
    /// The seat answered fewer requests than were sent: its session ended before the
    /// rest. This is said whether or not an answer it gave failed.
    Short { answered: usize },
}

/// What a single request came to.
#[derive(Debug)]
pub enum Reply {
    /// The seat answered, and its session goes on. The caller decides what a refused,
    /// missed or skipped answer means.
    Answer(Answer),
    /// The session ended, and the request was not served.
    Ended(Ended),
}

/// One answer of the seat.
#[derive(Clone, Debug, PartialEq)]
pub struct Answer {
    pub status: Status,
    pub response: Value,
    /// The line as it arrived, without its line ending.
    pub line: String,
}

/// How a session ended at a single request.
#[derive(Debug)]
pub enum Ended {
    /// The seat closed the connection before the request could be sent.
    Closed,
    /// The seat reset the connection: it ended with the request unread.
    Reset(io::Error),
    /// The seat closed the connection without answering.
    Unanswered,
    /// The seat answered that its session ended: `bye` or `exited`.
    Session(Status),
}

impl fmt::Display for Ended {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Closed => formatter.write_str("it closed the connection"),
            Self::Reset(error) => error.fmt(formatter),
            Self::Unanswered => formatter.write_str("it closed the connection without answering"),
            Self::Session(status) => write!(formatter, "its session ended with {status}"),
        }
    }
}

/// Why a whole exchange ended before every answer was read. `E` is the caller's
/// check's own error.
#[derive(Debug)]
#[non_exhaustive]
pub enum Unsent<E> {
    /// The caller's check ended the exchange, with its error.
    Stopped(E),
    /// Reading an answer failed.
    Unreadable(io::Error),
    /// Copying an answer to the caller's output failed.
    Unwritten(io::Error),
    /// The connection could not be set to wake for the caller's check.
    Unwoken(io::Error),
}

impl<E: fmt::Display> fmt::Display for Unsent<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stopped(error) => error.fmt(formatter),
            Self::Unreadable(_) => formatter.write_str("cannot read the seat's response"),
            Self::Unwritten(error) | Self::Unwoken(error) => error.fmt(formatter),
        }
    }
}

/// The caller's own error is its own: as a source it is not named.
impl<E: fmt::Debug + fmt::Display> Error for Unsent<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Stopped(_) => None,
            // Its text does not say the read's error; the others' texts do.
            Self::Unreadable(error) => Some(error),
            Self::Unwritten(error) | Self::Unwoken(error) => error.source(),
        }
    }
}

/// Why a single request has no reply. `E` is the caller's check's own error.
#[derive(Debug)]
pub enum RequestError<E> {
    /// The caller's check ended the wait, with its error.
    Stopped(E),
    /// Reading the answer failed, other than by a reset connection.
    Unreadable(io::Error),
    /// The answer is not JSON: the line, and why it does not parse.
    NotJson {
        line: String,
        error: serde_json::Error,
    },
    /// The answer is JSON with no status the protocol names.
    Unrecognised { request: String, line: String },
}

impl<E: fmt::Display> fmt::Display for RequestError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stopped(error) => error.fmt(formatter),
            Self::Unreadable(_) => formatter.write_str("cannot read the seat's response"),
            Self::NotJson { line, .. } => {
                write!(formatter, "the seat's answer is not JSON: {line}")
            }
            Self::Unrecognised { request, line } => {
                write!(formatter, "the seat answered {request} with: {line}")
            }
        }
    }
}

/// The caller's own error is its own: as a source it is not named.
impl<E: fmt::Debug + fmt::Display> Error for RequestError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Stopped(_) => None,
            Self::Unreadable(error) => Some(error),
            Self::NotJson { error, .. } => Some(error),
            Self::Unrecognised { .. } => None,
        }
    }
}

/// Why a line asked with [`Connection::ask`] has no line back. `E` is the caller's check's
/// own error.
#[derive(Debug)]
pub enum AskError<E> {
    /// The line could not be sent.
    Unsent(io::Error),
    /// The caller's check ended the wait, with its error.
    Stopped(E),
    /// Reading the line back failed.
    Unreadable(io::Error),
}

impl<E: fmt::Display> fmt::Display for AskError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsent(error) | Self::Unreadable(error) => error.fmt(formatter),
            Self::Stopped(error) => error.fmt(formatter),
        }
    }
}

/// The caller's own error is its own: as a source it is not named.
impl<E: fmt::Debug + fmt::Display> Error for AskError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Unsent(error) | Self::Unreadable(error) => error.source(),
            Self::Stopped(_) => None,
        }
    }
}

impl From<UnixStream> for Connection {
    fn from(stream: UnixStream) -> Self {
        Self { stream }
    }
}

impl Connection {
    /// Connect to the seat held on `socket`.
    pub fn open(socket: &Path) -> Result<Self, Unreached> {
        UnixStream::connect(socket)
            .map(Self::from)
            .map_err(|error| match error.kind() {
                ErrorKind::NotFound | ErrorKind::ConnectionRefused => {
                    Unreached::NotListening(error)
                }
                _ => Unreached::Failed(error),
            })
    }

    /// Send every request line, end the connection's input and copy each answer
    /// unchanged to `output`, flushing each one; pass [`io::sink`] to keep none. While
    /// the seat says nothing, `check` runs about five times a second, and its error
    /// ends the exchange. A failure to send only means fewer answers.
    pub fn send<E>(
        self,
        requests: &[String],
        output: &mut impl Write,
        mut check: impl FnMut() -> Result<(), E>,
    ) -> Result<Outcome, Unsent<E>> {
        self.stream
            .set_read_timeout(Some(WAKE))
            .map_err(Unsent::Unwoken)?;
        let (answered, failed) = exchange_lines(&self.stream, requests, output, &mut check)?;
        Ok(if answered < requests.len() {
            Outcome::Short { answered }
        } else if failed {
            Outcome::Failed
        } else {
            Outcome::Answered
        })
    }

    /// Send one request, end the connection's input and read its answer. While the seat
    /// says nothing, `check` runs about five times a second, and its error ends the
    /// wait.
    pub fn request<E>(
        self,
        request: &Request,
        check: impl FnMut() -> Result<(), E>,
    ) -> Result<Reply, RequestError<E>> {
        let request = request.to_string();
        let line = match self.ask(&request, check) {
            Ok(Some(line)) => line,
            Ok(None) => return Ok(Reply::Ended(Ended::Unanswered)),
            Err(AskError::Unsent(_)) => return Ok(Reply::Ended(Ended::Closed)),
            Err(AskError::Stopped(error)) => return Err(RequestError::Stopped(error)),
            Err(AskError::Unreadable(error)) if error.kind() == ErrorKind::ConnectionReset => {
                return Ok(Reply::Ended(Ended::Reset(error)));
            }
            Err(AskError::Unreadable(error)) => return Err(RequestError::Unreadable(error)),
        };
        let text = String::from_utf8_lossy(&line).trim_end().to_owned();
        let response = match serde_json::from_slice::<Value>(&line) {
            Ok(response) => response,
            Err(error) => return Err(RequestError::NotJson { line: text, error }),
        };
        match Status::of(&response) {
            None => Err(RequestError::Unrecognised {
                request,
                line: text,
            }),
            Some(status) if status.ends_session() => Ok(Reply::Ended(Ended::Session(status))),
            Some(status) => Ok(Reply::Answer(Answer {
                status,
                response,
                line: text,
            })),
        }
    }

    /// Send one line of any protocol, given without its line ending, end the connection's
    /// input and read one line back, as it arrived; `None` when the connection closed
    /// without one. While nothing arrives, `check` runs about five times a second, and its
    /// error ends the wait. `unfolded-agent-players` asks a game this way.
    pub fn ask<E>(
        self,
        request: &str,
        mut check: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<Vec<u8>>, AskError<E>> {
        self.stream
            .set_read_timeout(Some(WAKE))
            .map_err(AskError::Unreadable)?;
        (&self.stream)
            .write_all(format!("{request}\n").as_bytes())
            .map_err(AskError::Unsent)?;
        let _ = self.stream.shutdown(Shutdown::Write);
        let mut line = Vec::new();
        match wait_for_line(&mut BufReader::new(&self.stream), &mut line, &mut check) {
            Ok(arrived) => Ok(arrived.then_some(line)),
            Err(Waited::Stopped(error)) => Err(AskError::Stopped(error)),
            Err(Waited::Unreadable(error)) => Err(AskError::Unreadable(error)),
        }
    }
}

/// Why [`requests`] found none to send.
#[derive(Debug)]
#[non_exhaustive]
pub enum Unrequested {
    /// Standard input could not be read.
    Unreadable(io::Error),
    /// Every line given was blank.
    Empty,
}

impl fmt::Display for Unrequested {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Unreadable(_) => "cannot read requests from standard input",
            Self::Empty => "no requests: give them as arguments or as standard-input lines",
        })
    }
}

impl Error for Unrequested {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Unreadable(error) => Some(error),
            Self::Empty => None,
        }
    }
}

/// Split argument strings into lines, or read standard-input lines when no
/// arguments were supplied. Blank lines are not requests; preserve other bytes.
pub fn requests(given: &[String], input: &mut impl BufRead) -> Result<Vec<String>, Unrequested> {
    let lines: Vec<String> = if given.is_empty() {
        input
            .lines()
            .collect::<std::io::Result<_>>()
            .map_err(Unrequested::Unreadable)?
    } else {
        given
            .iter()
            .flat_map(|request| request.lines().map(str::to_owned))
            .collect()
    };
    let requests: Vec<String> = lines
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .collect();
    if requests.is_empty() {
        return Err(Unrequested::Empty);
    }
    Ok(requests)
}

fn exchange_lines<E>(
    stream: &UnixStream,
    requests: &[String],
    output: &mut impl Write,
    check: &mut impl FnMut() -> Result<(), E>,
) -> Result<(usize, bool), Unsent<E>> {
    thread::scope(|scope| {
        scope.spawn(|| {
            let mut writer = stream;
            for request in requests {
                if writer.write_all(format!("{request}\n").as_bytes()).is_err() {
                    return;
                }
            }
            let _ = stream.shutdown(Shutdown::Write);
        });
        let mut reader = BufReader::new(stream);
        let (mut answered, mut failed) = (0, false);
        let mut line = Vec::new();
        loop {
            line.clear();
            match wait_for_line(&mut reader, &mut line, check) {
                Ok(true) => {}
                Ok(false) => return Ok((answered, failed)),
                Err(Waited::Stopped(error)) => return Err(Unsent::Stopped(error)),
                Err(Waited::Unreadable(error)) => return Err(Unsent::Unreadable(error)),
            }
            output
                .write_all(&line)
                .and_then(|()| output.flush())
                .map_err(Unsent::Unwritten)?;
            answered += 1;
            // No status, or one that is no string, fails; so does a failed one the
            // protocol names. A string it does not name is no failure.
            failed |= serde_json::from_slice::<Value>(&line)
                .ok()
                .and_then(|response| response["status"].as_str().map(Status::named))
                .is_none_or(|status| status.is_some_and(Status::failed));
        }
    })
}

/// Why a wait for a line ended without one.
enum Waited<E> {
    /// The caller's check ended it.
    Stopped(E),
    /// The read failed other than by a timeout.
    Unreadable(io::Error),
}

/// Read one line into `line`, keeping partial bytes across read timeouts and running
/// `check` on each. False means the end of the connection with no bytes remaining.
fn wait_for_line<E>(
    reader: &mut impl BufRead,
    line: &mut Vec<u8>,
    check: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, Waited<E>> {
    loop {
        match reader.read_until(b'\n', line) {
            Ok(_) => return Ok(!line.is_empty()),
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                check().map_err(Waited::Stopped)?;
            }
            Err(error) => return Err(Waited::Unreadable(error)),
        }
    }
}
