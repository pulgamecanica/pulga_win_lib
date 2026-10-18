//! Wavefront OBJ parsing — our own code, no mesh-loading library, per the
//! subject. Ported from a from-scratch C++ OBJ loader written for an
//! earlier 42 project (~/scop): fan-triangulation, missing-normal
//! computation, and a centroid/main-axis heuristic used to frame the mesh.

use std::fmt;
use std::fs;
use std::path::Path;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub face_id: u32,
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub centroid: [f32; 3],
    pub main_axis: [f32; 3],
    pub has_uv: bool,
}

#[derive(Debug)]
pub enum ObjError {
    Io(std::io::Error),
    BadIndex(String),
}

impl fmt::Display for ObjError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjError::Io(e) => write!(f, "could not read OBJ file: {e}"),
            ObjError::BadIndex(tok) => write!(f, "malformed face index: {tok}"),
        }
    }
}

impl std::error::Error for ObjError {}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 0.0 { [v[0] / len, v[1] / len, v[2] / len] } else { [0.0, 0.0, 0.0] }
}

#[derive(Default, Clone, Copy)]
struct FaceIdx {
    v: i32,
    vt: i32,
    vn: i32,
}

/// Parses "v", "v/t", "v//n" or "v/t/n" (1-based OBJ indices -> 0-based, -1 = absent).
fn parse_face_elem(tok: &str) -> Result<FaceIdx, ObjError> {
    let parse = |s: &str| -> Result<i32, ObjError> {
        s.parse::<i32>().map(|i| i - 1).map_err(|_| ObjError::BadIndex(tok.to_string()))
    };
    let parts: Vec<&str> = tok.split('/').collect();
    match parts.as_slice() {
        [v] => Ok(FaceIdx { v: parse(v)?, vt: -1, vn: -1 }),
        [v, vt] => Ok(FaceIdx { v: parse(v)?, vt: parse(vt)?, vn: -1 }),
        [v, "", vn] => Ok(FaceIdx { v: parse(v)?, vt: -1, vn: parse(vn)? }),
        [v, vt, vn] => Ok(FaceIdx { v: parse(v)?, vt: parse(vt)?, vn: parse(vn)? }),
        _ => Err(ObjError::BadIndex(tok.to_string())),
    }
}

/// Parses OBJ text (triangulating n-gon faces as a fan, and computing a flat
/// normal per triangle when the file doesn't supply `vn`).
pub fn parse(text: &str) -> Result<Mesh, ObjError> {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();

    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut face_id: u32 = 0;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut tokens = line.split_whitespace();
        let Some(tag) = tokens.next() else { continue };

        match tag {
            "v" => {
                let nums: Vec<f32> = tokens.filter_map(|t| t.parse().ok()).collect();
                if nums.len() >= 3 {
                    positions.push([nums[0], nums[1], nums[2]]);
                }
            }
            "vt" => {
                let nums: Vec<f32> = tokens.filter_map(|t| t.parse().ok()).collect();
                if nums.len() >= 2 {
                    uvs.push([nums[0], nums[1]]);
                }
            }
            "vn" => {
                let nums: Vec<f32> = tokens.filter_map(|t| t.parse().ok()).collect();
                if nums.len() >= 3 {
                    normals.push([nums[0], nums[1], nums[2]]);
                }
            }
            "f" => {
                let fidx: Vec<FaceIdx> =
                    tokens.map(parse_face_elem).collect::<Result<_, _>>()?;
                if fidx.len() < 3 {
                    continue;
                }

                let resolve_p = |i: i32| -> [f32; 3] {
                    usize::try_from(i).ok().and_then(|i| positions.get(i).copied()).unwrap_or([0.0; 3])
                };
                let resolve_n = |i: i32| -> Option<[f32; 3]> {
                    usize::try_from(i).ok().and_then(|i| normals.get(i).copied())
                };
                let resolve_t = |i: i32| -> [f32; 2] {
                    usize::try_from(i).ok().and_then(|i| uvs.get(i).copied()).unwrap_or([0.0; 2])
                };

                // Fan triangulation: (0, i+1, i+2) for i in 0..len-2.
                for i in 0..fidx.len() - 2 {
                    let (i0, i1, i2) = (fidx[0], fidx[i + 1], fidx[i + 2]);
                    let (p0, p1, p2) = (resolve_p(i0.v), resolve_p(i1.v), resolve_p(i2.v));

                    let (n0, n1, n2) = match (resolve_n(i0.vn), resolve_n(i1.vn), resolve_n(i2.vn))
                    {
                        (Some(a), Some(b), Some(c)) => (a, b, c),
                        _ => {
                            let flat = normalize(cross(sub(p1, p0), sub(p2, p0)));
                            (flat, flat, flat)
                        }
                    };

                    let base = vertices.len() as u32;
                    vertices.push(Vertex { pos: p0, normal: n0, uv: resolve_t(i0.vt), face_id });
                    vertices.push(Vertex { pos: p1, normal: n1, uv: resolve_t(i1.vt), face_id });
                    vertices.push(Vertex { pos: p2, normal: n2, uv: resolve_t(i2.vt), face_id });
                    indices.push(base);
                    indices.push(base + 1);
                    indices.push(base + 2);

                    face_id += 1;
                }
            }
            _ => {}
        }
    }

    let centroid = if vertices.is_empty() {
        [0.0; 3]
    } else {
        let sum = vertices.iter().fold([0.0f32; 3], |acc, v| {
            [acc[0] + v.pos[0], acc[1] + v.pos[1], acc[2] + v.pos[2]]
        });
        let n = vertices.len() as f32;
        [sum[0] / n, sum[1] / n, sum[2] / n]
    };

    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for v in &vertices {
        for i in 0..3 {
            min[i] = min[i].min(v.pos[i]);
            max[i] = max[i].max(v.pos[i]);
        }
    }
    let extent = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
    let main_axis = if extent[0] >= extent[1] && extent[0] >= extent[2] {
        [1.0, 0.0, 0.0]
    } else if extent[2] >= extent[0] && extent[2] >= extent[1] {
        [0.0, 0.0, 1.0]
    } else {
        [0.0, 1.0, 0.0]
    };

    Ok(Mesh { vertices, indices, centroid, main_axis, has_uv: !uvs.is_empty() })
}

pub fn load(path: impl AsRef<Path>) -> Result<Mesh, ObjError> {
    let text = fs::read_to_string(path).map_err(ObjError::Io)?;
    parse(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_single_triangle() {
        let obj = "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n";
        let mesh = parse(obj).unwrap();
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.indices, vec![0, 1, 2]);
        assert!(!mesh.has_uv);
    }

    #[test]
    fn fan_triangulates_a_quad_into_two_triangles() {
        let obj = "v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4\n";
        let mesh = parse(obj).unwrap();
        assert_eq!(mesh.vertices.len(), 6); // two triangles, 3 verts each
        assert_eq!(mesh.indices, vec![0, 1, 2, 3, 4, 5]);
        // both triangles share face_id 0's successor pattern: 0, then 1
        assert_eq!(mesh.vertices[0].face_id, 0);
        assert_eq!(mesh.vertices[3].face_id, 1);
    }

    #[test]
    fn computes_flat_normal_when_missing() {
        let obj = "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n";
        let mesh = parse(obj).unwrap();
        // CCW triangle in the XY plane -> normal should point +Z.
        for v in &mesh.vertices {
            assert!((v.normal[2] - 1.0).abs() < 1e-6, "normal = {:?}", v.normal);
        }
    }

    #[test]
    fn uses_explicit_normals_and_uvs_when_present() {
        let obj = "v 0 0 0\nv 1 0 0\nv 0 1 0\nvt 0 0\nvt 1 0\nvt 0 1\nvn 0 0 1\nf 1/1/1 2/2/1 3/3/1\n";
        let mesh = parse(obj).unwrap();
        assert!(mesh.has_uv);
        assert_eq!(mesh.vertices[1].uv, [1.0, 0.0]);
        assert_eq!(mesh.vertices[0].normal, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn centroid_is_the_average_vertex_position() {
        let obj = "v 0 0 0\nv 2 0 0\nv 0 2 0\nf 1 2 3\n";
        let mesh = parse(obj).unwrap();
        let [x, y, z] = mesh.centroid;
        assert!((x - 2.0 / 3.0).abs() < 1e-6);
        assert!((y - 2.0 / 3.0).abs() < 1e-6);
        assert_eq!(z, 0.0);
    }

    #[test]
    fn rejects_malformed_face_index() {
        let obj = "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1/x 2 3\n";
        assert!(parse(obj).is_err());
    }

    #[test]
    fn loads_the_bundled_42_logo_asset() {
        let mesh = load(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/models/42.obj")).unwrap();
        assert!(!mesh.vertices.is_empty());
        assert_eq!(mesh.vertices.len() % 3, 0); // fully triangulated
        assert_eq!(mesh.indices.len(), mesh.vertices.len());
        assert!(!mesh.has_uv); // this asset ships without vt lines
    }
}
