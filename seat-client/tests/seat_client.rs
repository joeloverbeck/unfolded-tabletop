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
