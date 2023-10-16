use crate::spatial_indexer::kd_indexer::KdContainer;
use crate::surfaces::{gradient, on_surface};
use raylib::prelude::*;

pub struct Plane {
    o: Vector3,
    u: Vector3,
    v: Vector3,
}

impl Plane {
    pub fn from_origin_normal(o: Vector3, n: Vector3) -> Self {
        let u = n.perpendicular().normalized();
        let v = u.cross(n).normalized();

        Plane { o, u, v }
    }

    pub fn from(&self, p: Vector2) -> Vector3 {
        self.o + (self.u * p.x) + (self.v * p.y)
    }

    pub fn to(&self, p: Vector3) -> Vector2 {
        let op = p - self.o;

        rvec2(self.u.dot(op), self.v.dot(op))
    }
}

fn refine_point(
    surface: impl Fn(Vector3) -> f32,
    radius: f32,
    parent: Vector3,
    guess: Vector3,
) -> Vector3 {
    let mut point = guess;

    for _ in 0..10 {
        let grad = gradient(&surface, point);
        point -= grad.scale_by(surface(point) / grad.dot(grad));

        // Push point away from parent
        // The original paper did some fancy shit to rotate about the parent
        let mut away = point - parent;
        if away.length() < (radius * 2.0) {
            away.scale((radius * 2.0) - away.length());
            point += away;
        }

        if on_surface(&surface, point) {
            break;
        }
    }

    point
}

fn sibling_points(
    surface: impl Fn(Vector3) -> f32,
    parent: Vector3,
    repulsion_radius: f32,
) -> Vec<Vector3> {
    let normal = gradient(&surface, parent).normalized();
    let tangent_plane = Plane::from_origin_normal(parent, normal);

    let mut siblings = Vec::new();
    siblings.reserve(6);

    for i in 0..6 {
        let ipi3 = (i as f64 * PI) / 3.0;

        let point_guess = tangent_plane.from(rvec2(
            ipi3.cos() as f32 * (repulsion_radius * 2.0),
            ipi3.sin() as f32 * (repulsion_radius * 2.0),
        ));

        siblings.push(refine_point(
            &surface,
            repulsion_radius,
            parent,
            point_guess,
        ))
    }

    siblings
}

pub fn sample(
    surface: impl Fn(Vector3) -> f32,
    seed: Vector3,
    repulsion_radius: f32,
) -> Vec<Vector3> {
    if !on_surface(&surface, seed) {
        println!("surface({:?}) == {:?}", seed, surface(seed));
        panic!("Seed is not on the surface")
    }

    let initial_siblings = sibling_points(&surface, seed, repulsion_radius);

    let mut samples = KdContainer::new();
    samples.append(initial_siblings.clone());

    let mut untreated = initial_siblings;

    while !untreated.is_empty() {
        let next_seed = untreated.pop().unwrap();

        for point in sibling_points(&surface, next_seed, repulsion_radius) {
            if samples.any_items_in_radius(point, repulsion_radius * 1.9) {
                continue;
            }

            samples.push(point);
            untreated.push(point);
        }
    }

    samples.items
}
