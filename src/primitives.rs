//! Procedural meshes (not OBJ-loaded — gameplay geometry like the ground
//! and obstacles doesn't come from an asset file). Reuses [`crate::obj`]'s
//! `Mesh`/`Vertex` so [`crate::gl_mesh::GlMesh`] uploads them the same way.

use crate::obj::{Mesh, Vertex};

fn face(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    face_id: &mut u32,
    corners: [[f32; 3]; 4],
    normal: [f32; 3],
) {
    let base = vertices.len() as u32;
    for pos in corners {
        vertices.push(Vertex {
            pos,
            normal,
            uv: [0.0, 0.0],
            face_id: *face_id,
        });
    }
    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    *face_id += 1;
}

/// An axis-aligned cube centered on the origin, `size` units per side.
pub fn cube(size: f32) -> Mesh {
    let h = size / 2.0;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut face_id = 0u32;

    // +X, -X, +Y, -Y, +Z, -Z, each as a quad wound CCW when viewed from outside.
    face(
        &mut vertices,
        &mut indices,
        &mut face_id,
        [[h, -h, -h], [h, h, -h], [h, h, h], [h, -h, h]],
        [1.0, 0.0, 0.0],
    );
    face(
        &mut vertices,
        &mut indices,
        &mut face_id,
        [[-h, -h, h], [-h, h, h], [-h, h, -h], [-h, -h, -h]],
        [-1.0, 0.0, 0.0],
    );
    face(
        &mut vertices,
        &mut indices,
        &mut face_id,
        [[-h, h, -h], [-h, h, h], [h, h, h], [h, h, -h]],
        [0.0, 1.0, 0.0],
    );
    face(
        &mut vertices,
        &mut indices,
        &mut face_id,
        [[-h, -h, h], [-h, -h, -h], [h, -h, -h], [h, -h, h]],
        [0.0, -1.0, 0.0],
    );
    face(
        &mut vertices,
        &mut indices,
        &mut face_id,
        [[-h, -h, h], [h, -h, h], [h, h, h], [-h, h, h]],
        [0.0, 0.0, 1.0],
    );
    face(
        &mut vertices,
        &mut indices,
        &mut face_id,
        [[h, -h, -h], [-h, -h, -h], [-h, h, -h], [h, h, -h]],
        [0.0, 0.0, -1.0],
    );

    Mesh {
        vertices,
        indices,
        centroid: [0.0; 3],
        main_axis: [0.0, 1.0, 0.0],
        has_uv: false,
    }
}

/// A flat quad in the XZ plane, `width` x `length`, centered on the origin,
/// facing +Y. One face, so it renders as a single flat color.
pub fn ground_plane(width: f32, length: f32) -> Mesh {
    let hw = width / 2.0;
    let hl = length / 2.0;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut face_id = 0u32;
    face(
        &mut vertices,
        &mut indices,
        &mut face_id,
        [
            [-hw, 0.0, -hl],
            [-hw, 0.0, hl],
            [hw, 0.0, hl],
            [hw, 0.0, -hl],
        ],
        [0.0, 1.0, 0.0],
    );
    Mesh {
        vertices,
        indices,
        centroid: [0.0; 3],
        main_axis: [0.0, 1.0, 0.0],
        has_uv: false,
    }
}

/// A UV sphere centered on the origin: `rings` bands from pole to pole,
/// `segments` slices around the equator (at least 2 and 3).
pub fn sphere(radius: f32, rings: u32, segments: u32) -> Mesh {
    let (rings, segments) = (rings.max(2), segments.max(3));
    let mut vertices = Vec::new();
    for r in 0..=rings {
        let theta = std::f32::consts::PI * r as f32 / rings as f32; // 0 at +y
        for s in 0..=segments {
            let phi = std::f32::consts::TAU * s as f32 / segments as f32;
            let normal = [
                theta.sin() * phi.cos(),
                theta.cos(),
                theta.sin() * phi.sin(),
            ];
            vertices.push(Vertex {
                pos: normal.map(|n| n * radius),
                normal,
                uv: [s as f32 / segments as f32, r as f32 / rings as f32],
                face_id: r * segments + s,
            });
        }
    }
    let row = segments + 1;
    let mut indices = Vec::new();
    for r in 0..rings {
        for s in 0..segments {
            let (a, b) = (r * row + s, (r + 1) * row + s);
            // Counter-clockwise seen from outside.
            indices.extend_from_slice(&[a, a + 1, b, a + 1, b + 1, b]);
        }
    }
    Mesh {
        vertices,
        indices,
        centroid: [0.0; 3],
        main_axis: [0.0, 1.0, 0.0],
        has_uv: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sphere_vertices_sit_on_the_radius_with_outward_normals() {
        let m = sphere(2.0, 8, 12);
        assert_eq!(m.vertices.len(), 9 * 13);
        assert_eq!(m.indices.len(), (8 * 12 * 6) as usize);
        for v in &m.vertices {
            let len = (v.pos[0].powi(2) + v.pos[1].powi(2) + v.pos[2].powi(2)).sqrt();
            assert!((len - 2.0).abs() < 1e-4);
            assert!((0..3).all(|i| (v.pos[i] - v.normal[i] * 2.0).abs() < 1e-4));
        }
    }

    #[test]
    fn cube_has_six_faces_of_four_vertices() {
        let m = cube(1.0);
        assert_eq!(m.vertices.len(), 24); // 6 faces * 4 verts, unshared for flat normals
        assert_eq!(m.indices.len(), 36); // 6 faces * 2 triangles * 3 indices
    }

    #[test]
    fn cube_vertices_are_within_half_size_of_center() {
        let m = cube(2.0);
        for v in &m.vertices {
            for c in v.pos {
                assert!(c.abs() <= 1.0 + 1e-6, "{:?}", v.pos);
            }
        }
    }

    #[test]
    fn ground_plane_is_flat_and_faces_up() {
        let m = ground_plane(10.0, 20.0);
        assert_eq!(m.vertices.len(), 4);
        for v in &m.vertices {
            assert_eq!(v.pos[1], 0.0);
            assert_eq!(v.normal, [0.0, 1.0, 0.0]);
        }
    }
}
