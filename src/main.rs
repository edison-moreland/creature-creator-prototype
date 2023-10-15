mod kdtree;
mod particles;
mod sampling;
mod surfaces;

use crate::particles::ParticleStore;
use crate::sampling::sample;
use crate::surfaces::{ellipsoid, gradient, smooth_union, sphere, translate, union};
use raylib::prelude::*;
use std::time::Instant;

const RELAXATION_REPULSION_AMPLITUDE: f32 = 6.0;
const RELAXATION_SURFACE_FEEDBACK: f32 = 15.0;
const RELAXATION_T_STEP: f32 = 0.03;
const RELAXATION_ITERATIONS: i32 = 10;

// energy_contribution returns the energy of i due to j
fn energy_contribution(i_repulsion_radius: f32, i: Vector3, j: Vector3) -> f32 {
    RELAXATION_REPULSION_AMPLITUDE
        * ((i - j).length().powf(2.0) / (2.0 * i_repulsion_radius).powf(2.0))
}

fn velocity(particle: Vector3, neighbours: Vec<&Vector3>, radius: f32) -> Vector3 {
    neighbours
        .iter()
        .copied()
        .fold(rvec3(0, 0, 0), |dv, other_particle| {
            let rij = particle - *other_particle;

            let ra2 = (2.0 * radius).powf(2.0);

            let energy = RELAXATION_REPULSION_AMPLITUDE * ((-rij.length().powf(2.0) / ra2).exp());

            dv + rij.scale_by(energy)
        })
}

fn constrain_velocity(
    surface: impl Fn(Vector3) -> f32,
    position: Vector3,
    velocity: Vector3,
) -> Vector3 {
    let grad = gradient(&surface, position);
    velocity
        - grad.scale_by(
            (grad.dot(velocity) + (RELAXATION_SURFACE_FEEDBACK * surface(position)))
                / (grad.dot(grad)),
        )
}

#[derive(Copy, Clone)]
struct RelaxationAttributes {
    velocity: Vector3,
    radius: f32,
}

fn relax(
    particles: &mut ParticleStore<RelaxationAttributes>,
    surface: impl Fn(Vector3) -> f32 + Sync,
) -> Vec<Vector3> {
    for j in 0..RELAXATION_ITERATIONS {
        let start = Instant::now();

        // Calculate desired velocity to spread particles evenly on the surface
        // Neighbour radius is a guess based on when energy goes to 0
        particles.update_attributes(2.0, |particle, position, neighbours| {
            particle.velocity = velocity(position, neighbours, particle.radius);
        });

        // Constrain the velocity to the surface and add to the position
        particles.update_particles(|p, a| {
            p + constrain_velocity(&surface, p, a.velocity).scale_by(RELAXATION_T_STEP)
        });

        println!("Pass {:?}, {:?}", j, start.elapsed());
    }

    particles.positions()
}

fn surface_at(t: f32) -> impl Fn(Vector3) -> f32 {
    smooth_union(
        sphere(10.0),
        union(
            translate(
                rvec3((t / 10.0).sin() * 10.0, 0.0, 0.0),
                ellipsoid(10.0, 5.0, 5.0),
            ),
            translate(
                rvec3(0.0, 0.0, (t / 10.0).cos() * 10.0),
                ellipsoid(5.0, 5.0, 10.0),
            ),
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
    let sample_radius = 0.1;

    let surface = surface_at(0.0);
    let points = sample(surface, seed, sample_radius);

    let mut particles = ParticleStore::new(points, |_| RelaxationAttributes {
        velocity: rvec3(0, 0, 0),
        radius: sample_radius,
    });

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);
        let surface = surface_at(d.get_time() as f32);

        {
            let mut d3d = d.begin_mode3D(camera);
            for point in particles.positions() {
                let normal = gradient(&surface, point).normalized();

                let point_color =
                    Color::color_from_normalized(Vector4::new(normal.x, normal.y, normal.z, 1.0));
                d3d.draw_sphere(point, sample_radius, point_color)
            }
        }

        println!("Relaxing points...");
        let start = Instant::now();
        relax(&mut particles, &surface);
        println!("Done! {:?} elapsed", start.elapsed());
    }
}
