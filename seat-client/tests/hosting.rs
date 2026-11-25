//! Seam: a held seat's table kept running, against a stand-in seat on a real socket.
//! Expectations come from Arkham's `seat-wait` (Arkham #218), whose exchange moved here:
//! one connection per request, an observe and then settles, and how each ending shows.

use serde_json::{Value, json};
use std::{
    convert::Infallible,
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixListener,
    path::{Path, PathBuf},
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

/// Kept running for at most a minute, with no check, and stopped at revision `stop`.
/// What was seen: each answer's revision.
fn kept(socket: &Path, stop: u64) -> (Result<Kept<u64>, Unkept<Infallible>>, Vec<u64>) {
    let mut seen = Vec::new();
    let until = Instant::now() + Duration::from_secs(60);
    let kept = keep_running(
        socket,
        until,
        || Ok(()),
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
    let seat = seat(&[
        r#"{"status":"observed"}"#,
        r#"{"status":"settled"}"#,
        r#"{"status":"settled"}"#,
    ]);
    let began = Instant::now();
    let mut checks = 0;
    let kept = keep_running(
        &seat.socket,
        began + Duration::from_millis(1_200),
        || {
            checks += 1;
            Ok::<(), Infallible>(())
        },
        |_| Ok(Next::<()>::Rest),
    );
    let took = began.elapsed();
    assert!(matches!(kept, Ok(Kept::TimedOut)), "{kept:?}");
    assert!(took >= Duration::from_millis(1_200), "{took:?}");
    assert!(took < Duration::from_secs(10), "{took:?}");
    assert!(checks > 3, "an interrupt is noticed between requests");
    assert_eq!(seat.received.join().unwrap().len(), 3);
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
fn the_callers_check_and_its_reading_of_an_answer_end_it_with_their_own_errors() {
    let resting = seat(&[r#"{"status":"observed"}"#]);
    let until = Instant::now() + Duration::from_secs(60);
    let interrupted = keep_running(
        &resting.socket,
        until,
        || Err("interrupted"),
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
    let stopped = keep_running(&socket, until, || Err("too long"), |_| Ok(Next::<()>::Rest));
    assert!(matches!(stopped, Err(Unkept::Unasked(_))), "{stopped:?}");
}
