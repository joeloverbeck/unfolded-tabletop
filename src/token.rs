//! Solid body and textured face assembly extracted from Arkham counters.
//! Consumers select silhouettes, sizes, UVs, alpha, meanings and placement.
use bevy::prelude::*;

pub struct Body {
    pub name: String,
    pub mesh: Mesh,
    pub color: Color,
    pub transform: Transform,
}

pub fn spawn_body(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    body: Body,
) -> Entity {
    commands
        .spawn((
            Name::new(body.name),
            Mesh3d(meshes.add(body.mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: body.color,
                perceptual_roughness: 0.7,
                ..default()
            })),
            body.transform,
            Pickable::IGNORE,
        ))
        .id()
}

pub struct Face {
    pub mesh: Mesh,
    pub image: Handle<Image>,
    pub alpha: AlphaMode,
    pub transform: Transform,
}

pub fn add_face(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    parent: Entity,
    face: Face,
) {
    commands.entity(parent).with_child((
        Mesh3d(meshes.add(face.mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(face.image),
            unlit: true,
            alpha_mode: face.alpha,
            ..default()
        })),
        face.transform,
        Pickable::IGNORE,
    ));
}
