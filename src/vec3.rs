//! `[f32; 3]` convenience wrappers around
//! [`pulgamecanica_matrix`](https://crates.io/crates/pulgamecanica_matrix)'s
//! `Vector<f32>` — the actual math (dot/cross/norm/add/sub) lives there, not
//! reimplemented here. Shared by `obj`, `transform` and `camera` so none of
//! them hand-roll their own copy.

use pulgamecanica_matrix::pulgamecanica::Vector;

fn to_arr(v: Vector<f32>) -> [f32; 3] {
    [v.data[0], v.data[1], v.data[2]]
}

pub fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    let mut v = Vector::from(a);
    v.add(&Vector::from(b));
    to_arr(v)
}

pub fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    let mut v = Vector::from(a);
    v.sub(&Vector::from(b));
    to_arr(v)
}

pub fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    to_arr(Vector::cross_product(&Vector::from(a), &Vector::from(b)))
}

pub fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    Vector::from(a).dot(&Vector::from(b))
}

pub fn norm(v: [f32; 3]) -> f32 {
    Vector::from(v).norm()
}

pub fn normalize(v: [f32; 3]) -> [f32; 3] {
    let mut vv = Vector::from(v);
    let n = vv.norm();
    if n > 0.0 {
        vv.scl(1.0 / n);
    } else {
        vv.scl(0.0);
    }
    to_arr(vv)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cross_of_x_and_y_is_z() {
        assert_eq!(cross([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]), [0.0, 0.0, 1.0]);
    }

    #[test]
    fn normalize_zero_vector_stays_zero() {
        assert_eq!(normalize([0.0, 0.0, 0.0]), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn normalize_scales_to_unit_length() {
        let n = normalize([3.0, 0.0, 4.0]);
        assert!((norm(n) - 1.0).abs() < 1e-6);
    }
}
