use std::f32::consts::PI;

use nalgebra::{matrix, Matrix4, Rotation3, vector, Vector3};

pub fn gradient(surface: impl Fn(Vector3<f32>) -> f32, p: Vector3<f32>) -> Vector3<f32> {
    let h = 0.0001;

    let dx = (surface(vector![p.x + h, p.y, p.z]) - surface(p)) / h;
    let dy = (surface(vector![p.x, p.y + h, p.z]) - surface(p)) / h;
    let dz = (surface(vector![p.x, p.y, p.z + h]) - surface(p)) / h;

    return vector![dx, dy, dz];
}

pub fn on_surface(surface: impl Fn(Vector3<f32>) -> f32, point: Vector3<f32>) -> bool {
    surface(point).abs() <= f32::EPSILON
}

pub fn quadratic_surface(coefficients: Matrix4<f32>) -> impl Fn(Vector3<f32>) -> f32 {
    move |p| {
        let mp = matrix![p.x, p.y, p.z, 1.0];

        return (mp * coefficients * mp.transpose()).x;
    }
}

pub fn ellipsoid(a: f32, b: f32, c: f32) -> impl Fn(Vector3<f32>) -> f32 {
    move |p| {
        (p.x.powf(2.0) / a.powf(2.0))
            + (p.y.powf(2.0) / b.powf(2.0))
            + (p.z.powf(2.0) / c.powf(2.0))
            - 1.0
    }
}

pub fn sphere(r: f32) -> impl Fn(Vector3<f32>) -> f32 {
    return ellipsoid(r, r, r);
}

pub fn rotate(
    rotation: Vector3<f32>,
    surface: impl Fn(Vector3<f32>) -> f32,
) -> impl Fn(Vector3<f32>) -> f32 {
    let transform = Rotation3::new(rotation * (PI / 180.0));

    move |p| surface(transform.transform_vector(&p))
}

pub fn translate(
    origin: Vector3<f32>,
    surface: impl Fn(Vector3<f32>) -> f32,
) -> impl Fn(Vector3<f32>) -> f32 {
    move |p| surface(p - origin)
}

pub fn union(
    surface1: impl Fn(Vector3<f32>) -> f32,
    surface2: impl Fn(Vector3<f32>) -> f32,
) -> impl Fn(Vector3<f32>) -> f32 {
    move |p| surface1(p).min(surface2(p))
}

pub fn smooth_subtract(
    surface1: impl Fn(Vector3<f32>) -> f32,
    surface2: impl Fn(Vector3<f32>) -> f32,
    k: f32,
) -> impl Fn(Vector3<f32>) -> f32 {
    move |p| {
        let d1 = surface1(p);
        let d2 = surface2(p);

        let h = (k - (d1 - d2).abs()).max(0.0);

        d1.max(-d2) + (h * h * 0.25 / k)
    }
}

pub fn smooth_union(
    surface1: impl Fn(Vector3<f32>) -> f32,
    surface2: impl Fn(Vector3<f32>) -> f32,
    k: f32,
) -> impl Fn(Vector3<f32>) -> f32 {
    move |p| {
        let d1 = surface1(p);
        let d2 = surface2(p);

        let h = (k - (d1 - d2).abs()).max(0.0);

        d1.min(d2) - (h * h * 0.25 / k)
    }
}
