//! The seat protocol's vocabulary as it crosses the wire. The names and lines here
//! are the ones `tests/seat.rs` pins on the seat and `seat-client/tests` on the client.

use serde_json::{Value, json};
use unfolded_seat_protocol::Status;

#[test]
fn every_status_has_its_wire_name_and_is_read_back_from_it() {
    let names = [
        "ready",
        "observed",
        "activated",
        "pointed",
        "scrolled",
        "pressed",
        "dragged",
        "settled",
        "unsettled",
        "advanced",
        "captured",
        "refused",
        "missed",
        "skipped",
        "bye",
        "exited",
    ];
    assert_eq!(Status::ALL.map(Status::name), names);
    for (status, name) in Status::ALL.into_iter().zip(names) {
        assert_eq!(serde_json::to_value(status).unwrap(), json!(name));
        assert_eq!(
            serde_json::from_value::<Status>(json!(name)).unwrap(),
            status
        );
        assert_eq!(Status::named(name), Some(status));
        assert_eq!(status.to_string(), name);
        assert_eq!(
            Status::of(&json!({"ok": true, "status": name})),
            Some(status)
        );
    }
    assert_eq!(Status::named("Refused"), None);
    for response in [
        json!({}),
        json!({"status": 7}),
        json!({"status": "waved"}),
        Value::Null,
    ] {
        assert_eq!(Status::of(&response), None, "{response}");
    }
}

#[test]
fn a_status_says_whether_its_request_failed_or_ended_the_session() {
    let named = |test: fn(Status) -> bool| -> Vec<&str> {
        Status::ALL
            .into_iter()
            .filter(|status| test(*status))
            .map(Status::name)
            .collect()
    };
    assert_eq!(named(Status::failed), ["refused", "missed", "skipped"]);
    assert_eq!(named(Status::ends_session), ["bye", "exited"]);
}

#[test]
fn a_request_is_read_from_the_line_an_agent_writes_and_written_as_one() {
    use std::path::PathBuf;
    use unfolded_seat_protocol::Request;
    for (line, request) in [
        (r#"{"op":"observe"}"#, Request::Observe {}),
        (
            r#"{"op":"activate","revision":3,"component":"card:1"}"#,
            Request::Activate {
                revision: 3,
                component: Some("card:1".into()),
                choice: None,
            },
        ),
        (
            r#"{"op":"point","revision":0,"choice":"3:pass"}"#,
            Request::Point {
                revision: 0,
                component: None,
                choice: Some("3:pass".into()),
            },
        ),
        (
            r#"{"op":"scroll","revision":9,"choice":"GainCredit","x":2.0,"y":-4.0}"#,
            Request::Scroll {
                revision: 9,
                component: None,
                choice: Some("GainCredit".into()),
                x: 2.0,
                y: -4.0,
            },
        ),
        (
            r#"{"op":"drag","revision":7,"component":"table","button":"right","x":120.0,"y":-40.0}"#,
            Request::Drag {
                revision: 7,
                component: Some("table".into()),
                choice: None,
                button: "right".into(),
                x: 120.0,
                y: -40.0,
            },
        ),
        (
            r#"{"op":"key","key":"Enter","revision":1,"focused":"Continue"}"#,
            Request::Key {
                key: "Enter".into(),
                revision: 1,
                focused: Some("Continue".into()),
            },
        ),
        (r#"{"op":"settle"}"#, Request::Settle { steps: None }),
        (
            r#"{"op":"settle","steps":120}"#,
            Request::Settle { steps: Some(120) },
        ),
        (
            r#"{"op":"advance","steps":216}"#,
            Request::Advance { steps: 216 },
        ),
        (
            r#"{"op":"capture","path":"/tmp/table.png"}"#,
            Request::Capture {
                path: PathBuf::from("/tmp/table.png"),
            },
        ),
        (r#"{"op":"quit"}"#, Request::Quit {}),
    ] {
        assert_eq!(
            serde_json::from_str::<Request>(line).unwrap(),
            request,
            "{line}"
        );
        assert_eq!(request.to_string(), line);
    }
    // As an agent may write them: in any order and spacing, with `x` left out.
    assert_eq!(
        serde_json::from_str::<Request>(
            r#"{"y": -4, "op": "scroll", "component": "card", "revision": 0}"#
        )
        .unwrap(),
        Request::Scroll {
            revision: 0,
            component: Some("card".into()),
            choice: None,
            x: 0.0,
            y: -4.0,
        }
    );
    // A drag may leave out either axis, but never its button.
    assert_eq!(
        serde_json::from_str::<Request>(
            r#"{"op":"drag","revision":0,"choice":"deck","button":"left","y":5}"#
        )
        .unwrap(),
        Request::Drag {
            revision: 0,
            component: None,
            choice: Some("deck".into()),
            button: "left".into(),
            x: 0.0,
            y: 5.0,
        }
    );
    // An advance always says how many steps, and takes no revision.
    for refused in [
        r#"{"op":"advance"}"#,
        r#"{"op":"advance","steps":-1}"#,
        r#"{"op":"advance","steps":1.5}"#,
        r#"{"op":"advance","steps":3,"revision":0}"#,
        r#"{"op":"drag","revision":0,"component":"table","x":5}"#,
        r#"{"op":"drag","revision":0,"component":"table","button":"right","x":5,"steps":3}"#,
        r#"{"op":"observe","revision":0}"#,
        r#"{"op":"wave"}"#,
        r#"{"op":"activate","component":"card"}"#,
        r#"{"revision":0}"#,
    ] {
        assert!(
            serde_json::from_str::<Request>(refused).is_err(),
            "{refused}"
        );
    }
}

#[test]
fn a_start_refused_before_a_table_is_seated_is_one_line_a_launcher_reads_back() {
    use unfolded_seat_protocol::{StartLine, refusal};
    let refused = refusal("The table spawned no camera.");
    assert_eq!(
        refused.to_string(),
        r#"{"error":"The table spawned no camera.","mode":"seat","ok":false,"status":"refused"}"#
    );
    assert_eq!(
        StartLine::read(&refused.to_string()),
        Some(StartLine {
            status: Status::Refused,
            error: Some("The table spawned no camera.".into()),
        })
    );
    let ready = json!({"ok": true, "mode": "seat", "status": "ready", "revision": 0,
        "observation": null, "table": {"settled": true, "pending": []}});
    assert_eq!(
        StartLine::read(&format!("{ready}\n")),
        Some(StartLine {
            status: Status::Ready,
            error: None,
        })
    );
    for line in [
        "",
        "not JSON",
        r#"{"status":"waved"}"#,
        r#"{"error":"no status"}"#,
    ] {
        assert_eq!(StartLine::read(line), None, "{line}");
    }
}
