use nalgebra::{Matrix4, point, Point3, Transform3, vector, Vector3};

pub use pipeline::SurfacePipeline;

use crate::renderer::surfaces::primitives::ellipsoid;

mod pipeline;
mod primitives;
mod sampling;

pub enum Shape {
    Ellipsoid(Vector3<f32>),
}

pub struct Surface {
    shapes: Vec<(Matrix4<f32>, Shape)>,
}

impl Surface {
    pub fn new() -> Self {
        Self { shapes: vec![] }
    }

    pub fn push(&mut self, transform: Transform3<f32>, shape: Shape) {
        self.shapes.push((transform.to_homogeneous(), shape))
    }

    fn sample(&self, at: Point3<f32>) -> f32 {
        self.shapes
            .iter()
            .map(|(t, s)| {
                let tat = t.transform_point(&at);

                match s {
                    Shape::Ellipsoid(p) => ellipsoid(*p)(tat),
                }
            })
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .expect("no nans")
    }
}

pub fn seed(surface: &Surface) -> Point3<f32> {
    // TODO: This method is brittle and panics often (dividing by zero?)
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
        panic!("uh oh!")
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
    surface.sample(point).abs() <= f32::EPSILON
}
