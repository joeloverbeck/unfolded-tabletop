#![cfg(feature = "render")]

//! Table camera motion at `camera`'s public interface. The expected numbers were
//! recorded by Arkham's camera tests before the move; they are oracles, not
//! values derived from the implementation.

use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use unfolded_tabletop::camera::{CameraStep, Capture, FrameInput, Orbit, Pose, wheel_steps};

/// A home like Arkham's overview, so the recorded numbers apply unchanged.
const HOME: (Vec3, f32) = (Vec3::new(0.0, 0.0, 7.0), 72.0);
/// A nearer detail framing.
const DETAIL: (Vec3, f32) = (Vec3::new(0.0, 0.0, 8.5), 20.0);
/// Another framing, used only as a starting pose or follow target.
const ASIDE: (Vec3, f32) = (Vec3::new(-2.0, 0.0, -6.0), 30.0);

struct Inputs {
    keys: ButtonInput<KeyCode>,
    buttons: ButtonInput<MouseButton>,
    window: Window,
    capture: Capture,
    scroll_y: f32,
    dt: f32,
}

impl Default for Inputs {
    fn default() -> Self {
        let mut window = Window {
            focused: true,
            ..default()
        };
        window.set_cursor_position(Some(Vec2::new(800.0, 400.0)));
        Self {
            keys: default(),
            buttons: default(),
            window,
            capture: default(),
            scroll_y: 0.0,
            dt: 0.1,
        }
    }
}

impl Inputs {
    fn step(&mut self, orbit: &mut Orbit) -> CameraStep {
        let step = orbit.step(FrameInput {
            keys: &self.keys,
            buttons: &self.buttons,
            window: &self.window,
            scroll_y: self.scroll_y,
            dt: self.dt,
            capture: self.capture,
        });
        self.buttons.clear();
        self.keys.clear();
        self.scroll_y = 0.0;
        step
    }

    fn move_cursor(&mut self, delta: Vec2) {
        let position = self.window.cursor_position().unwrap() + delta;
        self.window.set_cursor_position(Some(position));
    }
}

fn looks_at(step: &CameraStep, (target, distance): (Vec3, f32)) {
    assert!((step.transform.translation.distance(target) - distance).abs() < 0.00001);
    assert!(
        step.transform
            .forward()
            .dot((target - step.transform.translation).normalize())
            > 0.99999
    );
}

#[test]
fn scene_drag_crosses_ui_and_home_returns_to_the_consumers_framing() {
    let home = (Vec3::new(3.0, 0.0, -4.0), 26.0);
    let mut orbit = Orbit::new(home);
    let mut keys = ButtonInput::default();
    let mut buttons = ButtonInput::default();
    let mut window = Window {
        focused: true,
        ..default()
    };
    window.set_cursor_position(Some(Vec2::new(300.0, 200.0)));
    buttons.press(MouseButton::Right);
    let step = orbit.step(FrameInput {
        keys: &keys,
        buttons: &buttons,
        window: &window,
        scroll_y: 0.0,
        dt: 0.1,
        capture: Capture::default(),
    });
    assert!(!step.manual_navigation);
    buttons.clear();
    window.set_cursor_position(Some(Vec2::new(340.0, 210.0)));
    let step = orbit.step(FrameInput {
        keys: &keys,
        buttons: &buttons,
        window: &window,
        scroll_y: 0.0,
        dt: 0.1,
        capture: Capture {
            over_ui: true,
            ..default()
        },
    });
    assert!(step.manual_navigation);
    assert!((orbit.pose().yaw + 0.24).abs() < 0.00001);
    assert!((orbit.pose().pitch - 1.04).abs() < 0.00001);

    buttons.release(MouseButton::Right);
    keys.press(KeyCode::Home);
    let step = orbit.step(FrameInput {
        keys: &keys,
        buttons: &buttons,
        window: &window,
        scroll_y: 0.0,
        dt: 0.1,
        capture: Capture::default(),
    });
    assert_eq!(orbit.pose().target, home.0);
    assert_eq!(orbit.pose().distance, home.1);
    assert_eq!(orbit.pose().yaw, 0.0);
    assert_eq!(orbit.pose().pitch, 1.0);
    looks_at(&step, home);
}

#[test]
fn a_new_orbit_starts_manual_at_home_and_home_restores_it() {
    let mut orbit = Orbit::new(HOME);
    let home = orbit.pose();
    assert_eq!(
        home,
        Pose {
            yaw: 0.0,
            pitch: 1.0,
            target: HOME.0,
            distance: HOME.1,
        }
    );
    assert!(!orbit.is_following());
    assert!(!orbit.is_moving());
    orbit.focus(DETAIL);
    assert_eq!(orbit.pose().target, DETAIL.0);
    assert_eq!(orbit.pose().distance, DETAIL.1);
    let mut input = Inputs::default();
    input.keys.press(KeyCode::Home);
    let step = input.step(&mut orbit);
    assert!(step.manual_navigation);
    assert_eq!(orbit.pose(), home);
    looks_at(&step, HOME);
}

#[test]
fn with_home_replaces_home_and_starts_there() {
    let mut orbit = Orbit::new(HOME).with_home(ASIDE);
    assert_eq!(orbit.pose().target, ASIDE.0);
    assert_eq!(orbit.pose().distance, ASIDE.1);
    assert!(!orbit.is_moving());
    orbit.focus(DETAIL);
    let mut input = Inputs::default();
    input.keys.press(KeyCode::Home);
    let step = input.step(&mut orbit);
    assert_eq!(orbit.pose().target, ASIDE.0);
    assert_eq!(orbit.pose().distance, ASIDE.1);
    looks_at(&step, ASIDE);
}

#[test]
fn focus_applies_once_without_replacing_home_or_live_following() {
    for framing in [HOME, DETAIL, ASIDE] {
        let mut orbit = Orbit::new(HOME);
        let normal = orbit.pose();
        orbit.focus(framing);
        let mut input = Inputs::default();
        let first_frame = input.step(&mut orbit);
        assert_eq!(orbit.pose().target, framing.0);
        assert_eq!(orbit.pose().distance, framing.1);
        assert!(!orbit.is_following());
        assert!(!orbit.is_moving());
        looks_at(&first_frame, framing);

        orbit.set_following(true);
        let follow_target = Vec3::new(3.0, 0.0, 7.0);
        orbit.update_follow_target(Some((follow_target, 15.0).into()));
        input.dt = 1.0;
        input.step(&mut orbit);
        assert_eq!(orbit.pose().target, follow_target);
        assert!(orbit.is_following());

        input.keys.press(KeyCode::Home);
        input.step(&mut orbit);
        assert_eq!(orbit.pose(), normal);
        assert!(!orbit.is_following());
        input.keys.release(KeyCode::Home);
        input.step(&mut orbit);
        assert_eq!(orbit.pose(), normal, "focus must not be reapplied");
    }
}

#[test]
fn wheel_zoom_scales_with_view_distance_and_reverses_without_drift() {
    for (distance, expected) in [(26.0, 24.7), (72.0, 68.4)] {
        let mut orbit = Orbit::new(HOME);
        orbit.focus((Vec3::ZERO, distance));
        let mut input = Inputs {
            scroll_y: 1.0,
            ..default()
        };
        input.step(&mut orbit);
        assert!((orbit.pose().distance - expected).abs() < 0.0001);
        input.scroll_y = -1.0;
        input.step(&mut orbit);
        assert!((orbit.pose().distance - distance).abs() < 0.0001);
    }
}

#[test]
fn pixel_scroll_keeps_fractional_motion_and_matches_line_scroll() {
    let mut line_orbit = Orbit::new(HOME);
    let mut pixel_orbit = Orbit::new(HOME);
    let mut input = Inputs {
        scroll_y: wheel_steps(&AccumulatedMouseScroll {
            unit: MouseScrollUnit::Line,
            delta: Vec2::new(0.0, 1.0),
        }),
        ..default()
    };
    input.step(&mut line_orbit);
    for _ in 0..10 {
        input.scroll_y = wheel_steps(&AccumulatedMouseScroll {
            unit: MouseScrollUnit::Pixel,
            delta: Vec2::new(0.0, 10.0),
        });
        input.step(&mut pixel_orbit);
    }
    assert!((pixel_orbit.pose().distance - 68.4).abs() < 0.0001);
    assert!((pixel_orbit.pose().distance - line_orbit.pose().distance).abs() < 0.0001);
}

#[test]
fn scene_drag_uses_logical_cursor_displacement_and_crosses_ui_until_release() {
    let mut orbit = Orbit::new(HOME);
    let mut input = Inputs::default();
    input.buttons.press(MouseButton::Right);
    assert!(!input.step(&mut orbit).manual_navigation);
    assert_eq!(orbit.pose().yaw, 0.0, "the press only captures the cursor");
    input.capture.over_ui = true;
    input.move_cursor(Vec2::new(40.0, 10.0));
    assert!(input.step(&mut orbit).manual_navigation);
    assert!((orbit.pose().yaw + 0.24).abs() < 0.00001);
    assert!((orbit.pose().pitch - 1.04).abs() < 0.00001);
    let moved = orbit.pose();
    input.buttons.release(MouseButton::Right);
    input.move_cursor(Vec2::splat(40.0));
    assert!(!input.step(&mut orbit).manual_navigation);
    assert_eq!(orbit.pose(), moved);
}

#[test]
fn drag_begun_on_ui_requires_a_fresh_scene_press() {
    let mut orbit = Orbit::new(HOME);
    let mut input = Inputs::default();
    let initial = orbit.pose();
    input.capture.over_ui = true;
    input.buttons.press(MouseButton::Right);
    input.step(&mut orbit);
    input.capture.over_ui = false;
    input.move_cursor(Vec2::splat(40.0));
    input.step(&mut orbit);
    assert_eq!(orbit.pose(), initial);
    input.buttons.release(MouseButton::Right);
    input.step(&mut orbit);
    input.buttons.press(MouseButton::Right);
    input.step(&mut orbit);
    input.move_cursor(Vec2::splat(40.0));
    assert!(input.step(&mut orbit).manual_navigation);
}

#[test]
fn interruptions_cancel_capture_without_a_resume_jump() {
    for interruption in 0..3 {
        let mut orbit = Orbit::new(HOME);
        let mut input = Inputs::default();
        input.buttons.press(MouseButton::Right);
        input.step(&mut orbit);
        input.move_cursor(Vec2::splat(10.0));
        input.step(&mut orbit);
        let before = orbit.pose();
        match interruption {
            0 => input.window.focused = false,
            1 => input.capture.pointer_blocked = true,
            _ => input.window.set_cursor_position(None),
        }
        input.step(&mut orbit);
        input.window.focused = true;
        input.capture.pointer_blocked = false;
        input
            .window
            .set_cursor_position(Some(Vec2::new(1000.0, 500.0)));
        input.step(&mut orbit);
        assert_eq!(orbit.pose(), before, "interruption {interruption}");
        input.buttons.release(MouseButton::Right);
        input.step(&mut orbit);
        input.buttons.press(MouseButton::Right);
        input.step(&mut orbit);
        input.move_cursor(Vec2::splat(10.0));
        assert!(input.step(&mut orbit).manual_navigation);
    }
}

#[test]
fn pan_is_camera_relative_normalized_and_stops_on_release() {
    let mut orbit = Orbit::new(HOME);
    orbit.focus((Vec3::ZERO, 16.0));
    let mut input = Inputs::default();
    let start = orbit.pose();
    input.keys.press(KeyCode::KeyW);
    input.step(&mut orbit);
    let panned = orbit.pose();
    assert!((panned.target.z - (start.target.z - 0.88)).abs() < 0.00001);
    assert_eq!(panned.pitch, start.pitch);
    assert_eq!(panned.distance, start.distance);
    input.keys.press(KeyCode::KeyD);
    input.step(&mut orbit);
    assert!((orbit.pose().target.distance(panned.target) - 0.88).abs() < 0.00001);
    input.keys.reset_all();
    let stopped = orbit.pose();
    input.step(&mut orbit);
    assert_eq!(orbit.pose(), stopped);
    input.keys.press(KeyCode::ArrowLeft);
    input.dt = std::f32::consts::FRAC_PI_2;
    input.step(&mut orbit);
    input.keys.reset_all();
    input.keys.press(KeyCode::KeyW);
    input.dt = 0.1;
    input.step(&mut orbit);
    let delta = orbit.pose().target - stopped.target;
    assert!((delta.x + 0.88).abs() < 0.00001 && delta.z.abs() < 0.00001);
}

#[test]
fn keyboard_capture_focus_loss_and_modified_wasd_preserve_the_pose() {
    let mut orbit = Orbit::new(HOME);
    let mut input = Inputs::default();
    orbit.focus(ASIDE);
    let before = orbit.pose();
    for key in [
        KeyCode::KeyW,
        KeyCode::ArrowLeft,
        KeyCode::Equal,
        KeyCode::Home,
    ] {
        input.keys.press(key);
    }
    input.capture.keyboard = true;
    input.step(&mut orbit);
    assert_eq!(orbit.pose(), before);
    input.capture.keyboard = false;
    input.window.focused = false;
    input.step(&mut orbit);
    assert_eq!(orbit.pose(), before);
    input.window.focused = true;
    for modifier in [
        KeyCode::ShiftLeft,
        KeyCode::ShiftRight,
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::AltLeft,
        KeyCode::AltRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ] {
        input.keys.reset_all();
        input.keys.press(KeyCode::KeyW);
        input.keys.press(modifier);
        assert!(!input.step(&mut orbit).manual_navigation);
        assert_eq!(orbit.pose(), before);
    }
}

#[test]
fn manual_navigation_overrides_following_and_freezes_pending_framing() {
    for gesture in 0..5 {
        let mut orbit = Orbit::new(HOME);
        let mut input = Inputs::default();
        orbit.set_following(true);
        orbit.update_follow_target(Some(ASIDE.into()));
        match gesture {
            0 => {
                input.buttons.press(MouseButton::Right);
                input.step(&mut orbit);
                input.move_cursor(Vec2::splat(10.0));
            }
            1 => input.scroll_y = 1.0,
            2 => input.keys.press(KeyCode::ArrowLeft),
            3 => input.keys.press(KeyCode::KeyW),
            _ => input.keys.press(KeyCode::Home),
        }
        let step = input.step(&mut orbit);
        assert!(step.manual_navigation && step.stopped_following);
        assert!(!orbit.is_following());
        input.keys.reset_all();
        let stopped = orbit.pose();
        assert!(!input.step(&mut orbit).manual_navigation);
        assert_eq!(orbit.pose(), stopped);
    }
}

#[test]
fn follow_interpolates_and_follow_off_freezes_framing() {
    let mut orbit = Orbit::new(HOME);
    let mut input = Inputs::default();
    orbit.focus(HOME);
    orbit.set_following(true);
    orbit.update_follow_target(Some(DETAIL.into()));
    assert_eq!(orbit.pose().distance, 72.0);
    assert!(orbit.is_moving());
    let step = input.step(&mut orbit);
    assert!(!step.manual_navigation && !step.stopped_following);
    assert!((orbit.pose().target.z - 7.6).abs() < 0.00001);
    assert!((orbit.pose().distance - 51.2).abs() < 0.00001);
    orbit.set_following(false);
    orbit.update_follow_target(None);
    input.step(&mut orbit);
    assert!((orbit.pose().target.z - 7.6).abs() < 0.00001);
    assert!(!orbit.is_moving());
    orbit.set_following(true);
    orbit.update_follow_target(Some(DETAIL.into()));
    input.step(&mut orbit);
    assert!((orbit.pose().target.z - 7.96).abs() < 0.00001);
}

#[test]
fn wheel_capture_and_distance_limits_are_preserved() {
    let mut orbit = Orbit::new(HOME);
    let mut input = Inputs::default();
    let initial = orbit.pose();
    for capture in [
        Capture {
            wheel: true,
            ..default()
        },
        Capture {
            over_ui: true,
            ..default()
        },
        Capture {
            pointer_blocked: true,
            ..default()
        },
    ] {
        input.capture = capture;
        input.scroll_y = 100.0;
        assert!(!input.step(&mut orbit).manual_navigation);
        assert_eq!(orbit.pose(), initial);
    }
    input.capture = default();
    input.scroll_y = -200.0;
    input.step(&mut orbit);
    assert_eq!(orbit.pose().distance, 90.0);
    input.scroll_y = 300.0;
    input.step(&mut orbit);
    assert_eq!(orbit.pose().distance, 6.0);
    input.keys.press(KeyCode::ArrowUp);
    input.dt = 0.1;
    for _ in 0..20 {
        input.step(&mut orbit);
    }
    assert_eq!(orbit.pose().pitch, 1.35);
    input.keys.reset_all();
    input.keys.press(KeyCode::ArrowDown);
    for _ in 0..20 {
        input.step(&mut orbit);
    }
    assert_eq!(orbit.pose().pitch, 0.35);
}

#[test]
fn a_configured_home_pitch_applies_at_start_and_when_home_is_pressed() {
    let mut orbit = Orbit::new(HOME)
        .with_navigation(0.2, 0.1..=0.8, 12.0..=80.0)
        .unwrap();
    assert_eq!(orbit.pose().pitch, 0.2);
    assert_eq!(orbit.pose().distance, 72.0);

    let mut input = Inputs::default();
    input.keys.press(KeyCode::ArrowUp);
    input.keys.press(KeyCode::ArrowRight);
    input.step(&mut orbit);
    assert!((orbit.pose().pitch - 0.27).abs() < 0.000_001);
    assert!((orbit.pose().yaw + 0.1).abs() < 0.000_001);

    orbit.set_home(ASIDE);
    input.keys.reset_all();
    input.keys.press(KeyCode::Home);
    let step = input.step(&mut orbit);
    assert_eq!(
        orbit.pose(),
        Pose {
            yaw: 0.0,
            pitch: 0.2,
            distance: 30.0,
            target: Vec3::new(-2.0, 0.0, -6.0),
        }
    );
    // A distance of 30 at 0.2 radians above the target on the positive Z side.
    let eye = Vec3::new(-2.0, 5.960_08, 23.402);
    assert!(step.transform.translation.abs_diff_eq(eye, 0.000_1));
}

#[test]
fn a_configured_pitch_range_limits_drag_and_keyboard_turning() {
    let mut orbit = Orbit::new(HOME)
        .with_navigation(0.2, 0.1..=0.8, 12.0..=80.0)
        .unwrap();
    let mut input = Inputs::default();

    input.buttons.press(MouseButton::Right);
    input.step(&mut orbit);
    input.move_cursor(Vec2::new(0.0, -100.0));
    input.step(&mut orbit);
    assert_eq!(orbit.pose().pitch, 0.1);
    input.move_cursor(Vec2::new(0.0, 250.0));
    input.step(&mut orbit);
    assert_eq!(orbit.pose().pitch, 0.8);

    input.buttons.release(MouseButton::Right);
    input.keys.press(KeyCode::ArrowDown);
    for _ in 0..20 {
        input.step(&mut orbit);
    }
    assert_eq!(orbit.pose().pitch, 0.1);

    input.keys.reset_all();
    input.keys.press(KeyCode::ArrowUp);
    for _ in 0..20 {
        input.step(&mut orbit);
    }
    assert_eq!(orbit.pose().pitch, 0.8);
}

#[test]
fn a_configured_distance_range_limits_wheel_and_keyboard_zoom() {
    let mut orbit = Orbit::new(DETAIL)
        .with_navigation(0.2, 0.1..=0.8, 18.0..=22.0)
        .unwrap();
    let mut input = Inputs {
        scroll_y: -100.0,
        ..default()
    };
    input.step(&mut orbit);
    assert_eq!(orbit.pose().distance, 22.0);

    input.scroll_y = 100.0;
    input.step(&mut orbit);
    assert_eq!(orbit.pose().distance, 18.0);

    input.keys.press(KeyCode::Minus);
    for _ in 0..10 {
        input.step(&mut orbit);
        assert!((18.0..=22.0).contains(&orbit.pose().distance));
    }
    assert_eq!(orbit.pose().distance, 22.0);

    input.keys.reset_all();
    input.keys.press(KeyCode::Equal);
    for _ in 0..10 {
        input.step(&mut orbit);
        assert!((18.0..=22.0).contains(&orbit.pose().distance));
    }
    assert_eq!(orbit.pose().distance, 18.0);
}
