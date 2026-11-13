//! The held session: the seat's protocol on a local Unix-domain socket, so one
//! session outlives the shell command that started it and keeps the table's camera,
//! menus, dialogs and readings between commands.
//!
//! The launching process claims the socket and starts the seat again as a fresh process
//! (never a fork of a running app) in its own session, handing it the listening socket
//! as standard input. The launching shell's standard input, output and error are not
//! passed on, so a harness waiting for their end is not held by the seat. The launcher
//! waits for admission and prints its result; the seat then answers one connection at a
//! time, in arrival order, through the same serving loop as batch form, until `quit`,
//! the table's own exit or its idle end.
//!
//! Linux only: the launcher starts the running executable again through
//! `/proc/self/exe`, and a socket path holds at most 107 bytes. The held seat's
//! standard error goes to a log beside the socket, at its path with `.log` added.

use super::{ConnectionEnd, Game, Seat, Status, emit};
use bevy::prelude::App;
use rustix::event::{PollFd, PollFlags, Timespec, poll};
use serde::Deserialize;
use serde_json::Value;
use std::{
    fs::{self, File, Metadata, OpenOptions, Permissions},
    io::{self, BufRead, BufReader, ErrorKind, Read},
    os::{
        fd::{AsFd, OwnedFd},
        unix::{
            fs::{FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt},
            net::{UnixListener, UnixStream},
            process::CommandExt,
        },
    },
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

/// A Unix-domain socket path is at most this many bytes.
const PATH_LIMIT: usize = 107;

/// Whether this process is the seat a launcher started. `marker` names the
/// environment variable the launcher sets on that process, which serves instead of
/// launching. Each game names its own.
pub fn detached(marker: &str) -> bool {
    std::env::var_os(marker).is_some()
}

/// The held seat's standard error: the socket path with `.log` appended.
fn log_path(socket: &Path) -> PathBuf {
    let mut log = socket.as_os_str().to_owned();
    log.push(".log");
    log.into()
}

/// Which file a path names: its device and inode. A path replaced since is another file.
#[derive(Clone, Copy, PartialEq, Eq)]
struct FileIdentity {
    device: u64,
    inode: u64,
}

impl FileIdentity {
    fn of(file: &Metadata) -> Self {
        Self {
            device: file.dev(),
            inode: file.ino(),
        }
    }
}

/// A socket path one seat listens on, accessible only to its owner. The path is removed
/// when the listener ends, unless something else has replaced it there.
pub struct Listener {
    listener: UnixListener,
    path: PathBuf,
    /// The socket file while this listener owns the path.
    file: Option<FileIdentity>,
}

impl Listener {
    /// `listener`, owning the socket file `socket` found at `path`.
    fn owning(listener: UnixListener, path: &Path, socket: &Metadata) -> Self {
        Self {
            listener,
            path: path.to_owned(),
            file: Some(FileIdentity::of(socket)),
        }
    }

    /// Listen on `path`. The socket is made in a private directory and linked into place
    /// only once owner-only, so the path never exists with wider access. A socket where
    /// nothing answers was left by a crashed seat and is replaced. A socket where a seat
    /// answers, and any other existing path, is refused and left unchanged.
    pub fn claim(path: &Path) -> Result<Self, String> {
        let cannot =
            |error: &dyn std::fmt::Display| format!("Cannot listen on {}: {error}", path.display());
        if path.as_os_str().len() > PATH_LIMIT {
            return Err(cannot(&format_args!(
                "a local socket path holds at most {PATH_LIMIT} bytes; choose a shorter one"
            )));
        }
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .ok_or_else(|| cannot(&"the path names no directory"))?;
        let private = tempfile::Builder::new()
            .prefix(".s")
            .permissions(Permissions::from_mode(0o700))
            .tempdir_in(parent)
            .map_err(|error| cannot(&error))?;
        let made = private.path().join("s");
        let listener = UnixListener::bind(&made).map_err(|error| cannot(&error))?;
        fs::set_permissions(&made, Permissions::from_mode(0o600))
            .map_err(|error| cannot(&error))?;
        // Seats claiming paths in one directory take turns, so the leftover found here is
        // the one replaced, never a seat that started meanwhile. A seat removes its own
        // socket while still listening, so a socket where nothing answers is left over.
        let _turn = File::open(parent)
            .and_then(|directory| directory.lock().map(|()| directory))
            .map_err(|error| cannot(&format_args!("cannot lock its directory: {error}")))?;
        let replaced = match fs::symlink_metadata(path) {
            Err(error) if error.kind() == ErrorKind::NotFound => false,
            Err(error) => return Err(cannot(&error)),
            Ok(file) if !file.file_type().is_socket() => {
                return Err(cannot(
                    &"the path exists and is not a socket; it was left unchanged",
                ));
            }
            Ok(_) => match UnixStream::connect(path) {
                Ok(_) => {
                    return Err(cannot(
                        &"an agent seat already answers there and was left untouched; quit that seat or choose another path",
                    ));
                }
                Err(error) if error.kind() == ErrorKind::ConnectionRefused => {
                    match fs::remove_file(path) {
                        Err(error) if error.kind() != ErrorKind::NotFound => {
                            return Err(cannot(&format_args!(
                                "cannot remove the socket a crashed seat left there: {error}"
                            )));
                        }
                        _ => true,
                    }
                }
                Err(error) => {
                    return Err(cannot(&format_args!(
                        "cannot tell whether a seat answers on the socket there ({error}); it was left unchanged"
                    )));
                }
            },
        };
        fs::hard_link(&made, path).map_err(|error| match error.kind() {
            ErrorKind::AlreadyExists => cannot(&"the path already exists; it was left unchanged"),
            _ => cannot(&error),
        })?;
        if replaced {
            eprintln!(
                "Replaced {}: a socket left by a seat that no longer answers.",
                path.display()
            );
        }
        let file = fs::symlink_metadata(path).map_err(|error| cannot(&error))?;
        Ok(Self::owning(listener, path, &file))
    }

    /// The listener a launcher handed this seat as its standard input, for `path`.
    fn inherited(path: &Path, marker: &str) -> Result<Self, String> {
        let not_handed = |error: &dyn std::fmt::Display| {
            format!(
                "The seat was not handed a listening socket for {} ({error}); start it without {marker}.",
                path.display()
            )
        };
        let listener = UnixListener::from(
            std::io::stdin()
                .as_fd()
                .try_clone_to_owned()
                .map_err(|error| not_handed(&error))?,
        );
        listener.local_addr().map_err(|error| not_handed(&error))?;
        let file = fs::symlink_metadata(path).map_err(|error| not_handed(&error))?;
        if !file.file_type().is_socket() {
            return Err(not_handed(&"the path is not a socket"));
        }
        Ok(Self::owning(listener, path, &file))
    }

    /// The listening socket, duplicated as the standard input of the seat process that
    /// will serve it.
    fn as_stdin(&self) -> Result<Stdio, String> {
        let socket = self
            .listener
            .try_clone()
            .map_err(|error| format!("Cannot hand over the socket: {error}"))?;
        Ok(Stdio::from(OwnedFd::from(socket)))
    }

    /// Answer connections one at a time, in arrival order, until `quit`, the table's own
    /// exit, or `idle` without any request, which ends the session as `quit` does. A
    /// client closing its connection does not end the session; an open connection that
    /// sends nothing does not keep it. The socket is removed before the last connection
    /// closes.
    pub fn serve<G: Game>(
        mut self,
        seat: &mut Seat<G>,
        app: &mut App,
        idle: Duration,
    ) -> Result<(), String> {
        eprintln!("The agent seat is listening on {}.", self.path.display());
        let stopped = |error: io::Error| {
            format!(
                "The seat stopped accepting connections on {}: {error}",
                self.path.display()
            )
        };
        let mut clock = IdleClock::new(idle);
        let last = loop {
            if !clock.wait(&self.listener).map_err(stopped)? {
                break None;
            }
            let stream = match self.listener.accept() {
                Ok((stream, _)) => stream,
                Err(error) if error.kind() == ErrorKind::ConnectionAborted => continue,
                Err(error) => return Err(stopped(error)),
            };
            // A client that stops reading cannot hold the seat past its idle end either.
            let _ = stream.set_write_timeout(Some(idle));
            let requests = Requests {
                stream: &stream,
                clock: &mut clock,
                heard: false,
            };
            match seat.converse(app, BufReader::new(requests), &stream) {
                Ok(ConnectionEnd::SessionEnded) => break Some(stream),
                Ok(ConnectionEnd::InputEnded) => {}
                Err(_) if clock.over => break Some(stream),
                Err(error) => eprintln!("A connection ended before all its responses: {error}"),
            }
        };
        if clock.over {
            eprintln!(
                "The agent seat is ending its session as quit does: no request arrived for {}.",
                period_in_words(idle)
            );
        }
        self.remove();
        drop(last);
        eprintln!("The agent seat's session ended.");
        seat.outcome(app)
    }

    /// The process started with this listener as standard input now owns the path.
    fn hand_over(mut self) {
        self.file = None;
    }

    fn remove(&mut self) {
        let Some(owned) = self.file.take() else {
            return;
        };
        if fs::symlink_metadata(&self.path).is_ok_and(|file| FileIdentity::of(&file) == owned) {
            let _ = fs::remove_file(&self.path);
        }
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        self.remove();
    }
}

/// The idle clock: the session ends once its period passes while the seat waits for a
/// request. Time spent answering does not count.
struct IdleClock {
    period: Duration,
    end: Instant,
    /// The period passed: the session is over.
    over: bool,
}

impl IdleClock {
    fn new(period: Duration) -> Self {
        Self {
            period,
            end: Instant::now() + period,
            over: false,
        }
    }

    /// The seat has answered what arrived and waits again.
    fn restart(&mut self) {
        self.end = Instant::now() + self.period;
    }

    /// Wait until `source` has something to read: true when it has, false once the
    /// period has passed first.
    fn wait(&mut self, source: &impl AsFd) -> io::Result<bool> {
        loop {
            let left = self.end.saturating_duration_since(Instant::now());
            if left.is_zero() {
                self.over = true;
                return Ok(false);
            }
            let timeout = Timespec::try_from(left).map_err(io::Error::other)?;
            match poll(&mut [PollFd::new(source, PollFlags::IN)], Some(&timeout)) {
                Ok(0) | Err(rustix::io::Errno::INTR) => {}
                Ok(_) => return Ok(true),
                Err(error) => return Err(error.into()),
            }
        }
    }
}

/// One connection's input, waiting for each request only until the idle end.
struct Requests<'a> {
    stream: &'a UnixStream,
    clock: &'a mut IdleClock,
    /// The last read brought something, so the seat reads again only once it has
    /// answered it.
    heard: bool,
}

impl Read for Requests<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.heard {
            self.clock.restart();
        }
        if !self.clock.wait(self.stream)? {
            return Err(io::Error::new(
                ErrorKind::TimedOut,
                "no request arrived within the idle period",
            ));
        }
        let mut stream = self.stream;
        let read = stream.read(buffer)?;
        self.heard = read > 0;
        Ok(read)
    }
}

/// The idle period as the log states it.
fn period_in_words(period: Duration) -> String {
    let minutes = period.as_secs() / 60;
    if minutes == 0 || period != Duration::from_secs(minutes * 60) {
        format!("{period:?}")
    } else if minutes == 1 {
        "1 minute".to_owned()
    } else {
        format!("{minutes} minutes")
    }
}

/// Start the seat held on `socket`, marked by the environment variable `marker`
/// ([`detached`]), and wait for its admission. The seat it starts says how its
/// start went in its first line of standard output: its `ready` response
/// ([`announce`]), or one `refused` response ([`super::refuse`]). On success print its
/// `ready` response unchanged and leave it serving; otherwise return why it could not
/// start, with no socket left and no seat running.
pub fn launch(socket: &Path, marker: &str) -> Result<(), String> {
    let listener = Listener::claim(socket)?;
    let log = log_path(socket);
    let mut command = Command::new("/proc/self/exe");
    if let Some(name) = std::env::args_os().next() {
        command.arg0(name);
    }
    // The same executable as this process, even if its file has since been replaced.
    let mut seat = command
        .args(std::env::args_os().skip(1))
        .env(marker, "1")
        .stdin(listener.as_stdin()?)
        .stdout(Stdio::piped())
        .stderr(create_log(&log)?)
        .spawn()
        .map_err(|error| format!("Cannot start the seat: {error}"))?;
    let mut line = String::new();
    let read = BufReader::new(seat.stdout.take().expect("piped")).read_line(&mut line);
    let startup: Option<Startup> = serde_json::from_str(&line).ok();
    if read.is_ok()
        && let Some(Startup {
            status: Status::Ready,
            ..
        }) = startup
    {
        print!("{line}");
        listener.hand_over();
        return Ok(());
    }
    // The seat did not start and is ending; the socket goes with this listener.
    let ended = seat.wait();
    drop(listener);
    match startup.and_then(|refused| refused.error) {
        Some(error) => Err(error),
        None => Err(format!(
            "The seat ended before admission ({}). Its diagnostics are in {}.",
            ended.map_or_else(|error| error.to_string(), |status| status.to_string()),
            log.display()
        )),
    }
}

/// The first line a starting seat prints: `ready`, or one `refused` response.
#[derive(Deserialize)]
struct Startup {
    status: Status,
    #[serde(default)]
    error: Option<String>,
}

/// Create the log afresh, accessible only to its owner. A log an earlier start left at
/// the path is removed first.
fn create_log(log: &Path) -> Result<File, String> {
    let cannot =
        |error: std::io::Error| format!("Cannot create the log {}: {error}", log.display());
    match fs::remove_file(log) {
        Err(error) if error.kind() != ErrorKind::NotFound => return Err(cannot(error)),
        _ => {}
    }
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(log)
        .map_err(cannot)
}

/// Become the seat a launcher started: leave the launching shell's session and process
/// group, and take the listener it handed over.
pub fn detach(socket: &Path, marker: &str) -> Result<Listener, String> {
    rustix::process::setsid()
        .map_err(|error| format!("The seat cannot leave the launching shell's session: {error}"))?;
    Listener::inherited(socket, marker)
}

/// Tell the launcher the session is ready. Standard output carries nothing else.
pub fn announce(ready: &Value) -> Result<(), String> {
    emit(&mut std::io::stdout().lock(), ready)
        .map_err(|error| format!("The launcher did not receive the ready response: {error}"))
}
