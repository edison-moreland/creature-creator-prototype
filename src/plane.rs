use std::f32::consts::PI;

use nalgebra::{vector, Vector2, Vector3};

pub struct Plane {
    o: Vector3<f32>,
    u: Vector3<f32>,
    v: Vector3<f32>,
}

impl Plane {
    pub fn from_origin_normal(o: Vector3<f32>, n: Vector3<f32>) -> Self {
        // TODO: This replacement might work differently
        let mut cardinal = vector![0.0, 0.0, 0.0];
        cardinal[n.imin()] = 1.0;

        let u = n.cross(&cardinal).normalize();
        let v = u.cross(&n).normalize();

        Plane { o, u, v }
    }

    pub fn from(&self, p: Vector2<f32>) -> Vector3<f32> {
        self.o + (self.u * p.x) + (self.v * p.y)
    }

    pub fn circle_points(&self, segments: usize, radius: f32) -> Vec<Vector3<f32>> {
        let segment_theta = (2.0 * PI) / (segments as f32);

        (0..segments)
            .map(|i| {
                let angle = segment_theta * (i as f32);

                self.from(vector![angle.cos(), angle.sin()] * radius)
            })
            .collect()
    }
}
