#![cfg(feature = "render")]
//! Card silhouettes at the public interface. Expectations come from Arkham's
//! recorded card-shape test and hand calculation of the rounded rectangle.
use bevy::{mesh::VertexAttributeValues, prelude::*};
use unfolded_tabletop::card::{
    CORNER_FRACTION, body, face, face_with_corners, outline, pile, pile_with_corners,
};

/// Arkham's investigator card and Netrunner's card, width by length.
const SIZES: [(f32, f32); 2] = [(2.0, 2.8), (1.8, 2.5)];

fn positions(mesh: &Mesh) -> Vec<Vec3> {
    let Some(VertexAttributeValues::Float32x3(values)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("card positions")
    };
    values.iter().copied().map(Vec3::from).collect()
}

fn normals(mesh: &Mesh) -> Vec<Vec3> {
    let Some(VertexAttributeValues::Float32x3(values)) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
    else {
        panic!("card normals")
    };
    values.iter().copied().map(Vec3::from).collect()
}

fn uvs(mesh: &Mesh) -> Vec<Vec2> {
    let Some(VertexAttributeValues::Float32x2(values)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0)
    else {
        panic!("card uvs")
    };
    values.iter().copied().map(Vec2::from).collect()
}

/// FNV-1a over a mesh's positions, normals, UVs and indices as little-endian
/// bytes, so a test can hold a mesh's exact output in one number.
fn digest(mesh: &Mesh) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    let mut feed = |bytes: [u8; 4]| {
        for byte in bytes {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
        }
    };
    for value in positions(mesh)
        .iter()
        .chain(&normals(mesh))
        .flat_map(|v| v.to_array())
        .chain(uvs(mesh).iter().flat_map(|v| v.to_array()))
    {
        feed(value.to_bits().to_le_bytes());
    }
    for index in mesh.indices().unwrap().iter() {
        feed(u32::try_from(index).unwrap().to_le_bytes());
    }
    hash
}

#[test]
fn existing_callers_get_the_meshes_they_got_before() {
    // Recorded, not computed here: each digest is the output of the revision
    // named beside it, at Arkham's and Netrunner's card sizes.
    // `face` and `body` from 15580d5, before `pile` existed; `pile` from
    // c9d5102, where it was introduced (#92). Recorded on x86_64 Linux, as CI
    // runs: another target's `cos` and `sin` may round differently.
    let recorded = [
        ("face(1.8, 2.5)", face(1.8, 2.5), 0x8cc741b669ca7eb2),
        ("face(1.35, 1.89)", face(1.35, 1.89), 0xd3637b2eecc26db3),
        ("face(2.0, 2.8)", face(2.0, 2.8), 0xf460f8390ae8204c),
        (
            "body(1.8, 0.06, 2.5)",
            body(1.8, 0.06, 2.5),
            0x38d74316597981d6,
        ),
        (
            "body(1.35, 0.005, 1.89)",
            body(1.35, 0.005, 1.89),
            0x9f6fc97bba501cd2,
        ),
        (
            "body(2.0, 0.06, 2.8)",
            body(2.0, 0.06, 2.8),
            0xb38477d4b311a3e6,
        ),
        (
            "pile(1.8, 2.5, 0.0065, 1)",
            pile(1.8, 2.5, 0.0065, 1),
            0x66c3d060d799806e,
        ),
        (
            "pile(1.8, 2.5, 0.0065, 2)",
            pile(1.8, 2.5, 0.0065, 2),
            0xb36c8225cdcac38a,
        ),
        (
            "pile(1.8, 2.5, 0.0065, 40)",
            pile(1.8, 2.5, 0.0065, 40),
            0x610821efd62d219e,
        ),
        (
            "pile(1.35, 1.89, 0.0065, 1)",
            pile(1.35, 1.89, 0.0065, 1),
            0x4cf982c4ca8f037e,
        ),
        (
            "pile(1.35, 1.89, 0.0065, 3)",
            pile(1.35, 1.89, 0.0065, 3),
            0x29c2d4044019b21e,
        ),
        (
            "pile(1.35, 1.89, 0.0065, 40)",
            pile(1.35, 1.89, 0.0065, 40),
            0xa0685c8506b7887e,
        ),
    ];
    let mut differ = Vec::new();
    for (call, mesh, expected) in recorded {
        let actual = digest(&mesh);
        if actual != expected {
            differ.push(format!("{call}: {actual:#018x}, recorded {expected:#018x}"));
        }
    }
    assert!(differ.is_empty(), "{differ:#?}");
}

#[test]
fn the_corner_fraction_is_the_inherited_silhouette() {
    assert_eq!(CORNER_FRACTION, 0.045);
}

#[test]
fn card_silhouettes_keep_image_scale_and_face_outward() {
    for (width, length) in SIZES {
        for mesh in [
            face(width, length),
            body(width, 0.06, length),
            outline(width, length),
            pile(width, length, 0.0065, 3),
        ] {
            let positions = positions(&mesh);
            let normals = normals(&mesh);
            let indices: Vec<_> = mesh.indices().unwrap().iter().collect();
            let (triangles, rest) = indices.as_chunks::<3>();
            assert!(rest.is_empty());
            for &[a, b, c] in triangles {
                let cross = (positions[b] - positions[a]).cross(positions[c] - positions[a]);
                assert!(
                    cross.dot(normals[a]) > 0.,
                    "back-face culling must retain the visible card surface"
                );
            }
        }
    }
}

#[test]
fn faces_cut_rounded_corners_and_map_the_whole_image() {
    for (width, length) in SIZES {
        let mesh = face(width, length);
        let positions = positions(&mesh);
        let uvs = uvs(&mesh);
        // One center vertex plus nine points on each of four quarter circles.
        assert_eq!(positions.len(), 1 + 4 * 9);
        assert_eq!(positions[0], Vec3::ZERO);
        assert_eq!(uvs[0], Vec2::splat(0.5));
        let radius = width.min(length) * 0.045;
        let half = Vec2::new(width, length) / 2.;
        for (point, uv) in positions.iter().zip(&uvs) {
            assert_eq!(point.y, 0.);
            let flat = Vec2::new(point.x, point.z);
            assert!(flat.x.abs() <= half.x + 1e-5 && flat.y.abs() <= half.y + 1e-5);
            // Inside each corner square, points lie on the corner's circle.
            let inset = half - Vec2::splat(radius);
            if flat.x.abs() > inset.x + 1e-4 && flat.y.abs() > inset.y + 1e-4 {
                let center = inset * flat.signum();
                assert!(((flat - center).length() - radius).abs() < 1e-5);
            }
            assert!(
                !(flat.x.abs() > half.x - 0.01 && flat.y.abs() > half.y - 0.01),
                "the rectangular corner is cut away"
            );
            assert!((uv.x - (point.x / width + 0.5)).abs() < 0.00001);
            assert!((uv.y - (point.z / length + 0.5)).abs() < 0.00001);
        }
        // The straight edges reach the full width and length: no image is rescaled.
        assert!(positions.iter().any(|p| (p.x - half.x).abs() < 1e-5));
        assert!(positions.iter().any(|p| (p.z + half.y).abs() < 1e-5));
        assert!(normals(&mesh).iter().all(|normal| *normal == Vec3::Y));
    }
}

#[test]
fn bodies_have_real_thickness_with_both_faces_and_outward_sides() {
    let (width, height, length) = (2.0, 0.06, 2.8);
    let mesh = body(width, height, length);
    let positions = positions(&mesh);
    let normals = normals(&mesh);
    // 36 side quads of four vertices, then two 37-vertex faces.
    assert_eq!(positions.len(), 36 * 4 + 2 * 37);
    for (point, normal) in positions.iter().zip(&normals) {
        assert!((point.y.abs() - height / 2.).abs() < 1e-6, "{point}");
        if normal.y.abs() > 0.5 {
            assert_eq!(normal.y.signum(), point.y.signum(), "faces point away");
            assert!((normal.y.abs() - 1.).abs() < 1e-6);
        } else {
            assert!(normal.y.abs() < 1e-6);
            assert!(
                Vec2::new(normal.x, normal.z).dot(Vec2::new(point.x, point.z)) > 0.,
                "sides face outward"
            );
        }
    }
    let top = positions
        .iter()
        .zip(&normals)
        .filter(|(_, n)| n.y > 0.5)
        .count();
    let bottom = positions
        .iter()
        .zip(&normals)
        .filter(|(_, n)| n.y < -0.5)
        .count();
    assert_eq!((top, bottom), (37, 37));
}

#[test]
fn outlines_ring_the_card_outside_its_edge() {
    for (width, length) in SIZES {
        let mesh = outline(width, length);
        let positions = positions(&mesh);
        assert_eq!(positions.len(), 2 * 36);
        let half = Vec2::new(width, length) / 2.;
        let max = |points: &mut dyn Iterator<Item = &Vec3>| {
            points.fold(Vec2::ZERO, |m, p| m.max(Vec2::new(p.x.abs(), p.z.abs())))
        };
        // Even vertices form the inner edge 0.03 outside the card, odd the outer 0.055.
        let inner = max(&mut positions.iter().step_by(2));
        let outer = max(&mut positions.iter().skip(1).step_by(2));
        assert!(
            (inner - (half + 0.03)).abs().max_element() < 1e-5,
            "{inner}"
        );
        assert!(
            (outer - (half + 0.055)).abs().max_element() < 1e-5,
            "{outer}"
        );
        assert!(positions.iter().all(|p| p.y == 0.));
        assert!(normals(&mesh).iter().all(|n| *n == Vec3::Y));
        assert!(uvs(&mesh).iter().all(|uv| *uv == Vec2::ZERO));
    }
}

#[test]
fn a_pile_of_one_card_is_the_card_body() {
    for (width, length) in SIZES {
        let (single, card) = (pile(width, length, 0.0065, 1), body(width, 0.0065, length));
        assert_eq!(positions(&single), positions(&card));
        assert_eq!(normals(&single), normals(&card));
        assert_eq!(uvs(&single), uvs(&card));
        let indices = |mesh: &Mesh| mesh.indices().unwrap().iter().collect::<Vec<_>>();
        assert_eq!(indices(&single), indices(&card));
    }
}

#[test]
#[should_panic(expected = "a pile has at least one card")]
fn an_empty_pile_is_the_callers_to_draw() {
    // What an empty pile shows is the game's: Arkham draws one card's
    // thickness, Netrunner draws nothing.
    pile(1.8, 2.5, 0.0065, 0);
}

#[test]
fn a_caller_rounds_its_own_corners_on_the_face_and_the_pile() {
    // Cuba Libre's printed faces need a radius of 5.6% of the card's width:
    // on a 0.63 by 0.88 card that is 0.03528, against 4.5%'s 0.02835.
    let (width, length, fraction) = (0.63, 0.88, 0.056);
    let radius = 0.03528;
    let half = Vec2::new(width, length) / 2.;
    // The outline's middle point on the corner nearest +X+Z lies on the
    // diagonal, `radius × (1 − 1/√2)` in from each straight edge.
    let cut = radius * (1. - std::f32::consts::FRAC_1_SQRT_2);
    for mesh in [
        face_with_corners(width, length, fraction),
        pile_with_corners(width, length, fraction, 0.0003, 5),
    ] {
        let positions = positions(&mesh);
        let reach = positions.iter().fold(Vec3::ZERO, |m, p| m.max(p.abs()));
        assert!((reach.x - half.x).abs() < 1e-6 && (reach.z - half.y).abs() < 1e-6);
        let corner = Vec2::new(half.x - cut, half.y - cut);
        assert!(
            positions
                .iter()
                .any(|p| (Vec2::new(p.x, p.z) - corner).length() < 1e-6),
            "a point on the 5.6% corner"
        );
        // Nothing reaches past that corner's circle.
        let center = half - Vec2::splat(radius);
        assert!(positions.iter().all(|p| {
            let flat = Vec2::new(p.x, p.z);
            flat.x <= center.x || flat.y <= center.y || (flat - center).length() < radius + 1e-6
        }));
    }
    // The face drawn on the pile's top has the pile's outline: the same
    // ring of points as the pile's top face.
    let top = |mesh: &Mesh| {
        let normals = normals(mesh);
        positions(mesh)
            .into_iter()
            .zip(normals)
            .filter(|(p, n)| n.y == 1. && p.y > 0.)
            .map(|(p, _)| Vec2::new(p.x, p.z))
            .collect::<Vec<_>>()
    };
    let face = face_with_corners(width, length, fraction);
    let face_ring: Vec<_> = positions(&face)
        .iter()
        .map(|p| Vec2::new(p.x, p.z))
        .collect();
    assert_eq!(
        top(&pile_with_corners(width, length, fraction, 0.0003, 5)),
        face_ring
    );
}

#[test]
fn the_default_corners_are_the_corner_fraction() {
    for (width, length) in SIZES {
        let same = |a: &Mesh, b: &Mesh| digest(a) == digest(b);
        assert!(same(
            &face(width, length),
            &face_with_corners(width, length, 0.045)
        ));
        assert!(same(
            &pile(width, length, 0.0065, 4),
            &pile_with_corners(width, length, 0.045, 0.0065, 4)
        ));
    }
}

#[test]
fn a_pile_drawn_in_fewer_thicker_layers_keeps_its_height_and_seams_each_layer() {
    // A caller may show a seam every few cards by passing fewer, thicker
    // layers: here a 0.63-wide card 0.0003 thick, one seam every 6 to 7 cards
    // (Cuba Libre #140's selection). Cards, layers, and the height by hand.
    let (width, length, fraction) = (0.63, 0.88, 0.056);
    for (cards, layers, height) in [
        (1, 2, 0.0003),
        (2, 2, 0.0006),
        (4, 2, 0.0012),
        (18, 3, 0.0054),
        (34, 5, 0.0102),
        (48, 7, 0.0144),
        (52, 7, 0.0156),
    ] {
        let layer = height / layers as f32;
        let mesh = pile_with_corners(width, length, fraction, layer, layers);
        let side = straight_side(&mesh, width);
        let reach = positions(&mesh)
            .iter()
            .fold(Vec3::ZERO, |m, p| m.max(p.abs()));
        assert!(
            (reach.y - height / 2.).abs() < 1e-7,
            "{cards} cards: {reach}"
        );
        assert!((reach.x - width / 2.).abs() < 1e-6 && (reach.z - length / 2.).abs() < 1e-6);
        let mut seams: Vec<f32> = side
            .iter()
            .filter(|(_, inset)| *inset > 1e-6)
            .map(|(y, _)| *y)
            .collect();
        seams.sort_by(f32::total_cmp);
        seams.dedup_by(|a, b| (*a - *b).abs() < 1e-8);
        assert_eq!(seams.len(), layers - 1, "{cards} cards in {layers} layers");
    }
}

#[test]
fn a_pile_keeps_the_card_footprint_and_stands_as_tall_as_its_cards() {
    // Netrunner's and Arkham's piles: 0.0065 per card on their own card sizes.
    for (width, length) in [(1.8, 2.5), (1.35, 1.89)] {
        for cards in [2, 3, 40] {
            let height = cards as f32 * 0.0065;
            let mesh = pile(width, length, 0.0065, cards);
            let positions = positions(&mesh);
            let normals = normals(&mesh);
            let reach = positions.iter().fold(Vec3::ZERO, |m, p| m.max(p.abs()));
            assert!((reach.x - width / 2.).abs() < 1e-6, "{reach}");
            assert!((reach.z - length / 2.).abs() < 1e-6, "{reach}");
            assert!((reach.y - height / 2.).abs() < 1e-6, "{reach}");
            // Both picture faces stay whole: 37 vertices each at the top and bottom.
            for side in [1., -1.] {
                let face = positions
                    .iter()
                    .zip(&normals)
                    .filter(|(p, n)| n.y == side && (p.y - side * height / 2.).abs() < 1e-6)
                    .count();
                assert_eq!(face, 37, "{cards} cards, face {side}");
            }
        }
    }
}

/// The side of the long straight edge at +X, whose faces turn only toward +X
/// and up or down: each vertex's height and how far it sits in from the
/// card's outline.
fn straight_side(mesh: &Mesh, width: f32) -> Vec<(f32, f32)> {
    positions(mesh)
        .into_iter()
        .zip(normals(mesh))
        .filter(|(_, n)| n.y.abs() < 0.99 && n.x > 0.1 && n.z.abs() < 1e-6)
        .map(|(p, _)| (p.y, width / 2. - p.x))
        .collect()
}

#[test]
fn a_pile_side_shows_one_seam_between_each_two_cards_as_geometry() {
    let thickness = 0.0065;
    for (width, length) in [(1.8, 2.5), (1.35, 1.89)] {
        for cards in [2_usize, 3, 40] {
            let bottom = -(cards as f32) * thickness / 2.;
            let mesh = pile(width, length, thickness, cards);
            let side = straight_side(&mesh, width);
            // Seams: the side sets in from the outline only at each boundary
            // between two cards, and stays within a card's thickness.
            let mut seams: Vec<f32> = side
                .iter()
                .filter(|(_, inset)| *inset > 1e-6)
                .map(|(y, inset)| {
                    assert!(*inset < thickness, "a seam is shallower than a card");
                    *y
                })
                .collect();
            seams.sort_by(f32::total_cmp);
            seams.dedup_by(|a, b| (*a - *b).abs() < 1e-7);
            let boundaries: Vec<f32> = (1..cards).map(|k| bottom + k as f32 * thickness).collect();
            assert_eq!(seams.len(), boundaries.len(), "{cards} cards");
            for (seam, boundary) in seams.iter().zip(&boundaries) {
                assert!((seam - boundary).abs() < 1e-6, "{seam} at {boundary}");
            }
            // Each card shows its own edge at the full outline, between its
            // two seams (or the pile's top or bottom).
            for k in 0..cards {
                let (low, high) = (
                    bottom + k as f32 * thickness,
                    bottom + (k + 1) as f32 * thickness,
                );
                let full = side
                    .iter()
                    .any(|(y, inset)| inset.abs() < 1e-6 && *y > low + 1e-7 && *y < high - 1e-7);
                assert!(full, "card {k} of {cards} shows its edge");
            }
        }
    }
}

#[test]
fn a_seam_turns_its_faces_toward_and_away_from_the_light() {
    // Each seam is a groove: the card below turns up into it and the card
    // above turns down, so a light from above draws a line at every seam
    // with no texture. 36 outline segments, one facet each way per seam.
    let cards = 5;
    let mesh = pile(1.8, 2.5, 0.0065, cards);
    let side: Vec<Vec3> = normals(&mesh)
        .into_iter()
        .filter(|n| n.y.abs() < 0.99)
        .collect();
    let up = side.iter().filter(|n| n.y > 0.1).count();
    let down = side.iter().filter(|n| n.y < -0.1).count();
    assert_eq!((up, down), (36 * 4 * (cards - 1), 36 * 4 * (cards - 1)));
    assert!(
        side.iter().all(|n| (n.length() - 1.).abs() < 1e-5),
        "unit normals"
    );
}
