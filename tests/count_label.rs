#![cfg(feature = "render")]

use bevy::{
    camera::{CameraProjection, ComputedCameraValues, RenderTargetInfo},
    ecs::world::CommandQueue,
    prelude::*,
};
use unfolded_tabletop::count_label::{self, CountLabelPlugin};

#[test]
fn counts_follow_scaled_roots_and_camera_then_hide_and_clean_up() {
    let mut app = App::new();
    app.add_plugins(CountLabelPlugin);
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
    let token = app
        .world_mut()
        .spawn(Transform::from_xyz(0., 0., -10.).with_scale(Vec3::splat(2.)))
        .id();
    let mut queue = CommandQueue::default();
    let label = count_label::spawn(
        &mut Commands::new(&mut queue, app.world()),
        token,
        123,
        Vec3::X,
    );
    queue.apply(app.world_mut());
    app.world_mut().entity_mut(label).insert(ComputedNode {
        size: Vec2::new(80., 24.),
        inverse_scale_factor: 0.5,
        ..default()
    });
    app.update();
    assert_eq!(app.world().get::<Text>(label).unwrap().0, "123");
    assert_position(&app, label, 530., 250.);

    app.world_mut()
        .entity_mut(camera)
        .insert(GlobalTransform::from_translation(Vec3::X * 2.));
    app.update();
    assert_position(&app, label, 480., 250.);

    app.world_mut()
        .entity_mut(token)
        .insert(Transform::from_xyz(4., 0., -10.));
    app.update();
    assert_position(&app, label, 555., 250.);
    app.world_mut()
        .get_mut::<Transform>(token)
        .unwrap()
        .translation
        .z = 10.;
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(label).unwrap(),
        Visibility::Hidden
    );
    app.world_mut()
        .get_mut::<Transform>(token)
        .unwrap()
        .translation
        .z = -10.;
    app.update();
    assert_position(&app, label, 555., 250.);
    app.world_mut().despawn(token);
    app.update();
    assert!(app.world().get_entity(label).is_err());
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
