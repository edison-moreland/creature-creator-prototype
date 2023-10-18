mod relaxation;
mod sampling;
mod spatial_indexer;
mod surfaces;

use crate::relaxation::RelaxationSystem;
use crate::sampling::sample;
use crate::surfaces::{ellipsoid, gradient, smooth_union, sphere, translate, union};
use raylib::prelude::*;

use nalgebra::{vector, Vector3};

fn surface_at(t: f32) -> impl Fn(Vector3<f32>) -> f32 {
    smooth_union(
        sphere(10.0),
        union(
            translate(
                vector![(t).sin() * 10.0, 0.0, 0.0],
                ellipsoid(10.0, 5.0, 5.0),
            ),
            translate(
                vector![0.0, 0.0, (t).cos() * 10.0],
                ellipsoid(5.0, 5.0, 10.0),
            ),
        ),
        0.5,
    )
}

fn main() {
    let (mut rl, thread) = raylib::init().size(640, 480).title("Hello, World").build();

    let camera = Camera3D::perspective(
        rvec3(25.0, 25.0, 25.0),
        rvec3(0.0, 0.0, 0.0),
        math::Vector3::up(),
        40.0,
    );

    let seed = vector![0.0, 10.0, 0.0];
    let sample_radius = 0.5;

    let mut t = 0.0;
    let surface = surface_at(t);
    let points = sample(surface, seed, sample_radius);

    let mut particles = RelaxationSystem::new(points, sample_radius);

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);

        t += 0.03;
        let surface = surface_at(t);

        {
            let mut d3d = d.begin_mode3D(camera);
            for (point, radius) in particles.positions() {
                let normal = gradient(&surface, point).normalize();

                let point_color = Color::color_from_normalized(Vector4::new(
                    normal.x.abs(),
                    normal.y.abs(),
                    normal.z.abs(),
                    1.0,
                ));

                let true_position = point - normal.scale(radius * 2.0);

                d3d.draw_sphere(
                    rvec3(true_position.x, true_position.y, true_position.z),
                    radius * 2.0,
                    point_color,
                )
            }
        }

        particles.update(sample_radius, &surface);

        d.draw_fps(0, 0);
    }
}
