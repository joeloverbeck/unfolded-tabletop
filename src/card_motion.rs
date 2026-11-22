//! A card's movement between two poses, lifted and turned on the way.
//!
//! [`CardMove::pose`] is a pure function of the time since the move began, read
//! from whatever clock the consumer keeps: a frame clock, a presentation clock
//! that pauses, or a fixed step a capture seeks to. Asking for the end of the
//! move, or any later time, gives the resting pose exactly, so a skip lands at
//! once. The move holds no rules and no sequence: which cards move, when each
//! begins, what happens when one lands, and the lift each takes are the game's.
//!
//! Poses are in the parent's coordinates, with +Y up, as cards lie in XZ in
//! [`crate::card`]. The path eases with smoothstep. A lift raises the card on a
//! half sine over the move's eased progress, and can sway it about +Y and +Z at
//! the top of its arc. A turn spins it about an axis of the parent's over a
//! share of the time, after the card's own rotation, as a card turning over
//! about its length does.

use bevy::prelude::*;
use std::f32::consts::PI;

/// One card's movement from one pose to another over a duration in seconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardMove {
    /// Where the card is at time zero, before any turn.
    pub from: Transform,
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

/// The rise of a card on its way: `height` at the middle of the move, and a
/// sway in radians about +Y (`yaw`) and +Z (`roll`) at that height, both
/// following the same half sine down to nothing as the card lands.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Lift {
    pub height: f32,
    pub yaw: f32,
    pub roll: f32,
}

/// A slide by `offset` over the first `share` of the move's time, a fraction
/// from 0 to 1. The crossing to the second pose takes the rest of the time and
/// carries the offset away with it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Departure {
    pub offset: Vec3,
    pub share: f32,
}

/// A spin about `axis`, a unit vector in the parent's coordinates, from angle
/// `from` to angle `to` in radians. It begins after the fraction `begins` of
/// the move's time and lasts the fraction `lasts`, easing with smoothstep.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Turn {
    pub axis: Vec3,
    pub from: f32,
    pub to: f32,
    pub begins: f32,
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

    fn rotation(&self, t: f32) -> Quat {
        Quat::from_axis_angle(self.axis, self.angle(t))
    }
}

impl CardMove {
    /// A flat move with no lift, departure or turn.
    pub fn new(from: Transform, to: Transform, duration: f32) -> Self {
        Self {
            from,
            to,
            duration,
            lift: Lift::NONE,
            departure: None,
            turn: None,
        }
    }

    pub fn lifted(self, lift: Lift) -> Self {
        Self { lift, ..self }
    }

    pub fn departing(self, departure: Departure) -> Self {
        Self {
            departure: Some(departure),
            ..self
        }
    }

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

    /// The pose the card lands in: the second pose, turned to the turn's end.
    pub fn rest(&self) -> Transform {
        match self.turn {
            Some(turn) => Transform {
                rotation: Quat::from_axis_angle(turn.axis, turn.to) * self.to.rotation,
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
                self.from.translation.lerp(
                    self.from.translation + departure.offset,
                    ease(t / departure.share),
                ),
                ease((t - departure.share) / (1. - departure.share)),
            ),
            _ => (self.from.translation, ease(t)),
        };
        let mut translation = start.lerp(self.to.translation, carry);
        let mut rotation = if self.from.rotation == self.to.rotation {
            self.to.rotation
        } else {
            self.from.rotation.slerp(self.to.rotation, carry)
        };
        let scale = if self.from.scale == self.to.scale {
            self.to.scale
        } else {
            self.from.scale.lerp(self.to.scale, carry)
        };
        if let Some(turn) = self.turn {
            rotation = turn.rotation(t) * rotation;
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
            scale,
        }
    }
}

/// Smoothstep of `v` clamped to 0..=1.
fn ease(v: f32) -> f32 {
    let v = v.clamp(0., 1.);
    v * v * (3. - 2. * v)
}
