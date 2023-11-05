use crate::renderer::widgets::strokes::StrokeSet;
use crate::renderer::widgets::Widget;
use nalgebra::{vector, Vector3};
use rand::random;

pub mod primitives;
pub mod relaxation;
pub mod sampling;

pub trait Surface {
    fn at(&self, t: f32, p: Vector3<f32>) -> f32;

    // a single point somewhere on the surface at t=0.0
    fn sample_point(&self) -> Option<Vector3<f32>> {
        None
    }
}

pub struct SurfaceFn<F>(F);

impl<F> SurfaceFn<F>
where
    F: Fn(f32, Vector3<f32>) -> f32,
{
    pub(crate) fn new(f: F) -> Self {
        Self(f)
    }
}

impl<F> Surface for SurfaceFn<F>
where
    F: Fn(f32, Vector3<f32>) -> f32,
{
    fn at(&self, t: f32, p: Vector3<f32>) -> f32 {
        (self.0)(t, p)
    }
}

impl<S> Widget for SurfaceFn<S> {
    fn strokes(&self) -> Option<&StrokeSet> {
        None
    }
}

pub fn seed(surface: &impl Surface, t: f32) -> Vector3<f32> {
    // TODO: This method is brittle and panics often (dividing by zero2)
    let mut seed_point = vector![random(), random(), random()];

    for _ in 0..100 {
        let grad = gradient(surface, t, seed_point);

        let gdg = grad.dot(&grad);
        if gdg.is_nan() {
            panic!("NANANANANANA")
        }

        seed_point -= grad.scale(surface.at(t, seed_point) / gdg);

        if on_surface(surface, t, seed_point) {
            return seed_point;
        }
    }

    if !on_surface(surface, t, seed_point) {
        dbg!(seed_point);
        panic!("uh oh!")
    }

    seed_point
}

pub fn gradient(surface: &impl Surface, t: f32, p: Vector3<f32>) -> Vector3<f32> {
    let h = 0.0001;

    let sp = surface.at(t, p);

    let dx = (surface.at(t, vector![p.x + h, p.y, p.z]) - sp) / h;
    let dy = (surface.at(t, vector![p.x, p.y + h, p.z]) - sp) / h;
    let dz = (surface.at(t, vector![p.x, p.y, p.z + h]) - sp) / h;

    vector![dx, dy, dz]
}

pub fn on_surface(surface: &impl Surface, t: f32, point: Vector3<f32>) -> bool {
    surface.at(t, point).abs() <= f32::EPSILON
}
