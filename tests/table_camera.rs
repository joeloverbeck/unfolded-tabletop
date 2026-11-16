#![cfg(feature = "render")]

//! The table camera's own input gathering: what a game's [`TableFacts`] and the
//! camera's pointer and focus facts stop, and what [`TableCameraPlugin`] reads and
//! writes. Motion itself is held by `tests/camera.rs`. Expected numbers are the ones
//! Arkham recorded there, or hand calculations from the motion they name.

use bevy::{
    input::mouse::AccumulatedMouseScroll,
    input_focus::InputFocus,
    picking::{backend::HitData, hover::HoverMap, pointer::PointerId},
    platform::collections::HashMap,
    prelude::*,
    ui_widgets::Button,
    window::PrimaryWindow,
};
use std::time::Duration;
use unfolded_tabletop::camera::{
    CameraStep, DISTANCE, Framing, InScene, Input, KEYS, Navigation, Orbit, PAN_KEYS, PITCH, Pose,
    Scene, TableCamera, TableCameraPlugin, TableCameraSystems, TableFacts, VIEW_KEYS,
};

/// A home like Arkham's overview, so the recorded numbers apply unchanged.
const HOME: Framing = Framing::new(Vec3::new(0.0, 0.0, 7.0), 72.0);
/// Where the cursor rests, inside every scene below but the one that excludes it.
const CURSOR: Vec2 = Vec2::new(800.0, 400.0);

/// Plain input values for one frame, with no Bevy world.
struct Frame {
    keys: ButtonInput<KeyCode>,
    buttons: ButtonInput<MouseButton>,
    window: Window,
    wheel_lines: f32,
    pointer_over_ui: bool,
    focus_on_button: bool,
}

impl Default for Frame {
    fn default() -> Self {
        let mut window = Window {
            focused: true,
            ..default()
        };
        window.set_cursor_position(Some(CURSOR));
        Self {
            keys: default(),
            buttons: default(),
            window,
            wheel_lines: 0.0,
            pointer_over_ui: false,
            focus_on_button: false,
        }
    }
}

impl Frame {
    fn drive(&mut self, orbit: &mut Orbit, table: &TableFacts) -> CameraStep {
        let step = orbit.drive(
            Input {
                keys: &self.keys,
                buttons: &self.buttons,
                window: &self.window,
                wheel_lines: self.wheel_lines,
                dt: 0.1,
                pointer_over_ui: self.pointer_over_ui,
                focus_on_button: self.focus_on_button,
            },
            table,
        );
        self.keys.clear();
        self.buttons.clear();
        self.wheel_lines = 0.0;
        step
    }

    /// Press and hold the right button, then move the cursor by `delta` while held.
    fn drag(&mut self, orbit: &mut Orbit, table: &TableFacts, delta: Vec2) -> CameraStep {
        self.buttons.press(MouseButton::Right);
        self.drive(orbit, table);
        let at = self.window.cursor_position().unwrap() + delta;
        self.window.set_cursor_position(Some(at));
        self.drive(orbit, table)
    }
}

fn pressed(key: KeyCode) -> Frame {
    let mut frame = Frame::default();
    frame.keys.press(key);
    frame
}

/// Away from home, so that Home moves the camera.
fn orbit() -> Orbit {
    let mut orbit = Orbit::new(HOME);
    orbit.focus(Framing::new(Vec3::new(2.0, 0.0, -3.0), 30.0));
    orbit
}

#[test]
fn every_camera_key_moves_it_and_a_key_it_does_not_name_does_not() {
    assert_eq!(KEYS[..4], PAN_KEYS);
    assert_eq!(KEYS[4..], VIEW_KEYS);
    for key in KEYS {
        let mut orbit = orbit();
        let before = orbit.pose();
        assert!(
            pressed(key)
                .drive(&mut orbit, &TableFacts::default())
                .manual_navigation,
            "{key:?}"
        );
        assert_ne!(orbit.pose(), before, "{key:?}");
    }
    for key in [
        KeyCode::KeyQ,
        KeyCode::Space,
        KeyCode::End,
        KeyCode::ShiftLeft,
    ] {
        let mut orbit = orbit();
        let before = orbit.pose();
        assert!(
            !pressed(key)
                .drive(&mut orbit, &TableFacts::default())
                .manual_navigation
        );
        assert_eq!(orbit.pose(), before);
    }
}

#[cfg(feature = "seat")]
#[test]
fn the_seat_keys_are_the_camera_keys_by_their_code_names_and_none_commits() {
    use unfolded_tabletop::camera::SEAT_KEYS;
    let codes: Vec<KeyCode> = SEAT_KEYS.iter().map(|key| key.code).collect();
    assert_eq!(codes, KEYS);
    for key in &SEAT_KEYS {
        assert_eq!(key.name, format!("{:?}", key.code));
        assert!(!key.commits, "{}", key.name);
    }
}

#[test]
fn a_blocking_table_stops_every_input_and_ends_a_held_drag() {
    let blocked = TableFacts {
        blocks_camera: true,
        ..default()
    };
    for key in KEYS {
        let mut orbit = orbit();
        let before = orbit.pose();
        assert!(!pressed(key).drive(&mut orbit, &blocked).manual_navigation);
        assert_eq!(orbit.pose(), before, "{key:?}");
    }
    let mut orbit = orbit();
    let before = orbit.pose();
    let mut frame = Frame {
        wheel_lines: 1.0,
        ..default()
    };
    frame.drive(&mut orbit, &blocked);
    assert!(
        !frame
            .drag(&mut orbit, &blocked, Vec2::splat(40.0))
            .manual_navigation
    );
    assert_eq!(orbit.pose(), before);

    // A drag held when the table blocks does not resume when it stops blocking.
    let mut frame = Frame::default();
    frame.drag(&mut orbit, &TableFacts::default(), Vec2::splat(10.0));
    let dragged = orbit.pose();
    frame.buttons.press(MouseButton::Right);
    frame.drive(&mut orbit, &blocked);
    frame.window.set_cursor_position(Some(CURSOR));
    frame.buttons.press(MouseButton::Right);
    assert!(
        !frame
            .drive(&mut orbit, &TableFacts::default())
            .manual_navigation
    );
    assert_eq!(orbit.pose(), dragged);
}

#[test]
fn taken_view_keys_and_focus_on_a_button_stop_view_keys_but_not_pan() {
    let taken = TableFacts {
        takes_view_keys: true,
        ..default()
    };
    for (table, focus_on_button) in [(taken, false), (TableFacts::default(), true)] {
        for key in VIEW_KEYS {
            let mut orbit = orbit();
            let before = orbit.pose();
            let mut frame = pressed(key);
            frame.focus_on_button = focus_on_button;
            assert!(!frame.drive(&mut orbit, &table).manual_navigation);
            assert_eq!(orbit.pose(), before, "{key:?}");
        }
        // Recorded: W at distance 16 and 0.1 s pans 0.88 toward negative Z.
        let mut orbit = Orbit::new(Framing::new(Vec3::ZERO, 16.0));
        let mut frame = pressed(KeyCode::KeyW);
        frame.focus_on_button = focus_on_button;
        assert!(frame.drive(&mut orbit, &table).manual_navigation);
        assert!((orbit.pose().target.z + 0.88).abs() < 0.00001);
        // The wheel is not a key.
        let mut orbit = Orbit::new(HOME);
        let mut frame = Frame {
            wheel_lines: 1.0,
            focus_on_button,
            ..default()
        };
        frame.drive(&mut orbit, &table);
        assert!((orbit.pose().distance - 68.4).abs() < 0.0001);
    }
}

#[test]
fn a_taken_wheel_stops_only_the_wheel() {
    let taken = TableFacts {
        takes_wheel: true,
        ..default()
    };
    let mut orbit = Orbit::new(HOME);
    let mut frame = Frame {
        wheel_lines: 1.0,
        ..default()
    };
    assert!(!frame.drive(&mut orbit, &taken).manual_navigation);
    assert_eq!(orbit.pose().distance, HOME.distance);
    // Recorded: a 40 by 10 pixel drag turns yaw by -0.24 and pitch to 1.04.
    assert!(
        frame
            .drag(&mut orbit, &taken, Vec2::new(40.0, 10.0))
            .manual_navigation
    );
    assert!((orbit.pose().yaw + 0.24).abs() < 0.00001);
    assert!((orbit.pose().pitch - 1.04).abs() < 0.00001);
}

#[test]
fn the_pointer_over_ui_or_outside_the_scene_neither_zooms_nor_starts_a_drag() {
    let scene = |min: Vec2, max: Vec2| TableFacts {
        scene: Scene::Within(Rect::from_corners(min, max)),
        ..default()
    };
    let outside = [
        (TableFacts::default(), true),
        (scene(Vec2::ZERO, Vec2::new(800.0, 900.0)), false),
        (scene(Vec2::ZERO, Vec2::new(1000.0, 400.0)), false),
        (scene(Vec2::new(801.0, 0.0), Vec2::splat(1000.0)), false),
    ];
    for (table, pointer_over_ui) in outside {
        let mut orbit = Orbit::new(HOME);
        let before = orbit.pose();
        let mut frame = Frame {
            wheel_lines: 1.0,
            pointer_over_ui,
            ..default()
        };
        assert!(!frame.drive(&mut orbit, &table).manual_navigation);
        assert!(
            !frame
                .drag(&mut orbit, &table, Vec2::splat(40.0))
                .manual_navigation
        );
        assert_eq!(orbit.pose(), before, "{table:?}");
        // Keys still move the camera.
        assert!(
            pressed(KeyCode::ArrowLeft)
                .drive(&mut orbit, &table)
                .manual_navigation
        );
    }
    // The scene holds its minimum corner, so a cursor on it zooms.
    let mut orbit = Orbit::new(HOME);
    let mut frame = Frame {
        wheel_lines: 1.0,
        ..default()
    };
    frame.drive(&mut orbit, &scene(CURSOR, Vec2::splat(1000.0)));
    assert!((orbit.pose().distance - 68.4).abs() < 0.0001);
}

#[test]
fn the_readable_limits_and_default_pose_are_arkhams() {
    // The numbers the motion README records, and Arkham's framing pose.
    assert_eq!((*PITCH.start(), *PITCH.end()), (0.35, 1.35));
    assert_eq!((*DISTANCE.start(), *DISTANCE.end()), (6.0, 90.0));
    assert_eq!((Pose::DEFAULT_YAW, Pose::DEFAULT_PITCH), (0.0, 1.0));
    let framing = Framing::new(Vec3::new(1.0, 0.0, 2.0), 20.0);
    let at = Pose {
        yaw: 0.0,
        pitch: 1.0,
        distance: 20.0,
        target: Vec3::new(1.0, 0.0, 2.0),
    };
    assert_eq!(Pose::at(framing), at);
    assert_eq!(Orbit::new(framing).pose(), at);
    // At yaw 0 and pitch 1 the camera sits above positive Z: (0, sin 1, cos 1).
    let direction = Vec3::new(0.0, 0.841_471, 0.540_302_3);
    assert!(at.direction().abs_diff_eq(direction, 0.000_001));
    let eye = at.transform().translation;
    assert!(eye.abs_diff_eq(Vec3::new(1.0, 0.0, 2.0) + direction * 20.0, 0.0001));
}

#[test]
fn set_home_moves_nothing_until_home_is_pressed() {
    let mut orbit = orbit();
    let before = orbit.pose();
    let fitted = Framing::new(Vec3::new(-1.0, 0.0, 4.0), 12.0);
    orbit.set_home(fitted);
    assert_eq!(orbit.home(), fitted);
    assert_eq!(orbit.pose(), before);
    pressed(KeyCode::Home).drive(&mut orbit, &TableFacts::default());
    assert_eq!(
        orbit.pose(),
        Pose {
            yaw: 0.0,
            pitch: 1.0,
            distance: 12.0,
            target: Vec3::new(-1.0, 0.0, 4.0),
        }
    );
}

/// A clock of the game's own, as Arkham keeps one.
#[derive(Default)]
struct Presentation;

fn table_app() -> App {
    let mut app = App::new();
    app.add_plugins(TableCameraPlugin::<Presentation>::default())
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<AccumulatedMouseScroll>()
        .insert_resource(Orbit::new(HOME));
    let mut presentation = Time::<Presentation>::default();
    presentation.advance_by(Duration::from_secs_f32(0.1));
    app.insert_resource(presentation);
    // Bevy's own clock says otherwise, so a turn shows which one the camera read.
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_secs(1));
    app.insert_resource(time);
    let mut window = Window {
        focused: true,
        ..default()
    };
    window.set_cursor_position(Some(CURSOR));
    app.world_mut().spawn((window, PrimaryWindow));
    app
}

#[test]
fn the_plugin_reads_the_games_clock_writes_the_table_camera_and_reports_navigation() {
    let mut app = table_app();
    let camera = app
        .world_mut()
        .spawn((TableCamera, Transform::default()))
        .id();
    let other = app.world_mut().spawn(Transform::default()).id();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ArrowLeft);
    app.update();
    let pose = app.world().resource::<Orbit>().pose();
    assert!((pose.yaw - 0.1).abs() < 0.00001, "yaw {}", pose.yaw);
    // Yaw 0.1 and pitch 1 at 72 from (0, 0, 7): (sin 0.1 cos 1, sin 1, cos 0.1 cos 1) * 72 on.
    let written = app.world().get::<Transform>(camera).unwrap();
    let eye = Vec3::new(0.0, 0.0, 7.0) + Vec3::new(0.053_940_2, 0.841_471, 0.537_603_0) * 72.0;
    assert!(written.translation.abs_diff_eq(eye, 0.0001), "{written:?}");
    assert_eq!(
        app.world().get::<Transform>(other),
        Some(&Transform::default())
    );
    assert_eq!(
        *app.world().resource::<Navigation>(),
        Navigation {
            manual: true,
            stopped_following: false,
        }
    );

    app.world_mut().resource_mut::<Orbit>().set_following(true);
    app.update();
    assert_eq!(
        *app.world().resource::<Navigation>(),
        Navigation {
            manual: true,
            stopped_following: true,
        }
    );
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::ArrowLeft);
    app.update();
    assert_eq!(*app.world().resource::<Navigation>(), Navigation::default());
}

#[test]
fn the_plugin_finds_the_pointer_over_a_ui_node_not_in_the_scene_and_focus_on_a_button() {
    let mut app = table_app();
    let node = app.world_mut().spawn(Node::default()).id();
    let in_scene = app.world_mut().spawn((Node::default(), InScene)).id();
    let mesh = app.world_mut().spawn(Transform::default()).id();
    let button = app.world_mut().spawn(Button).id();
    let hovering = |entity: Entity| {
        let hit = HitData::new(entity, 0.0, None, None);
        HoverMap(HashMap::from_iter([(
            PointerId::Mouse,
            [(entity, hit)].into_iter().collect(),
        )]))
    };

    // Hovering a scene entity, the wheel zooms; hovering a UI node, it does not.
    app.insert_resource(hovering(mesh));
    app.world_mut()
        .resource_mut::<AccumulatedMouseScroll>()
        .delta = Vec2::Y;
    app.update();
    let zoomed = app.world().resource::<Orbit>().pose().distance;
    assert!((zoomed - 68.4).abs() < 0.0001);
    app.insert_resource(hovering(node));
    app.update();
    assert_eq!(app.world().resource::<Orbit>().pose().distance, zoomed);
    // A UI node that stands for something in the scene: 68.4 * 0.95.
    app.insert_resource(hovering(in_scene));
    app.update();
    let distance = app.world().resource::<Orbit>().pose().distance;
    assert!((distance - 64.98).abs() < 0.0001, "{distance}");
    app.world_mut()
        .resource_mut::<AccumulatedMouseScroll>()
        .delta = Vec2::ZERO;

    // Focus on a button takes the view keys; focus on another entity does not.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ArrowLeft);
    app.insert_resource(InputFocus::from_entity(button));
    app.update();
    assert_eq!(app.world().resource::<Orbit>().pose().yaw, 0.0);
    app.insert_resource(InputFocus::from_entity(node));
    app.update();
    assert!((app.world().resource::<Orbit>().pose().yaw - 0.1).abs() < 0.00001);
}

#[test]
fn a_game_orders_its_facts_before_the_camera_and_can_stop_it() {
    let mut app = table_app();
    app.add_systems(
        Update,
        (|mut table: ResMut<TableFacts>| table.blocks_camera = true).before(TableCameraSystems),
    );
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ArrowLeft);
    app.update();
    assert_eq!(app.world().resource::<Orbit>().pose().yaw, 0.0);

    let mut app = table_app();
    app.configure_sets(Update, TableCameraSystems.run_if(|| false));
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ArrowLeft);
    app.update();
    assert_eq!(app.world().resource::<Orbit>().pose().yaw, 0.0);
}
