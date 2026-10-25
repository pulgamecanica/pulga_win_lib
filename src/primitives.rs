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

#[cfg(test)]
mod tests {
    use super::*;

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
