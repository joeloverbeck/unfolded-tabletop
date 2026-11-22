//! The table camera: its motion, and the input it gathers for itself each frame.
//!
//! A game adds [`TableCameraPlugin`], inserts an [`Orbit`] at its home [`Framing`],
//! marks its camera with [`TableCamera`], and writes [`TableFacts`] each frame before
//! [`TableCameraSystems`]: whether its table blocks the camera, takes the
//! [`VIEW_KEYS`] or the wheel, and which part of the window shows the scene. The
//! camera reads the keys, mouse buttons, wheel, primary window and frame time itself,
//! works out whether the pointer is over a UI node and whether keyboard focus is on a
//! button, moves, and writes the camera's transform. A UI node marked [`InScene`]
//! counts as part of the scene. After [`TableCameraSystems`],
//! [`Navigation`] says whether this frame's input moved it.
//!
//! [`Orbit`] owns the orbit, camera-relative WASD pan, wheel normalization, scene-drag
//! capture and follow interpolation. The game keeps its Home framing and navigation
//! settings, its focus and follow targets and when they change, its named views,
//! its table framing and pointer regions. Visibility, modal lifecycle and viewport
//! policy also stay with the game.
//!
//! With default navigation the pitch range is [`PITCH`] (0.35 to 1.35 radians) and the
//! distance clamp uses [`DISTANCE`] (6 to 90 world units). A game can choose its Home
//! pitch and manual limits with [`Orbit::with_navigation`]. Wheel distance scales by
//! 0.95 per line. Follow interpolation runs after the distance clamp, and a framing
//! or a follow target may be outside the manual distance range.
//!
//! [`Orbit::step`] with [`FrameInput`] and [`Capture`] is the motion alone, without
//! gathering. It is kept while games move to [`TableCameraPlugin`].
//!
//! `tests/seat.rs` drives a seated table's camera through Bevy's own input messages: a
//! right `drag` from the scene orbits it, and a `scroll` on the scene zooms it, while
//! the same input from a UI node, or a wheel the table takes, does not. A game's seat
//! test can do the same on its real table. What no test holds is that a real window
//! delivers that input: each game's own native inspection of orbit, pan, wheel, drag
//! and Home does.

use std::{marker::PhantomData, ops::RangeInclusive};

use bevy::{
    ecs::system::SystemParam,
    input::mouse::AccumulatedMouseScroll,
    input_focus::InputFocus,
    picking::{hover::HoverMap, pointer::PointerId},
    prelude::*,
    ui_widgets::Button,
    window::PrimaryWindow,
};

/// The default pitch limits, in radians above the table.
pub const PITCH: RangeInclusive<f32> = 0.35..=1.35;

/// The default distance limits, in world units. A framing may be
/// nearer or farther; follow interpolation is not clamped.
pub const DISTANCE: RangeInclusive<f32> = 6.0..=90.0;

/// Arrow keys turn the camera, `=` and `-` zoom, and Home returns to the home framing.
pub const VIEW_KEYS: [KeyCode; 7] = [
    KeyCode::Equal,
    KeyCode::Minus,
    KeyCode::ArrowLeft,
    KeyCode::ArrowRight,
    KeyCode::ArrowUp,
    KeyCode::ArrowDown,
    KeyCode::Home,
];

/// Unmodified WASD pans in the camera's horizontal plane.
pub const PAN_KEYS: [KeyCode; 4] = [KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD];

/// Every key that moves the camera: [`PAN_KEYS`], then [`VIEW_KEYS`]. Shift, Control,
/// Alt and Super stop WASD pan, but move nothing themselves.
pub const KEYS: [KeyCode; 11] = [
    PAN_KEYS[0],
    PAN_KEYS[1],
    PAN_KEYS[2],
    PAN_KEYS[3],
    VIEW_KEYS[0],
    VIEW_KEYS[1],
    VIEW_KEYS[2],
    VIEW_KEYS[3],
    VIEW_KEYS[4],
    VIEW_KEYS[5],
    VIEW_KEYS[6],
];

/// [`KEYS`] for the agent seat, in the same order: none of them commits a choice.
#[cfg(feature = "seat")]
pub static SEAT_KEYS: [crate::seat::SeatKey; 11] = {
    use crate::seat::{Logical, SeatKey};
    use bevy::input::keyboard::Key;
    const fn key(name: &'static str, code: KeyCode, logical: Logical) -> SeatKey {
        SeatKey {
            name,
            code,
            logical,
            commits: false,
        }
    }
    [
        key("KeyW", KEYS[0], Logical::Typed("w")),
        key("KeyA", KEYS[1], Logical::Typed("a")),
        key("KeyS", KEYS[2], Logical::Typed("s")),
        key("KeyD", KEYS[3], Logical::Typed("d")),
        key("Equal", KEYS[4], Logical::Typed("=")),
        key("Minus", KEYS[5], Logical::Typed("-")),
        key("ArrowLeft", KEYS[6], Logical::Named(Key::ArrowLeft)),
        key("ArrowRight", KEYS[7], Logical::Named(Key::ArrowRight)),
        key("ArrowUp", KEYS[8], Logical::Named(Key::ArrowUp)),
        key("ArrowDown", KEYS[9], Logical::Named(Key::ArrowDown)),
        key("Home", KEYS[10], Logical::Named(Key::Home)),
    ]
};

/// Use Bevy's standard pixel-to-line conversion so touchpad deltas do not become notches.
pub fn wheel_steps(scroll: &AccumulatedMouseScroll) -> f32 {
    scroll.to_lines(&default()).delta.y
}

/// A point on the table and the distance the camera views it from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Framing {
    pub target: Vec3,
    pub distance: f32,
}

impl Framing {
    pub const fn new(target: Vec3, distance: f32) -> Self {
        Self { target, distance }
    }
}

impl From<(Vec3, f32)> for Framing {
    fn from((target, distance): (Vec3, f32)) -> Self {
        Self::new(target, distance)
    }
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
    /// The yaw of every framing, in radians: looking along negative Z.
    pub const DEFAULT_YAW: f32 = 0.0;
    /// The default pitch of a framing, in radians.
    pub const DEFAULT_PITCH: f32 = 1.0;

    /// The pose the camera takes at a framing: the default yaw and pitch.
    pub fn at(framing: Framing) -> Self {
        Self {
            yaw: Self::DEFAULT_YAW,
            pitch: Self::DEFAULT_PITCH,
            distance: framing.distance,
            target: framing.target,
        }
    }

    /// The unit vector from the target to the camera.
    pub fn direction(self) -> Vec3 {
        Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.cos() * self.pitch.cos(),
        )
    }

    /// The camera's transform at this pose, looking at the target with Y up.
    pub fn transform(self) -> Transform {
        Transform::from_translation(self.target + self.direction() * self.distance)
            .looking_at(self.target, Vec3::Y)
    }
}

/// The part of the window that shows the scene. Elsewhere the pointer neither starts
/// a scene drag nor zooms.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Scene {
    #[default]
    WholeWindow,
    /// A rectangle in logical window pixels, holding its minimum corner but not its
    /// maximum one.
    Within(Rect),
}

impl Scene {
    fn holds(self, position: Vec2) -> bool {
        match self {
            Self::WholeWindow => true,
            Self::Within(rect) => position.cmpge(rect.min).all() && position.cmplt(rect.max).all(),
        }
    }
}

/// What the game says of its table this frame. Write it before [`TableCameraSystems`];
/// the camera reads whatever was last written.
///
/// The camera adds what it finds itself: the pointer over a UI node counts as outside
/// the scene unless the node is [`InScene`], and keyboard focus on a button takes the
/// [`VIEW_KEYS`].
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct TableFacts {
    /// No drag, wheel or key moves the camera, and a held scene drag ends.
    pub blocks_camera: bool,
    /// A state of the table, such as an open menu, uses the [`VIEW_KEYS`]. WASD still pans.
    pub takes_view_keys: bool,
    /// A state of the table, such as an open menu, uses the wheel.
    pub takes_wheel: bool,
    pub scene: Scene,
}

/// What one frame of input did to the camera. [`TableCameraSystems`] writes it each
/// frame it runs.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Navigation {
    /// A drag, wheel or key moved the camera.
    pub manual: bool,
    /// That move ended following.
    pub stopped_following: bool,
}

/// A UI node that stands for something in the scene, such as a count over a token:
/// the pointer over it is still over the scene.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct InScene;

/// The camera whose transform the table camera writes.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct TableCamera;

/// The systems that gather input and move the camera, in `Update`.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TableCameraSystems;

/// Gathers the table camera's input each `Update` and moves it, timed by `Time<Clock>`:
/// Bevy's `Time` unless the game keeps a clock of its own.
///
/// It initializes [`TableFacts`] and [`Navigation`]. The game inserts the [`Orbit`]
/// and marks its camera with [`TableCamera`]. Without a single primary window the
/// systems do nothing.
pub struct TableCameraPlugin<Clock = ()>(PhantomData<fn() -> Clock>);

impl<Clock> Default for TableCameraPlugin<Clock> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<Clock: Default + Send + Sync + 'static> Plugin for TableCameraPlugin<Clock> {
    fn build(&self, app: &mut App) {
        app.init_resource::<TableFacts>()
            .init_resource::<Navigation>()
            .add_systems(Update, drive::<Clock>.in_set(TableCameraSystems));
    }
}

/// One frame of what the camera reads, as plain values. [`TableCameraPlugin`] gathers
/// it from Bevy; a test builds it directly.
pub struct Input<'a> {
    pub keys: &'a ButtonInput<KeyCode>,
    pub buttons: &'a ButtonInput<MouseButton>,
    pub window: &'a Window,
    /// Lines of vertical wheel motion, as [`wheel_steps`] gives them.
    pub wheel_lines: f32,
    /// Seconds since the last frame.
    pub dt: f32,
    pub pointer_over_ui: bool,
    pub focus_on_button: bool,
}

#[derive(SystemParam)]
struct Devices<'w, 's> {
    keys: Res<'w, ButtonInput<KeyCode>>,
    buttons: Res<'w, ButtonInput<MouseButton>>,
    scroll: Res<'w, AccumulatedMouseScroll>,
    window: Single<'w, 's, &'static Window, With<PrimaryWindow>>,
}

#[derive(SystemParam)]
struct Interface<'w, 's> {
    hover: Option<Res<'w, HoverMap>>,
    nodes: Query<'w, 's, (), (With<Node>, Without<InScene>)>,
    focus: Option<Res<'w, InputFocus>>,
    controls: Query<'w, 's, (), With<Button>>,
}

impl Interface<'_, '_> {
    fn pointer_over_ui(&self) -> bool {
        self.hover
            .as_ref()
            .and_then(|hover| hover.get(&PointerId::Mouse))
            .is_some_and(|hits| hits.keys().any(|entity| self.nodes.contains(*entity)))
    }

    fn focus_on_button(&self) -> bool {
        self.focus
            .as_ref()
            .and_then(|focus| focus.get())
            .is_some_and(|entity| self.controls.contains(entity))
    }
}

fn drive<Clock: Default + Send + Sync + 'static>(
    devices: Devices,
    interface: Interface,
    time: Res<Time<Clock>>,
    table: Res<TableFacts>,
    mut orbit: ResMut<Orbit>,
    mut navigation: ResMut<Navigation>,
    mut cameras: Query<&mut Transform, With<TableCamera>>,
) {
    let step = orbit.drive(
        Input {
            keys: &devices.keys,
            buttons: &devices.buttons,
            window: &devices.window,
            wheel_lines: wheel_steps(&devices.scroll),
            dt: time.delta_secs(),
            pointer_over_ui: interface.pointer_over_ui(),
            focus_on_button: interface.focus_on_button(),
        },
        &table,
    );
    for mut transform in &mut cameras {
        *transform = step.transform;
    }
    *navigation = Navigation {
        manual: step.manual_navigation,
        stopped_following: step.stopped_following,
    };
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

/// A setting refused by [`Orbit::with_navigation`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidNavigation {
    /// The pitch limits must be finite and contain the Home pitch.
    Pitch,
    /// The distance limits must be positive, finite and ordered.
    Distance,
}

impl std::fmt::Display for InvalidNavigation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Pitch => "camera pitch limits must be finite and contain the Home pitch",
            Self::Distance => "camera distance limits must be positive, finite and ordered",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for InvalidNavigation {}

#[derive(Resource)]
pub struct Orbit {
    home: Framing,
    home_pitch: f32,
    pitch_limits: RangeInclusive<f32>,
    distance_limits: Option<RangeInclusive<f32>>,
    pose: Pose,
    desired_target: Vec3,
    desired_distance: f32,
    following: bool,
    drag: Option<Vec2>,
}

impl Orbit {
    /// Start at the game's home framing, with a manual camera.
    pub fn new(home: impl Into<Framing>) -> Self {
        let home = home.into();
        let pose = Pose::at(home);
        Self {
            home,
            home_pitch: Pose::DEFAULT_PITCH,
            pitch_limits: PITCH,
            distance_limits: None,
            pose,
            desired_target: pose.target,
            desired_distance: pose.distance,
            following: false,
            drag: None,
        }
    }

    /// Set the Home pitch and the inclusive limits for manual turning and zoom.
    /// Move to the existing Home framing with those settings.
    ///
    /// # Errors
    /// Returns [`InvalidNavigation`] for non-finite pitch limits, a Home pitch
    /// outside those limits, or a distance range with non-positive, non-finite
    /// or reversed endpoints.
    pub fn with_navigation(
        mut self,
        home_pitch: f32,
        pitch: RangeInclusive<f32>,
        distance: RangeInclusive<f32>,
    ) -> Result<Self, InvalidNavigation> {
        if !(pitch.start().is_finite() && pitch.end().is_finite() && pitch.contains(&home_pitch)) {
            return Err(InvalidNavigation::Pitch);
        }
        if !distance.start().is_finite()
            || !distance.end().is_finite()
            || distance.is_empty()
            || *distance.start() <= 0.0
        {
            return Err(InvalidNavigation::Distance);
        }
        self.home_pitch = home_pitch;
        self.pitch_limits = pitch;
        self.distance_limits = Some(distance);
        self.reset_home();
        Ok(self)
    }

    pub fn with_home(mut self, framing: impl Into<Framing>) -> Self {
        self.set_home(framing);
        self.reset_home();
        self
    }

    /// Where Home returns the camera.
    pub fn home(&self) -> Framing {
        self.home
    }

    /// Change where Home returns the camera, without moving it.
    pub fn set_home(&mut self, framing: impl Into<Framing>) {
        self.home = framing.into();
    }

    fn reset_home(&mut self) {
        self.focus(self.home);
        self.pose.pitch = self.home_pitch;
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

    /// Go to a framing at once, at the default yaw and pitch, and stop following.
    pub fn focus(&mut self, framing: impl Into<Framing>) {
        self.pose = Pose::at(framing.into());
        self.freeze_framing();
        self.following = false;
    }

    /// Called at the existing action boundary, never derived afresh each frame.
    /// Freeze framing on actions while following is off.
    pub fn update_follow_target(&mut self, framing: Option<Framing>) {
        if !self.following {
            self.freeze_framing();
        } else if let Some(framing) = framing {
            self.desired_target = framing.target;
            self.desired_distance = framing.distance;
        }
    }

    fn freeze_framing(&mut self) {
        self.desired_target = self.pose.target;
        self.desired_distance = self.pose.distance;
    }

    /// One frame of the table camera: what [`TableCameraPlugin`] does with the input
    /// it gathers. A pointer outside the scene counts as over UI, and WASD pan stops
    /// only when the table blocks the camera.
    pub fn drive(&mut self, input: Input<'_>, table: &TableFacts) -> CameraStep {
        let outside_scene = input
            .window
            .cursor_position()
            .is_some_and(|position| !table.scene.holds(position));
        let over_ui = input.pointer_over_ui || outside_scene;
        self.step(FrameInput {
            keys: input.keys,
            buttons: input.buttons,
            window: input.window,
            scroll_y: input.wheel_lines,
            dt: input.dt,
            capture: Capture {
                pointer_blocked: table.blocks_camera,
                over_ui,
                keyboard: table.blocks_camera || table.takes_view_keys || input.focus_on_button,
                pan: Some(table.blocks_camera),
                wheel: table.blocks_camera || table.takes_wheel || over_ui,
            },
        })
    }

    pub fn step(&mut self, input: FrameInput<'_>) -> CameraStep {
        let distance_limits = self.distance_limits.as_ref().unwrap_or(&DISTANCE);
        let (nearest, farthest) = (*distance_limits.start(), *distance_limits.end());
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
                (self.pose.distance * 0.95_f32.powf(input.scroll_y)).clamp(nearest, farthest);
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
            if self.distance_limits.is_some()
                && input.keys.any_pressed([KeyCode::Equal, KeyCode::Minus])
            {
                // Freeze configured zoom at its limit. Default zoom uses the later clamp.
                self.pose.distance = self.pose.distance.clamp(nearest, farthest);
            }
            if input.keys.just_pressed(KeyCode::Home) {
                // Home resets framing, not a held scene drag's capture.
                self.reset_home();
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
        self.pose.pitch = self
            .pose
            .pitch
            .clamp(*self.pitch_limits.start(), *self.pitch_limits.end());
        self.pose.distance = self.pose.distance.clamp(nearest, farthest);
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
