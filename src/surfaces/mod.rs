pub mod body;
pub mod primitives;

use nalgebra::{vector, Vector3};

pub trait Surface {
    fn at(&self, t: f32, p: Vector3<f32>) -> f32;

    // a single point somewhere on the surface at t=0.0
    fn sample_point(&self) -> Vector3<f32>;
}

pub struct SurfaceFn<F>(Vector3<f32>, F);

impl<F> SurfaceFn<F>
where
    F: Fn(f32, Vector3<f32>) -> f32,
{
    pub(crate) fn new(p: Vector3<f32>, f: F) -> Self {
        Self(p, f)
    }
}

impl<F> Surface for SurfaceFn<F>
where
    F: Fn(f32, Vector3<f32>) -> f32,
{
    fn at(&self, t: f32, p: Vector3<f32>) -> f32 {
        (self.1)(t, p)
    }

    fn sample_point(&self) -> Vector3<f32> {
        self.0
    }
}

pub fn gradient(surface: &impl Surface, t: f32, p: Vector3<f32>) -> Vector3<f32> {
    let h = 0.0001;

    let sp = surface.at(t, p);

    // TODO:
    let dx = (surface.at(t, vector![p.x + h, p.y, p.z]) - sp) / h;
    let dy = (surface.at(t, vector![p.x, p.y + h, p.z]) - sp) / h;
    let dz = (surface.at(t, vector![p.x, p.y, p.z + h]) - sp) / h;

    vector![dx, dy, dz]
}

pub fn on_surface(surface: &impl Surface, t: f32, point: Vector3<f32>) -> bool {
    surface.at(t, point).abs() <= f32::EPSILON
}
