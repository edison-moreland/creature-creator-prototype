use std::f64::consts::PI;

use nalgebra::{point, Point3};

use crate::geometry::Plane;
use crate::spatial_indexer::kd_indexer::KdContainer;
use crate::surfaces::{gradient, on_surface, seed, Surface};

// Use a technique similar to Delauany triangles to get a fast initial sampling of the entire surface
// Citation:
// Floriant Levet, Xavier Granier, Christophe Schlick. Fast sampling of implicit surfaces by particle systems.
// SMI ’06: Proceedings of the IEEE International Conference on Shape Modeling and Applications
// 2006, Jun 2006, Matsushima, Japan. pp.39, 10.1109/SMI.2006.13 . inria-00106853v1
pub fn sample(surface: &Surface, repulsion_radius: f32) -> Vec<Point3<f32>> {
    let seed = seed(surface);

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

fn sibling_points(
    surface: &Surface,
    parent: Point3<f32>,
    repulsion_radius: f32,
) -> Vec<Point3<f32>> {
    let normal = gradient(surface, parent).normalize();
    let tangent_plane = Plane::from_origin_normal(parent, normal);

    let mut siblings = Vec::new();
    siblings.reserve(6);

    for i in 0..6 {
        let ipi3 = (i as f64 * PI) / 3.0;

        let point_guess = tangent_plane.from(point![
            ipi3.cos() as f32 * (repulsion_radius * 2.0),
            ipi3.sin() as f32 * (repulsion_radius * 2.0),
        ]);

        siblings.push(refine_point(surface, repulsion_radius, parent, point_guess))
    }

    siblings
}

fn refine_point(
    surface: &Surface,
    radius: f32,
    parent: Point3<f32>,
    guess: Point3<f32>,
) -> Point3<f32> {
    let mut point = guess;

    for _ in 0..10 {
        let grad = gradient(surface, point);
        point -= grad.scale(surface.sample(point) / grad.dot(&grad));

        // Push point away from parent
        // The original paper did some fancy shit to rotate about the parent
        let mut away = point - parent;
        if away.magnitude() < (radius * 2.0) {
            away = away.scale((radius * 2.0) - away.magnitude());
            point += away;
        }

        if on_surface(surface, point) {
            break;
        }
    }

    point
}
