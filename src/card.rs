//! Shared physical-card silhouette. UVs retain the entire original image scale.
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

/// Corner radius of [`face`], [`body`], [`pile`] and [`outline`], as a share
/// of the card's shorter side.
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
    face_with_corners(width, length, CORNER_FRACTION)
}

/// [`face`] with a corner radius of `corner_fraction` of the card's shorter
/// side, for printed faces whose corners [`CORNER_FRACTION`] does not cut
/// away. It matches the top of [`pile_with_corners`] given the same fraction.
pub fn face_with_corners(width: f32, length: f32, corner_fraction: f32) -> Mesh {
    let outline = perimeter(width, length, width.min(length) * corner_fraction);
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
        length,
        CORNER_FRACTION,
        &[
            SidePoint {
                inset: 0.,
                y: -height / 2.,
            },
            SidePoint {
                inset: 0.,
                y: height / 2.,
            },
        ],
    )
}

/// How far above and below a seam the side starts to slope into its groove,
/// as a share of a card's thickness.
const SEAM_RISE: f32 = 0.35;
/// How far a seam's groove sets in from the outline, as a share of a card's
/// thickness.
const SEAM_DEPTH: f32 = 0.35;

/// A pile of `cards` cards, each `card_thickness` thick, as one solid with the
/// footprint of [`body`] and `cards × card_thickness` of height, centered on the
/// origin along Y.
///
/// Its side is cut by a V-shaped groove at each boundary between two cards:
/// two flat facets meet [`SEAM_DEPTH`] of a card's thickness in from the
/// outline, each sloping over [`SEAM_RISE`] of a card's thickness, so the
/// facet below faces partly up and the facet above partly down. Between its
/// grooves each card's edge stands at the full outline. This is geometry
/// only, with no texture; whether a seam can be seen depends on the light and
/// on how many pixels a card's thickness covers, which no test here holds.
/// The tests hold the seams' heights, their depth, the facets' normals and
/// that one card gives exactly [`body`] one card thick.
///
/// A caller that wants a seam every few cards passes fewer, thicker layers:
/// the height is still `cards × card_thickness` of whatever it passes.
///
/// # Panics
///
/// When `cards` is 0. What an empty pile shows is the caller's to decide.
pub fn pile(width: f32, length: f32, card_thickness: f32, cards: usize) -> Mesh {
    pile_with_corners(width, length, CORNER_FRACTION, card_thickness, cards)
}

/// [`pile`] with a corner radius of `corner_fraction` of the card's shorter
/// side, topped by the outline of [`face_with_corners`] with that fraction.
///
/// # Panics
///
/// When `cards` is 0.
pub fn pile_with_corners(
    width: f32,
    length: f32,
    corner_fraction: f32,
    card_thickness: f32,
    cards: usize,
) -> Mesh {
    assert!(cards > 0, "a pile has at least one card");
    let height = card_thickness * cards as f32;
    let bottom = -height / 2.;
    let (rise, depth) = (card_thickness * SEAM_RISE, card_thickness * SEAM_DEPTH);
    let flat = |y| SidePoint { inset: 0., y };
    let mut profile = vec![flat(bottom)];
    for k in 1..cards {
        let seam = bottom + card_thickness * k as f32;
        profile.extend([
            flat(seam - rise),
            SidePoint {
                inset: depth,
                y: seam,
            },
            flat(seam + rise),
        ]);
    }
    profile.push(flat(height / 2.));
    solid(width, length, corner_fraction, &profile)
}

/// A point on a solid's side, from bottom to top: how far the side sets in
/// from the card's outline, and at what height.
#[derive(Clone, Copy)]
struct SidePoint {
    inset: f32,
    y: f32,
}

/// A card-shaped solid whose side follows `profile`, with a face at its first
/// and last point's heights.
fn solid(width: f32, length: f32, corner_fraction: f32, profile: &[SidePoint]) -> Mesh {
    let radius = width.min(length) * corner_fraction;
    let outline = perimeter(width, length, radius);
    let set_in = |inset: f32| {
        if inset == 0. {
            outline.clone()
        } else {
            let inset = inset.min(radius);
            perimeter(width - 2. * inset, length - 2. * inset, radius - inset)
        }
    };
    let rings: Vec<_> = profile.iter().map(|point| set_in(point.inset)).collect();
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
            let (low, high) = (profile[band], profile[band + 1]);
            // A band that sets in as it rises faces up, and one that comes
            // out faces down; a straight band faces straight out.
            let normal = if low.inset == high.inset {
                outward
            } else {
                (outward * (high.y - low.y) + Vec3::Y * (high.inset - low.inset))
                    .normalize_or_zero()
            }
            .to_array();
            let (below, above) = (&rings[band], &rings[band + 1]);
            let start = positions.len() as u32;
            positions.extend([
                [below[i].x, low.y, below[i].y],
                [below[next].x, low.y, below[next].y],
                [above[next].x, high.y, above[next].y],
                [above[i].x, high.y, above[i].y],
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
    let (bottom, top) = (profile[0].y, profile[profile.len() - 1].y);
    let face = face_with_corners(width, length, corner_fraction);
    mesh.merge(&face.clone().translated_by(Vec3::Y * top))
        .expect("Matching card attributes");
    mesh.merge(
        &face
            .rotated_by(Quat::from_rotation_x(std::f32::consts::PI))
            .translated_by(Vec3::Y * bottom),
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
