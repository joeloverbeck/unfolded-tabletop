//! The camera and assertions the label tests share.
#![allow(dead_code)]

use bevy::{
    camera::{CameraProjection, ComputedCameraValues, RenderTargetInfo},
    prelude::*,
};

/// A 1000×500 logical target (2000×1000 physical at scale 2) with a 90° field
/// of view, looking down -Z from the origin. A world point (x, y, -10) lands at
/// (500 + 25x, 250 - 25y) logical pixels.
pub fn camera() -> impl Bundle {
    (
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
    )
}

/// A node as layout would have measured it: 80×24 physical pixels at scale 2,
/// so 40 logical pixels wide.
pub fn measured() -> ComputedNode {
    ComputedNode {
        size: Vec2::new(80., 24.),
        inverse_scale_factor: 0.5,
        ..default()
    }
}

pub fn assert_position(app: &App, label: Entity, x: f32, y: f32) {
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

pub fn assert_hidden(app: &App, label: Entity) {
    assert_eq!(
        *app.world().get::<Visibility>(label).unwrap(),
        Visibility::Hidden
    );
}
