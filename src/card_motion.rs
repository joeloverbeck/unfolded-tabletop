//! A card's movement to a resting pose, lifted and turned on the way.
//!
//! [`CardMove::pose`] is a pure function of the time since the move began, read
//! from whatever clock the consumer keeps: a frame clock, a presentation clock
//! that pauses, or a fixed step a capture seeks to. Asking for the end of the
//! move, or any later time, gives the resting pose exactly, so a skip lands at
//! once. The move holds no rules and no sequence: which cards move, when each
//! begins, what happens when one lands, and the lift each takes are the game's.
//!
//! Poses are in the parent's coordinates, with +Y up, as cards lie in XZ in
//! [`crate::card`]. The card travels from a starting position to its resting
//! pose and keeps that pose's rotation and scale on the way; only a [`Turn`] and
//! a [`Lift`]'s sway rotate it. The path eases with smoothstep. A lift raises
//! the card on a half sine over the move's eased progress, and can sway it about
//! +Y and +Z at the top of its arc. A turn spins it about an axis of the
//! parent's over a share of the time, after the card's own rotation, as a card
//! turning over about its length does.
//!
//! Nothing is checked or refused. Give [`Turn::axis`] as a unit vector, a
//! [`Departure::share`] from 0 up to but not including 1, and a duration that is
//! not negative; no test holds a pose for other values. No test holds a turn
//! that would end after the move does (`begins + lasts` above 1): its rest still
//! takes the turn's final angle.

use bevy::prelude::*;
use std::f32::consts::PI;

/// One card's movement from a position to a resting pose over a duration in
/// seconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardMove {
    /// Where the card is at time zero, in the resting pose's rotation and
    /// scale and before any turn.
    pub from: Vec3,
    /// Where the card rests at the end, before any turn.
    pub to: Transform,
    /// Seconds from the first pose to the second. Zero is already at rest.
    pub duration: f32,
    /// How the card rises off the table on its way.
    pub lift: Lift,
    /// An optional first slide, as a card drawn from a pile clears it.
    pub departure: Option<Departure>,
    /// An optional turn, as a card turning face up or face down.
    pub turn: Option<Turn>,
}

/// The rise of a card on its way, following a half sine of the move's eased
/// progress: nothing at either end and the full amount at the middle.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Lift {
    /// The rise along +Y at the middle of the move.
    pub height: f32,
    /// The sway about +Y at that height, in radians.
    pub yaw: f32,
    /// The sway about +Z at that height, in radians.
    pub roll: f32,
}

/// A first slide off a pile. The crossing to the resting pose takes the rest of
/// the time and carries the offset away with it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Departure {
    /// How far the card slides from its first position.
    pub offset: Vec3,
    /// The fraction of the move's time the slide takes, from 0 up to but not
    /// including 1.
    pub share: f32,
}

/// A spin about an axis of the parent's, easing with smoothstep over a share
/// of the move's time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Turn {
    /// The axis, a unit vector in the parent's coordinates.
    pub axis: Vec3,
    /// The angle in radians before the turn begins.
    pub from: f32,
    /// The angle in radians once the turn ends, kept at rest.
    pub to: f32,
    /// The fraction of the move's time before the turn begins.
    pub begins: f32,
    /// The fraction of the move's time the turn takes.
    pub lasts: f32,
}

impl Lift {
    /// No rise: the card slides flat.
    pub const NONE: Self = Self {
        height: 0.,
        yaw: 0.,
        roll: 0.,
    };

    /// A rise to `height` with no sway.
    pub fn arc(height: f32) -> Self {
        Self {
            height,
            ..Self::NONE
        }
    }
}

impl Turn {
    /// A half turn about `axis` over the whole move that ends at the resting
    /// rotation: a card that begins with its back up and rests face up.
    pub fn over(axis: Vec3) -> Self {
        Self {
            axis,
            from: -PI,
            to: 0.,
            begins: 0.,
            lasts: 1.,
        }
    }

    fn angle(&self, t: f32) -> f32 {
        let turned = if self.lasts > 0. {
            ease((t - self.begins) / self.lasts)
        } else if t >= self.begins {
            1.
        } else {
            0.
        };
        self.from * (1. - turned) + self.to * turned
    }

    fn rotation(&self, angle: f32) -> Quat {
        Quat::from_axis_angle(self.axis, angle)
    }
}

impl CardMove {
    /// A flat move with no lift, departure or turn, from `from` to rest at
    /// `to`. A card that turns where it lies starts at `to.translation`.
    pub fn new(from: Vec3, to: Transform, duration: f32) -> Self {
        Self {
            from,
            to,
            duration,
            lift: Lift::NONE,
            departure: None,
            turn: None,
        }
    }

    /// The same move, rising and swaying by `lift` on its way.
    pub fn lifted(self, lift: Lift) -> Self {
        Self { lift, ..self }
    }

    /// The same move, sliding by `departure` before it crosses.
    pub fn departing(self, departure: Departure) -> Self {
        Self {
            departure: Some(departure),
            ..self
        }
    }

    /// The same move, spinning by `turn` on its way.
    pub fn turned(self, turn: Turn) -> Self {
        Self {
            turn: Some(turn),
            ..self
        }
    }

    /// Whether the move has reached its rest at `elapsed` seconds.
    pub fn finished(&self, elapsed: f32) -> bool {
        elapsed >= self.duration
    }

    /// The pose the card lands in: the resting pose, turned to the turn's end.
    pub fn rest(&self) -> Transform {
        match self.turn {
            Some(turn) => Transform {
                rotation: turn.rotation(turn.to) * self.to.rotation,
                ..self.to
            },
            None => self.to,
        }
    }

    /// The card's pose `elapsed` seconds after the move began. Times before
    /// zero give the first pose; times from the duration on give [`Self::rest`].
    pub fn pose(&self, elapsed: f32) -> Transform {
        if self.finished(elapsed) {
            return self.rest();
        }
        let t = (elapsed / self.duration).clamp(0., 1.);
        let (start, carry) = match self.departure {
            Some(departure) if departure.share > 0. => (
                self.from
                    .lerp(self.from + departure.offset, ease(t / departure.share)),
                ease((t - departure.share) / (1. - departure.share)),
            ),
            _ => (self.from, ease(t)),
        };
        let mut translation = start.lerp(self.to.translation, carry);
        let mut rotation = self.to.rotation;
        if let Some(turn) = self.turn {
            rotation = turn.rotation(turn.angle(t)) * rotation;
        }
        if self.lift != Lift::NONE {
            let rise = (PI * ease(t)).sin();
            translation += Vec3::Y * self.lift.height * rise;
            rotation = Quat::from_rotation_y(self.lift.yaw * rise)
                * Quat::from_rotation_z(self.lift.roll * rise)
                * rotation;
        }
        Transform {
            translation,
            rotation,
            scale: self.to.scale,
        }
    }
}

/// Smoothstep of `v` clamped to 0..=1.
fn ease(v: f32) -> f32 {
    let v = v.clamp(0., 1.);
    v * v * (3. - 2. * v)
}
