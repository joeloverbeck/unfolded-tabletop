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

/// One card's solid: a flat side between its two faces, centered on the
/// origin with its thickness `height` along Y.
pub fn body(width: f32, height: f32, length: f32) -> Mesh {
    solid(
        width,
        height,
        length,
        &[(0., -height / 2.), (0., height / 2.)],
    )
}

/// Share of a card's thickness that each card's rounded edge takes above and
/// below a seam.
const SEAM_HEIGHT: f32 = 0.35;
/// How far a seam's groove sets in from the outline, as a share of a card's
/// thickness.
const SEAM_DEPTH: f32 = 0.35;

/// A pile of `cards` cards, each `card_thickness` thick, as one solid with the
/// footprint of [`body`] and `cards × card_thickness` of height, centered on the
/// origin along Y.
///
/// Its side shows each card's edge: between each two cards a shallow groove
/// sets in from the outline, so the card below turns up into it and the card
/// above turns down. Lit from above, each seam reads as a line, with no
/// texture. Fewer than two cards give exactly [`body`] one card thick.
pub fn pile(width: f32, length: f32, card_thickness: f32, cards: usize) -> Mesh {
    let cards = cards.max(1);
    let height = card_thickness * cards as f32;
    let bottom = -height / 2.;
    let (rise, depth) = (card_thickness * SEAM_HEIGHT, card_thickness * SEAM_DEPTH);
    let mut profile = vec![(0., bottom)];
    for k in 1..cards {
        let seam = bottom + card_thickness * k as f32;
        profile.extend([(0., seam - rise), (depth, seam), (0., seam + rise)]);
    }
    profile.push((0., height / 2.));
    solid(width, height, length, &profile)
}

/// A card-shaped solid whose side follows `profile` from bottom to top: each
/// point is how far the side sets in from the outline, and at what height.
fn solid(width: f32, height: f32, length: f32, profile: &[(f32, f32)]) -> Mesh {
    let radius = width.min(length) * CORNER_FRACTION;
    let outline = perimeter(width, length, radius);
    let set_in = |inset: f32| {
        if inset == 0. {
            outline.clone()
        } else {
            let inset = inset.min(radius);
            perimeter(width - 2. * inset, length - 2. * inset, radius - inset)
        }
    };
    let rings: Vec<_> = profile.iter().map(|&(inset, _)| set_in(inset)).collect();
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uv = Vec::new();
    let mut indices = Vec::new();
    for (i, from) in outline.iter().enumerate() {
        let to = outline[(i + 1) % outline.len()];
        let edge = to - *from;
        let outward = Vec3::new(edge.y, 0., -edge.x).normalize_or_zero();
        let next = (i + 1) % outline.len();
        for band in 0..profile.len() - 1 {
            let ((low_inset, low), (high_inset, high)) = (profile[band], profile[band + 1]);
            // A band that sets in as it rises faces up, and one that comes
            // out faces down; a straight band faces straight out.
            let normal = if low_inset == high_inset {
                outward
            } else {
                (outward * (high - low) + Vec3::Y * (high_inset - low_inset)).normalize_or_zero()
            }
            .to_array();
            let (below, above) = (&rings[band], &rings[band + 1]);
            let start = positions.len() as u32;
            positions.extend([
                [below[i].x, low, below[i].y],
                [below[next].x, low, below[next].y],
                [above[next].x, high, above[next].y],
                [above[i].x, high, above[i].y],
            ]);
            normals.extend([normal; 4]);
            uv.extend([[0., 0.], [1., 0.], [1., 1.], [0., 1.]]);
            indices.extend([start, start + 2, start + 1, start, start + 3, start + 2]);
        }
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
