#![cfg(feature = "render")]

use bevy::{
    camera::{CameraProjection, ComputedCameraValues, RenderTargetInfo},
    prelude::*,
};
use unfolded_tabletop::label_anchor::{LabelAnchor, LabelAnchorPlugin};

/// A 1000×500 logical target (2000×1000 physical at scale 2) with a 90° field
/// of view, looking down -Z from the origin. A world point (x, y, -10) lands at
/// (500 + 25x, 250 - 25y) logical pixels.
fn app_with_camera() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins(LabelAnchorPlugin);
    let camera = app
        .world_mut()
        .spawn((
            Camera3d::default(),
            Camera {
                computed: ComputedCameraValues {
                    clip_from_view: PerspectiveProjection {
                        fov: std::f32::consts::FRAC_PI_2,
                        aspect_ratio: 2.,
                        near: 0.1,
                        ..default()
                    }
                    .get_clip_from_view(),
                    target_info: Some(RenderTargetInfo {
                        physical_size: UVec2::new(2000, 1000),
                        scale_factor: 2.,
                    }),
                    ..default()
                },
                ..default()
            },
            GlobalTransform::IDENTITY,
        ))
        .id();
    (app, camera)
}

/// A node as a caller would spawn it, already measured by layout: 80×24
/// physical pixels at scale 2, so 40 logical pixels wide.
fn caller_node(app: &mut App, anchor: LabelAnchor) -> Entity {
    app.world_mut()
        .spawn((
            Text::new("T 3"),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            anchor,
        ))
        .insert(ComputedNode {
            size: Vec2::new(80., 24.),
            inverse_scale_factor: 0.5,
            ..default()
        })
        .id()
}

#[test]
fn an_entity_anchor_follows_its_scaled_root_then_hides_and_is_removed() {
    let (mut app, camera) = app_with_camera();
    let token = app
        .world_mut()
        .spawn(Transform::from_xyz(0., 0., -10.).with_scale(Vec3::splat(2.)))
        .id();
    let label = caller_node(&mut app, LabelAnchor::entity(token, Vec3::X));
    app.update();
    // (2, 0, -10) lands at x 550; half the 40-pixel width puts the left edge at 530.
    assert_position(&app, label, 530., 250.);

    app.world_mut()
        .entity_mut(camera)
        .insert(GlobalTransform::from_translation(Vec3::X * 2.));
    app.update();
    assert_position(&app, label, 480., 250.);

    // A rotated root still offsets along world X: the offset is not rotated.
    app.world_mut()
        .entity_mut(token)
        .insert(Transform::from_xyz(4., 0., -10.).with_rotation(Quat::from_rotation_y(1.)));
    app.update();
    assert_position(&app, label, 555., 250.);

    app.world_mut()
        .get_mut::<Transform>(token)
        .unwrap()
        .translation
        .z = 10.;
    app.update();
    assert_hidden(&app, label);
    app.world_mut()
        .get_mut::<Transform>(token)
        .unwrap()
        .translation
        .z = -10.;
    app.update();
    assert_position(&app, label, 555., 250.);

    app.world_mut().entity_mut(token).remove::<Transform>();
    app.update();
    assert!(app.world().get_entity(label).is_err());
}

#[test]
fn a_point_anchor_is_placed_the_same_way_and_can_be_kept_to_the_viewport() {
    let (mut app, camera) = app_with_camera();
    let free = caller_node(&mut app, LabelAnchor::point(Vec3::new(2., 4., -10.)));
    let outside = Vec3::new(24., 0., -10.);
    let unlimited = caller_node(&mut app, LabelAnchor::point(outside));
    let limited = caller_node(&mut app, LabelAnchor::point(outside).within_viewport());
    app.update();
    assert_position(&app, free, 530., 150.);
    // x 1100 is in front of the camera but right of the 1000-pixel viewport.
    assert_position(&app, unlimited, 1080., 250.);
    assert_hidden(&app, limited);

    // A point never moves with anything, and is never removed.
    app.world_mut()
        .entity_mut(camera)
        .insert(GlobalTransform::from_translation(Vec3::X * 20.));
    app.update();
    assert_position(&app, free, 30., 150.);
    assert_position(&app, limited, 580., 250.);

    app.world_mut()
        .entity_mut(camera)
        .insert(GlobalTransform::from_translation(Vec3::Z * -20.));
    app.update();
    assert_hidden(&app, free);
    assert_hidden(&app, limited);
    assert!(app.world().get_entity(free).is_ok());

    // What the caller set is its own.
    assert_eq!(app.world().get::<Text>(free).unwrap().0, "T 3");
    assert!(app.world().get::<Pickable>(free).is_none());
    assert_eq!(
        app.world().get::<Node>(free).unwrap().position_type,
        PositionType::Absolute
    );
}

#[test]
fn a_node_can_start_at_its_point_instead_of_centring_on_it() {
    let (mut app, _) = app_with_camera();
    let at = Vec3::new(2., 4., -10.);
    let started = caller_node(&mut app, LabelAnchor::point(at).left_edge());
    app.update();
    assert_position(&app, started, 550., 150.);
}

#[test]
fn without_one_3d_camera_nothing_changes() {
    let (mut app, camera) = app_with_camera();
    let token = app
        .world_mut()
        .spawn(Transform::from_xyz(0., 0., -10.))
        .id();
    let label = caller_node(&mut app, LabelAnchor::entity(token, Vec3::ZERO));
    let second = app.world_mut().spawn(Camera3d::default()).id();
    app.world_mut().entity_mut(token).remove::<Transform>();
    app.update();
    let node = app.world().get::<Node>(label).unwrap();
    assert_eq!((node.left, node.top), (Val::Auto, Val::Auto));
    assert_eq!(
        *app.world().get::<Visibility>(label).unwrap(),
        Visibility::Inherited
    );

    app.world_mut().despawn(second);
    app.world_mut().despawn(camera);
    app.update();
    assert!(app.world().get_entity(label).is_ok());
}

fn assert_position(app: &App, label: Entity, x: f32, y: f32) {
    let node = app.world().get::<Node>(label).unwrap();
    let (Val::Px(left), Val::Px(top)) = (node.left, node.top) else {
        panic!("label was not projected");
    };
    assert!((left - x).abs() < 0.001, "{left} != {x}");
    assert!((top - y).abs() < 0.001, "{top} != {y}");
    assert_eq!(
        *app.world().get::<Visibility>(label).unwrap(),
        Visibility::Visible
    );
}

fn assert_hidden(app: &App, label: Entity) {
    assert_eq!(
        *app.world().get::<Visibility>(label).unwrap(),
        Visibility::Hidden
    );
}
