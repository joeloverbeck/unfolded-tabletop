#![cfg(feature = "seat")]

use bevy::{prelude::*, ui::CalculatedClip};
use unfolded_tabletop::seat::{UiTarget, ui_target};

#[test]
fn visible_control_uses_logical_pixels_and_physical_clipping() {
    let mut world = World::new();
    let control = world
        .spawn((
            ComputedNode {
                size: Vec2::new(80., 40.),
                inverse_scale_factor: 0.5,
                ..default()
            },
            UiGlobalTransform::from_xy(200., 100.),
            InheritedVisibility::VISIBLE,
            CalculatedClip::default().with_rect(
                Rect::new(180., 80., 220., 120.),
                &UiGlobalTransform::default(),
            ),
        ))
        .id();
    assert_eq!(
        ui_target(&world, control, Some(Vec2::new(160., 90.))),
        Some(UiTarget {
            position: Vec2::new(100., 50.),
            on_screen: true,
        })
    );
}

#[test]
fn clipped_and_offscreen_controls_keep_their_positions() {
    let mut world = World::new();
    let control = world
        .spawn((
            ComputedNode {
                size: Vec2::splat(40.),
                inverse_scale_factor: 0.5,
                ..default()
            },
            UiGlobalTransform::from_xy(200., 100.),
            InheritedVisibility::VISIBLE,
        ))
        .id();
    let viewport = Some(Vec2::new(160., 90.));
    let unreachable = Some(UiTarget {
        position: Vec2::new(100., 50.),
        on_screen: false,
    });

    // This clip contains the logical center but not the physical one.
    world
        .entity_mut(control)
        .insert(CalculatedClip::default().with_rect(
            Rect::new(80., 30., 120., 70.),
            &UiGlobalTransform::default(),
        ));
    assert_eq!(ui_target(&world, control, viewport), unreachable);
    world
        .entity_mut(control)
        .insert(CalculatedClip::FullyClipped);
    assert_eq!(ui_target(&world, control, viewport), unreachable);
    world.entity_mut(control).remove::<CalculatedClip>();
    assert_eq!(ui_target(&world, control, None), unreachable);
    assert_eq!(
        ui_target(&world, control, Some(Vec2::new(99., 90.))),
        unreachable
    );

    for (physical, position, on_screen) in [
        (Vec2::ZERO, Vec2::ZERO, true),
        (Vec2::new(320., 180.), Vec2::new(160., 90.), true),
        (Vec2::new(-2., 100.), Vec2::new(-1., 50.), false),
        (Vec2::new(200., 182.), Vec2::new(100., 91.), false),
    ] {
        world
            .entity_mut(control)
            .insert(UiGlobalTransform::from_translation(physical));
        assert_eq!(
            ui_target(&world, control, viewport),
            Some(UiTarget {
                position,
                on_screen,
            })
        );
    }
}

#[test]
fn hidden_empty_and_missing_layout_controls_are_absent() {
    let mut world = World::new();
    let control = world
        .spawn((
            ComputedNode {
                size: Vec2::splat(40.),
                ..default()
            },
            UiGlobalTransform::from_xy(20., 20.),
            InheritedVisibility::HIDDEN,
        ))
        .id();
    let viewport = Some(Vec2::splat(100.));
    assert_eq!(ui_target(&world, control, viewport), None);
    world.entity_mut(control).remove::<InheritedVisibility>();
    assert_eq!(ui_target(&world, control, viewport), None);
    world
        .entity_mut(control)
        .insert(InheritedVisibility::VISIBLE);
    world.get_mut::<ComputedNode>(control).unwrap().size.x = 0.;
    assert_eq!(ui_target(&world, control, viewport), None);
    world.get_mut::<ComputedNode>(control).unwrap().size = Vec2::new(40., -1.);
    assert_eq!(ui_target(&world, control, viewport), None);
    world.get_mut::<ComputedNode>(control).unwrap().size = Vec2::splat(40.);
    world.entity_mut(control).remove::<UiGlobalTransform>();
    assert_eq!(ui_target(&world, control, viewport), None);
    world
        .entity_mut(control)
        .insert(UiGlobalTransform::default());
    world.entity_mut(control).remove::<ComputedNode>();
    assert_eq!(ui_target(&world, control, viewport), None);
}
