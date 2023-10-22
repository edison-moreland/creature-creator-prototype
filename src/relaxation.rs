use std::cell::RefCell;
use std::mem;
use std::ops::Neg;
use std::sync::RwLock;

use nalgebra::{vector, Vector3};
use rayon::prelude::*;

use crate::spatial_indexer::kd_indexer::KdIndexer;
use crate::spatial_indexer::{Positioned, SpatialIndexer};
use crate::surfaces::gradient;

const REPULSION_AMPLITUDE: f32 = 6.0;
const FEEDBACK: f32 = 15.0;
const NEIGHBOUR_RADIUS: f32 = 3.0;
const UPDATE_ITERATIONS: usize = 1;
const ITERATION_T_STEP: f32 = 0.03;
const EQUILIBRIUM_SPEED: f32 = 100.0;
const FISSION_COEFFICIENT: f32 = 0.2;
const DEATH_COEFFICIENT: f32 = 0.7;
const MAX_RADIUS_COEFFICIENT: f32 = 1.2;
const DESIRED_REPULSION_ENERGY: f32 = REPULSION_AMPLITUDE * 0.8;

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
    position_index: KdIndexer,

    particles_a: Vec<Particle>,
    particles_b: RwLock<Vec<Particle>>,
    // position_a: Vec<Vector3<f32>>,
    // position_b: RwLock<Vec<Vector3<f32>>>,
    // velocity_a: Vec<Vector3<f32>>,
    // velocity_b: RwLock<Vec<Vector3<f32>>>,
    // radius_a: Vec<f32>,
    // radius_b: RwLock<Vec<f32>>,
}

impl RelaxationSystem {
    pub fn new(positions: Vec<Vector3<f32>>, sample_radius: f32) -> Self {
        // let mut velocity = vec![];
        // velocity.resize(positions.len(), vector![0.0, 0.0, 0.0]);
        //
        // let mut radius = vec![];
        // radius.resize(positions.len(), sample_radius);
        //
        let particles: Vec<Particle> = positions
            .iter()
            .map(|position| Particle {
                position: *position,
                velocity: vector![0.0, 0.0, 0.0],
                radius: sample_radius,
            })
            .collect();

        let mut index = KdIndexer::new();
        index.reindex(&particles);

        RelaxationSystem {
            position_index: index,

            particles_a: particles.clone(),
            particles_b: RwLock::new(particles),
            // position_a: positions.clone(),
            // position_b: RwLock::new(positions),
            // velocity_a: velocity.clone(),
            // velocity_b: RwLock::new(velocity),
            // radius_a: radius.clone(),
            // radius_b: RwLock::new(radius),
        }
    }

    pub fn positions(&self) -> impl Iterator<Item = (Vector3<f32>, f32)> + ExactSizeIterator + '_ {
        self.particles_a.iter().map(|p| (p.position, p.radius))
    }

    pub fn update(
        &mut self,
        desired_radius: f32,
        surface: impl Fn(Vector3<f32>) -> f32 + Send + Sync,
    ) {
        for _ in 0..UPDATE_ITERATIONS {
            // let particle_count = self.radius_a.len();

            self.particles_b
                .write()
                .unwrap()
                .par_iter_mut()
                .enumerate()
                .for_each(|(i, particle)| {
                    let neighbour_indices = self.position_index.get_indices_within(
                        &self.particles_a,
                        self.particles_a[i].position,
                        NEIGHBOUR_RADIUS * self.particles_a[i].radius,
                    );

                    let neighbours = neighbour_indices.iter().filter(|j| **j != i).copied();

                    let velocity = constrain_to_surface(
                        &surface,
                        self.particles_a[i].position,
                        self.particle_velocity(
                            self.particles_a[i].position,
                            self.particles_a[i].radius,
                            neighbours.clone(),
                        ),
                    );

                    let position = self.particles_a[i].position + velocity.scale(ITERATION_T_STEP);

                    let radius = self.particle_radius(
                        position,
                        self.particles_a[i].radius,
                        neighbours.clone(),
                    );

                    *particle = Particle {
                        position,
                        velocity,
                        radius,
                    }

                    // self.velocity_b.write().unwrap()[i] = velocity;
                    // self.position_b.write().unwrap()[i] = position;
                    // self.radius_b.write().unwrap()[i] = radius;
                });

            // (0..particle_count).par_bridge().for_each(|i| {
            // });
            //
            // mem::swap(&mut self.position_a, &mut self.position_b.write().unwrap());
            // mem::swap(&mut self.velocity_a, &mut self.velocity_b.write().unwrap());
            // mem::swap(&mut self.radius_a, &mut self.radius_b.write().unwrap());

            // self.position_a = self.position_b.write().unwrap().clone();
            // self.velocity_a = self.velocity_b.write().unwrap().clone();
            // self.radius_a = self.radius_b.write().unwrap().clone();

            mem::swap(
                &mut self.particles_a,
                &mut self.particles_b.write().unwrap(),
            );

            self.position_index.reindex(&self.particles_a);

            self.split_kill_particles(desired_radius);

            // self.set_particle_velocities(&surface);
            // self.update_particle_positions();
            // self.update_particle_radii();
        }
    }

    fn repulsion_energy(
        &self,
        position: Vector3<f32>,
        radius: f32,
        neighbours: impl Iterator<Item = usize>,
    ) -> f32 {
        neighbours.fold(0.0, |energy, j| {
            energy + energy_contribution(radius, position, self.particles_a[j].position)
        })
    }

    fn particle_radius(
        &self,
        position: Vector3<f32>,
        radius: f32,
        neighbours: impl Iterator<Item = usize> + Clone,
    ) -> f32 {
        let re = self.repulsion_energy(position, radius, neighbours.clone());

        // desired change in energy
        let re_delta = -(FEEDBACK * (re - DESIRED_REPULSION_ENERGY));

        // change in energy with respect to change in radius
        let di_ai = (1.0 / radius.powf(3.0))
            * neighbours.fold(0.0, |sum, j| {
                let dist = (position - self.particles_a[j].position())
                    .magnitude()
                    .powf(2.0);

                sum + (dist * energy_contribution(radius, position, self.particles_a[j].position()))
            });

        // Radius change to bring us to desired energy
        let radius_delta = re_delta / (di_ai + 10.0);

        radius + (radius_delta * ITERATION_T_STEP)
    }

    fn particle_velocity(
        &self,
        position: Vector3<f32>,
        radius: f32,
        neighbours: impl Iterator<Item = usize> + Clone,
    ) -> Vector3<f32> {
        neighbours
            .fold(vector![0.0, 0.0, 0.0], |dv, i| {
                let rij = position - self.particles_a[i].position();

                let rei = (rij / radius.powf(2.0)).scale(energy_contribution(
                    radius,
                    position,
                    self.particles_a[i].position,
                ));

                let rej = (rij / self.particles_a[i].radius.powf(2.0)).scale(energy_contribution(
                    self.particles_a[i].radius,
                    self.particles_a[i].position,
                    position,
                ));

                // println!("{:?} - {:?} = {:?}", rej, rej, rei - rej);

                dv + (rei + rej)
            })
            .scale(radius.powf(2.0))
    }

    fn split_kill_particles(&mut self, desired_radius: f32) {
        // Apply fission/death
        let mut indices_to_die = vec![];
        let mut indices_to_fission = vec![];
        for (i, particle) in self.particles_a.iter().enumerate() {
            // Skip particles that are not at equilibrium
            if particle.velocity.magnitude() >= (EQUILIBRIUM_SPEED * particle.radius) {
                continue;
            }

            // We check if the particle needs to die first because it's cheaper
            if should_die(particle.radius, desired_radius) {
                indices_to_die.push(i);
                continue;
            }

            if should_fission_radius(particle.radius, desired_radius) {
                indices_to_fission.push(i);
                continue;
            }

            let neighbours = self.position_index.get_indices_within(
                &self.particles_a,
                particle.position,
                NEIGHBOUR_RADIUS,
            );

            let energy = self.repulsion_energy(
                particle.position,
                particle.radius,
                neighbours.iter().filter(|j| **j != i).copied(),
            );

            if should_fission_energy(particle.radius, energy, desired_radius) {
                indices_to_fission.push(i);
                continue;
            }
        }

        for i in &indices_to_fission {
            // We reuse this index for the first child. The second child gets pushed to the end
            let position = self.particles_a[*i].position();
            let radius = self.particles_a[*i].radius;

            let new_radius = radius / (2.0_f32).sqrt();

            let new_velocity = random_velocity().scale(radius);

            let sibling_1 = Particle {
                position: position + new_velocity,
                velocity: vector![0.0, 0.0, 0.0],
                radius: new_radius,
            };
            let sibling_2 = Particle {
                position: position - new_velocity,
                velocity: vector![0.0, 0.0, 0.0],
                radius: new_radius,
            };

            self.particles_a[*i] = sibling_1;

            self.particles_a.push(sibling_2);
            self.particles_b.write().unwrap().push(sibling_2);
        }

        for i in indices_to_die.iter().rev() {
            self.particles_a.remove(*i);
            self.particles_b.write().unwrap().remove(*i);
        }

        self.position_index.reindex(&self.particles_a);
    }
}
