#![cfg(feature = "render")]

mod common;

use bevy::prelude::*;
use common::{assert_hidden, assert_position, camera, measured};
#[allow(deprecated)]
use unfolded_tabletop::{
    count_label::{self, CountLabelPlugin},
    label_anchor::{LabelAnchor, LabelAnchorPlugin},
};

fn app_with_camera() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins(LabelAnchorPlugin);
    let camera = app.world_mut().spawn(camera()).id();
    (app, camera)
}

/// A node as a caller would spawn it, with a margin and width of its own,
/// already measured by layout.
fn caller_node(app: &mut App, anchor: LabelAnchor) -> Entity {
    app.world_mut()
        .spawn((
            Text::new("T 3"),
            Node {
                position_type: PositionType::Absolute,
                width: px(40),
                margin: UiRect::top(px(-16)),
                ..default()
            },
            anchor,
        ))
        .insert(measured())
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

    // Only the root's own transform is read, not a parent's.
    let parent = app.world_mut().spawn(Transform::from_xyz(8., 0., 0.)).id();
    app.world_mut().entity_mut(token).insert(ChildOf(parent));
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
    let node = app.world().get::<Node>(free).unwrap();
    assert_eq!(node.position_type, PositionType::Absolute);
    assert_eq!(node.width, px(40));
    assert_eq!(node.margin, UiRect::top(px(-16)));
}

#[test]
fn a_node_can_start_at_its_point_instead_of_centering_on_it() {
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

#[test]
#[allow(deprecated)]
fn counts_and_caller_nodes_share_one_tracker_whichever_plugin_comes_first() {
    for count_plugin_first in [true, false] {
        let mut app = App::new();
        if count_plugin_first {
            app.add_plugins((CountLabelPlugin, LabelAnchorPlugin));
        } else {
            app.add_plugins((LabelAnchorPlugin, CountLabelPlugin));
        }
        app.world_mut().spawn(camera());
        let token = app
            .world_mut()
            .spawn(Transform::from_xyz(0., 0., -10.))
            .id();
        let count = count_label::spawn(&mut app.world_mut().commands(), token, 7, Vec3::X);
        app.world_mut().flush();
        app.world_mut().entity_mut(count).insert(measured());
        let caller = caller_node(&mut app, LabelAnchor::entity(token, Vec3::X));
        app.update();
        assert_position(&app, count, 505., 250.);
        assert_position(&app, caller, 505., 250.);
    }
}

#[test]
fn a_moved_camera_and_a_hidden_node_take_effect_in_the_same_frame() {
    let mut app = App::new();
    app.add_plugins((
        TransformPlugin,
        bevy::camera::visibility::VisibilityPlugin,
        LabelAnchorPlugin,
    ))
    // What Bevy's visibility systems read, as a game's asset plugin supplies.
    .init_resource::<Assets<Mesh>>()
    .init_resource::<Assets<bevy::mesh::skinning::SkinnedMeshInverseBindposes>>();
    let camera = app.world_mut().spawn((camera(), Transform::IDENTITY)).id();
    let label = caller_node(&mut app, LabelAnchor::point(Vec3::new(2., 0., -10.)));
    app.update();
    assert_position(&app, label, 530., 250.);
    assert!(app.world().get::<InheritedVisibility>(label).unwrap().get());

    // Moved in Update: propagated, then projected, in this frame.
    app.world_mut()
        .get_mut::<Transform>(camera)
        .unwrap()
        .translation
        .x = 2.;
    app.update();
    assert_position(&app, label, 480., 250.);

    app.world_mut()
        .get_mut::<Transform>(camera)
        .unwrap()
        .translation
        .z = -20.;
    app.update();
    assert_hidden(&app, label);
    assert!(!app.world().get::<InheritedVisibility>(label).unwrap().get());
}
