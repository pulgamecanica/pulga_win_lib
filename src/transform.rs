//! Transformation matrices, built on the user's own
//! [`pulgamecanica_matrix`](https://crates.io/crates/pulgamecanica_matrix)
//! crate (per the subject: compute your own transforms, not a library like
//! glu/glm). That crate ships `projection()` for the perspective matrix;
//! this module adds the rest (translate/scale/rotate/look_at), ported from
//! the same formulas used in an earlier from-scratch C++ renderer
//! (~/scop/srcs/Math.cpp).

use crate::vec3::{cross, dot, normalize, sub};
use pulgamecanica_matrix::pulgamecanica::Matrix;

pub type Mat4 = Matrix<f32>;

pub fn identity() -> Mat4 {
    Matrix::from([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

pub fn translate(t: [f32; 3]) -> Mat4 {
    let mut m = identity();
    m.data[0][3] = t[0];
    m.data[1][3] = t[1];
    m.data[2][3] = t[2];
    m
}

pub fn scale(s: [f32; 3]) -> Mat4 {
    Matrix::from([
        [s[0], 0.0, 0.0, 0.0],
        [0.0, s[1], 0.0, 0.0],
        [0.0, 0.0, s[2], 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

/// Right-handed rotation of `radians` around `axis` (need not be normalized).
pub fn rotate_axis_angle(axis: [f32; 3], radians: f32) -> Mat4 {
    let [x, y, z] = normalize(axis);
    let (s, c) = radians.sin_cos();
    let t = 1.0 - c;

    Matrix::from([
        [t * x * x + c, t * x * y - s * z, t * x * z + s * y, 0.0],
        [t * x * y + s * z, t * y * y + c, t * y * z - s * x, 0.0],
        [t * x * z - s * y, t * y * z + s * x, t * z * z + c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

/// Right-handed view matrix looking from `eye` toward `center`.
pub fn look_at(eye: [f32; 3], center: [f32; 3], up: [f32; 3]) -> Mat4 {
    let f = normalize(sub(center, eye)); // forward
    let s = normalize(cross(f, up)); // right
    let u = cross(s, f); // corrected up

    Matrix::from([
        [s[0], s[1], s[2], -dot(s, eye)],
        [u[0], u[1], u[2], -dot(u, eye)],
        [-f[0], -f[1], -f[2], dot(f, eye)],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

/// Flattens to the 16 floats OpenGL expects, row-major as stored — pass
/// `transpose = GL_TRUE` to `glUniformMatrix4fv` (this matrix multiplies
/// column vectors on its left, like the formulas above assume).
pub fn to_gl(m: &Mat4) -> [f32; 16] {
    let mut out = [0.0f32; 16];
    for r in 0..4 {
        for c in 0..4 {
            out[r * 4 + c] = m.data[r][c];
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mul_point(m: &Mat4, p: [f32; 3]) -> [f32; 3] {
        let v = pulgamecanica_matrix::pulgamecanica::Vector::new(vec![p[0], p[1], p[2], 1.0]);
        let r = m.mul_vec(&v);
        [r.data[0], r.data[1], r.data[2]]
    }

    fn assert_close(a: [f32; 3], b: [f32; 3]) {
        for i in 0..3 {
            assert!((a[i] - b[i]).abs() < 1e-5, "{a:?} != {b:?}");
        }
    }

    #[test]
    fn translate_moves_a_point() {
        let m = translate([1.0, 2.0, 3.0]);
        assert_close(mul_point(&m, [0.0, 0.0, 0.0]), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn scale_scales_a_point() {
        let m = scale([2.0, 3.0, 4.0]);
        assert_close(mul_point(&m, [1.0, 1.0, 1.0]), [2.0, 3.0, 4.0]);
    }

    #[test]
    fn rotate_90_around_y_maps_plus_x_to_minus_z() {
        let m = rotate_axis_angle([0.0, 1.0, 0.0], std::f32::consts::FRAC_PI_2);
        assert_close(mul_point(&m, [1.0, 0.0, 0.0]), [0.0, 0.0, -1.0]);
    }

    #[test]
    fn translate_then_scale_composes_left_to_right() {
        // (T * S) applied to a point scales first, then translates.
        let m = translate([10.0, 0.0, 0.0]).mul_mat(&scale([2.0, 2.0, 2.0]));
        assert_close(mul_point(&m, [1.0, 0.0, 0.0]), [12.0, 0.0, 0.0]);
    }

    #[test]
    fn look_at_eye_maps_center_to_negative_z() {
        let m = look_at([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        // The look-at target sits straight down -Z in view space.
        assert_close(mul_point(&m, [0.0, 0.0, 0.0]), [0.0, 0.0, -5.0]);
    }
}
