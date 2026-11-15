#![cfg(feature = "render")]

use bevy::{
    camera::RenderTarget, ecs::world::CommandQueue, mesh::VertexAttributeValues, prelude::*,
    render::render_resource::TextureFormat,
};
use unfolded_tabletop::printed::{Atlas, Settings};

#[test]
fn caller_rows_become_transparent_mat_ink_with_their_own_orientation() {
    let mut world = World::new();
    let mut queue = CommandQueue::default();
    let mut images = Assets::default();
    let mut materials = Assets::default();
    let mut meshes = Assets::default();
    let mut atlas = Atlas::default();
    let transform = Transform::from_xyz(3., 0.071, -8.)
        .with_rotation(Quat::from_rotation_y(std::f32::consts::PI));
    let mut commands = Commands::new(&mut queue, &world);
    let printed = atlas.render(
        &mut commands,
        &mut materials,
        &mut images,
        Settings {
            width: 512,
            row_height: 64,
            camera_order: -20,
            ui_scale: 2.,
        },
        vec![
            (
                Text::new(String::from("Zone: 3")),
                TextFont::from_font_size(20.),
            ),
            (
                Text::new("Discard: 1 faceup"),
                TextFont::from_font_size(21.),
            ),
        ],
    );
    let plane = printed.spawn(&mut commands, &mut meshes, 1, 0.5, transform);
    queue.apply(&mut world);

    let camera = world.get::<Camera>(printed.camera).unwrap();
    assert_eq!(camera.order, -20);
    assert!(matches!(
        camera.clear_color,
        ClearColorConfig::Custom(Color::NONE)
    ));
    let RenderTarget::Image(target) = world.get::<RenderTarget>(printed.camera).unwrap() else {
        panic!("atlas camera must render to its image");
    };
    let image = images.get(&target.handle).unwrap();
    assert_eq!(image.size(), UVec2::new(512, 128));
    assert_eq!(
        image.texture_descriptor.format,
        TextureFormat::Bgra8UnormSrgb
    );
    assert_eq!(
        world.get::<UiTargetCamera>(printed.root).unwrap().0,
        printed.camera
    );
    let children = world.get::<Children>(printed.root).unwrap();
    for (index, row) in children.iter().enumerate() {
        let node = world.get::<Node>(row).unwrap();
        assert_eq!(node.width, px(256.));
        assert_eq!(node.height, px(32.));
        let text_entity = world.get::<Children>(row).unwrap()[0];
        assert_eq!(
            world.get::<Text>(text_entity).unwrap().0,
            ["Zone: 3", "Discard: 1 faceup"][index]
        );
        assert_eq!(
            world.get::<TextFont>(text_entity).unwrap().font_size,
            bevy::text::FontSize::Px([20., 21.][index])
        );
    }
    assert_eq!(world.get::<Transform>(plane).unwrap(), &transform);
    let material = materials
        .get(
            &world
                .get::<MeshMaterial3d<StandardMaterial>>(plane)
                .unwrap()
                .0,
        )
        .unwrap();
    assert!(material.unlit);
    assert_eq!(material.alpha_mode, AlphaMode::Blend);
    assert_eq!(
        material.base_color_texture.as_ref().unwrap(),
        &target.handle
    );
    let mesh = meshes.get(&world.get::<Mesh3d>(plane).unwrap().0).unwrap();
    let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) else {
        panic!("plane needs UVs");
    };
    assert!(
        uvs.iter()
            .all(|uv| [0., 1.].contains(&uv[0]) && [0.5, 1.].contains(&uv[1]))
    );
    assert!(uvs.iter().any(|uv| uv[1] == 0.5));
    assert!(uvs.iter().any(|uv| uv[1] == 1.));
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("plane needs positions");
    };
    assert!(
        positions
            .iter()
            .all(|p| p[0].abs() == 2. && p[1] == 0. && p[2].abs() == 0.25)
    );
}

#[test]
fn rebuilds_reuse_the_image_but_changed_row_counts_replace_it() {
    let mut world = World::new();
    let mut images = Assets::default();
    let mut materials = Assets::default();
    let mut atlas = Atlas::default();
    let mut previous = None;
    for (row_count, expected_size) in [
        (2, UVec2::new(512, 128)),
        (2, UVec2::new(512, 128)),
        (3, UVec2::new(512, 192)),
    ] {
        let mut queue = CommandQueue::default();
        let rendered = atlas.render(
            &mut Commands::new(&mut queue, &world),
            &mut materials,
            &mut images,
            Settings {
                width: 512,
                row_height: 64,
                camera_order: -20,
                ui_scale: 1.,
            },
            (0..row_count)
                .map(|row| Text::new(format!("Zone {row}")))
                .collect(),
        );
        queue.apply(&mut world);
        let RenderTarget::Image(target) = world.get::<RenderTarget>(rendered.camera).unwrap()
        else {
            panic!("atlas camera needs an image");
        };
        assert_eq!(images.get(&target.handle).unwrap().size(), expected_size);
        if let Some(previous) = previous {
            assert_eq!(target.handle == previous, row_count == 2);
        }
        previous = Some(target.handle.clone());
        world.despawn(rendered.camera);
        world.despawn(rendered.root);
        assert_eq!(world.query::<&Text>().iter(&world).count(), 0);
    }
}
