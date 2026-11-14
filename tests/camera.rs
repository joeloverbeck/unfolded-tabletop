#![cfg(feature = "render")]

use bevy::prelude::*;
use unfolded_tabletop::camera::{Capture, FrameInput, Orbit};

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
    assert!((step.transform.translation.distance(home.0) - 26.0).abs() < 0.00001);
    assert!(
        step.transform
            .forward()
            .dot((home.0 - step.transform.translation).normalize())
            > 0.99999
    );
}
