use std::mem;
use std::ops::Neg;

use nalgebra::{vector, Vector3};

use crate::buffer_allocator::{BufferAllocator, StackBufferAllocator};
use crate::spatial_indexer::{Positioned, SpatialIndexer};
use crate::spatial_indexer::kd_indexer::KdIndexer;
use crate::surfaces::gradient;

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
const MAX_PARTICLE_COUNT: usize = 20000;

fn random_velocity() -> Vector3<f32> {
    Vector3::new(rand::random(), rand::random(), rand::random()).normalize()
}

// energy_contribution returns the energy of i due to j
fn energy_contribution(i_repulsion_radius: f32, i: Vector3<f32>, j: Vector3<f32>) -> f32 {
    REPULSION_AMPLITUDE
        * ((i - j).magnitude().powf(2.0) / (2.0 * i_repulsion_radius).powf(2.0))
            .neg()
            .exp()
}

fn constrain_to_surface(
    surface: impl Fn(Vector3<f32>) -> f32,
    position: Vector3<f32>,
    velocity: Vector3<f32>,
) -> Vector3<f32> {
    let grad = gradient(&surface, position);
    velocity
        - grad.scale((grad.dot(&velocity) + (FEEDBACK * surface(position))) / (grad.dot(&grad)))
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
    position: Vector3<f32>,
    velocity: Vector3<f32>,
    radius: f32,
}

impl Positioned for Particle {
    fn position(&self) -> Vector3<f32> {
        self.position
    }
}

pub struct RelaxationSystem {
    living_particles: Vec<usize>,
    position_index: KdIndexer,
    index_allocator: StackBufferAllocator<MAX_PARTICLE_COUNT>,

    particles_a: [Particle; MAX_PARTICLE_COUNT],
    particles_b: [Particle; MAX_PARTICLE_COUNT],
}

impl RelaxationSystem {
    pub fn new(positions: Vec<Vector3<f32>>, sample_radius: f32) -> Self {
        if positions.len() > MAX_PARTICLE_COUNT {
            panic!("TOO DANG BIG!!")
        }

        let mut particles = [Particle {
            position: vector![0.0, 0.0, 0.0],
            velocity: vector![0.0, 0.0, 0.0],
            radius: sample_radius,
        }; MAX_PARTICLE_COUNT];

        let mut index_allocator = StackBufferAllocator::new();

        let mut living_particles = vec![];

        for p in positions {
            let i = index_allocator.insert();

            particles[i].position = p;
            living_particles.push(i)
        }

        let mut position_index = KdIndexer::new();
        position_index.reindex(&particles, living_particles.clone());

        RelaxationSystem {
            living_particles,
            position_index,
            index_allocator,

            particles_a: particles.clone(),
            particles_b: particles,
        }
    }

    pub fn positions(&self) -> impl Iterator<Item = (Vector3<f32>, f32)> + ExactSizeIterator + '_ {
        self.living_particles
            .iter()
            .map(|i| (self.particles_a[*i].position, self.particles_a[*i].radius))
        //
        // self.particles_a.iter().map(|p| (p.position, p.radius))
    }

    pub fn update(
        &mut self,
        desired_radius: f32,
        surface: impl Fn(Vector3<f32>) -> f32 + Send + Sync,
    ) {
        for _ in 0..UPDATE_ITERATIONS {
            for j in (0..self.living_particles.len()).rev() {
                let i = self.living_particles[j];

                let neighbour_indices = self.position_index.get_indices_within(
                    &self.particles_a,
                    self.particles_a[i].position,
                    NEIGHBOUR_RADIUS * self.particles_a[i].radius,
                );

                let neighbours: Vec<(usize, f32, f32)> = neighbour_indices
                    .iter()
                    .filter(|j| **j != i)
                    .map(|j| {
                        let pi = self.particles_a[i];
                        let pj = self.particles_a[*j];

                        (
                            *j,
                            energy_contribution(pi.radius, pi.position, pj.position),
                            energy_contribution(pj.radius, pj.position, pi.position),
                        )
                    })
                    .collect();
                let energy = self.repulsion_energy(&neighbours);

                if self.particles_a[i].velocity.magnitude()
                    < (EQUILIBRIUM_SPEED * self.particles_a[i].radius)
                {
                    let radius = self.particles_a[i].radius;
                    if should_die(radius, desired_radius) {
                        self.living_particles.remove(j);
                        self.index_allocator.remove(i);
                        continue;
                    }

                    if should_fission_energy(radius, energy, desired_radius)
                        || should_fission_radius(radius, desired_radius)
                    {
                        let position = self.particles_a[i].position();
                        let radius = self.particles_a[i].radius;

                        let new_radius = radius / (2.0_f32).sqrt();
                        let new_velocity = random_velocity().scale(radius);

                        self.particles_a[i] = Particle {
                            position: position + new_velocity,
                            velocity: vector![0.0, 0.0, 0.0],
                            radius: new_radius,
                        };

                        let sibling = Particle {
                            position: position - new_velocity,
                            velocity: vector![0.0, 0.0, 0.0],
                            radius: new_radius,
                        };
                        let sibling_i = self.index_allocator.insert();
                        self.particles_a[sibling_i] = sibling;
                        self.particles_b[sibling_i] = sibling;
                        self.living_particles.push(sibling_i);
                        continue;
                    }
                }

                let velocity = constrain_to_surface(
                    &surface,
                    self.particles_a[i].position,
                    self.particle_velocity(
                        self.particles_a[i].position,
                        self.particles_a[i].radius,
                        &neighbours,
                    ),
                );

                let position = self.particles_a[i].position + velocity.scale(ITERATION_T_STEP);

                let radius =
                    self.particle_radius(position, self.particles_a[i].radius, energy, &neighbours);

                self.particles_b[i] = Particle {
                    position,
                    velocity,
                    radius,
                }
            }

            mem::swap(&mut self.particles_a, &mut self.particles_b);

            self.position_index
                .reindex(&self.particles_a, self.living_particles.clone());
        }
    }

    fn repulsion_energy(&self, neighbours: &Vec<(usize, f32, f32)>) -> f32 {
        neighbours
            .iter()
            .fold(0.0, |energy, (_, energy_cont, _)| energy + energy_cont)
    }

    fn particle_radius(
        &self,
        position: Vector3<f32>,
        radius: f32,
        repulsion_energy: f32,
        neighbours: &Vec<(usize, f32, f32)>,
    ) -> f32 {
        // desired change in energy
        let re_delta = -(FEEDBACK * (repulsion_energy - DESIRED_REPULSION_ENERGY));

        // change in energy with respect to change in radius
        let di_ai = (1.0 / radius.powf(3.0))
            * neighbours.iter().fold(0.0, |sum, (j, energy_cont, _)| {
                let dist = (position - self.particles_a[*j].position())
                    .magnitude()
                    .powf(2.0);

                sum + (dist * energy_cont)
            });

        // Radius change to bring us to desired energy
        let radius_delta = re_delta / (di_ai + 10.0);

        radius + (radius_delta * ITERATION_T_STEP)
    }

    fn particle_velocity(
        &self,
        position: Vector3<f32>,
        radius: f32,
        neighbours: &Vec<(usize, f32, f32)>,
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
