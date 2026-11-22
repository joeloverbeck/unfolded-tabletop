#![cfg(feature = "render")]

use bevy::{
    asset::RenderAssetUsages,
    ecs::world::CommandQueue,
    mesh::VertexAttributeValues,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use unfolded_tabletop::slab::{Slab, spawn};

#[test]
fn a_solid_slab_shows_the_same_upright_picture_from_both_sides() {
    let mut world = World::new();
    let mut queue = CommandQueue::default();
    let mut meshes = Assets::<Mesh>::default();
    let mut materials = Assets::<StandardMaterial>::default();
    let mut images = Assets::<Image>::default();

    // The four corners are distinct, so either axis being flipped changes the picture.
    let picture = images.add(Image::new_fill(
        Extent3d {
            width: 2,
            height: 2,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255,
        ],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ));
    let body = meshes.add(Cuboid::new(2.1, 3.1, 0.1));
    let face = meshes.add(Rectangle::new(2.0, 3.0));
    let body_material = materials.add(Color::BLACK);
    let face_material = materials.add(StandardMaterial {
        base_color_texture: Some(picture.clone()),
        unlit: true,
        ..default()
    });

    let center = Vec3::new(4.0, 2.0, -1.0);
    let root = Commands::new(&mut queue, &world)
        .spawn_empty()
        .with_children(|parts| {
            spawn(
                parts,
                Slab {
                    body: body.clone(),
                    face,
                    body_material: body_material.clone(),
                    face_material: face_material.clone(),
                    center,
                    face_offset: 0.06,
                    pickable: Pickable::IGNORE,
                },
            );
        })
        .id();
    queue.apply(&mut world);

    let children = world.get::<Children>(root).unwrap();
    assert_eq!(children.len(), 3);
    let parts: Vec<_> = children
        .iter()
        .map(|child| {
            assert_eq!(world.get::<Pickable>(child), Some(&Pickable::IGNORE));
            (
                &world.get::<Mesh3d>(child).unwrap().0,
                &world
                    .get::<MeshMaterial3d<StandardMaterial>>(child)
                    .unwrap()
                    .0,
                world.get::<Transform>(child).unwrap(),
            )
        })
        .collect();

    let (_, material, transform) = parts
        .iter()
        .find(|(mesh, _, _)| *mesh == &body)
        .expect("the consumer's solid is assembled under its root");
    assert_eq!(*material, &body_material);
    assert_eq!(transform.translation, center);

    for outward in [Vec3::Z, Vec3::NEG_Z] {
        let (mesh, material, transform) = parts
            .iter()
            .find(|(_, _, transform)| (transform.translation - center).dot(outward) > 0.05)
            .expect("one picture surface sits outside each solid face");
        assert_eq!(*material, &face_material);
        assert_eq!(
            materials.get(*material).unwrap().base_color_texture,
            Some(picture.clone())
        );
        assert_picture_orientation(meshes.get(*mesh).unwrap(), transform, outward);
    }
}

fn assert_picture_orientation(mesh: &Mesh, transform: &Transform, outward: Vec3) {
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("face positions");
    };
    let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) else {
        panic!("face texture coordinates");
    };
    let right = Vec3::Y.cross(outward);
    let points: Vec<_> = positions
        .iter()
        .map(|&position| transform.transform_point(Vec3::from(position)) - transform.translation)
        .collect();

    for (corner, uv) in [
        (Vec2::new(-1.0, 1.5), [0.0, 0.0]),
        (Vec2::new(1.0, 1.5), [1.0, 0.0]),
        (Vec2::new(-1.0, -1.5), [0.0, 1.0]),
        (Vec2::new(1.0, -1.5), [1.0, 1.0]),
    ] {
        let vertex = points
            .iter()
            .position(|point| Vec2::new(point.dot(right), point.y).abs_diff_eq(corner, 1e-5))
            .expect("the supplied face reaches each picture corner");
        assert_eq!(
            uvs[vertex], uv,
            "picture corner at {corner} facing {outward}"
        );
    }

    let indices: Vec<_> = mesh.indices().unwrap().iter().collect();
    let (triangles, rest) = indices.as_chunks::<3>();
    assert!(!triangles.is_empty() && rest.is_empty());
    for &[a, b, c] in triangles {
        let normal = (points[b] - points[a]).cross(points[c] - points[a]);
        assert!(normal.dot(outward) > 0.0, "face winding points out");
    }
}
