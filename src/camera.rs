//! Table camera motion and input capture, with consumer-owned framing and UI policy.

use bevy::{input::mouse::AccumulatedMouseScroll, prelude::*};

/// Use Bevy's standard pixel-to-line conversion so touchpad deltas do not become notches.
pub fn wheel_steps(scroll: &AccumulatedMouseScroll) -> f32 {
    scroll.to_lines(&default()).delta.y
}

/// A read-only snapshot; the camera alone owns the mutable orbit state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub target: Vec3,
}

impl Pose {
    fn at((target, distance): (Vec3, f32)) -> Self {
        Self {
            yaw: 0.0,
            pitch: 1.0,
            distance,
            target,
        }
    }

    fn transform(self) -> Transform {
        let direction = Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.cos() * self.pitch.cos(),
        );
        Transform::from_translation(self.target + direction * self.distance)
            .looking_at(self.target, Vec3::Y)
    }
}

/// Facts supplied by the adapter's existing UI/picking systems.
#[derive(Clone, Copy, Default)]
pub struct Capture {
    pub pointer_blocked: bool,
    pub over_ui: bool,
    pub keyboard: bool,
    /// WASD capture when it differs from other camera keys; otherwise inherit keyboard capture.
    pub pan: Option<bool>,
    pub wheel: bool,
}

pub struct FrameInput<'a> {
    pub keys: &'a ButtonInput<KeyCode>,
    pub buttons: &'a ButtonInput<MouseButton>,
    pub window: &'a Window,
    pub scroll_y: f32,
    pub dt: f32,
    pub capture: Capture,
}

pub struct CameraStep {
    pub transform: Transform,
    pub manual_navigation: bool,
    pub stopped_following: bool,
}

#[derive(Resource)]
pub struct Orbit {
    home: (Vec3, f32),
    pose: Pose,
    desired_target: Vec3,
    desired_distance: f32,
    following: bool,
    drag: Option<Vec2>,
}

impl Orbit {
    /// Start at the consumer's home target and distance, with a manual camera.
    pub fn new(home: (Vec3, f32)) -> Self {
        let pose = Pose::at(home);
        Self {
            home,
            pose,
            desired_target: pose.target,
            desired_distance: pose.distance,
            following: false,
            drag: None,
        }
    }

    pub fn with_home(mut self, framing: (Vec3, f32)) -> Self {
        self.home = framing;
        self.focus(framing);
        self
    }

    pub fn pose(&self) -> Pose {
        self.pose
    }

    pub fn is_following(&self) -> bool {
        self.following
    }

    pub fn is_moving(&self) -> bool {
        !self.pose.target.abs_diff_eq(self.desired_target, 0.001)
            || (self.pose.distance - self.desired_distance).abs() >= 0.001
    }

    pub fn set_following(&mut self, following: bool) {
        self.following = following;
    }

    pub fn focus(&mut self, framing: (Vec3, f32)) {
        self.pose = Pose::at(framing);
        self.freeze_framing();
        self.following = false;
    }

    /// Called at the existing action boundary, never derived afresh each frame.
    /// Freeze framing on actions while following is off.
    pub fn update_follow_target(&mut self, framing: Option<(Vec3, f32)>) {
        if !self.following {
            self.freeze_framing();
        } else if let Some((target, distance)) = framing {
            self.desired_target = target;
            self.desired_distance = distance;
        }
    }

    fn freeze_framing(&mut self) {
        self.desired_target = self.pose.target;
        self.desired_distance = self.pose.distance;
    }

    pub fn step(&mut self, input: FrameInput<'_>) -> CameraStep {
        let pointer_enabled = input.window.focused && !input.capture.pointer_blocked;
        let cursor = input.window.cursor_position();
        let scene_pointer = pointer_enabled && cursor.is_some() && !input.capture.over_ui;
        // Logical window positions are the only motion source: raw device motion
        // must never be added to this displacement, including on high-DPI displays.
        let motion = match cursor {
            Some(position) if pointer_enabled && input.buttons.pressed(MouseButton::Right) => {
                if input.buttons.just_pressed(MouseButton::Right) {
                    self.drag = scene_pointer.then_some(position);
                    Vec2::ZERO
                } else if let Some(previous) = self.drag.as_mut() {
                    let delta = position - *previous;
                    *previous = position;
                    delta
                } else {
                    Vec2::ZERO
                }
            }
            _ => {
                self.drag = None;
                Vec2::ZERO
            }
        };
        let was_following = self.following;
        let mut changed = motion != Vec2::ZERO;
        if changed {
            self.pose.yaw -= motion.x * 0.006;
            self.pose.pitch += motion.y * 0.004;
        }
        if scene_pointer && !input.capture.wheel && input.scroll_y != 0.0 {
            // Equal opposite scrolls undo each other away from the limits. A fixed
            // proportion traverses the overview quickly while preserving close control.
            self.pose.distance =
                (self.pose.distance * 0.95_f32.powf(input.scroll_y)).clamp(6.0, 90.0);
            changed = true;
        }
        if input.window.focused && !input.capture.keyboard {
            for (key, yaw, pitch, zoom) in [
                (KeyCode::ArrowLeft, 1.0, 0.0, 0.0),
                (KeyCode::ArrowRight, -1.0, 0.0, 0.0),
                (KeyCode::ArrowUp, 0.0, 0.7, 0.0),
                (KeyCode::ArrowDown, 0.0, -0.7, 0.0),
                (KeyCode::Equal, 0.0, 0.0, -5.0),
                (KeyCode::Minus, 0.0, 0.0, 5.0),
            ] {
                if input.keys.pressed(key) {
                    self.pose.yaw += yaw * input.dt;
                    self.pose.pitch += pitch * input.dt;
                    self.pose.distance += zoom * input.dt;
                    changed = true;
                }
            }
            if input.keys.just_pressed(KeyCode::Home) {
                // Home resets framing, not a held scene drag's capture.
                self.focus(self.home);
                changed = true;
            }
        }
        if input.window.focused && !input.capture.pan.unwrap_or(input.capture.keyboard) {
            let delta = pan_delta(input.keys, self.pose.yaw, input.dt, self.pose.distance);
            if delta != Vec3::ZERO {
                self.pose.target += delta;
                changed = true;
            }
        }
        if changed {
            self.following = false;
            self.freeze_framing();
        }
        self.pose.pitch = self.pose.pitch.clamp(0.35, 1.35);
        self.pose.distance = self.pose.distance.clamp(6.0, 90.0);
        let blend = (input.dt * 4.0).min(1.0);
        // Preserve the original clamp-then-interpolate ordering.
        self.pose.distance += (self.desired_distance - self.pose.distance) * blend;
        self.pose.target = self.pose.target.lerp(self.desired_target, blend);
        if self.pose.target.abs_diff_eq(self.desired_target, 0.001) {
            self.pose.target = self.desired_target;
        }
        if (self.pose.distance - self.desired_distance).abs() < 0.001 {
            self.pose.distance = self.desired_distance;
        }
        CameraStep {
            transform: self.pose.transform(),
            manual_navigation: changed,
            stopped_following: changed && was_following,
        }
    }
}

/// Unmodified WASD pans in the camera's horizontal plane at a zoom-scaled rate.
fn pan_delta(keys: &ButtonInput<KeyCode>, yaw: f32, dt: f32, distance: f32) -> Vec3 {
    if keys.any_pressed([
        KeyCode::ShiftLeft,
        KeyCode::ShiftRight,
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::AltLeft,
        KeyCode::AltRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]) {
        return Vec3::ZERO;
    }
    let horizontal =
        f32::from(keys.pressed(KeyCode::KeyD)) - f32::from(keys.pressed(KeyCode::KeyA));
    let forward = f32::from(keys.pressed(KeyCode::KeyW)) - f32::from(keys.pressed(KeyCode::KeyS));
    let direction = Vec2::new(horizontal, forward).normalize_or_zero();
    let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
    let ahead = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
    (right * direction.x + ahead * direction.y) * dt * distance * 0.55
}
