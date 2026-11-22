#![cfg(feature = "render")]
//! A card's movement between two poses at the public interface. Expectations
//! come from hand calculation of the smoothstep and the lift's half sine, and
//! from Arkham Unfolded's own draw, move and location-flip formulas at
//! `1dab37d`, transcribed below as the behaviour its migration must keep.
use bevy::prelude::*;
use std::f32::consts::PI;
use unfolded_tabletop::card_motion::{CardMove, Departure, Lift, Turn};

const EPSILON: f32 = 1e-5;

fn smooth(v: f32) -> f32 {
    let v = v.clamp(0., 1.);
    v * v * (3. - 2. * v)
}

fn assert_pose(actual: Transform, expected: Transform) {
    assert!(
        actual
            .translation
            .abs_diff_eq(expected.translation, EPSILON),
        "translation {:?} != {:?}",
        actual.translation,
        expected.translation
    );
    assert!(
        actual.rotation.abs_diff_eq(expected.rotation, EPSILON)
            || actual.rotation.abs_diff_eq(-expected.rotation, EPSILON),
        "rotation {:?} != {:?}",
        actual.rotation,
        expected.rotation
    );
    assert!(actual.scale.abs_diff_eq(expected.scale, EPSILON));
}

fn up(pose: Transform) -> Vec3 {
    pose.rotation * Vec3::Y
}

#[test]
fn a_flat_move_eases_from_the_first_pose_to_the_second() {
    let from = Transform::from_xyz(1., 0., 2.).with_rotation(Quat::from_rotation_y(0.3));
    let to = Transform::from_xyz(5., 0., -2.).with_rotation(Quat::from_rotation_y(0.3));
    let motion = CardMove::new(from, to, 0.35);
    assert_pose(motion.pose(0.), from);
    // Halfway in time is halfway in place: smoothstep(0.5) = 0.5.
    assert_pose(
        motion.pose(0.175),
        Transform::from_xyz(3., 0., 0.).with_rotation(Quat::from_rotation_y(0.3)),
    );
    // A quarter of the time: smoothstep(0.25) = 0.15625 of the way.
    assert_pose(
        motion.pose(0.0875),
        Transform::from_xyz(1. + 4. * 0.15625, 0., 2. - 4. * 0.15625)
            .with_rotation(Quat::from_rotation_y(0.3)),
    );
    assert_eq!(motion.pose(0.35), to);
}

#[test]
fn the_clock_is_the_consumers_and_clamps_at_both_ends() {
    let from = Transform::from_xyz(0., 0., 0.);
    let to = Transform::from_xyz(2., 0., 0.);
    let motion = CardMove::new(from, to, 0.7).lifted(Lift::arc(0.5));
    assert_eq!(motion.pose(-1.), motion.pose(0.));
    assert_eq!(motion.pose(0.), from);
    assert_eq!(motion.pose(0.7), to);
    assert_eq!(motion.pose(50.), to);
    assert_eq!(motion.rest(), to);
    assert!(!motion.finished(0.69));
    assert!(motion.finished(0.7));
    // Seeking is pure: an earlier time read after a later one is the same pose.
    let late = motion.pose(0.5);
    let early = motion.pose(0.2);
    assert_eq!(motion.pose(0.5), late);
    assert_eq!(motion.pose(0.2), early);
}

#[test]
fn a_move_with_no_duration_is_already_at_rest() {
    let motion = CardMove::new(Transform::default(), Transform::from_xyz(1., 2., 3.), 0.)
        .lifted(Lift::arc(1.));
    assert_eq!(motion.pose(0.), Transform::from_xyz(1., 2., 3.));
    assert!(motion.finished(0.));
}

#[test]
fn the_lift_peaks_halfway_and_lands_flat() {
    let motion = CardMove::new(Transform::default(), Transform::from_xyz(4., 0., 0.), 1.)
        .lifted(Lift::arc(0.6));
    // smoothstep(0.5) = 0.5 and sin(π/2) = 1: the middle, at full height.
    assert_pose(motion.pose(0.5), Transform::from_xyz(2., 0.6, 0.));
    // smoothstep(0.25) = 0.15625; sin(0.15625π) = 0.4713967.
    assert_pose(
        motion.pose(0.25),
        Transform::from_xyz(0.625, 0.6 * 0.471_396_74, 0.),
    );
    assert_eq!(motion.pose(1.).translation, Vec3::new(4., 0., 0.));
}

#[test]
fn rotation_and_scale_follow_the_poses() {
    let from = Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(1.));
    let to = Transform::from_xyz(0., 0., 4.)
        .with_rotation(Quat::from_rotation_y(PI / 2.))
        .with_scale(Vec3::splat(2.));
    let motion = CardMove::new(from, to, 1.);
    let middle = motion.pose(0.5);
    assert_pose(
        middle,
        Transform::from_xyz(0., 0., 2.)
            .with_rotation(Quat::from_rotation_y(PI / 4.))
            .with_scale(Vec3::splat(1.5)),
    );
    assert_eq!(motion.pose(1.), to);
}

/// Cuba Libre's deck card (#145): it turns over where it lies, its back up
/// before the turn, edge-on halfway and its face up at rest.
#[test]
fn a_card_turns_over_in_place_with_its_back_showing_first() {
    let at = Transform::from_xyz(3., 0.1, -1.);
    let motion = CardMove::new(at, at, 1.)
        .lifted(Lift::arc(0.4))
        .turned(Turn::over(Vec3::Z));
    let start = motion.pose(0.);
    assert!(
        up(start).abs_diff_eq(Vec3::NEG_Y, EPSILON),
        "{:?}",
        up(start)
    );
    assert!(start.translation.abs_diff_eq(at.translation, EPSILON));
    let middle = motion.pose(0.5);
    assert!(up(middle).y.abs() < EPSILON, "{:?}", up(middle));
    assert!(
        middle
            .translation
            .abs_diff_eq(at.translation + Vec3::Y * 0.4, EPSILON)
    );
    // Before the middle the back still faces up; after it the face does.
    assert!(up(motion.pose(0.4)).y < 0.);
    assert!(up(motion.pose(0.6)).y > 0.);
    assert_eq!(motion.pose(1.), at);
    assert_eq!(motion.rest(), at);
}

#[test]
fn a_turn_keeps_to_its_share_of_the_time() {
    let at = Transform::default();
    let turn = Turn {
        axis: Vec3::Z,
        from: 0.,
        to: PI,
        begins: 0.2,
        lasts: 0.58,
    };
    let motion = CardMove::new(at, at, 1.).turned(turn);
    assert_pose(motion.pose(0.1), at);
    assert_pose(motion.pose(0.2), at);
    // Halfway through its share, smoothstep(0.5) = 0.5 of a half turn.
    assert_pose(
        motion.pose(0.49),
        at.with_rotation(Quat::from_rotation_z(PI / 2.)),
    );
    assert_pose(
        motion.pose(0.78),
        at.with_rotation(Quat::from_rotation_z(PI)),
    );
    assert_pose(
        motion.pose(0.9),
        at.with_rotation(Quat::from_rotation_z(PI)),
    );
    assert_eq!(motion.rest(), at.with_rotation(Quat::from_rotation_z(PI)));
}

#[test]
fn a_departure_slides_the_card_out_before_it_crosses() {
    let from = Transform::from_xyz(0., 0., 0.);
    let to = Transform::from_xyz(0., 0., 10.);
    let motion = CardMove::new(from, to, 1.).departing(Departure {
        offset: Vec3::X,
        share: 0.2,
    });
    // At the end of its share the card has left by the offset and not crossed.
    assert_pose(motion.pose(0.2), Transform::from_xyz(1., 0., 0.));
    // Halfway through the crossing: the slide is done, half the way is covered,
    // and the offset eases out toward the destination.
    assert_pose(motion.pose(0.6), Transform::from_xyz(0.5, 0., 5.));
    assert_eq!(motion.pose(1.), to);
}

/// Arkham's `CardMotion::transform` for a draw at `1dab37d`, `motion.rs`.
fn arkham_transfer(start: Vec3, end: Vec3, rotation: Quat, turn: f32, t: f32) -> Transform {
    let extracted = start + Vec3::X * 0.22;
    let carry = smooth((t - 0.12) / 0.88);
    let lift = (PI * smooth(t)).sin();
    let height = (0.45 + start.distance(end) * 0.035).min(1.2);
    let at = start.lerp(extracted, smooth(t / 0.12)).lerp(end, carry) + Vec3::Y * height * lift;
    let flip = smooth((t - 0.2) / 0.58);
    let angle = match turn {
        1. => -PI * (1. - flip),
        -1. => PI * flip,
        _ => 0.,
    };
    Transform::from_translation(at).with_rotation(
        Quat::from_rotation_y(0.10 * lift)
            * Quat::from_rotation_z(0.07 * lift)
            * Quat::from_rotation_z(angle)
            * rotation,
    )
}

#[test]
fn arkhams_draw_keeps_its_path_lift_sway_and_turn() {
    let start = Vec3::new(-6., 0.02, 4.);
    let end = Vec3::new(1.5, 0.3, -3.);
    let rotation = Quat::from_rotation_y(0.2);
    let height = (0.45 + start.distance(end) * 0.035).min(1.2);
    let lift = Lift {
        height,
        yaw: 0.10,
        roll: 0.07,
    };
    let departure = Departure {
        offset: Vec3::X * 0.22,
        share: 0.12,
    };
    let window = |from, to| Turn {
        axis: Vec3::Z,
        from,
        to,
        begins: 0.2,
        lasts: 0.58,
    };
    for (turn, sign) in [
        (Some(window(-PI, 0.)), 1.),
        (Some(window(0., PI)), -1.),
        (None, 0.),
    ] {
        let mut motion = CardMove::new(
            Transform::from_translation(start).with_rotation(rotation),
            Transform::from_translation(end).with_rotation(rotation),
            0.7,
        )
        .lifted(lift)
        .departing(departure);
        if let Some(turn) = turn {
            motion = motion.turned(turn);
        }
        for step in 0..=70 {
            let t = step as f32 / 70.;
            assert_pose(
                motion.pose(t * 0.7),
                arkham_transfer(start, end, rotation, sign, t),
            );
        }
    }
}

/// Arkham's location reveal at `1dab37d`, `map_motion.rs`: it rises and turns
/// face up around its length where it lies.
#[test]
fn arkhams_location_reveal_keeps_its_rise_and_turn() {
    let at = Vec3::new(2., 0.05, -1.);
    let motion = CardMove::new(
        Transform::from_translation(at),
        Transform::from_translation(at),
        1.,
    )
    .lifted(Lift::arc(0.86))
    .turned(Turn::over(Vec3::Z));
    for step in 0..=50 {
        let elapsed = step as f32 / 50.;
        let t = smooth(elapsed);
        let expected = Transform::from_translation(at + Vec3::Y * (0.86 * (PI * t).sin()))
            .with_rotation(Quat::from_rotation_z(-PI * (1. - t)));
        assert_pose(motion.pose(elapsed), expected);
    }
}
