//! Unix socket client for the held agent seat, extracted from Arkham's tools.
//! No renderer is required. A [`Connection`] answers the two questions its callers
//! ask: "send these requests and tell me how it went" ([`Connection::send`]) and
//! "send one request and give me its answer" ([`Connection::request`]). Both wake
//! regularly while the seat says nothing, to run the caller's check of its own
//! deadline or interrupt. Consumers own launch diagnostics, signal handling,
//! deadlines, wording and exit statuses. Response lines have no size limit, and
//! nothing bounds a write; a final line without a newline is accepted as it arrived.
//! The protocol's statuses and requests are `unfolded-seat-protocol`'s.
//!
//! [`requests`], [`connect`], [`exchange`] and [`read_line`] are the earlier calls,
//! kept while games move: with them the caller sets the socket's read timeout and
//! compares the count of answers with its requests.
#![cfg(unix)]

use anyhow::{Context, Result, bail};
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

/// Why a single request has no reply.
#[derive(Debug)]
pub enum RequestError {
    /// The caller's check ended the wait, with its error.
    Stopped(anyhow::Error),
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

impl fmt::Display for RequestError {
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

impl Error for RequestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Stopped(error) => error.source(),
            Self::Unreadable(error) => Some(error),
            Self::NotJson { error, .. } => Some(error),
            Self::Unrecognised { .. } => None,
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
    pub fn send(
        self,
        requests: &[String],
        output: &mut impl Write,
        check: impl FnMut() -> Result<()>,
    ) -> Result<Outcome> {
        self.stream.set_read_timeout(Some(WAKE))?;
        let (answered, failed) = exchange(&self.stream, requests, output, check)?;
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
    pub fn request(
        self,
        request: &Request,
        mut check: impl FnMut() -> Result<()>,
    ) -> Result<Reply, RequestError> {
        self.stream
            .set_read_timeout(Some(WAKE))
            .map_err(RequestError::Unreadable)?;
        let request = request.to_string();
        if (&self.stream)
            .write_all(format!("{request}\n").as_bytes())
            .is_err()
        {
            return Ok(Reply::Ended(Ended::Closed));
        }
        let _ = self.stream.shutdown(Shutdown::Write);
        let mut line = Vec::new();
        match wait_for_line(&mut BufReader::new(&self.stream), &mut line, &mut check) {
            Ok(true) => {}
            Ok(false) => return Ok(Reply::Ended(Ended::Unanswered)),
            Err(Waited::Stopped(error)) => return Err(RequestError::Stopped(error)),
            Err(Waited::Unreadable(error)) if error.kind() == ErrorKind::ConnectionReset => {
                return Ok(Reply::Ended(Ended::Reset(error)));
            }
            Err(Waited::Unreadable(error)) => return Err(RequestError::Unreadable(error)),
        }
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
}

/// Split argument strings into lines, or read standard-input lines when no
/// arguments were supplied. Blank lines are not requests; preserve other bytes.
pub fn requests(given: &[String], input: &mut impl BufRead) -> Result<Vec<String>> {
    let lines: Vec<String> = if given.is_empty() {
        input
            .lines()
            .collect::<std::io::Result<_>>()
            .context("cannot read requests from standard input")?
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
        bail!("no requests: give them as arguments or as standard-input lines");
    }
    Ok(requests)
}

/// Connect without replacing the OS error kind; the caller supplies launch help
/// or distinguishes a missing/refused socket from other failures.
pub fn connect(socket: &Path) -> std::io::Result<UnixStream> {
    UnixStream::connect(socket)
}

/// Send every request and half-close the connection's input while concurrently
/// copying responses unchanged to `output`, flushing each one. Returns the answer
/// count and whether a status was refused, missed, skipped, absent or malformed.
/// The caller compares the count with its requests and decides the exit status.
///
/// Set a read timeout on the stream to invoke `on_timeout` periodically; it may
/// check consumer-owned interrupts/deadlines. Without one, reads may wait forever.
/// As in the original client, a write failure only means fewer answers. This
/// does not impose a write timeout or interrupt a writer blocked on a large send.
pub fn exchange(
    stream: &UnixStream,
    requests: &[String],
    output: &mut impl Write,
    mut on_timeout: impl FnMut() -> Result<()>,
) -> Result<(usize, bool)> {
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
            if !read_line(&mut reader, &mut line, &mut on_timeout)? {
                return Ok((answered, failed));
            }
            output.write_all(&line)?;
            output.flush()?;
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

/// Read one line, retaining partial bytes across socket timeouts. Invoke the
/// consumer's callback on each timeout. False means EOF with no bytes remaining.
/// The caller clears `line` before reading its next response.
pub fn read_line(
    reader: &mut impl BufRead,
    line: &mut Vec<u8>,
    mut on_timeout: impl FnMut() -> Result<()>,
) -> Result<bool> {
    wait_for_line(reader, line, &mut on_timeout).or_else(|waited| match waited {
        Waited::Stopped(error) => Err(error),
        Waited::Unreadable(error) => Err(error).context("cannot read the seat's response"),
    })
}

/// Why a wait for a line ended without one.
enum Waited {
    /// The caller's check ended it.
    Stopped(anyhow::Error),
    /// The read failed other than by a timeout.
    Unreadable(io::Error),
}

/// Read one line into `line`, keeping partial bytes across read timeouts and running
/// `check` on each. False means the end of the connection with no bytes remaining.
fn wait_for_line(
    reader: &mut impl BufRead,
    line: &mut Vec<u8>,
    check: &mut impl FnMut() -> Result<()>,
) -> Result<bool, Waited> {
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
