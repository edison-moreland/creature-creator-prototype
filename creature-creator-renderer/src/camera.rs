use nalgebra::{Isometry3, Perspective3, Point3, Vector3};
use std::f32::consts::PI;

pub struct Camera {
    eye: Point3<f32>,
    target: Point3<f32>,
    fov: f32,
    aspect_ratio: f32,
}

impl Camera {
    pub fn new(eye: Point3<f32>, target: Point3<f32>, fov: f32) -> Self {
        Camera {
            eye,
            target,
            fov,
            aspect_ratio: 0.0,
        }
    }

    pub fn aspect_ratio_updated(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio
    }

    pub fn position(&self) -> [f32; 3] {
        self.eye.coords.data.0[0]
    }
    pub fn mvp_matrix(&self) -> [[f32; 4]; 4] {
        let view = Isometry3::look_at_rh(&self.eye, &self.target, &Vector3::y());

        let proj = Perspective3::new(self.aspect_ratio, self.fov * (180.0 / PI), 0.01, 10000.0);

        (proj.as_matrix() * view.to_homogeneous()).data.0
    }
}
