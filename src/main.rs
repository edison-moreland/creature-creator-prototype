mod relaxation;
mod sampling;
mod spatial_indexer;
mod surfaces;

use crate::relaxation::RelaxationSystem;
use crate::sampling::sample;
use crate::surfaces::{ellipsoid, gradient, smooth_union, sphere, translate, union};
use raylib::prelude::*;

fn surface_at(t: f32) -> impl Fn(Vector3) -> f32 {
    smooth_union(
        sphere(10.0),
        union(
            translate(rvec3((t).sin() * 10.0, 0.0, 0.0), ellipsoid(10.0, 5.0, 5.0)),
            translate(rvec3(0.0, 0.0, (t).cos() * 10.0), ellipsoid(5.0, 5.0, 10.0)),
        ),
        0.5,
    )
}

fn main() {
    let (mut rl, thread) = raylib::init().size(640, 480).title("Hello, World").build();

    let camera = Camera3D::perspective(
        Vector3::new(25.0, 25.0, 25.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::up(),
        40.0,
    );

    let seed = rvec3(0.0, 10.0, 0.0);
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
        // let surface = surface_at(0.0);

        {
            let mut d3d = d.begin_mode3D(camera);
            for (point, _, radius) in particles.positions() {
                // let energy_color = rvec3(1.0, 0, 0).lerp(rvec3(0, 0, 1.0), energy / (6.0 * 0.8));
                //
                // let point_color = Color::color_from_normalized(Vector4::new(
                //     energy_color.x,
                //     energy_color.y,
                //     energy_color.z,
                //     1.0,
                // ));

                let normal = gradient(&surface, point).normalized();

                let point_color =
                    Color::color_from_normalized(Vector4::new(normal.x, normal.y, normal.z, 1.0));

                // let point_color = if velocity.length() >= (8.0 * radius) {
                //     Color::BLUE
                // } else {
                //     Color::RED
                // };

                d3d.draw_sphere(point, radius * 2.0, point_color)
            }
        }

        particles.update(sample_radius / 1.5, &surface);

        d.draw_fps(0, 0);
    }
}
