use nalgebra::Vector3;

#[derive(Copy, Clone)]
pub enum Shape {
    Ellipsoid(Vector3<f32>),
    Sphere(f32),
    Cyliner(f32, f32),
}
