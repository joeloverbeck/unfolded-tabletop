// Several cases hold the earlier calls, deprecated and kept while a pin that uses them moves.
#![allow(deprecated)]

use std::{
    io::{BufReader, Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    thread,
    time::Duration,
};
use unfolded_seat_client as seat_client;

#[test]
fn sends_lines_then_half_closes_and_preserves_final_response_without_newline() {
    let directory = tempfile::tempdir().unwrap();
    let socket = directory.path().join("seat.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut input = String::new();
        stream.read_to_string(&mut input).unwrap();
        assert_eq!(input, "{\"op\":\"observe\"}\n{\"op\": \"settle\"}\n");
        stream
            .write_all(b"{\"status\": \"observed\"}\n{\"status\":\"settled\"}")
            .unwrap();
    });
    let requests = seat_client::requests(
        &["{\"op\":\"observe\"}\n \n{\"op\": \"settle\"}".into()],
        &mut std::io::empty(),
    )
    .unwrap();
    let stream = seat_client::connect(&socket).unwrap();
    let mut output = Vec::new();
    let (answered, failed) =
        seat_client::exchange(&stream, &requests, &mut output, || Ok(())).unwrap();
    assert_eq!((answered, failed), (2, false));
    assert_eq!(
        output,
        b"{\"status\": \"observed\"}\n{\"status\":\"settled\"}"
    );
    server.join().unwrap();
}

#[test]
fn refusal_and_unreadable_status_fail_without_changing_or_truncating_output() {
    for response in [
        "{\"status\":\"refused\"}\n",
        "{\"status\":\"missed\"}\n",
        "{\"status\":\"skipped\"}\n",
        "{}\n",
        "{\"status\":7}\n",
        "not JSON\n",
        "{\"status\":",
    ] {
        let (client, mut server) = UnixStream::pair().unwrap();
        let responder = thread::spawn(move || {
            let mut request = String::new();
            server.read_to_string(&mut request).unwrap();
            server.write_all(response.as_bytes()).unwrap();
        });
        let mut output = Vec::new();
        assert_eq!(
            seat_client::exchange(&client, &["{}".into()], &mut output, || Ok(())).unwrap(),
            (1, true)
        );
        assert_eq!(output, response.as_bytes());
        responder.join().unwrap();
    }
}

#[test]
fn early_eof_reports_too_few_answers_and_stdin_normalizes_blank_lines() {
    let requests = seat_client::requests(&[], &mut "{}\n \r\n{}\n".as_bytes()).unwrap();
    assert_eq!(requests, ["{}", "{}"]);
    assert!(seat_client::requests(&[], &mut " \n".as_bytes()).is_err());
    let (client, mut server) = UnixStream::pair().unwrap();
    let responder = thread::spawn(move || {
        let mut input = String::new();
        server.read_to_string(&mut input).unwrap();
        server.write_all(b"{\"status\":\"bye\"}\n").unwrap();
    });
    let (answered, failed) =
        seat_client::exchange(&client, &requests, &mut Vec::new(), || Ok(())).unwrap();
    assert_eq!(answered, 1);
    assert!(answered < requests.len());
    assert!(!failed);
    responder.join().unwrap();
}

#[test]
fn partial_line_survives_timeout_and_consumer_controls_interruption() {
    let (client, mut server) = UnixStream::pair().unwrap();
    client
        .set_read_timeout(Some(Duration::from_millis(10)))
        .unwrap();
    server.write_all(b"{\"status\":").unwrap();
    let mut reader = BufReader::new(client);
    let mut line = Vec::new();
    let mut timed_out = false;
    assert!(
        seat_client::read_line(&mut reader, &mut line, || {
            timed_out = true;
            server.write_all(b"\"observed\"}\n")?;
            Ok(())
        })
        .unwrap()
    );
    assert!(timed_out);
    assert_eq!(line, b"{\"status\":\"observed\"}\n");
    line.clear();
    let error = seat_client::read_line(&mut reader, &mut line, || {
        anyhow::bail!("consumer interrupted")
    })
    .unwrap_err();
    assert_eq!(error.to_string(), "consumer interrupted");
}

/// A stand-in seat on one end of a connection: it reads the client's requests to the
/// end of its input, then writes `answer` and closes.
fn answering(answer: &'static str) -> (seat_client::Connection, thread::JoinHandle<String>) {
    let (client, mut server) = UnixStream::pair().unwrap();
    let seat = thread::spawn(move || {
        let mut requests = String::new();
        server.read_to_string(&mut requests).unwrap();
        server.write_all(answer.as_bytes()).unwrap();
        requests
    });
    (client.into(), seat)
}

#[test]
fn a_whole_exchange_says_every_request_was_answered_and_copies_the_answers_unchanged() {
    use seat_client::Outcome;
    let directory = tempfile::tempdir().unwrap();
    let socket = directory.path().join("seat.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    let seat = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut requests = String::new();
        stream.read_to_string(&mut requests).unwrap();
        stream
            .write_all(b"{\"status\": \"observed\"}\n{\"status\":\"settled\"}")
            .unwrap();
        requests
    });
    let requests = [
        "{\"op\":\"observe\"}".to_owned(),
        "{\"op\": \"settle\"}".to_owned(),
    ];
    let mut output = Vec::new();
    let outcome = seat_client::Connection::open(&socket)
        .unwrap()
        .send(&requests, &mut output, || Ok(()))
        .unwrap();
    assert_eq!(outcome, Outcome::Answered);
    assert_eq!(
        output,
        b"{\"status\": \"observed\"}\n{\"status\":\"settled\"}"
    );
    assert_eq!(
        seat.join().unwrap(),
        "{\"op\":\"observe\"}\n{\"op\": \"settle\"}\n"
    );

    // A status the protocol does not name is no failure, as in Arkham's client.
    let (connection, seat) = answering("{\"status\":\"waved\"}\n");
    let outcome = connection.send(&["{}".into()], &mut std::io::sink(), || Ok(()));
    assert_eq!(outcome.unwrap(), Outcome::Answered);
    seat.join().unwrap();
}

#[test]
fn a_whole_exchange_fails_on_a_refused_missed_skipped_or_malformed_answer() {
    for answer in [
        "{\"status\":\"refused\"}\n",
        "{\"status\":\"missed\"}\n",
        "{\"status\":\"skipped\"}\n",
        "{}\n",
        "{\"status\":7}\n",
        "not JSON\n",
        "{\"status\":",
    ] {
        let (connection, seat) = answering(answer);
        let mut output = Vec::new();
        let outcome = connection.send(&["{}".into()], &mut output, || Ok(()));
        assert_eq!(outcome.unwrap(), seat_client::Outcome::Failed, "{answer}");
        assert_eq!(output, answer.as_bytes());
        seat.join().unwrap();
    }
}

#[test]
fn a_whole_exchange_says_how_many_were_answered_when_the_session_ends_early() {
    for answers in ["{\"status\":\"bye\"}\n", "{\"status\":\"refused\"}\n", ""] {
        let (connection, seat) = answering(answers);
        let outcome = connection.send(&["{}".into(), "{}".into()], &mut Vec::new(), || Ok(()));
        let answered = answers.lines().count();
        assert_eq!(
            outcome.unwrap(),
            seat_client::Outcome::Short { answered },
            "{answers:?}"
        );
        seat.join().unwrap();
    }
}

#[test]
fn both_calls_wake_for_the_callers_check_while_the_seat_says_nothing() {
    use seat_client::RequestError;
    use unfolded_seat_protocol::Request;
    for single in [false, true] {
        let (client, server) = UnixStream::pair().unwrap();
        let connection = seat_client::Connection::from(client);
        let mut checks = 0;
        let check = || {
            checks += 1;
            if checks == 3 {
                anyhow::bail!("the caller's deadline passed");
            }
            Ok(())
        };
        let error = if single {
            match connection.request(&Request::Observe {}, check) {
                Err(RequestError::Stopped(error)) => error,
                other => panic!("{other:?}"),
            }
        } else {
            connection
                .send(&["{}".into()], &mut Vec::new(), check)
                .unwrap_err()
        };
        assert_eq!(error.to_string(), "the caller's deadline passed");
        assert_eq!(checks, 3);
        drop(server);
    }
}

#[test]
fn connecting_tells_no_seat_listening_from_any_other_failure() {
    use seat_client::Unreached;
    let directory = tempfile::tempdir().unwrap();
    let absent = directory.path().join("absent.sock");
    // A socket file nothing listens on any more, as a crashed seat leaves it.
    let left = directory.path().join("left.sock");
    drop(UnixListener::bind(&left).unwrap());
    for socket in [&absent, &left] {
        match seat_client::Connection::open(socket) {
            Err(Unreached::NotListening(_)) => {}
            other => panic!("{}: {:?}", socket.display(), other.err()),
        }
    }
    let file = directory.path().join("file");
    std::fs::write(&file, "").unwrap();
    // A path through a file, and one longer than a socket path can be.
    for (socket, kind) in [
        (file.join("seat.sock"), std::io::ErrorKind::NotADirectory),
        (
            directory.path().join("s".repeat(120)),
            std::io::ErrorKind::InvalidInput,
        ),
    ] {
        match seat_client::Connection::open(&socket) {
            Err(Unreached::Failed(error)) => assert_eq!(error.kind(), kind),
            other => panic!("{}: {:?}", socket.display(), other.err()),
        }
    }
    // Either is said in the operating system's words, which a game's message quotes.
    let unreached = Unreached::NotListening(std::io::Error::other("nothing answers"));
    assert_eq!(unreached.to_string(), "nothing answers");
}

#[test]
fn a_single_request_returns_its_parsed_answer() {
    use seat_client::Reply;
    use unfolded_seat_protocol::{Request, Status};
    for (line, status, response) in [
        (
            "{\"ok\":true,  \"status\":\"observed\",\"mode\":\"seat\"}\n",
            Status::Observed,
            serde_json::json!({"ok": true, "status": "observed", "mode": "seat"}),
        ),
        (
            "{\"status\":\"refused\",\"error\":\"Outdated\"}",
            Status::Refused,
            serde_json::json!({"status": "refused", "error": "Outdated"}),
        ),
    ] {
        let (connection, seat) = answering(line);
        let request = Request::Settle { steps: Some(120) };
        match connection.request(&request, || Ok(())).unwrap() {
            Reply::Answer(answer) => {
                assert_eq!(answer.status, status);
                assert_eq!(answer.line, line.trim_end());
                assert_eq!(answer.response, response);
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(seat.join().unwrap(), "{\"op\":\"settle\",\"steps\":120}\n");
    }
}

#[test]
fn a_single_request_says_the_session_ended() {
    use seat_client::{Ended, Reply};
    use unfolded_seat_protocol::{Request, Status};
    for (answer, said) in [
        ("{\"status\":\"bye\"}\n", "its session ended with bye"),
        (
            "{\"ok\":true,\"status\":\"exited\"}",
            "its session ended with exited",
        ),
        ("", "it closed the connection without answering"),
    ] {
        let (connection, seat) = answering(answer);
        match connection.request(&Request::Quit {}, || Ok(())).unwrap() {
            Reply::Ended(ended) => {
                assert_eq!(ended.to_string(), said);
                if !answer.is_empty() {
                    assert!(matches!(
                        ended,
                        Ended::Session(Status::Bye | Status::Exited)
                    ));
                }
            }
            other => panic!("{other:?}"),
        }
        seat.join().unwrap();
    }

    // A seat that ends with the request unread resets the connection.
    let (client, mut server) = UnixStream::pair().unwrap();
    let seat = thread::spawn(move || {
        let mut first = [0];
        server.read_exact(&mut first).unwrap();
    });
    let reply = seat_client::Connection::from(client).request(&Request::Observe {}, || Ok(()));
    seat.join().unwrap();
    assert!(matches!(
        reply.unwrap(),
        Reply::Ended(Ended::Reset(_) | Ended::Closed)
    ));
}

#[test]
fn a_single_request_refuses_an_answer_with_no_status_it_knows() {
    use seat_client::RequestError;
    use unfolded_seat_protocol::Request;
    for (answer, said) in [
        ("not JSON\n", "the seat's answer is not JSON: not JSON"),
        (
            "{\"status\":\"waved\"}\n",
            "the seat answered {\"op\":\"observe\"} with: {\"status\":\"waved\"}",
        ),
        ("{}", "the seat answered {\"op\":\"observe\"} with: {}"),
    ] {
        let (connection, seat) = answering(answer);
        let error = connection
            .request(&Request::Observe {}, || Ok(()))
            .unwrap_err();
        assert!(
            matches!(
                error,
                RequestError::NotJson { .. } | RequestError::Unrecognised { .. }
            ),
            "{error:?}"
        );
        assert_eq!(error.to_string(), said);
        seat.join().unwrap();
    }
}
