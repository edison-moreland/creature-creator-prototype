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

pub struct RelaxationSystem {
    position_index: KdIndexer,
    position_a: Vec<Vector3<f32>>,
    position_b: Vec<Vector3<f32>>,
    velocity_a: Vec<Vector3<f32>>,
    velocity_b: Vec<Vector3<f32>>,
    radius_a: Vec<f32>,
    radius_b: Vec<f32>,
}

impl RelaxationSystem {
    pub fn new(positions: Vec<Vector3<f32>>, sample_radius: f32) -> Self {
        let mut velocity = vec![];
        velocity.resize(positions.len(), vector![0.0, 0.0, 0.0]);

        let mut radius = vec![];
        radius.resize(positions.len(), sample_radius);

        let mut index = KdIndexer::new();
        index.reindex(&positions);

        RelaxationSystem {
            position_index: index,
            position_a: positions.clone(),
            position_b: positions,
            velocity_a: velocity.clone(),
            velocity_b: velocity,
            radius_a: radius.clone(),
            radius_b: radius,
        }
    }

    pub fn positions(&self) -> impl Iterator<Item = (Vector3<f32>, f32)> + ExactSizeIterator + '_ {
        self.position_a
            .iter()
            .copied()
            .zip(self.radius_a.iter().copied())
    }

    pub fn update(
        &mut self,
        desired_radius: f32,
        surface: impl Fn(Vector3<f32>) -> f32 + Send + Sync,
    ) {
        for _ in 0..UPDATE_ITERATIONS {
            let particle_count = self.radius_a.len();

            for i in 0..particle_count {
                let neighbour_indices = self.position_index.get_indices_within(
                    &self.position_a,
                    self.position_a[i],
                    NEIGHBOUR_RADIUS * self.radius_a[i],
                );

                let neighbours = neighbour_indices.iter().filter(|j| **j != i).copied();

                let velocity = constrain_to_surface(
                    &surface,
                    self.position_a[i],
                    self.particle_velocity(
                        self.position_a[i],
                        self.radius_a[i],
                        neighbours.clone(),
                    ),
                );

                let position = self.position_a[i] + velocity.scale(ITERATION_T_STEP);

                let radius = self.particle_radius(position, self.radius_a[i], neighbours.clone());

                self.velocity_b[i] = velocity;
                self.position_b[i] = position;
                self.radius_b[i] = radius;
            }

            self.position_a = self.position_b.clone();
            self.velocity_a = self.velocity_b.clone();
            self.radius_a = self.radius_b.clone();

            self.position_index.reindex(&self.position_a);

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
            energy + energy_contribution(radius, position, self.position_a[j])
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
                let dist = (position - self.position_a[j]).magnitude().powf(2.0);

                sum + (dist * energy_contribution(radius, position, self.position_a[j]))
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
                let rij = position - self.position_a[i];

                let rei = (rij / radius.powf(2.0)).scale(energy_contribution(
                    radius,
                    position,
                    self.position_a[i],
                ));

                let rej = (rij / self.radius_a[i].powf(2.0)).scale(energy_contribution(
                    self.radius_a[i],
                    self.position_a[i],
                    position,
                ));

                // println!("{:?} - {:?} = {:?}", rej, rej, rei - rej);

                dv + (rei + rej)
            })
            .scale(radius.powf(2.0))
    }

    // fn set_particle_velocities(&mut self, surface: impl Fn(Vector3<f32>) -> f32 + Sync) {
    //     // Update velocity to push samples away from each other
    //
    //     self.velocity
    //         .par_iter_mut()
    //         .enumerate()
    //         .for_each(|(i, velocity)| {
    //             let position = self.position[i];
    //             let radius = self.radius[i];
    //
    //             let neighbour_indices = self.position_index.get_indices_within(
    //                 &self.position,
    //                 position,
    //                 NEIGHBOUR_RADIUS * radius,
    //             );
    //
    //             let neighbours = neighbour_indices
    //                 .iter()
    //                 .filter(|j| **j != i)
    //                 .map(|j| (self.position[*j], self.radius[*j]));
    //
    //             *velocity = constrain_to_surface(
    //                 &surface,
    //                 position,
    //                 particle_velocity(position, radius, neighbours),
    //             )
    //         });
    // }
    //
    // fn update_particle_positions(&mut self) {
    //     self.position
    //         .par_iter_mut()
    //         .enumerate()
    //         .for_each(|(i, p)| *p += self.velocity[i].scale(ITERATION_T_STEP));
    //
    //     self.position_index.reindex(&self.position);
    // }

    // fn update_particle_radii(&mut self) {
    //     self.radius
    //         .par_iter_mut()
    //         .enumerate()
    //         .for_each(|(i, radius)| {
    //             let position = self.position[i];
    //
    //             let neighbour_indices = self.position_index.get_indices_within(
    //                 &self.position,
    //                 position,
    //                 NEIGHBOUR_RADIUS * *radius,
    //             );
    //
    //             let neighbours = neighbour_indices
    //                 .iter()
    //                 .filter(|j| **j != i)
    //                 .map(|j| self.position[*j]);
    //
    //             *radius = particle_radius(position, *radius, neighbours)
    //         });
    // }

    fn split_kill_particles(&mut self, desired_radius: f32) {
        // Apply fission/death
        let mut indices_to_die = vec![];
        let mut indices_to_fission = vec![];
        for i in 0..self.position_a.len() {
            let position = self.position_a[i];
            let velocity = self.velocity_a[i];
            let radius = self.radius_a[i];

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

            let neighbours = self.position_index.get_indices_within(
                &self.position_a,
                position,
                NEIGHBOUR_RADIUS,
            );

            let energy = self.repulsion_energy(
                position,
                radius,
                neighbours.iter().filter(|j| **j != i).copied(),
            );

            if should_fission_energy(radius, energy, desired_radius) {
                indices_to_fission.push(i);
                continue;
            }
        }

        for i in &indices_to_fission {
            // We reuse this index for the first child. The second child gets pushed to the end
            let position = self.position_a[*i];
            let radius = self.radius_a[*i];

            let new_radius = radius / (2.0_f32).sqrt();

            let new_velocity = random_velocity().scale(radius);
            self.position_a[*i] += new_velocity;
            self.position_a.push(position - new_velocity);
            self.position_b.push(position - new_velocity);

            self.radius_a[*i] = new_radius;
            self.radius_a.push(new_radius);
            self.radius_b.push(new_radius);

            self.velocity_a[*i] = vector![0.0, 0.0, 0.0];
            self.velocity_a.push(vector![0.0, 0.0, 0.0]);
            self.velocity_b.push(vector![0.0, 0.0, 0.0]);
        }

        for i in indices_to_die.iter().rev() {
            self.position_a.remove(*i);
            self.velocity_a.remove(*i);
            self.radius_a.remove(*i);
            self.position_b.remove(*i);
            self.velocity_b.remove(*i);
            self.radius_b.remove(*i);
        }

        self.position_index.reindex(&self.position_a);
    }
}
