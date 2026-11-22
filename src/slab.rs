//! A solid with the same picture facing out from both sides.
//!
//! The consumer supplies the solid, face mesh and materials. This module sets
//! the depth and orientation of the two picture surfaces. The solid and faces
//! stay under the consumer's entity so they can move, turn and be picked as one.

use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};

/// Meshes and appearance of a slab in its parent's coordinates.
pub struct Slab {
    /// Solid mesh centered on the slab's origin, with its thickness along Z.
    pub body: Handle<Mesh>,
    /// Flat mesh in XY at Z = 0, with +Y up and winding toward +Z. Its UVs
    /// orient the picture as viewed from +Z. Its outline must fit the body
    /// both as supplied and after a half turn around Y.
    pub face: Handle<Mesh>,
    /// Material for the solid, including the visible edges.
    pub body_material: Handle<StandardMaterial>,
    /// Material applied to both faces, normally textured with the picture.
    pub face_material: Handle<StandardMaterial>,
    /// Center of the slab relative to its parent.
    pub center: Vec3,
    /// Positive distance from the center to each picture surface. Choose
    /// a distance just beyond the solid's surfaces to avoid depth fighting.
    pub face_offset: f32,
    /// Picking behavior of the body and both faces.
    pub pickable: Pickable,
}

/// Spawns the solid and both outward picture faces as direct children.
///
/// The rear face turns around the up axis so the same UVs read upright and
/// left to right from either side. The parent controls the slab's orientation.
///
/// The library test checks the face geometry and texture mapping without a
/// renderer. Artwork, fit and appearance at the consumer's camera stay with
/// the consumer.
pub fn spawn(parts: &mut ChildSpawnerCommands, slab: Slab) {
    parts.spawn((
        Mesh3d(slab.body),
        MeshMaterial3d(slab.body_material),
        Transform::from_translation(slab.center),
        slab.pickable,
    ));

    for (side, yaw) in [(1.0, 0.0), (-1.0, std::f32::consts::PI)] {
        parts.spawn((
            Mesh3d(slab.face.clone()),
            MeshMaterial3d(slab.face_material.clone()),
            Transform::from_translation(slab.center + Vec3::Z * (side * slab.face_offset))
                .with_rotation(Quat::from_rotation_y(yaw)),
            slab.pickable,
        ));
    }
}
