//! Unix socket client for the held agent seat, extracted from Arkham's tools.
//! No renderer is required. Consumers own launch diagnostics, signal handling,
//! deadlines and the policy for incomplete exchanges. Response lines have no size
//! limit; a final line without a newline is returned and parsed as it arrived.
#![cfg(unix)]

use anyhow::{Context, Result, bail};
use std::{
    io::{BufRead, BufReader, ErrorKind, Write},
    net::Shutdown,
    os::unix::net::UnixStream,
    path::Path,
    thread,
};

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
            let status = serde_json::from_slice::<serde_json::Value>(&line)
                .ok()
                .and_then(|response| response["status"].as_str().map(str::to_owned));
            failed |= matches!(
                status.as_deref(),
                None | Some("refused" | "missed" | "skipped")
            );
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
    loop {
        match reader.read_until(b'\n', line) {
            Ok(_) => return Ok(!line.is_empty()),
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                on_timeout()?;
            }
            Err(error) => return Err(error).context("cannot read the seat's response"),
        }
    }
}
