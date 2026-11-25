//! Seam: the client at a real socket, against a stand-in game. Expectations come from
//! Cuba Libre #199: one request, one answer, exit statuses the caller maps.

use std::{
    convert::Infallible,
    io::{Read, Write},
    os::unix::net::UnixListener,
    path::PathBuf,
    thread,
    time::Duration,
};
use unfolded_agent_players::client::{Answer, Unanswered, request, session_in};

struct Game {
    socket: PathBuf,
    heard: thread::JoinHandle<String>,
    _directory: tempfile::TempDir,
}

/// A stand-in game that reads one client's requests to the end of its input, pauses,
/// writes `answer` in two parts with a pause between and closes.
fn game(answer: &'static str) -> Game {
    let directory = tempfile::tempdir().unwrap();
    let socket = directory.path().join("players.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    let heard = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut heard = String::new();
        stream.read_to_string(&mut heard).unwrap();
        let (first, rest) = answer.split_at(answer.len() / 2);
        stream.write_all(first.as_bytes()).unwrap();
        thread::sleep(Duration::from_millis(500));
        // A client that has its line is gone.
        let _ = stream.write_all(rest.as_bytes());
        heard
    });
    Game {
        socket,
        heard,
        _directory: directory,
    }
}

fn answered(answer: &'static str) -> Answer {
    let game = game(answer);
    let answer = request(&game.socket, "{\"op\":\"look\"}", || {
        Ok::<(), Infallible>(())
    })
    .unwrap();
    assert_eq!(game.heard.join().unwrap(), "{\"op\":\"look\"}\n");
    answer
}

fn printed(answer: &Answer) -> String {
    let mut output = Vec::new();
    answer.write_to(&mut output).unwrap();
    String::from_utf8(output).unwrap()
}

#[test]
fn one_request_is_sent_and_its_answer_printed_as_it_arrived() {
    let answer = answered("{\"status\": \"decision\",  \"view\":{}}\n");
    assert_eq!(answer.status().as_deref(), Some("decision"));
    assert_eq!(
        printed(&answer),
        "{\"status\": \"decision\",  \"view\":{}}\n"
    );
}

#[test]
fn an_answer_ends_with_a_line_ending_and_its_status_may_be_absent() {
    for (line, status) in [
        ("{\"status\":\"refused\"}", Some("refused")),
        ("{\"status\":3}", None),
        ("{\"ok\":true}", None),
        ("not JSON", None),
    ] {
        let answer = answered(line);
        assert_eq!(printed(&answer), format!("{line}\n"));
        assert_eq!(answer.status().as_deref(), status, "{line}");
    }
}

#[test]
fn only_the_first_line_is_the_answer() {
    let answer = answered("{\"status\":\"seen\"}\n{\"status\":\"again\"}\n");
    assert_eq!(printed(&answer), "{\"status\":\"seen\"}\n");
}

#[test]
fn a_game_that_closes_without_answering_is_no_answer() {
    let game = game("");
    let closed = request(&game.socket, "{}", || Ok::<(), Infallible>(())).unwrap_err();
    assert!(matches!(closed, Unanswered::Closed), "{closed}");
}

#[test]
fn a_socket_nobody_answers_on_and_one_that_cannot_be_reached_are_told_apart() {
    let directory = tempfile::tempdir().unwrap();
    let missing = directory.path().join("missing.sock");
    let dead = directory.path().join("dead.sock");
    drop(UnixListener::bind(&dead).unwrap());
    for socket in [missing, dead] {
        let refused = request(&socket, "{}", || Ok::<(), Infallible>(())).unwrap_err();
        assert!(
            matches!(refused, Unanswered::NotListening(_)),
            "{}: {refused}",
            socket.display()
        );
    }
    let file = directory.path().join("file");
    std::fs::write(&file, "").unwrap();
    let refused = request(
        &file.join("players.sock"),
        "{}",
        || Ok::<(), Infallible>(()),
    )
    .unwrap_err();
    assert!(matches!(refused, Unanswered::Unreached(_)), "{refused}");
}

#[test]
fn the_callers_check_runs_during_the_wait_and_its_error_ends_it() {
    let directory = tempfile::tempdir().unwrap();
    let socket = directory.path().join("players.sock");
    let _silent = UnixListener::bind(&socket).unwrap();
    let mut checks = 0;
    let stopped = request(&socket, "{}", || {
        checks += 1;
        if checks == 3 { Err("enough") } else { Ok(()) }
    })
    .unwrap_err();
    assert!(
        matches!(stopped, Unanswered::Stopped("enough")),
        "{stopped}"
    );
    assert_eq!(checks, 3);
}

/// The variables are those Cuba Libre #198 found each tool to give its commands.
#[test]
fn a_sessions_identifiers_are_those_its_tool_gives() {
    let given = |variables: &'static [(&'static str, &'static str)]| {
        session_in(|name| {
            variables
                .iter()
                .find(|(variable, _)| *variable == name)
                .map(|(_, value)| (*value).to_owned())
        })
    };
    assert_eq!(given(&[]), None);
    assert_eq!(given(&[("CODEX_THREAD_ID", "")]), None);
    assert_eq!(
        given(&[("CODEX_THREAD_ID", "t1"), ("CODEX_SESSION_ID", "t1")]).as_deref(),
        Some("CODEX_THREAD_ID=t1")
    );
    assert_eq!(
        given(&[("CLAUDE_CODE_SESSION_ID", "c1")]).as_deref(),
        Some("CLAUDE_CODE_SESSION_ID=c1")
    );
    // A Codex session started from a Claude Code session's command inherits its
    // identifier, and is another session than one started beside it.
    let nested = |thread| match thread {
        "t1" => given(&[("CLAUDE_CODE_SESSION_ID", "c1"), ("CODEX_THREAD_ID", "t1")]),
        _ => given(&[("CLAUDE_CODE_SESSION_ID", "c1"), ("CODEX_THREAD_ID", "t2")]),
    };
    assert_ne!(nested("t1"), nested("t2"));
}
