//! Shared physical-card silhouette. UVs retain the entire original image scale.
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

pub const CORNER_FRACTION: f32 = 0.045;

fn perimeter(width: f32, length: f32, radius: f32) -> Vec<Vec2> {
    let half = Vec2::new(width, length) * 0.5;
    let mut points = Vec::with_capacity(36);
    for (corner, center) in [
        Vec2::new(half.x - radius, half.y - radius),
        Vec2::new(-half.x + radius, half.y - radius),
        Vec2::new(-half.x + radius, -half.y + radius),
        Vec2::new(half.x - radius, -half.y + radius),
    ]
    .into_iter()
    .enumerate()
    {
        for step in 0..=8 {
            let angle = (corner as f32 + step as f32 / 8.) * std::f32::consts::FRAC_PI_2;
            points.push(center + Vec2::new(angle.cos(), angle.sin()) * radius);
        }
    }
    points
}

pub fn face(width: f32, length: f32) -> Mesh {
    let outline = perimeter(width, length, width.min(length) * CORNER_FRACTION);
    let mut positions = vec![[0., 0., 0.]];
    let mut uv = vec![[0.5, 0.5]];
    let mut indices = Vec::new();
    for point in &outline {
        positions.push([point.x, 0., point.y]);
        uv.push([point.x / width + 0.5, point.y / length + 0.5]);
    }
    for i in 0..outline.len() as u32 {
        indices.extend([0, (i + 1) % outline.len() as u32 + 1, i + 1]);
    }
    let count = positions.len();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
    .with_inserted_indices(Indices::U32(indices))
}

pub fn body(width: f32, height: f32, length: f32) -> Mesh {
    let outline = perimeter(width, length, width.min(length) * CORNER_FRACTION);
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uv = Vec::new();
    let mut indices = Vec::new();
    for (i, from) in outline.iter().enumerate() {
        let to = outline[(i + 1) % outline.len()];
        let edge = to - *from;
        let normal = Vec3::new(edge.y, 0., -edge.x)
            .normalize_or_zero()
            .to_array();
        let start = positions.len() as u32;
        positions.extend([
            [from.x, -height / 2., from.y],
            [to.x, -height / 2., to.y],
            [to.x, height / 2., to.y],
            [from.x, height / 2., from.y],
        ]);
        normals.extend([normal; 4]);
        uv.extend([[0., 0.], [1., 0.], [1., 1.], [0., 1.]]);
        indices.extend([start, start + 2, start + 1, start, start + 3, start + 2]);
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
    .with_inserted_indices(Indices::U32(indices));
    mesh.merge(&face(width, length).translated_by(Vec3::Y * height / 2.))
        .expect("Matching card attributes");
    mesh.merge(
        &face(width, length)
            .rotated_by(Quat::from_rotation_x(std::f32::consts::PI))
            .translated_by(-Vec3::Y * height / 2.),
    )
    .expect("Matching card attributes");
    mesh
}

pub fn outline(width: f32, length: f32) -> Mesh {
    let radius = width.min(length) * CORNER_FRACTION;
    let inner = perimeter(width + 0.06, length + 0.06, radius + 0.03);
    let outer = perimeter(width + 0.11, length + 0.11, radius + 0.055);
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    for (a, b) in inner.iter().zip(&outer) {
        positions.extend([[a.x, 0., a.y], [b.x, 0., b.y]]);
    }
    for i in 0..inner.len() as u32 {
        let a = i * 2;
        let b = ((i + 1) % inner.len() as u32) * 2;
        indices.extend([a, b, a + 1, a + 1, b, b + 1]);
    }
    let count = positions.len();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.]; count])
    .with_inserted_indices(Indices::U32(indices))
}
