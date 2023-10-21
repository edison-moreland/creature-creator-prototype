use std::ops::Neg;

use nalgebra::{vector, Vector3};
use rayon::prelude::*;

use crate::spatial_indexer::kd_indexer::KdIndexer;
use crate::spatial_indexer::SpatialIndexer;
use crate::surfaces::gradient;

const REPULSION_AMPLITUDE: f32 = 6.0;
const FEEDBACK: f32 = 15.0;
const NEIGHBOUR_RADIUS: f32 = 3.0;
const UPDATE_ITERATIONS: usize = 1;
const ITERATION_T_STEP: f32 = 0.03;
const EQUILIBRIUM_SPEED: f32 = 6.0;
const FISSION_COEFFICIENT: f32 = 0.2;
const DEATH_COEFFICIENT: f32 = 0.7;
const MAX_RADIUS_COEFFICIENT: f32 = 1.2;
const DESIRED_REPULSION_ENERGY: f32 = REPULSION_AMPLITUDE * 0.8;

fn random_velocity() -> Vector3<f32> {
    Vector3::new(rand::random(), rand::random(), rand::random()).normalize()
}

// energy_contribution returns the energy of i due to j
pub fn energy_contribution(i_repulsion_radius: f32, i: Vector3<f32>, j: Vector3<f32>) -> f32 {
    REPULSION_AMPLITUDE
        * ((i - j).magnitude().powf(2.0) / (2.0 * i_repulsion_radius).powf(2.0))
            .neg()
            .exp()
}

fn repulsion_energy(
    position: Vector3<f32>,
    radius: f32,
    neighbours: impl Iterator<Item = Vector3<f32>>,
) -> f32 {
    neighbours.fold(0.0, |energy, n_position| {
        energy + energy_contribution(radius, position, n_position)
    })
}

fn particle_radius(
    position: Vector3<f32>,
    radius: f32,
    neighbours: impl Iterator<Item = Vector3<f32>> + Clone,
) -> f32 {
    let re = repulsion_energy(position, radius, neighbours.clone());

    // desired change in energy
    let re_delta = -(FEEDBACK * (re - DESIRED_REPULSION_ENERGY));

    // change in energy with respect to change in radius
    let di_ai = (1.0 / radius.powf(3.0))
        * neighbours.fold(0.0, |sum, n_position| {
            let dist = (position - n_position).magnitude().powf(2.0);

            sum + (dist * energy_contribution(radius, position, n_position))
        });

    // Radius change to bring us to desired energy
    let radius_delta = re_delta / (di_ai + 10.0);

    radius + (radius_delta * ITERATION_T_STEP)
}

fn particle_velocity(
    position: Vector3<f32>,
    radius: f32,
    neighbours: impl Iterator<Item = (Vector3<f32>, f32)>,
) -> Vector3<f32> {
    neighbours
        .fold(vector![0.0, 0.0, 0.0], |dv, (n_position, n_radius)| {
            let rij = position - n_position;

            let rei =
                (rij / radius.powf(2.0)).scale(energy_contribution(radius, position, n_position));

            let rej = (rij / n_radius.powf(2.0))
                .scale(energy_contribution(n_radius, n_position, position));

            // println!("{:?} - {:?} = {:?}", rej, rej, rei - rej);

            dv + (rei + rej)
        })
        .scale(radius.powf(2.0))
}

pub fn constrain_to_surface(
    surface: impl Fn(Vector3<f32>) -> f32,
    position: Vector3<f32>,
    velocity: Vector3<f32>,
) -> Vector3<f32> {
    let grad = gradient(&surface, position);
    velocity
        - grad.scale((grad.dot(&velocity) + (FEEDBACK * surface(position))) / (grad.dot(&grad)))
}

pub fn should_die(radius: f32, desired_radius: f32) -> bool {
    // Assuming particle is at equilibrium
    let death_radius = desired_radius * DEATH_COEFFICIENT;
    radius < death_radius && dbg!(rand::random::<f32>()) > radius / death_radius
}

pub fn should_fission_radius(radius: f32, desired_radius: f32) -> bool {
    let fission_radius = desired_radius * MAX_RADIUS_COEFFICIENT;
    radius > fission_radius
}

pub fn should_fission_energy(radius: f32, energy: f32, desired_radius: f32) -> bool {
    let fission_energy = DESIRED_REPULSION_ENERGY * FISSION_COEFFICIENT;
    energy > fission_energy && radius > desired_radius
}

pub struct RelaxationSystem {
    position_index: KdIndexer,
    position: Vec<Vector3<f32>>,
    velocity: Vec<Vector3<f32>>,
    radius: Vec<f32>,
}

impl RelaxationSystem {
    pub fn new(positions: Vec<Vector3<f32>>, sample_radius: f32) -> Self {
        let mut velocity = vec![];
        velocity.resize(positions.len(), vector![0.0, 0.0, 0.0]);

        let mut radius = vec![];
        radius.resize(positions.len(), sample_radius);

        let mut index = KdIndexer::new();
        index.reindex(&positions, &(0..positions.len()).collect::<Vec<usize>>());

        RelaxationSystem {
            position_index: index,
            position: positions,
            velocity,
            radius,
        }
    }

    pub fn positions(&self) -> impl Iterator<Item = (Vector3<f32>, f32)> + ExactSizeIterator + '_ {
        self.position
            .iter()
            .copied()
            .zip(self.radius.iter().copied())
    }

    pub fn update(
        &mut self,
        desired_radius: f32,
        surface: impl Fn(Vector3<f32>) -> f32 + Send + Sync,
    ) {
        for _ in 0..UPDATE_ITERATIONS {
            self.set_particle_velocities(&surface);
            self.update_particle_positions();
            self.update_particle_radii();
            self.split_kill_particles(desired_radius);
        }
    }

    fn set_particle_velocities(&mut self, surface: impl Fn(Vector3<f32>) -> f32 + Sync) {
        // Update velocity to push samples away from each other

        self.velocity
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, velocity)| {
                let position = self.position[i];
                let radius = self.radius[i];

                let neighbour_indices = self.position_index.get_indices_within(
                    &self.position,
                    position,
                    NEIGHBOUR_RADIUS * radius,
                );

                let neighbours = neighbour_indices
                    .iter()
                    .filter(|j| **j != i)
                    .map(|j| (self.position[*j], self.radius[*j]));

                *velocity = constrain_to_surface(
                    &surface,
                    position,
                    particle_velocity(position, radius, neighbours),
                )
            });
    }

    fn update_particle_positions(&mut self) {
        self.position
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, p)| *p += self.velocity[i].scale(ITERATION_T_STEP));

        self.position_index.reindex(
            &self.position,
            &(0..self.position.len()).collect::<Vec<usize>>(),
        );
    }

    fn update_particle_radii(&mut self) {
        self.radius
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, radius)| {
                let position = self.position[i];

                let neighbour_indices = self.position_index.get_indices_within(
                    &self.position,
                    position,
                    NEIGHBOUR_RADIUS * *radius,
                );

                let neighbours = neighbour_indices
                    .iter()
                    .filter(|j| **j != i)
                    .map(|j| self.position[*j]);

                *radius = particle_radius(position, *radius, neighbours)
            });
    }

    fn split_kill_particles(&mut self, desired_radius: f32) {
        // Apply fission/death
        let mut indices_to_die = vec![];
        let mut indices_to_fission = vec![];
        for i in 0..self.position.len() {
            let position = self.position[i];
            let velocity = self.velocity[i];
            let radius = self.radius[i];

            // Skip particles that are not at equilibrium
            if velocity.magnitude() >= (EQUILIBRIUM_SPEED * radius) {
                continue;
            }

            // We check if the particle needs to die first because it's cheaper
            if should_die(radius, desired_radius) {
                indices_to_die.push(i);
                continue;
            }

            if should_fission_radius(radius, desired_radius) {
                indices_to_fission.push(i);
                continue;
            }

            let neighbours =
                self.position_index
                    .get_indices_within(&self.position, position, NEIGHBOUR_RADIUS);

            let energy = repulsion_energy(
                position,
                radius,
                neighbours
                    .iter()
                    .filter(|j| **j != i)
                    .map(|j| self.position[*j]),
            );

            if should_fission_energy(radius, energy, desired_radius) {
                indices_to_fission.push(i);
                continue;
            }
        }

        for i in &indices_to_fission {
            // We reuse this index for the first child. The second child gets pushed to the end
            let position = self.position[*i];
            let radius = self.radius[*i];

            let new_radius = radius / (2.0_f32).sqrt();

            let new_velocity = random_velocity().scale(radius);
            self.position[*i] += new_velocity;
            self.position.push(position - new_velocity);

            self.radius[*i] = new_radius;
            self.radius.push(new_radius);

            self.velocity[*i] = vector![0.0, 0.0, 0.0];
            self.velocity.push(vector![0.0, 0.0, 0.0]);
        }

        for i in indices_to_die.iter().rev() {
            self.position.remove(*i);
            self.velocity.remove(*i);
            self.radius.remove(*i);
        }

        self.position_index.reindex(
            &self.position,
            &(0..self.position.len()).collect::<Vec<usize>>(),
        );
    }
}
