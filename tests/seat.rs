//! The agent seat's protocol as Arkham had it at `0b8e6fb`, before its game-neutral
//! half moved here, on a table that belongs to no game. Arkham's own tests hold the
//! same protocol on its real table. The table here reads keys and mouse buttons
//! through Bevy's own input, and its camera is the real table camera; its picking and
//! its renderer are stand-ins, so no test needs a graphics adapter.
#![cfg(feature = "seat")]

use bevy::{
    app::AppExit,
    asset::RenderAssetUsages,
    camera::RenderTarget,
    input::{
        InputPlugin,
        keyboard::Key,
        mouse::{MouseScrollUnit, MouseWheel},
    },
    picking::{
        backend::HitData,
        hover::HoverMap,
        pointer::{PointerAction, PointerButton, PointerId, PointerInput},
    },
    prelude::*,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        view::screenshot::{Screenshot, ScreenshotCaptured},
    },
    window::PrimaryWindow,
};
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    fs::Permissions,
    io::{BufRead, BufReader, Write},
    os::unix::{
        fs::{FileTypeExt, PermissionsExt},
        net::{UnixListener, UnixStream},
    },
    path::{Path, PathBuf},
    time::Duration,
};
use unfolded_tabletop::{
    camera::{
        Framing, Orbit, Pose, TableCamera, TableCameraPlugin, TableCameraSystems, TableFacts,
    },
    capture::Pipelines,
    seat::{
        Admission, Exited, Focus, Game, Hit, Interrupt, Logical, Name, Observed, STEP, Seat,
        SeatKey, SeatedAgent, Unadmitted, Unshown,
        socket::{Listener, Unclaimable},
    },
};

/// The table's clock after `steps` of the seat's steps, in nanoseconds.
fn clock_after(steps: u32) -> u64 {
    u64::try_from((STEP * steps).as_nanos()).unwrap()
}

/// What the table of no game holds.
#[derive(Resource)]
struct Table {
    revision: u64,
    /// Steps of presentation still to run, during which the table drops input.
    presenting: u32,
    /// The table's own clock, which only the seat advances.
    clock: Duration,
    /// The focused control: its choice identity and its label.
    focus: Option<(Option<String>, String)>,
    /// The pieces and keys that may reach the table while it presents. Clicking or
    /// pressing one ends the presentation and leaves the revision where it was.
    interrupts: Vec<&'static str>,
    /// The keys the table read through Bevy's keyboard input, in order.
    keys: Vec<String>,
    /// The pieces clicked: pressed and released while under the pointer.
    clicked: Vec<String>,
    wheels: Vec<(f32, f32)>,
    scrolls: Vec<(String, f32, f32)>,
    /// The mouse buttons the table read going down and up through Bevy's mouse input.
    buttons: Vec<String>,
    /// Every press, release and cancel picking saw, and every move made while a
    /// button was down, with the piece nearest the pointer.
    gestures: Vec<String>,
    admission: Admission,
    /// What the stand-in render world finds of the pipelines on its next frames, one
    /// frame each. The last finding holds once these run out.
    pipelines: VecDeque<bool>,
    /// How many of those findings were still to come when a capture was first asked
    /// for.
    findings_left_when_asked: Option<usize>,
    /// The table exits, instead of handing the picture back, once a capture is asked for.
    exits_when_asked: bool,
}

impl Default for Table {
    fn default() -> Self {
        Self {
            revision: 0,
            presenting: 0,
            clock: Duration::ZERO,
            focus: None,
            interrupts: Vec::new(),
            keys: Vec::new(),
            clicked: Vec::new(),
            wheels: Vec::new(),
            scrolls: Vec::new(),
            buttons: Vec::new(),
            gestures: Vec::new(),
            admission: Admission::Admitted,
            pipelines: VecDeque::new(),
            findings_left_when_asked: None,
            exits_when_asked: false,
        }
    }
}

/// Something shown on the table that the pointer can be over.
#[derive(Component)]
struct Piece {
    name: &'static str,
    area: Rect,
    /// Nearer pieces have smaller depths.
    depth: f32,
    /// Where a pointer could rest on it, its middle first.
    reach: Vec<Vec2>,
    /// It arrives under the pointer only once something has been pressed.
    late: bool,
}

fn piece(name: &'static str, area: Rect, depth: f32) -> Piece {
    Piece {
        name,
        area,
        depth,
        reach: vec![area.center()],
        late: false,
    }
}

/// The game's side of the seat, for the table of no game.
struct Plain;

impl Game for Plain {
    fn observe(&self, world: &mut World) -> Observed {
        let table = world.resource::<Table>();
        Observed {
            observation: json!({
                "revision": table.revision,
                "keys": table.keys,
                "clicked": table.clicked,
            }),
            table: json!({
                "settled": table.presenting == 0,
                "clock_nanos": u64::try_from(table.clock.as_nanos()).unwrap(),
            }),
        }
    }

    fn revision(&self, world: &mut World) -> u64 {
        world.resource::<Table>().revision
    }

    fn resolve(&self, world: &mut World, name: &Name) -> Result<Vec<Vec2>, String> {
        let Name::Component(wanted) = name else {
            return Err("This table shows no choice.".into());
        };
        let mut pieces = world.query::<&Piece>();
        pieces
            .iter(world)
            .find(|piece| piece.name == wanted)
            .map(|piece| piece.reach.clone())
            .ok_or_else(|| format!("No shown component is named {wanted}."))
    }

    fn identify(&self, world: &mut World, entity: Entity) -> Hit {
        let name = world.get::<Piece>(entity).map(|piece| piece.name);
        Hit {
            name: name.map(|name| Name::Component(name.into())),
            description: json!(name),
        }
    }

    fn focus(&self, world: &mut World) -> Focus {
        match &world.resource::<Table>().focus {
            None => Focus::Nothing,
            Some((choice, label)) => Focus::Control {
                choice: choice.clone(),
                label: label.clone(),
            },
        }
    }

    fn unsettled(&self, world: &mut World) -> Vec<String> {
        if world.resource::<Table>().presenting > 0 {
            vec!["presentation".into()]
        } else {
            Vec::new()
        }
    }

    fn advance(&self, world: &mut World, by: Duration) {
        let mut table = world.resource_mut::<Table>();
        table.clock += by;
        if !by.is_zero() {
            table.presenting = table.presenting.saturating_sub(1);
        }
    }

    fn admission(&self, world: &World) -> Admission {
        world.resource::<Table>().admission.clone()
    }

    fn keys(&self) -> &[SeatKey] {
        &KEYS
    }

    fn interrupts(&self, world: &mut World, input: Interrupt<'_>) -> bool {
        let named = match input {
            Interrupt::Activate(Name::Component(name)) => name.as_str(),
            Interrupt::Activate(Name::Choice(_)) => return false,
            Interrupt::Key(name) => name,
        };
        world.resource::<Table>().interrupts.contains(&named)
    }
}

/// [`Plain`] in all but [`Game::interrupts`], which it leaves to the seat's default.
struct Uninterrupted;

impl Game for Uninterrupted {
    fn observe(&self, world: &mut World) -> Observed {
        Plain.observe(world)
    }

    fn revision(&self, world: &mut World) -> u64 {
        Plain.revision(world)
    }

    fn resolve(&self, world: &mut World, name: &Name) -> Result<Vec<Vec2>, String> {
        Plain.resolve(world, name)
    }

    fn identify(&self, world: &mut World, entity: Entity) -> Hit {
        Plain.identify(world, entity)
    }

    fn unsettled(&self, world: &mut World) -> Vec<String> {
        Plain.unsettled(world)
    }

    fn advance(&self, world: &mut World, by: Duration) {
        Plain.advance(world, by);
    }

    fn admission(&self, world: &World) -> Admission {
        Plain.admission(world)
    }

    fn focus(&self, world: &mut World) -> Focus {
        Plain.focus(world)
    }

    fn keys(&self) -> &[SeatKey] {
        Plain.keys()
    }
}

/// The keys this table reads, which are the keys the seat may press at it. Enter,
/// Space and Y commit here: which keys do is each table's own.
static KEYS: [SeatKey; 6] = [
    key("Tab", KeyCode::Tab, Logical::Named(Key::Tab), false),
    key("Enter", KeyCode::Enter, Logical::Named(Key::Enter), true),
    key("Space", KeyCode::Space, Logical::Named(Key::Space), true),
    key("KeyY", KeyCode::KeyY, Logical::Typed("y"), true),
    key("Home", KeyCode::Home, Logical::Named(Key::Home), false),
    key("End", KeyCode::End, Logical::Named(Key::End), false),
];

const fn key(name: &'static str, code: KeyCode, logical: Logical, commits: bool) -> SeatKey {
    SeatKey {
        name,
        code,
        logical,
        commits,
    }
}

/// The table's own key handling, behind Bevy's keyboard input. Enter and Space
/// commit, which moves the revision on. End is this table's way to end with an
/// error, and Home its way to end well.
fn read_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut table: ResMut<Table>,
    mut exit: MessageWriter<AppExit>,
) {
    for key in keys.get_just_pressed() {
        let name = format!("{key:?}");
        if table.interrupts.contains(&name.as_str()) {
            table.presenting = 0;
        }
        table.keys.push(name);
        match key {
            KeyCode::Enter | KeyCode::Space | KeyCode::KeyY => table.revision += 1,
            KeyCode::End => {
                exit.write(AppExit::error());
            }
            KeyCode::Home => {
                exit.write(AppExit::Success);
            }
            _ => {}
        }
    }
}

/// Stands in for Bevy's picking, which needs a renderer: what is under the pointer
/// after each pointer message, and a click when a primary press and its release land
/// on the same nearest piece.
fn pick(
    mut inputs: MessageReader<PointerInput>,
    mut hover: ResMut<HoverMap>,
    pieces: Query<(Entity, &Piece)>,
    mut table: ResMut<Table>,
    mut held: Local<Option<&'static str>>,
    mut pressed_once: Local<bool>,
    mut down: Local<Vec<PointerButton>>,
) {
    for input in inputs.read() {
        if matches!(input.action, PointerAction::Press(_)) {
            *pressed_once = true;
        }
        let at = input.location.position;
        // Every piece blocks what lies under it, so only the nearest is hovered.
        let nearest = pieces
            .iter()
            .filter(|(_, piece)| piece.area.contains(at) && (!piece.late || *pressed_once))
            .min_by(|a, b| a.1.depth.total_cmp(&b.1.depth));
        hover.insert(
            input.pointer_id,
            nearest
                .iter()
                .map(|(entity, piece)| {
                    (
                        *entity,
                        HitData::new(Entity::PLACEHOLDER, piece.depth, None, None),
                    )
                })
                .collect(),
        );
        let nearest = nearest.map(|(_, piece)| piece.name);
        let over = nearest.unwrap_or("nothing");
        match input.action {
            PointerAction::Press(button) => {
                down.push(button);
                table.gestures.push(format!("press {button:?} on {over}"));
            }
            PointerAction::Release(button) => {
                down.retain(|held| *held != button);
                table.gestures.push(format!("release {button:?} on {over}"));
            }
            PointerAction::Move { .. } if !down.is_empty() => {
                table.gestures.push(format!("move on {over}"));
            }
            PointerAction::Cancel => table.gestures.push("cancel".into()),
            _ => {}
        }
        match input.action {
            PointerAction::Press(PointerButton::Primary) => *held = nearest,
            PointerAction::Release(PointerButton::Primary) => {
                if let Some(name) = held.take().filter(|name| Some(*name) == nearest) {
                    table.clicked.push(name.into());
                    if table.interrupts.contains(&name) {
                        table.presenting = 0;
                    } else {
                        table.revision += 1;
                    }
                }
            }
            PointerAction::Cancel => *held = None,
            PointerAction::Scroll { unit, x, y, .. } => {
                assert_eq!(unit, MouseScrollUnit::Line);
                table.scrolls.push((nearest.unwrap().into(), x, y));
            }
            _ => {}
        }
    }
}

/// The table's own mouse-button handling, behind Bevy's mouse input.
fn read_buttons(buttons: Res<ButtonInput<MouseButton>>, mut table: ResMut<Table>) {
    for button in buttons.get_just_pressed() {
        table.buttons.push(format!("{button:?} down"));
    }
    for button in buttons.get_just_released() {
        table.buttons.push(format!("{button:?} up"));
    }
}

fn read_wheel(mut events: MessageReader<MouseWheel>, mut table: ResMut<Table>) {
    for event in events.read() {
        assert_eq!(event.unit, MouseScrollUnit::Line);
        table.wheels.push((event.x, event.y));
    }
}

/// Stands in for Bevy's render world: what it finds of the pipelines after a frame.
fn render_world_reports(mut table: ResMut<Table>, pipelines: Res<Pipelines>) {
    if let Some(ready) = table.pipelines.pop_front() {
        pipelines.report(ready);
    }
}

/// Stands in for Bevy's renderer: hand a picture back to every capture asked for.
fn renderer_hands_back(
    mut commands: Commands,
    asked: Query<Entity, With<Screenshot>>,
    mut table: ResMut<Table>,
    mut exit: MessageWriter<AppExit>,
) {
    for entity in &asked {
        let left = table.pipelines.len();
        table.findings_left_when_asked.get_or_insert(left);
        if table.exits_when_asked {
            exit.write(AppExit::Success);
            continue;
        }
        commands.trigger(ScreenshotCaptured {
            entity,
            image: Image::new_fill(
                Extent3d {
                    width: 2,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                &[255, 0, 0, 255],
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::MAIN_WORLD,
            ),
        });
    }
}

struct Sitting<G = Plain> {
    app: App,
    seat: Seat<G>,
}

impl<G: Game> Sitting<G> {
    /// The response to `request`, from a table that has not exited.
    fn ask(&mut self, request: Value) -> Value {
        self.seat
            .respond(&mut self.app, &request.to_string())
            .unwrap_or_else(|exited| panic!("{request}: {exited}"))
    }

    /// The exit that answers `request`, which the table must have made.
    fn expect_exit(&mut self, request: Value) -> Exited {
        self.seat
            .respond(&mut self.app, &request.to_string())
            .expect_err("the table should have exited")
    }

    fn table(&mut self) -> Mut<'_, Table> {
        self.app.world_mut().resource_mut::<Table>()
    }

    fn show(&mut self, piece: Piece) {
        self.app.world_mut().spawn(piece);
    }
}

/// The table with no renderer, and an agent seated at it. With `image`, the seat has
/// an image to capture and the stand-in renderer hands pictures back.
fn sit_with(image: Option<Handle<Image>>) -> Sitting {
    sit_at(Plain, image)
}

/// [`sit_with`], with `game` as the game's side of the seat.
fn sit_at<G: Game>(game: G, image: Option<Handle<Image>>) -> Sitting<G> {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin))
        .add_message::<PointerInput>()
        .init_resource::<HoverMap>()
        .init_resource::<Table>()
        .add_systems(Update, (read_keys, pick, read_wheel, read_buttons));
    if image.is_some() {
        app.add_systems(Update, (render_world_reports, renderer_hands_back).chain());
    }
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    let camera = app.world_mut().spawn(RenderTarget::default()).id();
    let seat = Seat::new(game, &mut app, camera, image);
    Sitting { app, seat }
}

fn sit() -> Sitting {
    sit_with(None)
}

/// A directory of this test's own, removed when the test ends.
struct Scratch(PathBuf);

impl Scratch {
    fn new(test: &str) -> Self {
        let path = std::env::temp_dir().join(format!("ut-seat-{test}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn join(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn lines(output: &[u8]) -> Vec<Value> {
    output
        .lines()
        .map(|line| serde_json::from_str(&line.unwrap()).unwrap())
        .collect()
}

#[test]
fn the_seats_step_is_one_sixtieth_of_a_second() {
    // 1,000,000,000 ns / 60 = 16,666,666.67 ns, rounded up, as Arkham had it.
    assert_eq!(STEP, Duration::from_nanos(16_666_667));
}

#[test]
fn sitting_marks_the_table_and_an_observation_is_the_games_own() {
    let mut seat = sit();
    assert!(seat.app.world().contains_resource::<SeatedAgent>());

    let response = seat.ask(json!({"op": "observe"}));

    assert_eq!(
        response,
        json!({
            "ok": true, "mode": "seat", "status": "observed",
            "observation": {"revision": 0, "keys": [], "clicked": []},
            "table": {"settled": true, "clock_nanos": 0},
            "revision": 0, "settled": true, "pending": [],
        })
    );
}

#[test]
fn a_key_is_pressed_and_released_through_bevys_keyboard_input_in_two_fixed_steps() {
    let mut seat = sit();

    let response = seat.ask(json!({"op": "key", "revision": 0, "key": "Tab"}));

    assert_eq!(response["status"], "pressed", "{response}");
    assert_eq!(response["ok"], true);
    // Read once: the key went down in one update and came up in the next.
    assert_eq!(response["observation"]["keys"], json!(["Tab"]));
    assert_eq!(response["table"]["clock_nanos"], clock_after(2));
}

#[test]
fn a_request_decided_on_another_revision_is_refused_and_nothing_is_pressed() {
    let mut seat = sit();

    let response = seat.ask(json!({"op": "key", "revision": 7, "key": "Tab"}));

    assert_eq!(response["status"], "refused");
    assert_eq!(response["ok"], false);
    assert_eq!(
        response["error"],
        "Outdated: the request names revision 7, but the attempt is at revision 0. Observe again."
    );
    assert_eq!(response["observation"]["keys"], json!([]));
    assert_eq!(response["table"]["clock_nanos"], 0);
}

#[test]
fn a_key_that_can_commit_presses_only_the_control_it_names() {
    let mut seat = sit();

    let unnamed = seat.ask(json!({"op": "key", "revision": 0, "key": "Enter"}));
    assert_eq!(unnamed["status"], "refused");
    assert_eq!(
        unnamed["error"],
        "Enter can commit a choice or dismiss a reading, so its request must name the control it expects to press in \"focused\": the control's choice identity, or its label when it has none. Nothing was pressed; nothing has focus."
    );

    seat.table().focus = Some((Some("turn:end".into()), "End turn".into()));
    let other =
        seat.ask(json!({"op": "key", "revision": 0, "key": "Enter", "focused": "Continue"}));
    assert_eq!(other["status"], "refused");
    assert_eq!(
        other["error"],
        "Enter names \"Continue\", but focus is on choice turn:end (\"End turn\"). Nothing was pressed."
    );
    // A control with a choice identity is not named by its label.
    let label =
        seat.ask(json!({"op": "key", "revision": 0, "key": "Space", "focused": "End turn"}));
    assert_eq!(label["status"], "refused");
    assert_eq!(label["observation"]["keys"], json!([]));

    let named =
        seat.ask(json!({"op": "key", "revision": 0, "key": "Space", "focused": "turn:end"}));
    assert_eq!(named["status"], "pressed", "{named}");
    assert_eq!(named["observation"]["keys"], json!(["Space"]));
    assert_eq!(named["observation"]["revision"], 1);

    // A control without a choice identity is named by its label, and any key given
    // `focused` is checked the same way.
    seat.table().focus = Some((None, "Continue".into()));
    let dismissed =
        seat.ask(json!({"op": "key", "revision": 1, "key": "Enter", "focused": "Continue"}));
    assert_eq!(dismissed["status"], "pressed", "{dismissed}");
    let checked = seat.ask(json!({"op": "key", "revision": 2, "key": "Tab", "focused": "Menu"}));
    assert_eq!(checked["status"], "refused");
    assert_eq!(
        checked["error"],
        "Tab names \"Menu\", but focus is on \"Continue\", which has no choice identity. Nothing was pressed."
    );
}

#[test]
fn a_key_the_table_does_not_read_is_refused_with_the_keys_it_does() {
    let mut seat = sit();

    // Bevy knows Escape, and this table does not read it.
    let response = seat.ask(json!({"op": "key", "revision": 0, "key": "Escape"}));

    assert_eq!(response["status"], "refused");
    assert_eq!(
        response["error"],
        "Unknown key \"Escape\". Known keys: Tab, Enter, Space, KeyY, Home, End."
    );
    assert_eq!(response["observation"]["keys"], json!([]));
}

#[test]
fn which_keys_can_commit_is_the_tables_own() {
    let mut seat = sit();
    seat.table().focus = Some((Some("wager:yes".into()), "Yes".into()));

    // Y commits at this table, so it must name the control it expects to press.
    let unnamed = seat.ask(json!({"op": "key", "revision": 0, "key": "KeyY"}));
    assert_eq!(unnamed["status"], "refused");
    assert_eq!(
        unnamed["error"],
        "KeyY can commit a choice or dismiss a reading, so its request must name the control it expects to press in \"focused\": the control's choice identity, or its label when it has none. Nothing was pressed; focus is on choice wager:yes (\"Yes\")."
    );
    assert_eq!(unnamed["observation"]["keys"], json!([]));

    let named =
        seat.ask(json!({"op": "key", "revision": 0, "key": "KeyY", "focused": "wager:yes"}));
    assert_eq!(named["status"], "pressed", "{named}");
    assert_eq!(named["observation"]["keys"], json!(["KeyY"]));
    assert_eq!(named["observation"]["revision"], 1);
}

#[test]
fn a_busy_table_refuses_input_and_settles_in_fixed_steps() {
    let mut seat = sit();
    seat.table().presenting = 5;

    let refused = seat.ask(json!({"op": "key", "revision": 0, "key": "Tab"}));
    assert_eq!(refused["status"], "refused");
    assert_eq!(
        refused["error"],
        "The table is busy (presentation); settle first."
    );
    assert_eq!(refused["observation"]["keys"], json!([]));

    let unsettled = seat.ask(json!({"op": "settle", "steps": 2}));
    assert_eq!(unsettled["status"], "unsettled");
    assert_eq!(unsettled["ok"], false);
    assert_eq!(unsettled["steps"], 2);
    assert_eq!(
        unsettled["error"],
        "Still in progress after 2 steps: presentation."
    );

    let settled = seat.ask(json!({"op": "settle"}));
    assert_eq!(settled["status"], "settled", "{settled}");
    assert_eq!(settled["steps"], 3);
    assert_eq!(settled["table"]["clock_nanos"], clock_after(5));
}

#[test]
fn an_advance_steps_a_settled_tables_clock_exactly_as_asked() {
    let mut seat = sit();
    // Settling a settled table takes no step, so it cannot move the clock.
    assert_eq!(seat.ask(json!({"op": "settle"}))["steps"], 0);

    let advanced = seat.ask(json!({"op": "advance", "steps": 216}));

    assert_eq!(advanced["status"], "advanced", "{advanced}");
    assert_eq!(advanced["ok"], true);
    assert_eq!(advanced["steps"], 216);
    assert_eq!(advanced["settled"], true);
    // 216 × 16,666,667 ns: 3.6 s and 72 ns.
    assert_eq!(advanced["table"]["clock_nanos"], 3_600_000_072_u64);
    assert_eq!(advanced["observation"]["keys"], json!([]));
    assert_eq!(advanced["observation"]["clicked"], json!([]));
}

#[test]
fn an_advance_steps_a_busy_table_as_asked_and_does_not_wait_for_it_to_settle() {
    let mut seat = sit();
    seat.table().presenting = 5;

    let short = seat.ask(json!({"op": "advance", "steps": 2}));
    assert_eq!(short["status"], "advanced", "{short}");
    assert_eq!(short["ok"], true);
    assert_eq!(short["steps"], 2);
    assert_eq!(short["settled"], false);
    assert_eq!(short["pending"], json!(["presentation"]));
    assert_eq!(short["table"]["clock_nanos"], clock_after(2));

    // It settles after three more steps, and the advance goes on past that.
    let past = seat.ask(json!({"op": "advance", "steps": 10}));
    assert_eq!(past["steps"], 10);
    assert_eq!(past["settled"], true);
    assert_eq!(past["table"]["clock_nanos"], clock_after(12));
}

#[test]
fn an_advance_of_no_steps_too_many_or_none_said_is_refused_and_steps_nothing() {
    let mut seat = sit();
    for steps in [0, 7_201] {
        let refused = seat.ask(json!({"op": "advance", "steps": steps}));
        assert_eq!(refused["status"], "refused", "{refused}");
        assert_eq!(
            refused["error"],
            format!(
                "advance needs between 1 and 7200 steps, not {steps}. The table was not stepped."
            )
        );
        assert_eq!(refused["table"]["clock_nanos"], 0);
    }
    // A count left out is refused as the line is read, naming the missing field.
    let unsaid = seat.ask(json!({"op": "advance"}));
    assert_eq!(unsaid["status"], "refused");
    assert!(
        unsaid["error"]
            .as_str()
            .unwrap()
            .contains("missing field `steps`"),
        "{unsaid}"
    );
    assert_eq!(unsaid["table"]["clock_nanos"], 0);
    let most = seat.ask(json!({"op": "advance", "steps": 7_200}));
    assert_eq!(most["steps"], 7_200);
    assert_eq!(most["table"]["clock_nanos"], clock_after(7_200));
}

#[test]
fn an_advance_sends_no_input_so_a_refusal_before_it_does_not_skip_it() {
    let mut seat = sit();
    let input = concat!(
        "{\"op\":\"key\",\"revision\":0,\"key\":\"F1\"}\n",
        "{\"op\":\"advance\",\"steps\":3}\n",
        "{\"op\":\"quit\"}\n",
    );
    let mut output = Vec::new();

    let ended = seat
        .seat
        .serve(&mut seat.app, input.as_bytes(), &mut output);

    assert_eq!(ended, Ok(()));
    let responses = lines(&output);
    let statuses: Vec<&str> = responses
        .iter()
        .map(|response| response["status"].as_str().unwrap())
        .collect();
    assert_eq!(statuses, ["ready", "refused", "advanced", "bye"]);
    assert_eq!(responses[2]["steps"], 3);
    assert_eq!(responses[2]["table"]["clock_nanos"], clock_after(3));
}

#[test]
fn batch_form_is_ready_first_and_stops_acting_after_its_first_refusal() {
    let mut seat = sit();
    let input = concat!(
        "{\"op\":\"key\",\"revision\":0,\"key\":\"F1\"}\n",
        "\n",
        "{\"op\":\"key\",\"revision\":0,\"key\":\"Tab\"}\n",
        "{\"op\":\"observe\"}\n",
        "{\"op\":\"settle\"}\n",
        "{\"op\":\"quit\"}\n",
        "{\"op\":\"observe\"}\n",
    );
    let mut output = Vec::new();

    let ended = seat
        .seat
        .serve(&mut seat.app, input.as_bytes(), &mut output);

    assert_eq!(ended, Ok(()));
    let responses = lines(&output);
    let statuses: Vec<&str> = responses
        .iter()
        .map(|response| response["status"].as_str().unwrap())
        .collect();
    // Nothing follows `quit`.
    assert_eq!(
        statuses,
        ["ready", "refused", "skipped", "observed", "settled", "bye"]
    );
    // No renderer drew this table.
    assert_eq!(responses[0]["adapter"], Value::Null);
    assert_eq!(responses[2]["ok"], false);
    assert_eq!(
        responses[2]["error"],
        format!(
            "Skipped: an earlier request on this connection was refused, so its activations, drags, keys and scrolls are no longer acted on. No input was sent. Refused request: {{\"op\":\"key\",\"revision\":0,\"key\":\"F1\"}}. Reason: {}",
            responses[1]["error"].as_str().unwrap()
        )
    );
    assert_eq!(responses[5]["observation"]["keys"], json!([]));
}

#[test]
fn a_line_that_is_no_request_is_refused() {
    let mut seat = sit();

    for line in [
        "nonsense",
        "{\"op\":\"dance\"}",
        "{\"op\":\"observe\",\"more\":1}",
    ] {
        let response = seat.seat.respond(&mut seat.app, line).unwrap();
        assert_eq!(response["status"], "refused", "{line}");
    }
    let both = seat.ask(json!({"op": "activate", "revision": 0, "component": "a", "choice": "b"}));
    assert_eq!(
        both["error"],
        "activate needs exactly one of component or choice"
    );
}

#[test]
fn a_presenting_table_takes_the_activation_and_key_it_lets_interrupt_it() {
    let mut seat = sit();
    seat.show(piece("tray", Rect::new(0., 0., 20., 20.), 1.));
    seat.table().interrupts = vec!["tray", "Tab"];

    seat.table().presenting = 100;
    let clicked = seat.ask(json!({"op": "activate", "revision": 0, "component": "tray"}));
    assert_eq!(clicked["status"], "activated", "{clicked}");
    assert_eq!(clicked["observation"]["clicked"], json!(["tray"]));
    // The click ended the presentation and acted on nothing, so the reply reads a
    // settled table at the same revision.
    assert_eq!(clicked["revision"], 0);
    assert_eq!(clicked["settled"], true, "{clicked}");
    assert_eq!(clicked["pending"], json!([]));

    seat.table().presenting = 100;
    let pressed = seat.ask(json!({"op": "key", "revision": 0, "key": "Tab"}));
    assert_eq!(pressed["status"], "pressed", "{pressed}");
    assert_eq!(pressed["observation"]["keys"], json!(["Tab"]));
    assert_eq!(pressed["settled"], true, "{pressed}");
}

#[test]
fn a_presenting_table_refuses_what_it_does_not_let_interrupt_it_and_any_stale_or_unnamed_press() {
    let mut seat = sit();
    seat.show(piece("tray", Rect::new(0., 0., 20., 20.), 1.));
    seat.show(piece("card", Rect::new(40., 0., 60., 20.), 1.));
    seat.table().interrupts = vec!["tray", "Tab", "Enter"];
    seat.table().presenting = 100;
    let busy = "The table is busy (presentation); settle first.";

    for request in [
        json!({"op": "activate", "revision": 0, "component": "card"}),
        json!({"op": "key", "revision": 0, "key": "Home"}),
        // Only an activation or a key interrupts, even on the tray.
        json!({"op": "point", "revision": 0, "component": "tray"}),
        json!({"op": "scroll", "revision": 0, "component": "tray", "x": 0.0, "y": 1.0}),
        json!({"op": "drag", "revision": 0, "component": "tray", "button": "left", "x": 30.0, "y": 0.0}),
    ] {
        let refused = seat.ask(request.clone());
        assert_eq!(refused["status"], "refused", "{request}: {refused}");
        assert_eq!(refused["error"], busy, "{request}");
    }

    let stale = seat.ask(json!({"op": "activate", "revision": 3, "component": "tray"}));
    assert_eq!(stale["status"], "refused");
    assert_eq!(
        stale["error"],
        "Outdated: the request names revision 3, but the attempt is at revision 0. Observe again."
    );
    let stale = seat.ask(json!({"op": "key", "revision": 3, "key": "Tab"}));
    assert_eq!(stale["status"], "refused");
    assert!(
        stale["error"].as_str().unwrap().starts_with("Outdated:"),
        "{stale}"
    );

    // Enter commits at this table, so it still names the control it expects to press.
    let unnamed = seat.ask(json!({"op": "key", "revision": 0, "key": "Enter"}));
    assert_eq!(unnamed["status"], "refused");
    assert!(
        unnamed["error"]
            .as_str()
            .unwrap()
            .starts_with("Enter can commit a choice"),
        "{unnamed}"
    );

    let seen = seat.ask(json!({"op": "observe"}));
    assert_eq!(seen["observation"]["clicked"], json!([]), "{seen}");
    assert_eq!(seen["observation"]["keys"], json!([]));
    assert_eq!(seen["table"]["clock_nanos"], 0, "nothing stepped");
    assert_eq!(seen["settled"], false);
}

#[test]
fn with_no_interrupts_of_its_own_a_presenting_table_refuses_every_activation_and_key_as_busy() {
    let mut seat = sit_at(Uninterrupted, None);
    seat.show(piece("tray", Rect::new(0., 0., 20., 20.), 1.));
    // What the table itself would take; only the game can let it through.
    seat.table().interrupts = vec!["tray", "Tab"];
    seat.table().presenting = 100;

    for request in [
        json!({"op": "activate", "revision": 0, "component": "tray"}),
        json!({"op": "key", "revision": 0, "key": "Tab"}),
    ] {
        let refused = seat.ask(request.clone());
        assert_eq!(refused["status"], "refused", "{request}: {refused}");
        assert_eq!(
            refused["error"], "The table is busy (presentation); settle first.",
            "{request}"
        );
    }
    let seen = seat.ask(json!({"op": "observe"}));
    assert_eq!(seen["observation"]["clicked"], json!([]), "{seen}");
    assert_eq!(seen["observation"]["keys"], json!([]));
    assert_eq!(seen["settled"], false);
}

#[test]
fn an_activation_rests_on_the_named_component_then_presses_and_releases() {
    let mut seat = sit();
    seat.show(piece("card", Rect::new(0., 0., 20., 20.), 1.));

    let absent = seat.ask(json!({"op": "activate", "revision": 0, "component": "coin"}));
    assert_eq!(absent["status"], "refused");
    assert_eq!(absent["error"], "No shown component is named coin.");

    let activated = seat.ask(json!({"op": "activate", "revision": 0, "component": "card"}));

    assert_eq!(activated["status"], "activated", "{activated}");
    assert_eq!(activated["position"], json!([10.0, 10.0]));
    assert_eq!(activated["covered"], Value::Null);
    assert_eq!(activated["observation"]["clicked"], json!(["card"]));
    // One step to rest the pointer, one to press and one to release.
    assert_eq!(activated["table"]["clock_nanos"], clock_after(3));
}

#[test]
fn a_point_rests_the_pointer_on_the_named_component_and_presses_nothing() {
    let mut seat = sit();
    seat.show(piece("card", Rect::new(0., 0., 20., 20.), 1.));

    let absent = seat.ask(json!({"op": "point", "revision": 0, "component": "coin"}));
    assert_eq!(absent["status"], "refused");
    assert_eq!(absent["error"], "No shown component is named coin.");
    let outdated = seat.ask(json!({"op": "point", "revision": 3, "component": "card"}));
    assert_eq!(
        outdated["error"],
        "Outdated: the request names revision 3, but the attempt is at revision 0. Observe again."
    );
    let both = seat.ask(json!({"op": "point", "revision": 0, "component": "a", "choice": "b"}));
    assert_eq!(
        both["error"],
        "point needs exactly one of component or choice"
    );

    let pointed = seat.ask(json!({"op": "point", "revision": 0, "component": "card"}));

    assert_eq!(pointed["status"], "pointed", "{pointed}");
    assert_eq!(pointed["ok"], true);
    assert_eq!(pointed["position"], json!([10.0, 10.0]));
    // Everything under the pointer, nearest first, as the game describes it.
    assert_eq!(pointed["hit"], json!(["card"]));
    assert_eq!(pointed["covered"], Value::Null);
    assert_eq!(pointed["observation"]["clicked"], json!([]));
    assert_eq!(pointed["observation"]["revision"], 0);
    // One step to rest the pointer, and none to press.
    assert_eq!(pointed["table"]["clock_nanos"], clock_after(1));
    // The pointer stays where it rests: the table sees it over the card.
    let hovered = seat.app.world().resource::<HoverMap>();
    let over: Vec<Entity> = hovered[&PointerId::Mouse].keys().copied().collect();
    let mut pieces = seat.app.world_mut().query::<(Entity, &Piece)>();
    let card = pieces
        .iter(seat.app.world())
        .find(|(_, piece)| piece.name == "card");
    assert_eq!(over, [card.unwrap().0]);
}

#[test]
fn scrolling_delivers_normal_wheel_and_pointer_input_without_clicking() {
    let mut seat = sit();
    seat.show(piece("card", Rect::new(0., 0., 20., 20.), 1.));
    let response =
        seat.ask(json!({"op":"scroll", "revision":0, "component":"card", "x":2, "y":-4}));
    assert_eq!(response["status"], "scrolled", "{response}");
    assert_eq!(response["position"], json!([10., 10.]));
    assert_eq!(response["delta"], json!({"unit":"line", "x":2., "y":-4.}));
    assert_eq!(seat.table().wheels, vec![(2., -4.)]);
    assert_eq!(seat.table().scrolls, vec![("card".into(), 2., -4.)]);
    assert!(seat.table().clicked.is_empty());
    assert_eq!(seat.table().revision, 0);
}

#[test]
fn scrolling_refuses_invalid_deltas_names_stale_busy_and_covered_targets() {
    let mut seat = sit();
    seat.show(piece("card", Rect::new(0., 0., 20., 20.), 1.));
    for request in [
        json!({"op":"scroll", "revision":0, "component":"card", "y":0}),
        json!({"op":"scroll", "revision":0, "component":"card", "y":33}),
        json!({"op":"scroll", "revision":0, "component":"card", "x":-33, "y":1}),
        json!({"op":"scroll", "revision":0, "component":"card", "y":1e40}),
        json!({"op":"scroll", "revision":0, "y":1}),
        json!({"op":"scroll", "revision":0, "component":"card", "choice":"x", "y":1}),
        json!({"op":"scroll", "revision":0, "component":"absent", "y":1}),
        json!({"op":"scroll", "revision":1, "component":"card", "y":1}),
    ] {
        assert_eq!(seat.ask(request)["status"], "refused");
    }
    seat.table().presenting = 2;
    assert_eq!(
        seat.ask(json!({"op":"scroll", "revision":0, "component":"card", "y":1}))["status"],
        "refused"
    );
    seat.table().presenting = 0;
    seat.show(piece("modal", Rect::new(0., 0., 20., 20.), 0.));
    assert_eq!(
        seat.ask(json!({"op":"scroll", "revision":0, "component":"card", "y":1}))["status"],
        "missed"
    );
    assert!(seat.table().wheels.is_empty());
    assert!(seat.table().scrolls.is_empty());
    assert!(seat.table().clicked.is_empty());
}

#[test]
fn a_scroll_miss_stops_later_scrolls_and_actions_on_that_connection() {
    let mut seat = sit();
    seat.show(piece("card", Rect::new(0., 0., 20., 20.), 1.));
    seat.show(piece("modal", Rect::new(0., 0., 20., 20.), 0.));
    let input = concat!(
        "{\"op\":\"scroll\",\"revision\":0,\"component\":\"card\",\"y\":-4}\n",
        "{\"op\":\"scroll\",\"revision\":0,\"component\":\"modal\",\"y\":-4}\n",
        "{\"op\":\"activate\",\"revision\":0,\"component\":\"modal\"}\n",
        "{\"op\":\"key\",\"revision\":0,\"key\":\"Tab\"}\n",
        "{\"op\":\"observe\"}\n",
    );
    let mut output = Vec::new();
    seat.seat
        .serve(&mut seat.app, input.as_bytes(), &mut output)
        .unwrap();
    let responses = lines(&output);
    assert_eq!(
        responses
            .iter()
            .map(|r| r["status"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "ready", "missed", "skipped", "skipped", "skipped", "observed"
        ]
    );
    assert!(seat.table().wheels.is_empty());
    assert!(seat.table().scrolls.is_empty());
    assert!(seat.table().clicked.is_empty());
    assert_eq!(
        seat.ask(json!({"op":"scroll", "revision":0, "component":"modal", "y":-4}))["status"],
        "scrolled"
    );
}

#[test]
fn a_drag_presses_moves_and_releases_one_button_through_mouse_and_picking_input() {
    let mut seat = sit();
    seat.show(piece("felt", Rect::new(0., 0., 40., 40.), 1.));

    let response = seat.ask(
        json!({"op":"drag", "revision":0, "component":"felt", "button":"right", "x":10, "y":-8}),
    );

    assert_eq!(response["status"], "dragged", "{response}");
    assert_eq!(response["ok"], true);
    assert_eq!(response["button"], "right");
    assert_eq!(response["from"], json!([20., 20.]));
    assert_eq!(response["to"], json!([30., 12.]));
    // Bevy's mouse input saw the button go down once and come up once.
    assert_eq!(seat.table().buttons, ["Right down", "Right up"]);
    let world = seat.app.world_mut();
    assert_eq!(
        world
            .resource::<ButtonInput<MouseButton>>()
            .get_pressed()
            .count(),
        0
    );
    // Picking saw one press, a move each step while it was held, and one release.
    assert_eq!(
        seat.table().gestures,
        [
            "press Secondary on felt",
            "move on felt",
            "move on felt",
            "move on felt",
            "move on felt",
            "release Secondary on felt",
        ]
    );
    let world = seat.app.world_mut();
    let window = world
        .query_filtered::<&Window, With<PrimaryWindow>>()
        .single(world)
        .unwrap();
    assert_eq!(window.cursor_position(), Some(Vec2::new(30., 12.)));
    // A secondary press and release is no click.
    assert!(seat.table().clicked.is_empty());
    assert_eq!(seat.table().revision, 0);

    // Each named button is its own: middle is picking's middle, left its primary.
    seat.table().gestures.clear();
    let middle =
        seat.ask(json!({"op":"drag", "revision":0, "component":"felt", "button":"middle", "y":4}));
    assert_eq!(middle["status"], "dragged", "{middle}");
    assert_eq!(seat.table().gestures[0], "press Middle on felt");
    seat.table().gestures.clear();
    let left =
        seat.ask(json!({"op":"drag", "revision":0, "component":"felt", "button":"left", "x":4}));
    assert_eq!(left["status"], "dragged", "{left}");
    assert_eq!(seat.table().gestures[0], "press Primary on felt");
    assert_eq!(
        seat.table().buttons,
        [
            "Right down",
            "Right up",
            "Middle down",
            "Middle up",
            "Left down",
            "Left up"
        ]
    );
}

#[test]
fn a_drag_refuses_unknown_buttons_empty_or_outside_offsets_names_stale_and_busy_tables() {
    let mut seat = sit();
    seat.show(piece("felt", Rect::new(0., 0., 40., 40.), 1.));
    for (request, error) in [
        (
            json!({"op":"drag", "revision":0, "component":"felt", "button":"back", "x":5}),
            "drag needs a button of left, right or middle, not \"back\". Nothing was pressed.",
        ),
        (
            json!({"op":"drag", "revision":0, "component":"felt", "button":"right"}),
            "drag needs a nonzero finite offset in logical pixels. Nothing was pressed.",
        ),
        (
            json!({"op":"drag", "revision":0, "component":"felt", "button":"right", "y":1e40}),
            "drag needs a nonzero finite offset in logical pixels. Nothing was pressed.",
        ),
        (
            json!({"op":"drag", "revision":0, "button":"right", "x":5}),
            "drag needs exactly one of component or choice",
        ),
        (
            json!({"op":"drag", "revision":0, "component":"felt", "choice":"x", "button":"right", "x":5}),
            "drag needs exactly one of component or choice",
        ),
        (
            json!({"op":"drag", "revision":0, "component":"felt", "button":"right", "x":-30}),
            "The drag would end at (-10, 20), outside the 1280 by 720 window. Nothing was pressed.",
        ),
        // A window reports no cursor on its far edge.
        (
            json!({"op":"drag", "revision":0, "component":"felt", "button":"right", "x":1260}),
            "The drag would end at (1280, 20), outside the 1280 by 720 window. Nothing was pressed.",
        ),
        (
            json!({"op":"drag", "revision":1, "component":"felt", "button":"right", "x":5}),
            "Outdated: the request names revision 1, but the attempt is at revision 0. Observe again.",
        ),
    ] {
        let response = seat.ask(request.clone());
        assert_eq!(response["status"], "refused", "{request}");
        assert_eq!(response["error"], error, "{request}");
    }
    seat.table().presenting = 2;
    assert_eq!(
        seat.ask(json!({"op":"drag", "revision":0, "component":"felt", "button":"right", "x":5}))["status"],
        "refused"
    );
    seat.table().presenting = 0;
    // Something over the target when the pointer rests: nothing is pressed.
    seat.show(piece("modal", Rect::new(0., 0., 40., 40.), 0.));
    let missed =
        seat.ask(json!({"op":"drag", "revision":0, "component":"felt", "button":"right", "x":5}));
    assert_eq!(missed["status"], "missed", "{missed}");
    assert_eq!(missed["pressed"], false);
    assert_eq!(
        missed["error"],
        "The pointer at (20, 20) is over \"modal\", not component felt. Nothing was pressed."
    );
    assert!(seat.table().buttons.is_empty());
    assert!(seat.table().gestures.is_empty());
}

#[test]
fn a_drag_whose_press_lands_on_something_else_lets_go_without_moving() {
    let mut seat = sit();
    seat.show(piece("felt", Rect::new(0., 0., 40., 40.), 1.));
    seat.show(Piece {
        late: true,
        ..piece("curtain", Rect::new(0., 0., 40., 40.), 0.)
    });

    let missed =
        seat.ask(json!({"op":"drag", "revision":0, "component":"felt", "button":"right", "x":5}));

    assert_eq!(missed["status"], "missed", "{missed}");
    assert_eq!(missed["pressed"], true);
    assert_eq!(
        missed["error"],
        "The press at (20, 20) landed on \"curtain\", not component felt."
    );
    assert_eq!(seat.table().buttons, ["Right down", "Right up"]);
    assert_eq!(
        seat.table().gestures,
        [
            "press Secondary on curtain",
            "cancel",
            "release Secondary on curtain"
        ]
    );
}

#[test]
fn a_drag_is_skipped_after_a_refusal_and_its_miss_stops_later_presses() {
    let mut seat = sit();
    seat.show(piece("felt", Rect::new(0., 0., 40., 40.), 1.));
    let input = concat!(
        "{\"op\":\"key\",\"revision\":0,\"key\":\"F1\"}\n",
        "{\"op\":\"drag\",\"revision\":0,\"component\":\"felt\",\"button\":\"right\",\"x\":5}\n",
    );
    let mut output = Vec::new();
    seat.seat
        .serve(&mut seat.app, input.as_bytes(), &mut output)
        .unwrap();
    let responses = lines(&output);
    assert_eq!(responses[2]["status"], "skipped", "{}", responses[2]);
    assert_eq!(
        responses[2]["error"],
        format!(
            "Skipped: an earlier request on this connection was refused, so its activations, drags, keys and scrolls are no longer acted on. No input was sent. Refused request: {{\"op\":\"key\",\"revision\":0,\"key\":\"F1\"}}. Reason: {}",
            responses[1]["error"].as_str().unwrap()
        )
    );
    assert!(seat.table().buttons.is_empty());

    let mut seat = sit();
    seat.show(piece("felt", Rect::new(0., 0., 40., 40.), 1.));
    seat.show(piece("modal", Rect::new(0., 0., 40., 40.), 0.));
    let input = concat!(
        "{\"op\":\"drag\",\"revision\":0,\"component\":\"felt\",\"button\":\"right\",\"x\":5}\n",
        "{\"op\":\"activate\",\"revision\":0,\"component\":\"modal\"}\n",
        "{\"op\":\"drag\",\"revision\":0,\"component\":\"modal\",\"button\":\"right\",\"x\":5}\n",
    );
    let mut output = Vec::new();
    seat.seat
        .serve(&mut seat.app, input.as_bytes(), &mut output)
        .unwrap();
    assert_eq!(
        lines(&output)
            .iter()
            .map(|r| r["status"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["ready", "missed", "skipped", "skipped"]
    );
    assert!(seat.table().buttons.is_empty());
    assert!(seat.table().clicked.is_empty());
}

/// A home like Arkham's overview, as the camera's own tests have it.
const HOME: Framing = Framing::new(Vec3::new(0.0, 0.0, 7.0), 72.0);

/// The table with the table camera: felt fills the window, and a UI panel that is not
/// part of the scene lies over one corner of it.
fn sit_at_the_camera() -> Sitting {
    let mut seat = sit();
    seat.app
        .add_plugins(TableCameraPlugin::<()>::default())
        .insert_resource(Orbit::new(HOME))
        // The stand-in picking finds what is under the pointer before the camera asks.
        .configure_sets(Update, TableCameraSystems.after(pick));
    seat.app
        .world_mut()
        .spawn((TableCamera, Transform::default()));
    seat.show(piece("felt", Rect::new(0., 0., 1280., 720.), 2.));
    seat.app.world_mut().spawn((
        piece("panel", Rect::new(0., 0., 100., 100.), 0.),
        Node::default(),
    ));
    seat
}

fn pose(seat: &Sitting) -> Pose {
    seat.app.world().resource::<Orbit>().pose()
}

#[test]
fn a_right_drag_on_the_scene_orbits_the_table_camera_and_one_from_ui_does_not() {
    let mut seat = sit_at_the_camera();

    let dragged = seat.ask(
        json!({"op":"drag", "revision":0, "component":"felt", "button":"right", "x":40, "y":10}),
    );

    assert_eq!(dragged["status"], "dragged", "{dragged}");
    // As the camera's own scene drag of (40, 10) logical pixels: yaw -0.24, pitch 1.04.
    let moved = pose(&seat);
    assert!((moved.yaw + 0.24).abs() < 0.00001, "yaw {}", moved.yaw);
    assert!(
        (moved.pitch - 1.04).abs() < 0.00001,
        "pitch {}",
        moved.pitch
    );
    assert_eq!(moved.distance, 72.0);

    // A drag begun over a UI node outside the scene captures nothing.
    let from_ui = seat.ask(
        json!({"op":"drag", "revision":0, "component":"panel", "button":"right", "x":40, "y":10}),
    );
    assert_eq!(from_ui["status"], "dragged", "{from_ui}");
    assert_eq!(pose(&seat), moved);

    // The camera orbits on the right button only.
    let left = seat.ask(
        json!({"op":"drag", "revision":0, "component":"felt", "button":"left", "x":40, "y":10}),
    );
    assert_eq!(left["status"], "dragged", "{left}");
    assert_eq!(pose(&seat), moved);
}

#[test]
fn a_scroll_on_the_scene_zooms_the_table_camera_and_one_on_ui_or_a_taken_wheel_does_not() {
    let mut seat = sit_at_the_camera();

    let scrolled = seat.ask(json!({"op":"scroll", "revision":0, "component":"felt", "y":1}));

    assert_eq!(scrolled["status"], "scrolled", "{scrolled}");
    // One line in: 72 * 0.95.
    assert!(
        (pose(&seat).distance - 68.4).abs() < 0.0001,
        "{:?}",
        pose(&seat)
    );
    let zoomed = pose(&seat);

    let on_ui = seat.ask(json!({"op":"scroll", "revision":0, "component":"panel", "y":1}));
    assert_eq!(on_ui["status"], "scrolled", "{on_ui}");
    assert_eq!(pose(&seat), zoomed);

    seat.app.insert_resource(TableFacts {
        takes_wheel: true,
        ..default()
    });
    let taken = seat.ask(json!({"op":"scroll", "revision":0, "component":"felt", "y":1}));
    assert_eq!(taken["status"], "scrolled", "{taken}");
    assert_eq!(pose(&seat), zoomed);
}

#[test]
fn a_covered_component_is_missed_by_a_point_and_a_partly_covered_one_is_pointed_where_it_shows() {
    let mut seat = sit();
    seat.show(piece("card", Rect::new(0., 0., 20., 20.), 1.));
    seat.show(piece("lid", Rect::new(0., 0., 20., 12.), 0.));

    let missed = seat.ask(json!({"op": "point", "revision": 0, "component": "card"}));
    assert_eq!(missed["status"], "missed", "{missed}");
    assert_eq!(missed["ok"], false);
    assert_eq!(missed["pressed"], false);
    assert_eq!(missed["hit"], json!(["lid"]));
    assert_eq!(
        missed["error"],
        "The pointer at (10, 10) is over \"lid\", not component card. Nothing was pressed."
    );

    let mut pieces = seat.app.world_mut().query::<&mut Piece>();
    for mut piece in pieces.iter_mut(seat.app.world_mut()) {
        if piece.name == "card" {
            piece.reach.push(Vec2::new(10., 16.));
        }
    }
    let pointed = seat.ask(json!({"op": "point", "revision": 0, "component": "card"}));
    assert_eq!(pointed["status"], "pointed", "{pointed}");
    assert_eq!(pointed["position"], json!([10.0, 16.0]));
    assert_eq!(pointed["hit"], json!(["card"]));
    assert_eq!(
        pointed["covered"],
        json!({"position": [10.0, 10.0], "hit": ["lid"]})
    );
    assert_eq!(pointed["observation"]["clicked"], json!([]));
}

#[test]
fn a_point_presses_nothing_so_a_refusal_before_it_does_not_skip_it() {
    let mut seat = sit();
    seat.show(piece("card", Rect::new(0., 0., 20., 20.), 1.));
    seat.show(piece("lid", Rect::new(0., 0., 20., 20.), 0.));
    let input = concat!(
        "{\"op\":\"key\",\"revision\":0,\"key\":\"F1\"}\n",
        "{\"op\":\"point\",\"revision\":0,\"component\":\"lid\"}\n",
        "{\"op\":\"point\",\"revision\":0,\"component\":\"card\"}\n",
        "{\"op\":\"key\",\"revision\":0,\"key\":\"Tab\"}\n",
    );
    let mut output = Vec::new();

    seat.seat
        .serve(&mut seat.app, input.as_bytes(), &mut output)
        .unwrap();

    let responses = lines(&output);
    let statuses: Vec<&str> = responses
        .iter()
        .map(|response| response["status"].as_str().unwrap())
        .collect();
    // A missed point stops the connection's later keys as a missed activation does.
    assert_eq!(
        statuses,
        ["ready", "refused", "pointed", "missed", "skipped"]
    );
}

#[test]
fn after_a_missed_point_a_point_is_served_and_activations_and_keys_are_skipped() {
    let mut seat = sit();
    seat.show(piece("card", Rect::new(0., 0., 20., 20.), 1.));
    seat.show(piece("lid", Rect::new(0., 0., 20., 20.), 0.));
    let input = concat!(
        "{\"op\":\"point\",\"revision\":0,\"component\":\"card\"}\n",
        "{\"op\":\"point\",\"revision\":0,\"component\":\"lid\"}\n",
        "{\"op\":\"activate\",\"revision\":0,\"component\":\"lid\"}\n",
        "{\"op\":\"key\",\"revision\":0,\"key\":\"Tab\"}\n",
    );
    let mut output = Vec::new();

    seat.seat
        .serve(&mut seat.app, input.as_bytes(), &mut output)
        .unwrap();

    let responses = lines(&output);
    let statuses: Vec<&str> = responses
        .iter()
        .map(|response| response["status"].as_str().unwrap())
        .collect();
    assert_eq!(
        statuses,
        ["ready", "missed", "pointed", "skipped", "skipped"]
    );
    assert!(
        responses[3]["error"]
            .as_str()
            .unwrap()
            .starts_with("Skipped: an earlier request on this connection missed its target"),
        "{}",
        responses[3]["error"]
    );
    // Nothing was pressed, so nothing was clicked.
    assert_eq!(responses[4]["observation"]["clicked"], json!([]));
    assert_eq!(responses[4]["observation"]["keys"], json!([]));
}

#[test]
fn a_covered_component_is_missed_and_a_partly_covered_one_is_pressed_where_it_shows() {
    let mut seat = sit();
    seat.show(piece("card", Rect::new(0., 0., 20., 20.), 1.));
    seat.show(piece("lid", Rect::new(0., 0., 20., 12.), 0.));

    let missed = seat.ask(json!({"op": "activate", "revision": 0, "component": "card"}));
    assert_eq!(missed["status"], "missed", "{missed}");
    assert_eq!(missed["ok"], false);
    assert_eq!(missed["pressed"], false);
    assert_eq!(missed["position"], json!([10.0, 10.0]));
    assert_eq!(missed["hit"], json!(["lid"]));
    assert_eq!(
        missed["error"],
        "The pointer at (10, 10) is over \"lid\", not component card. Nothing was pressed."
    );
    assert_eq!(missed["observation"]["clicked"], json!([]));

    // The game also offers a point below the lid.
    let mut pieces = seat.app.world_mut().query::<&mut Piece>();
    for mut piece in pieces.iter_mut(seat.app.world_mut()) {
        if piece.name == "card" {
            piece.reach.push(Vec2::new(10., 16.));
        }
    }
    let activated = seat.ask(json!({"op": "activate", "revision": 0, "component": "card"}));
    assert_eq!(activated["status"], "activated", "{activated}");
    assert_eq!(activated["position"], json!([10.0, 16.0]));
    assert_eq!(
        activated["covered"],
        json!({"position": [10.0, 10.0], "hit": ["lid"]})
    );
    assert_eq!(activated["observation"]["clicked"], json!(["card"]));
}

#[test]
fn a_press_that_lands_on_something_else_is_cancelled_before_release() {
    let mut seat = sit();
    seat.show(piece("card", Rect::new(0., 0., 20., 20.), 1.));
    seat.show(Piece {
        late: true,
        ..piece("curtain", Rect::new(0., 0., 20., 20.), 0.)
    });

    let missed = seat.ask(json!({"op": "activate", "revision": 0, "component": "card"}));

    assert_eq!(missed["status"], "missed", "{missed}");
    assert_eq!(missed["pressed"], true);
    assert_eq!(
        missed["error"],
        "The press at (10, 10) landed on \"curtain\", not component card."
    );
    // Not a click on either piece.
    assert_eq!(missed["observation"]["clicked"], json!([]));
    assert_eq!(missed["observation"]["revision"], 0);
}

#[test]
fn a_miss_stops_the_connections_later_activations_and_keys() {
    let mut seat = sit();
    seat.show(piece("card", Rect::new(0., 0., 20., 20.), 1.));
    seat.show(piece("lid", Rect::new(0., 0., 20., 20.), 0.));
    let input = concat!(
        "{\"op\":\"activate\",\"revision\":0,\"component\":\"card\"}\n",
        "{\"op\":\"key\",\"revision\":0,\"key\":\"Tab\"}\n",
    );
    let mut output = Vec::new();

    seat.seat
        .serve(&mut seat.app, input.as_bytes(), &mut output)
        .unwrap();

    let responses = lines(&output);
    assert_eq!(responses[1]["status"], "missed");
    assert_eq!(responses[2]["status"], "skipped");
    assert!(
        responses[2]["error"]
            .as_str()
            .unwrap()
            .starts_with("Skipped: an earlier request on this connection missed its target, so its activations, drags, keys and scrolls are no longer acted on. No input was sent. Missed request: "),
        "{}",
        responses[2]["error"]
    );
}

#[test]
fn a_table_that_ends_itself_ends_the_session_with_its_reason() {
    let mut seat = sit();
    let ended = seat
        .seat
        .respond(
            &mut seat.app,
            "{\"op\":\"key\",\"revision\":0,\"key\":\"Home\"}",
        )
        .unwrap_err();
    assert!(!ended.is_error());
    assert_eq!(ended.exit(), &AppExit::Success);
    assert_eq!(ended.response()["status"], "exited");
    assert_eq!(ended.response()["ok"], true);
    assert_eq!(ended.to_string(), "The table exited (Some(Success)).");

    // An error exit says why, in the response and in the error itself: Bevy's
    // `AppExit::error()` is code 1. Every later request, whatever it is, is answered
    // with the exit and acts on nothing.
    let mut seat = sit();
    let failed = seat.expect_exit(json!({"op": "key", "revision": 0, "key": "End"}));
    let reason = "The table exited (Some(Error(1))).";
    assert!(failed.is_error());
    assert_eq!(failed.response()["ok"], false);
    assert_eq!(failed.to_string(), reason);
    assert_eq!(failed.response()["error"], reason);
    for later in [
        json!({"op": "observe"}),
        json!({"op": "quit"}),
        json!({"op": "advance", "steps": 5}),
        json!({"op": "settle", "steps": 5}),
        json!({"op": "key", "revision": 0, "key": "Enter"}),
    ]
    .iter()
    .map(Value::to_string)
    .chain(["nonsense".to_owned()])
    {
        let (clock, keys, revision) = {
            let table = seat.table();
            (table.clock, table.keys.clone(), table.revision)
        };
        let again = seat.seat.respond(&mut seat.app, &later).unwrap_err();
        assert_eq!(again.to_string(), reason, "{later}");
        assert_eq!(again.response()["status"], "exited", "{later}");
        let table = seat.table();
        assert_eq!(table.clock, clock, "{later}");
        assert_eq!(table.keys, keys, "{later}");
        assert_eq!(table.revision, revision, "{later}");
    }

    let mut seat = sit();
    let mut output = Vec::new();
    let failed = seat.seat.serve(
        &mut seat.app,
        &b"{\"op\":\"key\",\"revision\":0,\"key\":\"End\"}\n{\"op\":\"observe\"}\n"[..],
        &mut output,
    );
    let responses = lines(&output);
    assert_eq!(responses.len(), 2, "nothing follows the table's exit");
    assert_eq!(responses[1]["status"], "exited");
    assert_eq!(responses[1]["ok"], false);
    assert_eq!(responses[1]["error"], reason);
    assert_eq!(failed, Err(reason.to_owned()));

    // A game that refused play gives its own reason for the exit.
    let mut seat = sit();
    seat.table().admission = Admission::Refused("The artwork went away.".into());
    let refused = seat.expect_exit(json!({"op": "key", "revision": 0, "key": "End"}));
    assert_eq!(refused.response()["status"], "exited");
    assert_eq!(refused.response()["error"], "The artwork went away.");
    assert_eq!(refused.to_string(), "The artwork went away.");
}

/// Bevy's default render-error handler quits by writing `AppExit::error()` on every
/// update after the renderer fails, from the render world's extract: an update the
/// seat may never run itself, as when a test resizes the table and updates it. Here a
/// system writes it as that handler does, in an update the test runs.
fn exit_outside_the_seat(app: &mut App) {
    app.add_systems(Update, |mut exit: MessageWriter<AppExit>| {
        exit.write(AppExit::error());
    });
    app.update();
}

#[test]
fn a_table_that_exited_outside_the_seats_steps_answers_even_an_observe_with_its_exit() {
    let mut seat = sit();
    exit_outside_the_seat(&mut seat.app);
    let clock = seat.table().clock;

    let exited = seat.expect_exit(json!({"op": "observe"}));

    assert!(exited.is_error());
    assert_eq!(exited.response()["status"], "exited");
    assert_eq!(exited.response()["ok"], false);
    assert_eq!(exited.to_string(), "The table exited (Some(Error(1))).");
    assert_eq!(seat.table().clock, clock, "nothing stepped");

    // On the wire the observe is answered with the exit, and nothing follows it.
    let mut seat = sit();
    exit_outside_the_seat(&mut seat.app);
    let mut output = Vec::new();
    let failed = seat.seat.serve(
        &mut seat.app,
        &b"{\"op\":\"observe\"}\n{\"op\":\"settle\"}\n"[..],
        &mut output,
    );
    let responses = lines(&output);
    assert_eq!(responses.len(), 2, "ready, then the exit: {responses:?}");
    assert_eq!(responses[1]["status"], "exited");
    assert_eq!(responses[1]["error"], "The table exited (Some(Error(1))).");
    assert_eq!(failed, Err("The table exited (Some(Error(1))).".to_owned()));
}

#[test]
fn admission_is_the_games_and_an_admitted_table_is_settled_first() {
    let mut seat = sit();
    seat.table().presenting = 4;
    assert_eq!(
        seat.seat.admit(&mut seat.app, Duration::from_secs(30)),
        Ok(())
    );
    assert_eq!(seat.table().presenting, 0);

    let mut seat = sit();
    seat.table().admission = Admission::Refused("No artwork.".into());
    assert_eq!(
        seat.seat.admit(&mut seat.app, Duration::from_secs(30)),
        Err(Unadmitted::Refused("No artwork.".into()))
    );

    // Still checking when the time allowed is over: the words are the game's.
    let mut seat = sit();
    seat.table().admission = Admission::Checking;
    assert_eq!(
        seat.seat.admit(&mut seat.app, Duration::ZERO),
        Err(Unadmitted::Unfinished)
    );
}

#[test]
fn a_host_that_renders_no_image_refuses_a_capture() {
    let mut seat = sit();
    let scratch = Scratch::new("no-image");
    let path = scratch.join("table.png");

    let response = seat.ask(json!({"op": "capture", "path": path}));

    assert_eq!(response["status"], "refused");
    assert_eq!(
        response["error"],
        "This host renders no image; capture needs the windowless seat."
    );
    assert!(!path.exists());
}

#[test]
fn a_capture_waits_for_the_pipelines_on_a_frozen_clock_and_answers_after_the_file() {
    let mut seat = sit_with(Some(Handle::default()));
    let scratch = Scratch::new("capture");
    // The directories above the file do not exist: the seat makes them.
    let path = scratch.join("pictures/table.png");
    seat.ask(json!({"op": "key", "revision": 0, "key": "Tab"}));
    // Ready on one frame is not enough: the finding trails the renderer by a frame.
    seat.table().pipelines = [false, true, false, true, true, true, true].into();

    let response = seat.ask(json!({"op": "capture", "path": path}));

    assert_eq!(response["status"], "captured", "{response}");
    assert_eq!(response["path"], json!(path));
    // Ready on the fourth and fifth frames running, so the picture is asked for on the
    // sixth, which takes the sixth finding first: one is left. Asked for after the
    // first ready frame, four would be left.
    assert_eq!(seat.table().findings_left_when_asked, Some(1));
    // Only the key's two steps moved the clock.
    assert_eq!(response["table"]["clock_nanos"], clock_after(2));
    let picture = image::open(&path).unwrap().to_rgb8();
    assert_eq!(picture.dimensions(), (2, 1));
    assert_eq!(picture.get_pixel(0, 0).0, [255, 0, 0]);
}

#[test]
fn a_capture_whose_pipelines_are_never_ready_two_frames_running_is_refused_and_writes_nothing() {
    let mut seat = sit_with(Some(Handle::default()));
    let scratch = Scratch::new("never-ready");
    let path = scratch.join("table.png");
    // Ready twice, never on consecutive frames, then never again.
    seat.table().pipelines = [true, false, true, false].into();

    let response = seat.ask(json!({"op": "capture", "path": path}));

    assert_eq!(response["status"], "refused", "{response}");
    assert_eq!(response["error"], "Render pipelines did not become ready.");
    assert_eq!(seat.table().findings_left_when_asked, None);
    assert!(!path.exists());
}

#[test]
fn a_capture_whose_picture_cannot_be_written_says_so() {
    let mut seat = sit_with(Some(Handle::default()));
    let scratch = Scratch::new("unsaved");
    // No image format is named by this ending.
    let path = scratch.join("table.unsupported");

    let response = seat.ask(json!({"op": "capture", "path": path}));

    assert_eq!(response["status"], "refused", "{response}");
    let error = response["error"].as_str().unwrap();
    assert!(error.starts_with("Cannot save the capture: "), "{error}");
    assert!(!path.exists());
}

#[test]
fn a_table_that_exits_during_a_capture_answers_that_it_exited_and_writes_nothing() {
    let mut seat = sit_with(Some(Handle::default()));
    let scratch = Scratch::new("exits");
    let path = scratch.join("table.png");
    seat.table().exits_when_asked = true;

    let exited = seat.expect_exit(json!({"op": "capture", "path": path}));

    // The picture was asked for, then the table exited before it came back.
    assert!(seat.table().findings_left_when_asked.is_some());
    assert_eq!(exited.response()["status"], "exited", "{exited}");
    assert!(!path.exists());
}

/// One connection to a held seat: send every request, then read every response.
fn connection(socket: &Path, requests: &[Value]) -> Vec<Value> {
    let mut stream = UnixStream::connect(socket).unwrap();
    for request in requests {
        writeln!(stream, "{request}").unwrap();
    }
    stream.shutdown(std::net::Shutdown::Write).unwrap();
    BufReader::new(stream)
        .lines()
        .map(|line| serde_json::from_str(&line.unwrap()).unwrap())
        .collect()
}

#[test]
fn a_held_session_answers_one_connection_after_another_until_quit() {
    let mut seat = sit();
    let scratch = Scratch::new("held");
    let socket = scratch.join("seat.sock");
    let listener = Listener::claim(&socket).unwrap();
    let file = std::fs::symlink_metadata(&socket).unwrap();
    assert!(file.file_type().is_socket());
    assert_eq!(file.permissions().mode() & 0o777, 0o600);

    let client = std::thread::spawn({
        let socket = socket.clone();
        move || {
            let first = connection(
                &socket,
                &[
                    json!({"op": "key", "revision": 7, "key": "Tab"}),
                    json!({"op": "key", "revision": 0, "key": "Tab"}),
                ],
            );
            // A refusal stops only its own connection, and the table is kept.
            let second = connection(
                &socket,
                &[
                    json!({"op": "key", "revision": 0, "key": "Tab"}),
                    json!({"op": "quit"}),
                ],
            );
            (first, second)
        }
    });
    let ended = listener.serve(&mut seat.seat, &mut seat.app, Duration::from_secs(60));
    let (first, second) = client.join().unwrap();

    assert_eq!(ended, Ok(()));
    assert_eq!(first[0]["status"], "refused");
    assert_eq!(first[1]["status"], "skipped");
    assert_eq!(second[0]["status"], "pressed", "{}", second[0]);
    assert_eq!(second[1]["status"], "bye");
    assert_eq!(second[1]["observation"]["keys"], json!(["Tab"]));
    assert!(!socket.exists(), "the socket goes with the session");
}

#[test]
fn a_held_session_with_no_request_ends_itself_after_its_idle_period() {
    let mut seat = sit();
    let scratch = Scratch::new("idle");
    let socket = scratch.join("seat.sock");
    let listener = Listener::claim(&socket).unwrap();
    // Connected, and sending nothing: it does not keep the session.
    let silent = UnixStream::connect(&socket).unwrap();

    let ended = listener.serve(&mut seat.seat, &mut seat.app, Duration::from_millis(60));

    assert_eq!(ended, Ok(()));
    assert!(!socket.exists());
    drop(silent);
}

#[test]
fn a_socket_path_is_claimed_only_where_no_seat_answers() {
    let scratch = Scratch::new("claim");

    // Each refusal names its cause as data and keeps the path it concerns; its
    // text is the seat's default wording, which a game may replace.
    let long = scratch.join(&"s".repeat(120));
    let refused = Listener::claim(&long).err().unwrap();
    assert!(matches!(refused.cause(), Unclaimable::TooLong), "{refused}");
    assert_eq!(refused.path(), long);
    assert_eq!(
        refused.to_string(),
        format!(
            "Cannot listen on {}: a local socket path holds at most 107 bytes; choose a shorter one",
            long.display()
        )
    );

    let file = scratch.join("not-a-socket");
    std::fs::write(&file, "kept").unwrap();
    let refused = Listener::claim(&file).err().unwrap();
    assert!(
        matches!(refused.cause(), Unclaimable::NotSocket),
        "{refused}"
    );
    assert_eq!(
        refused.to_string(),
        format!(
            "Cannot listen on {}: the path exists and is not a socket; it was left unchanged",
            file.display()
        )
    );
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "kept");

    // A directory its owner may write and search but not read can hold the socket,
    // but cannot be opened to take turns in it.
    let unreadable = scratch.join("unreadable");
    std::fs::create_dir(&unreadable).unwrap();
    std::fs::set_permissions(&unreadable, Permissions::from_mode(0o300)).unwrap();
    let refused = Listener::claim(&unreadable.join("s")).err().unwrap();
    std::fs::set_permissions(&unreadable, Permissions::from_mode(0o700)).unwrap();
    assert!(
        matches!(refused.cause(), Unclaimable::Unlocked(cause) if cause.kind() == std::io::ErrorKind::PermissionDenied),
        "{refused}"
    );
    assert!(
        refused
            .to_string()
            .contains(": cannot lock its directory: "),
        "{refused}"
    );
    assert_eq!(std::fs::read_dir(&unreadable).unwrap().count(), 0);

    let socket = scratch.join("seat.sock");
    let answering = Listener::claim(&socket).unwrap();
    let refused = Listener::claim(&socket).err().unwrap();
    assert!(
        matches!(refused.cause(), Unclaimable::Answered),
        "{refused}"
    );
    assert_eq!(
        refused.to_string(),
        format!(
            "Cannot listen on {}: an agent seat already answers there and was left untouched; quit that seat or choose another path",
            socket.display()
        )
    );
    drop(answering);
    assert!(!socket.exists(), "a listener that ends removes its socket");

    // A crashed seat leaves its socket file, where nothing answers: it is replaced.
    let leftover = scratch.join("leftover.sock");
    drop(UnixListener::bind(&leftover).unwrap());
    assert!(leftover.exists());
    let replaced = Listener::claim(&leftover).unwrap();
    assert!(UnixStream::connect(&leftover).is_ok());
    drop(replaced);
}

#[test]
fn a_claimed_path_answers_a_protocol_of_the_games_own_until_its_listener_drops() {
    let scratch = Scratch::new("own");
    let socket = scratch.join("own.sock");
    let listener = Listener::claim(&socket).unwrap();

    // The game's own loop, on a thread of its own instead of the seat's serving loop.
    let host = std::thread::spawn(move || {
        let (stream, _) = listener.unix_listener().accept().unwrap();
        let mut line = String::new();
        BufReader::new(&stream).read_line(&mut line).unwrap();
        writeln!(&stream, "heard {}", line.trim_end()).unwrap();
        listener
    });
    let mut stream = UnixStream::connect(&socket).unwrap();
    writeln!(stream, "hello").unwrap();
    let mut answer = String::new();
    BufReader::new(&stream).read_line(&mut answer).unwrap();
    let listener = host.join().unwrap();

    assert_eq!(answer, "heard hello\n");
    assert!(
        socket.exists(),
        "the path stays claimed while its listener lives"
    );
    drop(listener);
    assert!(!socket.exists(), "the path goes with its listener");
}

/// A table that supplies only what needs knowledge of its game: its observation and
/// revision, its names, its hits, what is unsettled and its admission. The seat answers
/// its clock, its focus and its keys. Its presentation runs on Bevy's time.
struct Bare;

impl Game for Bare {
    fn observe(&self, world: &mut World) -> Observed {
        Observed {
            observation: json!({"keys": world.resource::<Table>().keys}),
            table: Value::Null,
        }
    }

    fn revision(&self, world: &mut World) -> u64 {
        world.resource::<Table>().revision
    }

    fn resolve(&self, _: &mut World, _: &Name) -> Result<Vec<Vec2>, String> {
        Err("This table shows nothing.".into())
    }

    fn identify(&self, _: &mut World, _: Entity) -> Hit {
        Hit {
            name: None,
            description: Value::Null,
        }
    }

    fn unsettled(&self, world: &mut World) -> Vec<String> {
        Plain.unsettled(world)
    }

    fn admission(&self, world: &World) -> Admission {
        Plain.admission(world)
    }
}

/// [`Bare`] with keys of its own; the seat still answers its focus.
struct Keyed;

impl Game for Keyed {
    fn observe(&self, world: &mut World) -> Observed {
        Bare.observe(world)
    }

    fn revision(&self, world: &mut World) -> u64 {
        Bare.revision(world)
    }

    fn resolve(&self, world: &mut World, name: &Name) -> Result<Vec<Vec2>, String> {
        Bare.resolve(world, name)
    }

    fn identify(&self, world: &mut World, entity: Entity) -> Hit {
        Bare.identify(world, entity)
    }

    fn unsettled(&self, world: &mut World) -> Vec<String> {
        Bare.unsettled(world)
    }

    fn admission(&self, world: &World) -> Admission {
        Bare.admission(world)
    }

    fn keys(&self) -> &[SeatKey] {
        &KEYS
    }
}

/// The bare table's presentation: one step of it for each update that Bevy's time moves.
fn present(time: Res<Time<Real>>, mut table: ResMut<Table>) {
    if !time.delta().is_zero() {
        table.presenting = table.presenting.saturating_sub(1);
    }
}

/// A table of `game` with no clock of its own, and an agent seated at it.
fn sit_bare<G: Game>(game: G) -> (App, Seat<G>) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin))
        .add_message::<PointerInput>()
        .init_resource::<HoverMap>()
        .init_resource::<Table>()
        .add_systems(Update, (read_keys, present));
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    let camera = app.world_mut().spawn(RenderTarget::default()).id();
    // Bevy's first update only starts its clock, and the seat's opening makes it.
    app.update();
    let seat = Seat::new(game, &mut app, camera, None);
    (app, seat)
}

#[test]
fn with_no_clock_of_its_own_a_tables_steps_move_bevys_time_in_place_of_the_wall_clock() {
    let (mut app, mut seat) = sit_bare(Bare);
    app.world_mut().resource_mut::<Table>().presenting = 3;
    // Long enough for a wall clock to be told from the seat's steps.
    std::thread::sleep(Duration::from_millis(50));

    let settled = seat.respond(&mut app, r#"{"op":"settle"}"#).unwrap();

    assert_eq!(settled["status"], "settled", "{settled}");
    assert_eq!(settled["steps"], 3);
    assert_eq!(app.world().resource::<Time<Real>>().elapsed(), 3 * STEP);
}

#[test]
fn with_no_keys_of_its_own_a_table_lets_the_seat_press_none() {
    let (mut app, mut seat) = sit_bare(Bare);

    let refused = seat
        .respond(&mut app, r#"{"op":"key","revision":0,"key":"Tab"}"#)
        .unwrap();

    assert_eq!(refused["status"], "refused");
    assert_eq!(refused["error"], "Unknown key \"Tab\". Known keys: .");
    assert_eq!(app.world().resource::<Table>().keys, Vec::<String>::new());
}

#[test]
fn with_no_focus_of_its_own_a_table_has_nothing_focused_for_a_key_that_can_commit() {
    let (mut app, mut seat) = sit_bare(Keyed);

    let refused = seat
        .respond(
            &mut app,
            r#"{"op":"key","revision":0,"key":"Enter","focused":"Done"}"#,
        )
        .unwrap();
    let pressed = seat
        .respond(&mut app, r#"{"op":"key","revision":0,"key":"Tab"}"#)
        .unwrap();

    assert_eq!(
        refused["error"],
        "Enter names \"Done\", but nothing has focus. Nothing was pressed."
    );
    assert_eq!(pressed["status"], "pressed", "{pressed}");
    assert_eq!(pressed["observation"]["keys"], json!(["Tab"]));
}

#[test]
fn every_response_states_the_revision_and_what_is_pending_without_the_game() {
    let (mut app, mut seat) = sit_bare(Bare);
    {
        let mut table = app.world_mut().resource_mut::<Table>();
        table.revision = 7;
        table.presenting = 2;
    }

    let busy = seat.respond(&mut app, r#"{"op":"observe"}"#).unwrap();
    let settled = seat.respond(&mut app, r#"{"op":"settle"}"#).unwrap();

    assert_eq!(
        busy,
        json!({
            "ok": true, "mode": "seat", "status": "observed",
            "observation": {"keys": []}, "table": null,
            "revision": 7, "settled": false, "pending": ["presentation"],
        })
    );
    assert_eq!(settled["revision"], 7);
    assert_eq!(settled["settled"], true);
    assert_eq!(settled["pending"], json!([]));
}

/// A game's admission, as a game's test of that one concern supplies it.
fn refusing(_: &World) -> Admission {
    Admission::Refused("No artwork.".into())
}

#[test]
fn a_test_of_a_tables_admission_alone_supplies_only_its_admission() {
    let (mut app, mut seat) = sit_bare(Unshown(refusing));

    let observed = seat.respond(&mut app, r#"{"op":"observe"}"#).unwrap();
    let activated = seat
        .respond(
            &mut app,
            r#"{"op":"activate","revision":0,"component":"Deck"}"#,
        )
        .unwrap();

    assert_eq!(
        seat.admit(&mut app, Duration::from_secs(30)),
        Err(Unadmitted::Refused("No artwork.".into()))
    );
    assert_eq!(
        observed,
        json!({
            "ok": true, "mode": "seat", "status": "observed",
            "observation": null, "table": null,
            "revision": 0, "settled": true, "pending": [],
        })
    );
    assert_eq!(activated["error"], "component Deck is not shown.");
}
