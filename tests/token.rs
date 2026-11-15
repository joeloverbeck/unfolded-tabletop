#![cfg(feature = "render")]
//! Solid body and textured face assembly at the public interface, after
//! Arkham's clue/resource face test and Netrunner's credit-token test.
use bevy::{
    asset::RenderAssetUsages,
    ecs::world::CommandQueue,
    mesh::VertexAttributeValues,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use unfolded_tabletop::token::{Body, Face, add_face, spawn_body};

#[test]
fn a_solid_body_carries_a_textured_face_above_it_with_the_chosen_alpha() {
    let mut world = World::new();
    let mut queue = CommandQueue::default();
    let mut meshes = Assets::<Mesh>::default();
    let mut materials = Assets::<StandardMaterial>::default();
    let mut images = Assets::<Image>::default();
    // Arkham's clue face: visible RGB stored with zero alpha.
    let image = images.add(Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[254, 253, 197, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ));
    let color = Color::srgb(0.42, 0.09, 0.14);
    let at = Transform::from_xyz(1.0, 0.2, -3.0).with_scale(Vec3::splat(0.8));
    let roots = [AlphaMode::Opaque, AlphaMode::Blend].map(|alpha| {
        let mut commands = Commands::new(&mut queue, &world);
        let root = spawn_body(
            &mut commands,
            &mut meshes,
            &mut materials,
            Body {
                name: format!("Token {alpha:?}"),
                mesh: Mesh::from(Cylinder::new(0.31, 0.12)),
                color,
                transform: at,
            },
        );
        add_face(
            &mut commands,
            &mut meshes,
            &mut materials,
            root,
            Face {
                mesh: Mesh::from(Circle::new(0.31)),
                image: image.clone(),
                alpha,
                transform: Transform::from_xyz(0.0, 0.061, 0.0)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            },
        );
        (root, alpha)
    });
    queue.apply(&mut world);

    for (root, alpha) in roots {
        assert_eq!(
            world.get::<Name>(root).unwrap().as_str(),
            format!("Token {alpha:?}")
        );
        assert_eq!(*world.get::<Transform>(root).unwrap(), at);
        assert_eq!(world.get::<Pickable>(root), Some(&Pickable::IGNORE));
        let body = materials
            .get(
                &world
                    .get::<MeshMaterial3d<StandardMaterial>>(root)
                    .unwrap()
                    .0,
            )
            .unwrap();
        assert_eq!(body.base_color, color);
        assert_eq!(body.perceptual_roughness, 0.7);
        assert!(body.base_color_texture.is_none() && !body.unlit);
        let mesh = meshes.get(&world.get::<Mesh3d>(root).unwrap().0).unwrap();
        let Some(VertexAttributeValues::Float32x3(points)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("body positions")
        };
        // A solid body, not a flat backing.
        assert!(points.iter().any(|p| p[1] < 0.) && points.iter().any(|p| p[1] > 0.));

        let children = world.get::<Children>(root).unwrap();
        assert_eq!(children.len(), 1);
        let face = children[0];
        assert_eq!(world.get::<ChildOf>(face).unwrap().parent(), root);
        assert_eq!(world.get::<Pickable>(face), Some(&Pickable::IGNORE));
        let material = materials
            .get(
                &world
                    .get::<MeshMaterial3d<StandardMaterial>>(face)
                    .unwrap()
                    .0,
            )
            .unwrap();
        assert_eq!(material.base_color_texture, Some(image.clone()));
        assert!(material.unlit);
        assert_eq!(material.alpha_mode, alpha);
        let transform = world.get::<Transform>(face).unwrap();
        assert!(transform.translation.y > 0.06, "the face rests on the body");
        assert!((transform.rotation * Vec3::Z).dot(Vec3::Y) > 0.99);
        assert!(meshes.get(&world.get::<Mesh3d>(face).unwrap().0).is_some());
    }
}
