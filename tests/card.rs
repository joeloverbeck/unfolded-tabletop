#![cfg(feature = "render")]
//! Card silhouettes at the public interface. Expectations come from Arkham's
//! recorded card-shape test and hand calculation of the rounded rectangle.
use bevy::{mesh::VertexAttributeValues, prelude::*};
use unfolded_tabletop::card::{CORNER_FRACTION, body, face, outline};

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
