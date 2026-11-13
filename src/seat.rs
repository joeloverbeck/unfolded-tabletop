//! The agent seat: an agent sits at a game's real table through one JSON request per
//! line and one JSON response per line, with no window: on standard input and output in batch form, or held across shell
//! commands on a local socket ([`socket`]). This module owns fixed-step stepping,
//! pointer and key synthesis, capture and the protocol loop both forms share. It reaches
//! the game only through [`Game`]: produce an observation and its revision, resolve a
//! name to a screen position (and a hit back to a name), say what has keyboard focus,
//! report whether the table is settled, advance the table's own clock, and say whether
//! the table is admitted to play and its render pipelines are ready, and name the keys
//! the seat may press. The table's surface and the capture of its image come from
//! [`crate::capture`].
//!
//! Each game keeps its policy in its adapter: what it observes and names, the keys
//! it reads and which of them can commit, its clock, its admission and the words of
//! an admission that ran out of time, the plugins of its windowless host, how it is
//! launched, and how long a held session may idle.
//!
//! The module was extracted from Arkham, its first table. The protocol's vocabulary
//! and the wording of its responses are as Arkham had them, and so are its fixed
//! numbers: the step, the settling limits and the time a capture may take. No second
//! use has shaped them yet.
//!
//! The protocol's guards live here, so every transport gets them: activations and
//! keys act only on the revision they were decided on, a key that can commit presses
//! only the control it names, and a connection stops acting after its first refusal or
//! missed activation.
//!
//! Input enters where Bevy's own window-event conversion writes it: `PointerInput` and
//! `KeyboardInput` messages plus the logical window's cursor position. Everything after
//! that is the game's production code. The seat never triggers observers, queues table
//! actions or calls handlers.

use crate::capture::{self, Failure, Surface};
use bevy::{
    app::AppExit,
    camera::RenderTarget,
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    picking::{
        hover::HoverMap,
        pointer::{Location, PointerAction, PointerButton, PointerId, PointerInput},
    },
    prelude::*,
    render::renderer::RenderAdapterInfo,
    window::PrimaryWindow,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    io::{BufRead, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub mod socket;

/// One step of the table's clock. The seat, not wall-clock time, advances it.
const STEP: Duration = Duration::from_nanos(16_666_667);
/// Thirty seconds of table time.
const DEFAULT_SETTLE_STEPS: u32 = 1_800;
const MAX_SETTLE_STEPS: u32 = 7_200;
const CAPTURE_LIMIT: Duration = Duration::from_secs(30);

/// What an agent names. Lookup is exact; a nearest match is never used.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Name {
    /// A table component, by the presentation's source identity.
    Component(String),
    /// A choice-bearing control, by its choice identity.
    Choice(String),
}

impl Name {
    fn describe(&self) -> String {
        match self {
            Self::Component(name) => format!("component {name}"),
            Self::Choice(id) => format!("choice {id}"),
        }
    }
}

/// What the pointer is over, named when the game can name it.
pub struct Hit {
    pub name: Option<Name>,
    pub description: Value,
}

/// What has keyboard focus, which a key that can commit must name.
pub enum Focus {
    Nothing,
    /// A shown control or contextual-menu row.
    Control {
        choice: Option<String>,
        label: String,
    },
    /// Something that is not a shown control or menu row, as the game describes it.
    Other(Value),
}

impl Focus {
    /// A control is named by its choice identity, or by its label when it has none.
    /// The match is exact.
    fn is_named(&self, expected: &str) -> bool {
        match self {
            Self::Control {
                choice: Some(id), ..
            } => id == expected,
            Self::Control {
                choice: None,
                label,
            } => label == expected,
            Self::Nothing | Self::Other(_) => false,
        }
    }

    fn describe(&self) -> String {
        match self {
            Self::Nothing => "nothing has focus".into(),
            Self::Control {
                choice: Some(id),
                label,
            } => format!("focus is on choice {id} ({label:?})"),
            Self::Control {
                choice: None,
                label,
            } => format!("focus is on {label:?}, which has no choice identity"),
            Self::Other(description) => {
                format!("focus is on {description}, which is not a shown control or menu row")
            }
        }
    }
}

pub struct Observed {
    /// The player-visible observation, in the game's own form.
    pub observation: Value,
    /// What the table shows, such as components, controls, modal state and settling.
    pub table: Value,
}

/// Whether a game admits its table to play, by the game's own conditions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Admission {
    /// The table may be played.
    Admitted,
    /// The game is still checking.
    Checking,
    /// The game refused play, for this reason.
    Refused(String),
}

/// Why [`Seat::admit`] did not admit the table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unadmitted {
    /// The game refused play or the table exited: the reason.
    Refused(String),
    /// The game was still checking when the time allowed ran out. What to say of it
    /// is the game's.
    Unfinished,
}

/// A key a game lets the seat press.
pub struct SeatKey {
    /// The name a request gives it: its `KeyCode` name.
    pub name: &'static str,
    /// The physical key.
    pub code: KeyCode,
    /// The logical key a keyboard would send with it.
    pub logical: Logical,
    /// Whether it can commit a choice or dismiss a reading at this table. A request
    /// for such a key must name the control it expects to press.
    pub commits: bool,
}

/// The logical key a keyboard sends with a physical one.
pub enum Logical {
    Named(Key),
    /// A character key, by the text it types.
    Typed(&'static str),
}

/// The whole game-facing interface of the seat.
pub trait Game {
    fn observe(&self, world: &mut World) -> Observed;
    /// The observation revision the attempt is at, which a request names to act.
    fn revision(&self, world: &mut World) -> u64;
    /// Logical screen positions on exactly one shown component or control: its
    /// middle first, then other points where a pointer could rest. Absent, ambiguous
    /// and off-screen names are errors, never a near match.
    fn resolve(&self, world: &mut World, name: &Name) -> Result<Vec<Vec2>, String>;
    /// The inverse of `resolve` for one picked entity.
    fn identify(&self, world: &mut World, entity: Entity) -> Hit;
    /// What a key press would reach now.
    fn focus(&self, world: &mut World) -> Focus;
    /// The conditions still in progress; empty when the table will accept input.
    fn unsettled(&self, world: &mut World) -> Vec<String>;
    /// Advance the table's own clock by `by`, which is zero while a capture is read
    /// back. Only the seat's steps move that clock, never wall-clock time.
    fn advance(&self, world: &mut World, by: Duration);
    /// Whether the game admits the table to play.
    fn admission(&self, world: &World) -> Admission;
    /// Whether the render pipelines of what the table shows are ready, so that a
    /// capture omits nothing.
    fn pipelines_ready(&self, world: &World) -> bool;
    /// The keys the seat may press at this table, in the order a refusal lists them.
    /// Any other key is refused as unknown.
    fn keys(&self) -> &[SeatKey];
}

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
enum Request {
    Observe {},
    Activate {
        revision: u64,
        #[serde(default)]
        component: Option<String>,
        #[serde(default)]
        choice: Option<String>,
    },
    Key {
        key: String,
        revision: u64,
        #[serde(default)]
        focused: Option<String>,
    },
    Settle {
        #[serde(default)]
        steps: Option<u32>,
    },
    Capture {
        path: PathBuf,
    },
    Quit {},
}

/// A response's status, which decides what follows it on its connection and in the
/// session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    Ready,
    Observed,
    Activated,
    Pressed,
    Settled,
    Unsettled,
    Captured,
    /// The request was not acted on, for the reason in its error.
    Refused,
    /// Something else was over the activation's target: nothing was pressed, or the
    /// press was cancelled before release.
    Missed,
    /// An earlier refusal or miss on the connection stopped its activations and keys.
    Skipped,
    /// `quit` ended the session.
    Bye,
    /// The table ended the session itself.
    Exited,
}

impl Status {
    /// The table was not as the request assumed and the request clicked nothing, so the
    /// rest of the connection stops acting.
    fn stops_connection(self) -> bool {
        matches!(self, Self::Refused | Self::Missed)
    }

    fn ends_session(self) -> bool {
        matches!(self, Self::Bye | Self::Exited)
    }
}

/// One response line and its status.
struct Response {
    status: Status,
    /// The line itself: `ok`, `mode`, `status`, `observation`, `table`, any `error` and
    /// the status's own details.
    json: Value,
}

impl Response {
    /// The error the agent reads, or nothing.
    fn error(&self) -> &str {
        self.json["error"].as_str().unwrap_or_default()
    }
}

/// How one connection ended: whether the session goes on after it.
#[derive(Debug, PartialEq, Eq)]
enum ConnectionEnd {
    /// The connection's input ended; another connection may continue the session.
    InputEnded,
    /// `quit` or the table's own exit ended the session.
    SessionEnded,
}

/// One connection's requests: standard input in batch form, or one connection to the
/// seat. After its first refused or missed response the connection stops acting: every
/// later activation and key is skipped, so one wrong assumption cannot cascade into
/// several commits. Observing, settling, capturing and quitting are still served.
#[derive(Default)]
struct Connection {
    stopped: Option<Stop>,
}

/// The request that stopped a connection.
struct Stop {
    line: String,
    status: Status,
    reason: String,
}

impl Connection {
    /// Why `request` is skipped, if it is.
    fn skips(&self, request: &Request) -> Option<String> {
        let Stop {
            line,
            status,
            reason,
        } = self.stopped.as_ref()?;
        let (stopped, request_was) = match status {
            Status::Missed => ("missed its target", "Missed"),
            _ => ("was refused", "Refused"),
        };
        matches!(request, Request::Activate { .. } | Request::Key { .. }).then(|| {
            format!(
                "Skipped: an earlier request on this connection {stopped}, so its activations and keys are no longer acted on. Nothing was pressed. {request_was} request: {line}. Reason: {reason}"
            )
        })
    }

    fn answered(&mut self, line: &str, response: &Response) {
        if self.stopped.is_none() && response.status.stops_connection() {
            self.stopped = Some(Stop {
                line: line.trim().to_owned(),
                status: response.status,
                reason: response.error().to_owned(),
            });
        }
    }
}

/// Present while an agent sits at the table. The agent operates the table, never the
/// desktop it runs on, so a game can refuse to the agent what it would otherwise hand
/// to the desktop, such as an address to open.
#[derive(Resource)]
pub struct SeatedAgent;

pub struct Seat<G> {
    game: G,
    camera: Entity,
    image: Option<Handle<Image>>,
    exit: Option<AppExit>,
}

impl<G: Game> Seat<G> {
    /// Sit at a table whose camera already has a render target. Without an image
    /// (the no-GPU test host) every operation but `capture` is available.
    pub fn new(game: G, app: &mut App, camera: Entity, image: Option<Handle<Image>>) -> Self {
        app.insert_resource(SeatedAgent);
        Self {
            game,
            camera,
            image,
            exit: None,
        }
    }

    /// Finish building a table whose windowless surface is installed
    /// ([`capture::install`]) and sit at it. No runner ever starts: from here the
    /// seat alone steps updates.
    pub fn open(game: G, app: &mut App) -> Result<Self, String> {
        app.finish();
        app.cleanup();
        app.update();
        let surface = app.world().resource::<Surface>();
        let camera = surface
            .camera()
            .ok_or("The table spawned no camera for the seat's image.")?;
        let image = surface.image();
        Ok(Self::new(game, app, camera, Some(image)))
    }

    /// Advance the table's clock by `advance` and run one update. Returns false once
    /// the table has asked to exit.
    fn step(&mut self, app: &mut App, advance: Duration) -> bool {
        if self.exit.is_some() {
            return false;
        }
        self.game.advance(app.world_mut(), advance);
        app.update();
        self.exit = app.should_exit();
        self.exit.is_none()
    }

    /// Step in wall-clock time until the game admits the table, which can wait on
    /// asynchronous loading, then until the admitted table settles. `limit` is the
    /// time allowed for admission.
    pub fn admit(&mut self, app: &mut App, limit: Duration) -> Result<(), Unadmitted> {
        let deadline = Instant::now() + limit;
        loop {
            if !self.step(app, STEP) {
                return Err(Unadmitted::Refused(self.exit_reason(app)));
            }
            match self.game.admission(app.world()) {
                Admission::Admitted => break,
                Admission::Refused(reason) => return Err(Unadmitted::Refused(reason)),
                Admission::Checking => {}
            }
            if Instant::now() >= deadline {
                return Err(Unadmitted::Unfinished);
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        for _ in 0..DEFAULT_SETTLE_STEPS {
            if self.game.unsettled(app.world_mut()).is_empty() {
                break;
            }
            if !self.step(app, STEP) {
                return Err(Unadmitted::Refused(self.exit_reason(app)));
            }
        }
        Ok(())
    }

    /// Why the table exited: the game's refusal when it refused play.
    fn exit_reason(&self, app: &App) -> String {
        match self.game.admission(app.world()) {
            Admission::Refused(reason) => reason,
            Admission::Admitted | Admission::Checking => {
                format!("The table exited ({:?}).", self.exit)
            }
        }
    }

    /// Batch form: `ready`, then standard input as one connection until `quit`, its end
    /// or the table's own exit.
    pub fn serve(
        &mut self,
        app: &mut App,
        input: impl BufRead,
        mut output: impl Write,
    ) -> Result<(), String> {
        emit(&mut output, &self.ready(app))?;
        self.converse(app, input, &mut output)?;
        self.outcome(app)
    }

    /// The first response of a session: the admitted table and its graphics adapter.
    pub fn ready(&self, app: &mut App) -> Value {
        let mut ready = self.response(app, Status::Ready, None);
        ready.json["adapter"] = adapter(app.world());
        ready.json
    }

    /// Answer one connection's requests in order until `quit`, the end of its input or
    /// the table's own exit. Each call is a connection of its own, so a refusal or miss
    /// stops only the rest of this call's actions. An error is this connection's input
    /// or output failing.
    fn converse(
        &mut self,
        app: &mut App,
        input: impl BufRead,
        mut output: impl Write,
    ) -> Result<ConnectionEnd, String> {
        let mut connection = Connection::default();
        for line in input.lines() {
            let line = line.map_err(|error| error.to_string())?;
            if line.trim().is_empty() {
                continue;
            }
            let response = self.respond_on(app, &mut connection, &line);
            let sent = emit(&mut output, &response.json);
            if response.status.ends_session() {
                // Over even if its last response could not be delivered.
                return Ok(ConnectionEnd::SessionEnded);
            }
            sent?;
        }
        Ok(ConnectionEnd::InputEnded)
    }

    /// How an ended session went: the table's reason when it exited with an error.
    fn outcome(&self, app: &App) -> Result<(), String> {
        match &self.exit {
            Some(exit) if exit.is_error() => Err(self.exit_reason(app)),
            _ => Ok(()),
        }
    }

    /// One request on a connection of its own, one response, for a consumer's protocol
    /// tests, which send requests one at a time. The flag is false when the session
    /// has ended.
    pub fn respond(&mut self, app: &mut App, line: &str) -> (Value, bool) {
        let response = self.respond_on(app, &mut Connection::default(), line);
        (response.json, !response.status.ends_session())
    }

    /// One request on `connection`, one response.
    fn respond_on(&mut self, app: &mut App, connection: &mut Connection, line: &str) -> Response {
        let response = match serde_json::from_str::<Request>(line) {
            Err(error) => self.refused(app, error.to_string()),
            Ok(request) => match connection.skips(&request) {
                Some(error) => self.response(app, Status::Skipped, Some(error)),
                None => self.perform(app, request),
            },
        };
        connection.answered(line, &response);
        response
    }

    fn perform(&mut self, app: &mut App, request: Request) -> Response {
        let response = match request {
            Request::Observe {} => self.response(app, Status::Observed, None),
            Request::Activate {
                revision,
                component,
                choice,
            } => match (component, choice) {
                (Some(name), None) => self.activate(app, Name::Component(name), revision),
                (None, Some(id)) => self.activate(app, Name::Choice(id), revision),
                _ => self.refused(
                    app,
                    "activate needs exactly one of component or choice".into(),
                ),
            },
            Request::Key {
                key,
                revision,
                focused,
            } => self.key(app, &key, revision, focused.as_deref()),
            Request::Settle { steps } => self.settle(app, steps),
            Request::Capture { path } => self.capture(app, &path),
            Request::Quit {} => return self.response(app, Status::Bye, None),
        };
        if let Some(exit) = &self.exit {
            // The table ended the session itself, for example by returning to selection.
            let error = exit.is_error().then(|| self.exit_reason(app));
            return self.response(app, Status::Exited, error);
        }
        response
    }

    fn response(&self, app: &mut App, status: Status, error: Option<String>) -> Response {
        let Observed { observation, table } = self.game.observe(app.world_mut());
        let mut json = json!({
            "ok": error.is_none(), "mode": "seat", "status": status,
            "observation": observation, "table": table,
        });
        if let Some(error) = error {
            json["error"] = json!(error);
        }
        Response { status, json }
    }

    fn refused(&self, app: &mut App, error: String) -> Response {
        self.response(app, Status::Refused, Some(error))
    }

    /// Why a request decided on `revision` must not act now, if it must not: the table
    /// is presenting and would drop the input, or the attempt has moved on.
    fn unready(&self, app: &mut App, revision: u64) -> Option<Response> {
        let pending = self.game.unsettled(app.world_mut());
        if !pending.is_empty() {
            return Some(self.busy(app, &pending));
        }
        let current = self.game.revision(app.world_mut());
        (revision != current).then(|| {
            self.refused(
                app,
                format!(
                    "Outdated: the request names revision {revision}, but the attempt is at revision {current}. Observe again."
                ),
            )
        })
    }

    fn activate(&mut self, app: &mut App, name: Name, revision: u64) -> Response {
        if let Some(refusal) = self.unready(app, revision) {
            return refusal;
        }
        let candidates = match self.game.resolve(app.world_mut(), &name) {
            Ok(candidates) => candidates,
            Err(error) => return self.refused(app, error),
        };
        // Rest the pointer on the named target as a player would: on a part of it
        // that nothing covers. One update per point establishes real hover.
        let mut covered = None;
        let mut reached = None;
        for position in candidates {
            if let Err(error) =
                self.pointer(app, position, PointerAction::Move { delta: Vec2::ZERO })
            {
                return self.refused(app, error);
            }
            self.step(app, STEP);
            let hovered = self.under_pointer(app);
            if hovered.iter().any(|hit| hit.name.as_ref() == Some(&name)) {
                reached = Some(position);
                break;
            }
            covered.get_or_insert((position, hovered));
        }
        let Some(position) = reached else {
            // Pressing here would act on whatever covers the named target.
            let Some((position, hovered)) = covered else {
                return self.refused(app, format!("{} has no screen position.", name.describe()));
            };
            return self.missed(app, &name, position, hovered, false);
        };
        for action in [
            PointerAction::Press(PointerButton::Primary),
            PointerAction::Release(PointerButton::Primary),
        ] {
            let pressing = matches!(action, PointerAction::Press(_));
            if let Err(error) = self.pointer(app, position, action) {
                return self.refused(app, error);
            }
            self.step(app, STEP);
            if pressing {
                let pressed = self.under_pointer(app);
                if !pressed.iter().any(|hit| hit.name.as_ref() == Some(&name)) {
                    // Something moved under the pointer after hover. A release here
                    // would click it, so cancel the press before letting go.
                    for undo in [
                        PointerAction::Cancel,
                        PointerAction::Release(PointerButton::Primary),
                    ] {
                        let _ = self.pointer(app, position, undo);
                        self.step(app, STEP);
                    }
                    return self.missed(app, &name, position, pressed, true);
                }
            }
        }
        let mut response = self.response(app, Status::Activated, None);
        response.json["position"] = json!([position.x, position.y]);
        if let Some((tried, hits)) = covered {
            // Pressed at a later point: say what the pointer found at the first one,
            // so a partly covered target is reported, not hidden by the success.
            response.json["covered"] = json!({
                "position": [tried.x, tried.y],
                "hit": hits.into_iter().map(|hit| hit.description).collect::<Vec<_>>(),
            });
        }
        response
    }

    fn busy(&self, app: &mut App, pending: &[String]) -> Response {
        // The table drops input while presenting; never let it vanish silently.
        self.refused(
            app,
            format!("The table is busy ({}); settle first.", pending.join(", ")),
        )
    }

    fn missed(
        &self,
        app: &mut App,
        name: &Name,
        position: Vec2,
        hits: Vec<Hit>,
        pressed: bool,
    ) -> Response {
        let hit: Vec<Value> = hits.into_iter().map(|hit| hit.description).collect();
        let over = if hit.is_empty() {
            "nothing".to_owned()
        } else {
            hit.iter()
                .map(Value::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        };
        let error = if pressed {
            format!(
                "The press at ({:.0}, {:.0}) landed on {over}, not {}.",
                position.x,
                position.y,
                name.describe()
            )
        } else {
            format!(
                "The pointer at ({:.0}, {:.0}) is over {over}, not {}. Nothing was pressed.",
                position.x,
                position.y,
                name.describe()
            )
        };
        let mut response = self.response(app, Status::Missed, Some(error));
        response.json["hit"] = json!(hit);
        response.json["pressed"] = json!(pressed);
        response.json["position"] = json!([position.x, position.y]);
        response
    }

    /// Everything under the pointer after the last update, nearest first.
    fn under_pointer(&self, app: &mut App) -> Vec<Hit> {
        let mut hovered: Vec<(Entity, f32)> = app
            .world()
            .get_resource::<HoverMap>()
            .and_then(|hover| hover.get(&PointerId::Mouse))
            .map(|hits| {
                hits.iter()
                    .map(|(entity, hit)| (*entity, hit.depth))
                    .collect()
            })
            .unwrap_or_default();
        hovered.sort_by(|a, b| a.1.total_cmp(&b.1));
        hovered
            .into_iter()
            .map(|(entity, _)| self.game.identify(app.world_mut(), entity))
            .collect()
    }

    /// The same message Bevy's window-event conversion writes, aimed at the camera's
    /// own render target, plus the logical window's cursor position.
    fn pointer(&self, app: &mut App, position: Vec2, action: PointerAction) -> Result<(), String> {
        let world = app.world_mut();
        let (window, mut cursor) = world
            .query_filtered::<(Entity, &mut Window), With<PrimaryWindow>>()
            .single_mut(world)
            .map_err(|_| "The seat has no logical primary window.")?;
        cursor.set_cursor_position(Some(position));
        let target = world
            .get::<RenderTarget>(self.camera)
            .and_then(|target| target.normalize(Some(window)))
            .ok_or("The seat's camera has no render target.")?;
        world.write_message(PointerInput::new(
            PointerId::Mouse,
            Location { target, position },
            action,
        ));
        Ok(())
    }

    /// Press and release one key decided on `revision`. A key that can commit, or any
    /// key given `focused`, presses only when the named control has focus.
    fn key(&mut self, app: &mut App, name: &str, revision: u64, focused: Option<&str>) -> Response {
        let pressable = self.game.keys().iter().find(|key| key.name == name);
        let pressable = pressable.map(|key| {
            let logical = match &key.logical {
                Logical::Named(key) => key.clone(),
                Logical::Typed(text) => Key::Character((*text).into()),
            };
            (key.code, logical, key.commits)
        });
        let Some((code, logical, commits)) = pressable else {
            let known: Vec<&str> = self.game.keys().iter().map(|key| key.name).collect();
            return self.refused(
                app,
                format!("Unknown key {name:?}. Known keys: {}.", known.join(", ")),
            );
        };
        if let Some(refusal) = self.unready(app, revision) {
            return refusal;
        }
        if commits || focused.is_some() {
            let focus = self.game.focus(app.world_mut());
            let error = match focused {
                None => Some(format!(
                    "{name} can commit a choice or dismiss a reading, so its request must name the control it expects to press in \"focused\": the control's choice identity, or its label when it has none. Nothing was pressed; {}.",
                    focus.describe()
                )),
                Some(expected) if !focus.is_named(expected) => Some(format!(
                    "{name} names {expected:?}, but {}. Nothing was pressed.",
                    focus.describe()
                )),
                Some(_) => None,
            };
            if let Some(error) = error {
                return self.refused(app, error);
            }
        }
        let world = app.world_mut();
        let Ok(window) = world
            .query_filtered::<Entity, With<PrimaryWindow>>()
            .single(world)
        else {
            return self.refused(app, "The seat has no logical primary window.".into());
        };
        for state in [ButtonState::Pressed, ButtonState::Released] {
            app.world_mut().write_message(KeyboardInput {
                key_code: code,
                logical_key: logical.clone(),
                state,
                text: None,
                repeat: false,
                window,
            });
            self.step(app, STEP);
        }
        self.response(app, Status::Pressed, None)
    }

    fn settle(&mut self, app: &mut App, steps: Option<u32>) -> Response {
        let limit = steps.unwrap_or(DEFAULT_SETTLE_STEPS).min(MAX_SETTLE_STEPS);
        let mut used = 0;
        loop {
            let pending = self.game.unsettled(app.world_mut());
            let done = pending.is_empty();
            if done || used == limit || self.exit.is_some() {
                let mut response = if done {
                    self.response(app, Status::Settled, None)
                } else {
                    self.response(
                        app,
                        Status::Unsettled,
                        Some(format!(
                            "Still in progress after {used} steps: {}.",
                            pending.join(", ")
                        )),
                    )
                };
                response.json["steps"] = json!(used);
                return response;
            }
            self.step(app, STEP);
            used += 1;
        }
    }

    /// Write a picture of the seat's image. The clock is frozen while the capture is
    /// read back, and the response follows the completed file.
    fn capture(&mut self, app: &mut App, path: &Path) -> Response {
        let Some(image) = self.image.clone() else {
            return self.refused(
                app,
                "This host renders no image; capture needs the windowless seat.".into(),
            );
        };
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            && let Err(error) = std::fs::create_dir_all(parent)
        {
            return self.refused(app, format!("Cannot create {}: {error}", parent.display()));
        }
        let deadline = Instant::now() + CAPTURE_LIMIT;
        // Frozen updates draw anything the last request changed. The readiness flag
        // trails the renderer by a frame, so require it on two consecutive frames.
        let mut ready = 0;
        while ready < 2 {
            if Instant::now() >= deadline || !self.step(app, Duration::ZERO) {
                return self.refused(app, "Render pipelines did not become ready.".into());
            }
            if self.game.pipelines_ready(app.world()) {
                ready += 1;
            } else {
                ready = 0;
                std::thread::sleep(Duration::from_millis(2));
            }
        }
        // The shared capture steps the seat's frozen clock until the file is written,
        // inside the limit the readiness wait began.
        let captured = capture::image_to_file(app, image, path, deadline, |app| {
            self.step(app, Duration::ZERO)
        });
        match captured {
            Ok(()) => {
                let mut response = self.response(app, Status::Captured, None);
                response.json["path"] = json!(path);
                response
            }
            Err(Failure::Unsaved(error)) => {
                self.refused(app, format!("Cannot save the capture: {error}"))
            }
            Err(Failure::Incomplete) => self.refused(
                app,
                format!(
                    "The capture did not complete within {} seconds; no file was written.",
                    CAPTURE_LIMIT.as_secs()
                ),
            ),
        }
    }
}

/// Refuse to start: one `refused` response on standard output, which the agent or a
/// launching command ([`socket::launch`]) reads as the seat's first line. The error
/// is returned for the caller's standard error and unsuccessful exit.
pub fn refuse<T>(error: String) -> Result<T, String> {
    println!(
        "{}",
        json!({"ok": false, "mode": "seat", "status": "refused", "error": error})
    );
    Err(error)
}

fn emit(output: &mut impl Write, value: &Value) -> Result<(), String> {
    serde_json::to_writer(&mut *output, value).map_err(|error| error.to_string())?;
    output.write_all(b"\n").map_err(|error| error.to_string())?;
    output.flush().map_err(|error| error.to_string())
}

/// The graphics adapter that rendered the seat's image, when a renderer exists.
fn adapter(world: &World) -> Value {
    world
        .get_resource::<RenderAdapterInfo>()
        .map_or(Value::Null, |info| {
            json!({
                "name": info.name,
                "backend": format!("{:?}", info.backend),
                "device_type": format!("{:?}", info.device_type),
                "driver": info.driver,
                "driver_info": info.driver_info,
            })
        })
}
