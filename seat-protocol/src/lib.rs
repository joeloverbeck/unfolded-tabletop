//! The agent seat's wire vocabulary: the requests an agent sends, the statuses its
//! responses carry and the one line of a start refused before a table is seated. The
//! seat (`unfolded-tabletop`'s `seat` feature), its launcher and the held-seat client
//! (`unfolded-seat-client`) read and write the protocol through these types, and a
//! game's tools can name them too, so a status or a request field is spelled once. Nothing here needs
//! Bevy, a socket or a game.
//!
//! The vocabulary is Arkham's, as the seat had it when it moved; see `seat` in the
//! table package for what each request does.

use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use std::{
    fmt,
    path::{Path, PathBuf},
};

/// One request line. The seat reads it exactly: an unknown `op` or field is refused,
/// and a field left out takes its default. Written, it is the line an agent writes,
/// with the fields in the order below and a field that is `None` left out.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Observe {},
    /// Press and release the named component or choice, at this revision.
    Activate {
        revision: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        component: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        choice: Option<String>,
    },
    /// Rest the pointer on the named component or choice, at this revision.
    Point {
        revision: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        component: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        choice: Option<String>,
    },
    /// Turn the wheel over the named component or choice, in lines, at this revision.
    Scroll {
        revision: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        component: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        choice: Option<String>,
        #[serde(default)]
        x: f32,
        y: f32,
    },
    /// Press a key, at this revision, on the named focused control if it can commit.
    Key {
        key: String,
        revision: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        focused: Option<String>,
    },
    /// Step the table until it settles, or for at most this many steps.
    Settle {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        steps: Option<u32>,
    },
    /// Write the table's image to this path.
    Capture {
        #[serde(serialize_with = "lossy")]
        path: PathBuf,
    },
    Quit {},
}

/// A JSON string holds only Unicode, so a path that is not is written as it displays.
fn lossy<S: Serializer>(path: &Path, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&path.to_string_lossy())
}

impl fmt::Display for Request {
    /// The request as one line, without its newline.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let line = serde_json::to_string(self).map_err(|_| fmt::Error)?;
        formatter.write_str(&line)
    }
}

/// A response's status: what the seat did with the request, which decides what follows
/// it on its connection and in the session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// The seat is seated and admitted, and serves requests.
    Ready,
    Observed,
    Activated,
    /// The pointer rests on the named target, and nothing was pressed.
    Pointed,
    Scrolled,
    Pressed,
    Settled,
    Unsettled,
    Captured,
    /// The request was not acted on, for the reason in its error.
    Refused,
    /// Something else was over the target of an activation or a point: nothing was
    /// pressed, or the press was cancelled before release.
    Missed,
    /// An earlier refusal or miss stopped its activations, keys and scrolls.
    Skipped,
    /// `quit` ended the session.
    Bye,
    /// The table ended the session itself.
    Exited,
}

impl Status {
    /// Every status, in the order above.
    pub const ALL: [Self; 14] = [
        Self::Ready,
        Self::Observed,
        Self::Activated,
        Self::Pointed,
        Self::Scrolled,
        Self::Pressed,
        Self::Settled,
        Self::Unsettled,
        Self::Captured,
        Self::Refused,
        Self::Missed,
        Self::Skipped,
        Self::Bye,
        Self::Exited,
    ];

    /// The status as a response spells it.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Observed => "observed",
            Self::Activated => "activated",
            Self::Pointed => "pointed",
            Self::Scrolled => "scrolled",
            Self::Pressed => "pressed",
            Self::Settled => "settled",
            Self::Unsettled => "unsettled",
            Self::Captured => "captured",
            Self::Refused => "refused",
            Self::Missed => "missed",
            Self::Skipped => "skipped",
            Self::Bye => "bye",
            Self::Exited => "exited",
        }
    }

    /// The status a response spells `name`, if the protocol has one. Exact: case and
    /// spacing count.
    pub fn named(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|status| status.name() == name)
    }

    /// The request was not done as asked: it was refused, it missed its target or it
    /// was skipped. A client counts such an exchange as failed.
    pub const fn failed(self) -> bool {
        matches!(self, Self::Refused | Self::Missed | Self::Skipped)
    }

    /// No request follows this one in the session: `quit` or the table's own exit.
    pub const fn ends_session(self) -> bool {
        matches!(self, Self::Bye | Self::Exited)
    }

    /// The status a parsed response carries, if it carries one the protocol has.
    pub fn of(response: &Value) -> Option<Self> {
        response.get("status")?.as_str().and_then(Self::named)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

/// Every response's `mode`.
pub const MODE: &str = "seat";

/// The one response of a start that fails before a table is seated: `ok` false, the
/// seat's `mode`, `refused` and why. It carries no revision, observation or table.
pub fn refusal(error: &str) -> Value {
    serde_json::json!({"ok": false, "mode": MODE, "status": Status::Refused, "error": error})
}

/// What a launcher reads from the first line a starting seat prints: its `ready`
/// response, or the one `refused` response of [`refusal`].
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct StartLine {
    pub status: Status,
    /// Why the start was refused.
    #[serde(default)]
    pub error: Option<String>,
}

impl StartLine {
    /// The line read as a start's first line, or nothing when it is no response with a
    /// status the protocol has.
    pub fn read(line: &str) -> Option<Self> {
        serde_json::from_str(line).ok()
    }
}
