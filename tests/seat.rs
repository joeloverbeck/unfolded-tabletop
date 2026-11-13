//! The agent seat's protocol as Arkham had it at `0b8e6fb`, before its game-neutral
//! half moved here, on a table that belongs to no game. Arkham's own tests hold the
//! same protocol on its real table. The table here reads keys through Bevy's own
//! keyboard input; its picking and its renderer are stand-ins, so no test needs a
//! graphics adapter.
#![cfg(feature = "seat")]

use bevy::{
    app::AppExit,
    asset::RenderAssetUsages,
    camera::RenderTarget,
    input::{InputPlugin, keyboard::Key},
    picking::{
        backend::HitData,
        hover::HoverMap,
        pointer::{PointerAction, PointerId, PointerInput},
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
    io::{BufRead, BufReader, Write},
    os::unix::{
        fs::{FileTypeExt, PermissionsExt},
        net::{UnixListener, UnixStream},
    },
    path::{Path, PathBuf},
    time::Duration,
};
use unfolded_tabletop::seat::{
    Admission, Focus, Game, Hit, Logical, Name, Observed, Seat, SeatKey, SeatedAgent, Unadmitted,
    socket::Listener,
};

/// One sixtieth of a second: the seat's step of the table's clock.
const STEP_NANOS: u64 = 16_666_667;

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
    /// The keys the table read through Bevy's keyboard input, in order.
    keys: Vec<String>,
    /// The pieces clicked: pressed and released while under the pointer.
    clicked: Vec<String>,
    admission: Admission,
    /// Frozen steps still to pass before the render pipelines are ready.
    compiling: u32,
}

impl Default for Table {
    fn default() -> Self {
        Self {
            revision: 0,
            presenting: 0,
            clock: Duration::ZERO,
            focus: None,
            keys: Vec::new(),
            clicked: Vec::new(),
            admission: Admission::Admitted,
            compiling: 0,
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
        if by.is_zero() {
            table.compiling = table.compiling.saturating_sub(1);
        } else {
            table.presenting = table.presenting.saturating_sub(1);
        }
    }

    fn admission(&self, world: &World) -> Admission {
        world.resource::<Table>().admission.clone()
    }

    fn pipelines_ready(&self, world: &World) -> bool {
        world.resource::<Table>().compiling == 0
    }

    fn keys(&self) -> &[SeatKey] {
        &KEYS
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
        table.keys.push(format!("{key:?}"));
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
/// after each pointer message, and a click when a press and its release land on the
/// same nearest piece.
fn pick(
    mut inputs: MessageReader<PointerInput>,
    mut hover: ResMut<HoverMap>,
    pieces: Query<(Entity, &Piece)>,
    mut table: ResMut<Table>,
    mut held: Local<Option<&'static str>>,
    mut pressed_once: Local<bool>,
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
        match input.action {
            PointerAction::Press(_) => *held = nearest,
            PointerAction::Release(_) => {
                if held.take().is_some_and(|name| Some(name) == nearest) {
                    table.clicked.extend(nearest.map(String::from));
                    table.revision += 1;
                }
            }
            PointerAction::Cancel => *held = None,
            _ => {}
        }
    }
}

/// Stands in for Bevy's renderer: hand a picture back to every capture asked for.
fn renderer_hands_back(mut commands: Commands, asked: Query<Entity, With<Screenshot>>) {
    for entity in &asked {
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

struct Sitting {
    app: App,
    seat: Seat<Plain>,
}

impl Sitting {
    fn ask(&mut self, request: Value) -> Value {
        self.seat.respond(&mut self.app, &request.to_string()).0
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
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin))
        .add_message::<PointerInput>()
        .init_resource::<HoverMap>()
        .init_resource::<Table>()
        .add_systems(Update, (read_keys, pick));
    if image.is_some() {
        app.add_systems(Update, renderer_hands_back);
    }
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    let camera = app.world_mut().spawn(RenderTarget::default()).id();
    let seat = Seat::new(Plain, &mut app, camera, image);
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
    assert_eq!(response["table"]["clock_nanos"], 2 * STEP_NANOS);
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
    assert_eq!(settled["table"]["clock_nanos"], 5 * STEP_NANOS);
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
            "Skipped: an earlier request on this connection was refused, so its activations and keys are no longer acted on. Nothing was pressed. Refused request: {{\"op\":\"key\",\"revision\":0,\"key\":\"F1\"}}. Reason: {}",
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
        let (response, goes_on) = seat.seat.respond(&mut seat.app, line);
        assert_eq!(response["status"], "refused", "{line}");
        assert!(goes_on);
    }
    let both = seat.ask(json!({"op": "activate", "revision": 0, "component": "a", "choice": "b"}));
    assert_eq!(
        both["error"],
        "activate needs exactly one of component or choice"
    );
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
    assert_eq!(activated["table"]["clock_nanos"], 3 * STEP_NANOS);
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
    assert_eq!(pointed["table"]["clock_nanos"], STEP_NANOS);
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
            .starts_with("Skipped: an earlier request on this connection missed its target, so its activations and keys are no longer acted on. Nothing was pressed. Missed request: "),
        "{}",
        responses[2]["error"]
    );
}

#[test]
fn a_table_that_ends_itself_ends_the_session_with_its_reason() {
    let mut seat = sit();
    let (ended, goes_on) = seat.seat.respond(
        &mut seat.app,
        "{\"op\":\"key\",\"revision\":0,\"key\":\"Home\"}",
    );
    assert_eq!(ended["status"], "exited");
    assert_eq!(ended["ok"], true);
    assert!(!goes_on);

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
    let reason = responses[1]["error"].as_str().unwrap();
    assert!(
        reason.starts_with("The table exited (Some(Error("),
        "{reason}"
    );
    assert_eq!(failed, Err(reason.to_owned()));

    // A game that refused play gives its own reason for the exit.
    let mut seat = sit();
    seat.table().admission = Admission::Refused("The artwork went away.".into());
    let refused = seat.ask(json!({"op": "key", "revision": 0, "key": "End"}));
    assert_eq!(refused["status"], "exited");
    assert_eq!(refused["error"], "The artwork went away.");
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
    seat.table().compiling = 3;

    let response = seat.ask(json!({"op": "capture", "path": path}));

    assert_eq!(response["status"], "captured", "{response}");
    assert_eq!(response["path"], json!(path));
    assert_eq!(seat.table().compiling, 0);
    // Only the key's two steps moved the clock.
    assert_eq!(response["table"]["clock_nanos"], 2 * STEP_NANOS);
    let picture = image::open(&path).unwrap().to_rgb8();
    assert_eq!(picture.dimensions(), (2, 1));
    assert_eq!(picture.get_pixel(0, 0).0, [255, 0, 0]);
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

    let long = scratch.join(&"s".repeat(120));
    let refused = Listener::claim(&long).err().unwrap();
    assert!(refused.contains("at most 107 bytes"), "{refused}");

    let file = scratch.join("not-a-socket");
    std::fs::write(&file, "kept").unwrap();
    let refused = Listener::claim(&file).err().unwrap();
    assert!(
        refused.contains("the path exists and is not a socket; it was left unchanged"),
        "{refused}"
    );
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "kept");

    let socket = scratch.join("seat.sock");
    let answering = Listener::claim(&socket).unwrap();
    let refused = Listener::claim(&socket).err().unwrap();
    assert!(
        refused.contains("an agent seat already answers there and was left untouched"),
        "{refused}"
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
