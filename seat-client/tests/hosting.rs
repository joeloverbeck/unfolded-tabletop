//! Seam: a held seat's table kept running, against a stand-in seat on a real socket.
//! Expectations come from Arkham's `seat-wait` (Arkham #218), whose loop moved here: one
//! connection per request, an observe and then settles, what runs in the pause between
//! them, and how each ending shows.

use serde_json::{Value, json};
use std::{
    cell::Cell,
    convert::Infallible,
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixListener,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use unfolded_seat_client::{
    Ended,
    hosting::{Gone, Kept, Next, Unkept, keep_running},
};

/// A stand-in seat: each connection's one request is answered with the next line, and
/// after the last its socket is removed. An empty line closes the connection unanswered.
struct Seat {
    socket: PathBuf,
    received: thread::JoinHandle<Vec<Value>>,
    _directory: tempfile::TempDir,
}

fn seat(answers: &'static [&'static str]) -> Seat {
    let directory = tempfile::tempdir().unwrap();
    let socket = directory.path().join("seat.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    let path = socket.clone();
    let received = thread::spawn(move || {
        let received = answers
            .iter()
            .map(|answer| {
                let (stream, _) = listener.accept().unwrap();
                let mut request = String::new();
                BufReader::new(&stream).read_line(&mut request).unwrap();
                if !answer.is_empty() {
                    writeln!(&stream, "{answer}").unwrap();
                }
                serde_json::from_str(&request).unwrap()
            })
            .collect();
        drop(listener);
        std::fs::remove_file(&path).unwrap();
        received
    });
    Seat {
        socket,
        received,
        _directory: directory,
    }
}

/// A caller's pause: it sleeps.
fn paused<E>(duration: Duration) -> Result<(), E> {
    thread::sleep(duration);
    Ok(())
}

/// Kept running for at most a minute, with no check, and stopped at revision `stop`.
/// What was seen: each answer's revision.
fn kept(socket: &Path, stop: u64) -> (Result<Kept<u64>, Unkept<Infallible>>, Vec<u64>) {
    let mut seen = Vec::new();
    let until = Instant::now() + Duration::from_secs(60);
    let kept = keep_running(
        socket,
        until,
        || Ok(()),
        paused,
        |answer| {
            let revision = answer["revision"].as_u64().unwrap();
            seen.push(revision);
            Ok(if revision == stop {
                Next::Stop(revision)
            } else {
                Next::Go
            })
        },
    );
    (kept, seen)
}

#[test]
fn the_table_is_observed_once_then_settled_until_the_game_stops_it() {
    let seat = seat(&[
        r#"{"status":"observed","revision":3}"#,
        r#"{"status":"settled","revision":3}"#,
        r#"{"status":"unsettled","revision":4}"#,
        r#"{"status":"settled","revision":5}"#,
    ]);
    let (kept, seen) = kept(&seat.socket, 5);
    assert!(matches!(kept, Ok(Kept::Stopped(5))), "{kept:?}");
    assert_eq!(seen, [3, 3, 4, 5]);
    let settle = json!({"op": "settle", "steps": 120});
    assert_eq!(
        seat.received.join().unwrap(),
        [
            json!({"op": "observe"}),
            settle.clone(),
            settle.clone(),
            settle
        ]
    );
}

#[test]
fn a_resting_table_is_settled_twice_a_second_and_the_time_ends_it() {
    let directory = tempfile::tempdir().unwrap();
    let socket = directory.path().join("seat.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    let asked = Arc::new(AtomicUsize::new(0));
    let counted = asked.clone();
    // It answers for as long as it is asked, so it ends with the test's process.
    thread::spawn(move || {
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let mut request = String::new();
            BufReader::new(&stream).read_line(&mut request).unwrap();
            counted.fetch_add(1, Ordering::SeqCst);
            let _ = writeln!(&stream, r#"{{"status":"settled"}}"#);
        }
    });
    let began = Instant::now();
    let mut pauses = Vec::new();
    let kept = keep_running(
        &socket,
        began + Duration::from_millis(1_200),
        || Ok::<(), Infallible>(()),
        |pause| {
            pauses.push(pause);
            paused(pause)
        },
        |_| Ok(Next::<()>::Rest),
    );
    let took = began.elapsed();
    assert!(matches!(kept, Ok(Kept::TimedOut)), "{kept:?}");
    assert!(took >= Duration::from_millis(1_200), "{took:?}");
    assert!(took < Duration::from_secs(10), "{took:?}");
    // At 0, 0.5 and 1 second, and fewer on a slow host.
    let asked = asked.load(Ordering::SeqCst);
    assert!((2..=3).contains(&asked), "asked {asked} times");
    // A pause of at most half a second after each answer, the last cut at the time.
    let half = Duration::from_millis(500);
    assert_eq!(pauses.len(), asked);
    assert!(pauses.iter().all(|pause| *pause <= half), "{pauses:?}");
    assert!(*pauses.last().unwrap() < half, "{pauses:?}");
}

#[test]
fn a_seat_that_goes_away_is_told_by_how_it_went() {
    for last in ["", r#"{"status":"bye"}"#, r#"{"status":"exited"}"#] {
        let directory = tempfile::tempdir().unwrap();
        let socket = directory.path().join("seat.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        let answering = thread::spawn(move || {
            for answer in [r#"{"status":"observed","revision":3}"#, last] {
                let (stream, _) = listener.accept().unwrap();
                let mut request = String::new();
                BufReader::new(&stream).read_line(&mut request).unwrap();
                if !answer.is_empty() {
                    writeln!(&stream, "{answer}").unwrap();
                }
            }
        });
        let (kept, seen) = kept(&socket, 9);
        answering.join().unwrap();
        assert_eq!(seen, [3]);
        let Ok(Kept::Gone(Gone::Ended(ended))) = kept else {
            panic!("{kept:?}");
        };
        assert_eq!(matches!(ended, Ended::Unanswered), last.is_empty());
        assert_eq!(matches!(ended, Ended::Session(_)), !last.is_empty());
    }

    // Nothing listens, from the start or once it is gone.
    let seat = seat(&[r#"{"status":"observed","revision":3}"#]);
    let socket = seat.socket.clone();
    let (gone, seen) = kept(&socket, 9);
    assert!(
        matches!(gone, Ok(Kept::Gone(Gone::NotListening(_)))),
        "{gone:?}"
    );
    assert_eq!(seen, [3]);
    seat.received.join().unwrap();
    let (gone, seen) = kept(&socket, 9);
    assert!(
        matches!(gone, Ok(Kept::Gone(Gone::NotListening(_)))),
        "{gone:?}"
    );
    assert!(seen.is_empty());
}

#[test]
fn an_answer_that_is_neither_an_observation_nor_a_settle_fails() {
    let seat = seat(&[r#"{"status":"refused","error":"Malformed request."}"#]);
    let (kept, seen) = kept(&seat.socket, 9);
    let Err(unkept @ Unkept::Unexpected { .. }) = kept else {
        panic!("{kept:?}");
    };
    assert_eq!(
        unkept.to_string(),
        r#"the seat answered {"op":"observe"} with: {"status":"refused","error":"Malformed request."}"#
    );
    assert!(seen.is_empty(), "the game sees only a table's answers");
    seat.received.join().unwrap();
}

#[test]
fn the_callers_check_pause_and_reading_of_an_answer_end_it_with_their_own_errors() {
    let resting = seat(&[r#"{"status":"observed"}"#]);
    let until = Instant::now() + Duration::from_secs(60);
    let interrupted = keep_running(
        &resting.socket,
        until,
        || Ok(()),
        |_| Err("interrupted"),
        |_| Ok(Next::<()>::Rest),
    );
    assert!(
        matches!(interrupted, Err(Unkept::Stopped("interrupted"))),
        "{interrupted:?}"
    );
    resting.received.join().unwrap();

    let unread = seat(&[r#"{"status":"observed"}"#]);
    let failed = keep_running(
        &unread.socket,
        until,
        || Ok(()),
        paused,
        |_| Err::<Next<()>, _>("no revision"),
    );
    assert!(
        matches!(failed, Err(Unkept::Stopped("no revision"))),
        "{failed:?}"
    );
    unread.received.join().unwrap();

    // A seat that stopped answering: the check ends the request itself.
    let directory = tempfile::tempdir().unwrap();
    let socket = directory.path().join("seat.sock");
    let _silent = UnixListener::bind(&socket).unwrap();
    let stopped = keep_running(
        &socket,
        until,
        || Err("too long"),
        paused,
        |_| Ok(Next::<()>::Rest),
    );
    assert!(matches!(stopped, Err(Unkept::Request(_))), "{stopped:?}");
}

#[test]
fn an_answer_that_came_late_is_not_a_seat_that_did_not_answer() {
    // The caller gives up on a request past the time. Its seat answered, and the moment
    // to give up comes in the pause that follows.
    let seat = seat(&[r#"{"status":"observed"}"#]);
    let given_up = Cell::new(false);
    let kept = keep_running(
        &seat.socket,
        Instant::now() + Duration::from_millis(100),
        || {
            if given_up.get() {
                return Err("it did not answer in time");
            }
            Ok(())
        },
        |pause| {
            given_up.set(true);
            paused(pause)
        },
        |_| Ok(Next::<()>::Rest),
    );
    assert!(matches!(kept, Ok(Kept::TimedOut)), "{kept:?}");
    seat.received.join().unwrap();
}
