use std::f32::consts::PI;

use nalgebra::{Isometry3, Perspective3, Point3, Vector3};

// Uniforms are shared between all render passes

#[repr(C)]
pub struct Uniforms {
    camera: [[f32; 4]; 4],
    camera_position: [f32; 3],
}

impl Uniforms {
    pub fn new(camera: &Camera) -> Self {
        Self {
            camera: camera.mvp_matrix(),
            camera_position: camera.eye.coords.data.0[0],
        }
    }

    pub fn camera_updated(&mut self, camera: &Camera) {
        self.camera = camera.mvp_matrix();
        self.camera_position = camera.eye.coords.data.0[0];
    }
}

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

    fn mvp_matrix(&self) -> [[f32; 4]; 4] {
        let view = Isometry3::look_at_rh(&self.eye, &self.target, &Vector3::y());

        let proj = Perspective3::new(self.aspect_ratio, self.fov * (180.0 / PI), 0.01, 10000.0);

        (proj.as_matrix() * view.to_homogeneous()).data.0
    }
}
