use std::mem;
use std::ops::Neg;

use nalgebra::{vector, Point3, Vector3};

use crate::renderer::shared::new_zeroed_box;
use crate::renderer::surfaces::sampling::buffer_allocator::{
    BufferAllocator, StackBufferAllocator,
};
use crate::renderer::surfaces::sampling::initial_sampling::sample;
use crate::renderer::surfaces::{gradient, Surface};
use crate::spatial_indexer::kd_indexer::KdIndexer;
use crate::spatial_indexer::{Positioned, SpatialIndexer};

// Using Particles to Sample and Control Implicit Surfaces.
// Andrew P. Witkin, Paul S. Heckbert
// https://dl.acm.org/doi/pdf/10.1145/192161.192227

const REPULSION_AMPLITUDE: f32 = 6.0;
const FEEDBACK: f32 = 15.0;
const NEIGHBOUR_RADIUS: f32 = 3.0;
const UPDATE_ITERATIONS: usize = 4;
const ITERATION_T_STEP: f32 = 0.03;
const EQUILIBRIUM_SPEED: f32 = 100.0;
const FISSION_COEFFICIENT: f32 = 0.2;
const DEATH_COEFFICIENT: f32 = 0.7;
const MAX_RADIUS_COEFFICIENT: f32 = 1.2;
const DESIRED_REPULSION_ENERGY: f32 = REPULSION_AMPLITUDE * 0.8;
pub const MAX_PARTICLE_COUNT: usize = 100000;

fn random_velocity() -> Vector3<f32> {
    Vector3::new(rand::random(), rand::random(), rand::random()).normalize()
}

// energy_contribution returns the energy of i due to j
fn energy_contribution(i_repulsion_radius: f32, i: Point3<f32>, j: Point3<f32>) -> f32 {
    REPULSION_AMPLITUDE
        * ((i - j).magnitude().powf(2.0) / (2.0 * i_repulsion_radius).powf(2.0))
            .neg()
            .exp()
}

fn constrain_to_surface(
    surface: &Surface,
    position: Point3<f32>,
    normal: Vector3<f32>,
    velocity: Vector3<f32>,
) -> Vector3<f32> {
    velocity
        - normal.scale(
            (normal.dot(&velocity) + (FEEDBACK * surface.sample(position))) / (normal.dot(&normal)),
        )
}

fn should_die(radius: f32, desired_radius: f32) -> bool {
    // Assuming particle is at equilibrium
    let death_radius = desired_radius * DEATH_COEFFICIENT;
    radius < death_radius && rand::random::<f32>() > radius / death_radius
}

fn should_fission_radius(radius: f32, desired_radius: f32) -> bool {
    let fission_radius = desired_radius * MAX_RADIUS_COEFFICIENT;
    radius > fission_radius
}

fn should_fission_energy(radius: f32, energy: f32, desired_radius: f32) -> bool {
    let fission_energy = DESIRED_REPULSION_ENERGY * FISSION_COEFFICIENT;
    energy > fission_energy && radius > desired_radius
}

#[derive(Copy, Clone, Debug)]
struct Particle {
    position: Point3<f32>,
    velocity: Vector3<f32>,
    normal: Vector3<f32>,
    radius: f32,
}

impl Positioned for Particle {
    fn position(&self) -> Point3<f32> {
        self.position
    }
}

pub struct SamplingSystem {
    living_particles: Vec<usize>,
    position_index: KdIndexer,
    index_allocator: StackBufferAllocator<MAX_PARTICLE_COUNT>,

    // These are boxed so we don't blow out the stack
    particles_a: Box<[Particle; MAX_PARTICLE_COUNT]>,
    particles_b: Box<[Particle; MAX_PARTICLE_COUNT]>,

    pub t: f32,
}

impl SamplingSystem {
    pub fn new() -> Self {
        // let particles = [Particle {
        //     position: point![0.0, 0.0, 0.0],
        //     velocity: vector![0.0, 0.0, 0.0],
        //     normal: vector![0.0, 0.0, 0.0],
        //     radius: 0.0,
        // }; MAX_PARTICLE_COUNT];
        //
        SamplingSystem {
            living_particles: vec![],
            position_index: KdIndexer::new(),
            index_allocator: StackBufferAllocator::new(),

            particles_a: new_zeroed_box(),
            particles_b: new_zeroed_box(),

            t: 0.0,
        }
    }

    fn initial_sampling(&mut self, desired_radius: f32, surface: &Surface) {
        // The initial sampling isn't taken until the particle system is stepped for the first time
        // This way we don't need to know what surface we're sampling when the particle system is allocated
        println!("Initial sampling...");
        let positions = sample(surface, desired_radius * 2.0);
        if positions.len() > MAX_PARTICLE_COUNT {
            panic!("TOO DANG BIG!!")
        }

        for p in positions {
            let normal = gradient(surface, p).normalize();

            let i = self.index_allocator.insert();
            self.living_particles.push(i);

            self.particles_a[i].position = p;
            self.particles_a[i].normal = normal;
            self.particles_a[i].radius = desired_radius;
            self.particles_b[i].position = p;
            self.particles_b[i].normal = normal;
            self.particles_b[i].radius = desired_radius;
        }

        let mut position_index = KdIndexer::new();
        position_index.reindex(self.particles_a.as_slice(), self.living_particles.clone());

        println!("Done!")
    }

    pub fn positions(
        &self,
    ) -> impl Iterator<Item = (Point3<f32>, Vector3<f32>, f32)> + ExactSizeIterator + '_ {
        self.living_particles.iter().map(|i| {
            let particle = self.particles_a[*i];
            (particle.position, particle.normal, particle.radius)
        })
    }

    pub fn update(&mut self, desired_radius: f32, surface: &Surface) {
        if self.t == 0.0 && self.living_particles.is_empty() {
            self.initial_sampling(desired_radius, surface)
        }

        for _ in 0..UPDATE_ITERATIONS {
            for j in (0..self.living_particles.len()).rev() {
                let i = self.living_particles[j];
                let particle = self.particles_a[i];

                let neighbour_indices = self.position_index.get_indices_within(
                    self.particles_a.as_slice(),
                    particle.position,
                    NEIGHBOUR_RADIUS * particle.radius,
                );

                let neighbours: Vec<(usize, f32, f32)> = neighbour_indices
                    .iter()
                    .filter(|j| **j != i)
                    .map(|j| {
                        assert!(*j < MAX_PARTICLE_COUNT);

                        let pj = self.particles_a[*j];

                        (
                            *j,
                            energy_contribution(particle.radius, particle.position, pj.position),
                            energy_contribution(pj.radius, pj.position, particle.position),
                        )
                    })
                    .collect();
                let energy = self.repulsion_energy(&neighbours);

                if particle.velocity.magnitude() < (EQUILIBRIUM_SPEED * particle.radius) {
                    if should_die(particle.radius, desired_radius) {
                        self.living_particles.remove(j);
                        self.index_allocator.remove(i);
                        continue;
                    }

                    if should_fission_energy(particle.radius, energy, desired_radius)
                        || should_fission_radius(particle.radius, desired_radius)
                    {
                        let position = particle.position();
                        let radius = particle.radius;

                        let new_radius = radius / (2.0_f32).sqrt();
                        let new_velocity = random_velocity().scale(radius);

                        let new_position = Point3::from(position + new_velocity);
                        self.particles_b[i] = Particle {
                            position: new_position,
                            velocity: vector![0.0, 0.0, 0.0],
                            normal: gradient(surface, new_position).normalize(),
                            radius: new_radius,
                        };

                        let sibling_position = Point3::from(position - new_velocity);
                        let sibling = Particle {
                            position: sibling_position,
                            velocity: vector![0.0, 0.0, 0.0],
                            normal: gradient(surface, sibling_position).normalize(),
                            radius: new_radius,
                        };
                        let sibling_i = self.index_allocator.insert();
                        self.particles_b[sibling_i] = sibling;
                        self.living_particles.push(sibling_i);
                        continue;
                    }
                }

                let velocity = constrain_to_surface(
                    surface,
                    particle.position,
                    particle.normal,
                    self.particle_velocity(particle.position, particle.radius, &neighbours),
                );

                let position = particle.position + velocity.scale(ITERATION_T_STEP);

                let normal = gradient(surface, position).normalize();

                let radius = self.particle_radius(position, particle.radius, energy, &neighbours);

                self.particles_b[i] = Particle {
                    position,
                    velocity,
                    normal,
                    radius,
                }
            }

            mem::swap(&mut self.particles_a, &mut self.particles_b);

            self.position_index
                .reindex(self.particles_a.as_slice(), self.living_particles.clone());
        }

        self.t += ITERATION_T_STEP
    }

    fn repulsion_energy(&self, neighbours: &[(usize, f32, f32)]) -> f32 {
        neighbours.iter().map(|(_, energy, _)| energy).sum()
    }

    fn particle_radius(
        &self,
        position: Point3<f32>,
        radius: f32,
        repulsion_energy: f32,
        neighbours: &[(usize, f32, f32)],
    ) -> f32 {
        // desired change in energy
        let re_delta = -(FEEDBACK * (repulsion_energy - DESIRED_REPULSION_ENERGY));

        // change in energy with respect to change in radius
        let di_ai = (1.0 / radius.powf(3.0))
            * neighbours
                .iter()
                .map(|(j, energy_cont, _)| {
                    let dist = (position - self.particles_a[*j].position())
                        .magnitude()
                        .powf(2.0);

                    dist * energy_cont
                })
                .sum::<f32>();

        // Radius change to bring us to desired energy
        let radius_delta = re_delta / (di_ai + 10.0);

        radius + (radius_delta * ITERATION_T_STEP)
    }

    fn particle_velocity(
        &self,
        position: Point3<f32>,
        radius: f32,
        neighbours: &[(usize, f32, f32)],
    ) -> Vector3<f32> {
        neighbours
            .iter()
            .fold(
                vector![0.0, 0.0, 0.0],
                |dv, (i, energy_cont, rev_energy_cont)| {
                    let rij = position - self.particles_a[*i].position();

                    let rei = (rij / radius.powf(2.0)).scale(*energy_cont);

                    let rej = (rij / self.particles_a[*i].radius.powf(2.0)).scale(*rev_energy_cont);

                    dv + (rei + rej)
                },
            )
            .scale(radius.powf(2.0))
    }
}
