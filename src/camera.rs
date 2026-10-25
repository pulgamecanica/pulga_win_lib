//! A perspective camera with a dolly zoom, built on [`crate::transform`].
//! Design mirrors an earlier from-scratch C++ `Camera` (`~/scop`): cache the
//! look-at params so `zoom` can recompute the view matrix without the
//! caller re-supplying eye/center/up every frame.

use crate::transform::{self, Mat4};
use crate::vec3::{add, cross, norm, normalize, sub};
use pulgamecanica_matrix::pulgamecanica::projection_matrix::projection;

pub struct Camera {
    fov: f32,
    aspect: f32,
    znear: f32,
    zfar: f32,
    eye: [f32; 3],
    center: [f32; 3],
    up: [f32; 3],
}

const MIN_DISTANCE: f32 = 0.05;

impl Camera {
    pub fn new(fov_radians: f32, aspect: f32, znear: f32, zfar: f32) -> Self {
        Self {
            fov: fov_radians,
            aspect,
            znear,
            zfar,
            eye: [0.0, 0.0, 1.0],
            center: [0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
        }
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }

    pub fn look_at(&mut self, eye: [f32; 3], center: [f32; 3], up: [f32; 3]) {
        self.eye = eye;
        self.center = center;
        self.up = up;
    }

    /// Dollies the eye toward/away from `center` along the view axis.
    /// `factor < 1` zooms in, `factor > 1` zooms out; `center` stays fixed.
    /// Distance is clamped so the eye never reaches (or passes) the target.
    pub fn zoom(&mut self, factor: f32) {
        let offset = sub(self.eye, self.center);
        let distance = (norm(offset) * factor).max(MIN_DISTANCE);
        let direction = if norm(offset) > 0.0 {
            normalize(offset)
        } else {
            [0.0, 0.0, 1.0]
        };
        self.eye = add(
            self.center,
            [
                direction[0] * distance,
                direction[1] * distance,
                direction[2] * distance,
            ],
        );
    }

    pub fn proj(&self) -> Mat4 {
        projection(self.fov, self.aspect, self.znear, self.zfar)
    }

    pub fn view(&self) -> Mat4 {
        transform::look_at(self.eye, self.center, self.up)
    }

    pub fn eye(&self) -> [f32; 3] {
        self.eye
    }

    /// The world-space ray from the eye through a point on screen, for mouse
    /// picking. `ndc_x`/`ndc_y` are normalized device coordinates: -1..1,
    /// left to right and bottom to top. Returns `(origin, unit direction)`.
    pub fn ray(&self, ndc_x: f32, ndc_y: f32) -> ([f32; 3], [f32; 3]) {
        let forward = normalize(sub(self.center, self.eye));
        let right = normalize(cross(forward, self.up));
        let up = cross(right, forward);
        let half_h = (self.fov / 2.0).tan(); // fov is vertical, like projection()
        let (sx, sy) = (ndc_x * half_h * self.aspect, ndc_y * half_h);
        let dir = [0, 1, 2].map(|i| forward[i] + right[i] * sx + up[i] * sy);
        (self.eye, normalize(dir))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_in_moves_eye_closer_to_center() {
        let mut cam = Camera::new(1.0, 1.0, 0.1, 100.0);
        cam.look_at([0.0, 0.0, 10.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        cam.zoom(0.5);
        assert!((cam.eye()[2] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn zoom_out_moves_eye_away_from_center() {
        let mut cam = Camera::new(1.0, 1.0, 0.1, 100.0);
        cam.look_at([0.0, 0.0, 10.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        cam.zoom(2.0);
        assert!((cam.eye()[2] - 20.0).abs() < 1e-5);
    }

    #[test]
    fn ray_through_center_and_right_edge() {
        let mut cam = Camera::new(std::f32::consts::FRAC_PI_2, 2.0, 0.1, 100.0);
        cam.look_at([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let (origin, center) = cam.ray(0.0, 0.0);
        assert_eq!(origin, [0.0, 0.0, 5.0]);
        assert!((center[2] + 1.0).abs() < 1e-5);
        // 90° vertical fov, aspect 2: the right edge is 2 units right per unit forward.
        let (_, right) = cam.ray(1.0, 0.0);
        assert!((right[0] / -right[2] - 2.0).abs() < 1e-5);
        let (_, top) = cam.ray(0.0, 1.0);
        assert!((top[1] / -top[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn zoom_never_crosses_the_target() {
        let mut cam = Camera::new(1.0, 1.0, 0.1, 100.0);
        cam.look_at([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        for _ in 0..50 {
            cam.zoom(0.1);
        }
        assert!(cam.eye()[2] >= MIN_DISTANCE);
    }
}
