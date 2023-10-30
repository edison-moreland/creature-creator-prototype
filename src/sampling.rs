use std::f64::consts::PI;

use crate::plane::Plane;
use nalgebra::{vector, Vector3};

use crate::spatial_indexer::kd_indexer::KdContainer;
use crate::surfaces::{gradient, on_surface, Surface};

fn refine_point(
    surface: &impl Surface,
    radius: f32,
    parent: Vector3<f32>,
    guess: Vector3<f32>,
) -> Vector3<f32> {
    let mut point = guess;

    for _ in 0..10 {
        let grad = gradient(surface, 0.0, point);
        point -= grad.scale(surface.at(0.0, point) / grad.dot(&grad));

        // Push point away from parent
        // The original paper did some fancy shit to rotate about the parent
        let mut away = point - parent;
        if away.magnitude() < (radius * 2.0) {
            away = away.scale((radius * 2.0) - away.magnitude());
            point += away;
        }

        if on_surface(surface, 0.0, point) {
            break;
        }
    }

    point
}

fn sibling_points(
    surface: &impl Surface,
    parent: Vector3<f32>,
    repulsion_radius: f32,
) -> Vec<Vector3<f32>> {
    let normal = gradient(surface, 0.0, parent).normalize();
    let tangent_plane = Plane::from_origin_normal(parent, normal);

    let mut siblings = Vec::new();
    siblings.reserve(6);

    for i in 0..6 {
        let ipi3 = (i as f64 * PI) / 3.0;

        let point_guess = tangent_plane.from(vector![
            ipi3.cos() as f32 * (repulsion_radius * 2.0),
            ipi3.sin() as f32 * (repulsion_radius * 2.0),
        ]);

        siblings.push(refine_point(surface, repulsion_radius, parent, point_guess))
    }

    siblings
}

pub fn sample(
    surface: &impl Surface,
    seed: Vector3<f32>,
    repulsion_radius: f32,
) -> Vec<Vector3<f32>> {
    if !on_surface(surface, 0.0, seed) {
        println!("surface({:?}) == {:?}", seed, surface.at(0.0, seed));
        panic!("Seed is not on the surface")
    }

    let initial_siblings = sibling_points(surface, seed, repulsion_radius);

    let mut samples = KdContainer::new();
    samples.append(initial_siblings.clone());

    let mut untreated = initial_siblings;

    while let Some(next_seed) = untreated.pop() {
        for point in sibling_points(surface, next_seed, repulsion_radius) {
            if samples.any_items_in_radius(point, repulsion_radius * 1.9) {
                continue;
            }

            samples.push(point);
            untreated.push(point);
        }
    }

    samples.items
}
