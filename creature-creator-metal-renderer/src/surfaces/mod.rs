use creature_creator_renderer::shapes::Shape;
use nalgebra::{point, vector, Matrix4, Point3, Vector3};

use crate::surfaces::primitives::{cylinder, ellipsoid, sphere};
pub use pipeline::SurfacePipeline;

mod pipeline;
mod primitives;
mod sampling;

pub struct Surface {
    shapes: Vec<(Matrix4<f32>, Shape)>,
}

impl Surface {
    pub fn new() -> Self {
        Self { shapes: vec![] }
    }

    pub fn push(&mut self, transform: Matrix4<f32>, shape: Shape) {
        self.shapes.push((transform, shape))
    }

    pub(crate) fn empty(&self) -> bool {
        self.shapes.is_empty()
    }

    fn eval_shape(&self, index: usize, at: Point3<f32>) -> f32 {
        let (t, s) = self.shapes[index];

        let tat = t.transform_point(&at);

        match s {
            Shape::Ellipsoid(p) => ellipsoid(p)(tat),
            Shape::Sphere(r) => sphere(r)(tat),
            Shape::Cyliner(r, h) => cylinder(r, h)(tat),
        }
    }

    fn sample(&self, at: Point3<f32>) -> f32 {
        match self.shapes.len() {
            0 => {
                panic!("No shapes! Nothing to sample.")
            }
            1 => self.eval_shape(0, at),
            2 => smooth_min(self.eval_shape(0, at), self.eval_shape(1, at), 0.5),
            _ => {
                let mut min_1 = f32::MAX;
                let mut min_2 = f32::MAX;

                for i in 0..self.shapes.len() {
                    let t = self.eval_shape(i, at);

                    if t < min_1 {
                        min_2 = min_1;
                        min_1 = t;
                    }
                }

                smooth_min(min_1, min_2, 0.5)
            }
        }
    }
}

fn smooth_min(a: f32, b: f32, k: f32) -> f32 {
    let h = (k - (a - b).abs()).max(0.0);

    a.min(b) - (h * h * 0.25 / k)
}

pub fn seed(surface: &Surface) -> Point3<f32> {
    let mut seed_point = point![rand::random(), rand::random(), rand::random()];

    for _ in 0..100 {
        let grad = gradient(surface, seed_point);

        let gdg = grad.dot(&grad);
        if gdg.is_nan() {
            panic!("NANANANANANA")
        }

        seed_point -= grad.scale(surface.sample(seed_point) / gdg);

        if on_surface(surface, seed_point) {
            return seed_point;
        }
    }

    if !on_surface(surface, seed_point) {
        dbg!(seed_point);
        panic!("could not find a seed point")
    }

    seed_point
}

pub fn gradient(surface: &Surface, p: Point3<f32>) -> Vector3<f32> {
    let h = 0.0001;

    let sp = surface.sample(p);

    let dx = (surface.sample(point![p.x + h, p.y, p.z]) - sp) / h;
    let dy = (surface.sample(point![p.x, p.y + h, p.z]) - sp) / h;
    let dz = (surface.sample(point![p.x, p.y, p.z + h]) - sp) / h;

    vector![dx, dy, dz]
}

pub fn on_surface(surface: &Surface, point: Point3<f32>) -> bool {
    surface.sample(point).abs() <= f32::EPSILON * 2.0
}
