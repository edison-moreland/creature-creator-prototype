use crate::spatial_indexer::kd_indexer::{KdContainer, KdIndexer};
use crate::spatial_indexer::SpatialIndexer;
use crate::surfaces::gradient;
use raylib::prelude::*;
use rayon::prelude::*;
use std::ops::Neg;
use std::time::Instant;

const REPULSION_AMPLITUDE: f32 = 6.0;
const FEEDBACK: f32 = 15.0;
const NEIGHBOUR_RADIUS: f32 = 2.0;
const UPDATE_ITERATIONS: usize = 1;
const ITERATION_T_STEP: f32 = 0.03;

// energy_contribution returns the energy of i due to j
fn energy_contribution(i_repulsion_radius: f32, i: Vector3, j: Vector3) -> f32 {
    REPULSION_AMPLITUDE
        * (i.distance_to(j).powf(2.0) / (2.0 * i_repulsion_radius).powf(2.0))
            .neg()
            .exp()
}

fn particle_radius(position: Vector3, radius: f32, neighbours: Vec<Vector3>) -> f32 {
    let repulsion_energy: f32 = neighbours.iter().fold(0.0, |energy, n_position| {
        energy + energy_contribution(radius, position, *n_position)
    });

    let desired_re = REPULSION_AMPLITUDE * 0.8;

    let re_delta = -(FEEDBACK * (repulsion_energy - desired_re));

    // change in energy with respect to change in radius
    let temp: f32 = neighbours.iter().fold(0.0, |sum, n_position| {
        let dist = position.distance_to(*n_position).powf(2.0);

        sum + (dist * energy_contribution(radius, position, *n_position))
    });
    let di_ai = (1.0 / radius.powf(3.0)) * temp;

    // Radius change to bring us to desired energy
    let radius_delta = re_delta / (di_ai + 10.0);

    // println!("{:?}", radius_delta);

    let new_radius = radius + (radius_delta * ITERATION_T_STEP);

    println!("{:?} = {:?} + {:?}", new_radius, radius, radius_delta);

    new_radius
}

fn particle_velocity(position: Vector3, radius: f32, neighbours: Vec<(Vector3, f32)>) -> Vector3 {
    // return rvec3(0, 0, 0); // Not yet

    // println!("{:?}", radius);

    neighbours
        .iter()
        .copied()
        .fold(rvec3(0, 0, 0), |dv, (n_position, n_radius)| {
            let rij = position - n_position;

            let rei = (rij / radius.powf(2.0))
                .scale_by(energy_contribution(radius, position, n_position));

            let rej = (rij / n_radius.powf(2.0))
                .scale_by(energy_contribution(n_radius, n_position, position));

            dv + (rei - rej)
        })
        .scale_by(radius.powf(2.0))
}

fn constrain_to_surface(
    surface: impl Fn(Vector3) -> f32,
    position: Vector3,
    velocity: Vector3,
) -> Vector3 {
    let grad = gradient(&surface, position);
    velocity
        - grad.scale_by((grad.dot(velocity) + (FEEDBACK * surface(position))) / (grad.dot(grad)))
}

pub struct RelaxationSystem {
    position_index: KdIndexer,
    position: Vec<Vector3>,
    velocity: Vec<Vector3>,
    radius: Vec<f32>,
}

impl RelaxationSystem {
    pub fn new(positions: Vec<Vector3>, sample_radius: f32) -> Self {
        let mut velocity = vec![];
        velocity.resize(positions.len(), rvec3(0, 0, 0));

        let mut radius = vec![];
        radius.resize(positions.len(), sample_radius);

        let mut index = KdIndexer::new();
        index.reindex(&positions);

        RelaxationSystem {
            position_index: index,
            position: positions,
            velocity,
            radius,
        }
    }

    pub fn positions(&self) -> &Vec<Vector3> {
        &self.position
    }

    pub fn update(&mut self, surface: impl Fn(Vector3) -> f32 + Send + Sync) {
        for i in 0..UPDATE_ITERATIONS {
            let start = Instant::now();

            // Update velocity to push samples away from each other
            self.update_velocity(|(position, radius), neighbours| {
                particle_velocity(position, radius, neighbours)
            });

            // Apply the velocity to the position
            self.update_positions(|position, velocity| {
                position
                    + constrain_to_surface(&surface, position, velocity).scale_by(ITERATION_T_STEP)
            });
            self.position_index.reindex(&self.position);

            // Update each particles radius
            // self.update_radius(|(position, radius), neighbours| {
            //     particle_radius(position, radius, neighbours)
            // });

            println!("Pass {:?}, {:?}", i, start.elapsed());
        }
    }

    fn update_radius(&mut self, f: impl Fn((Vector3, f32), Vec<Vector3>) -> f32 + Send + Sync) {
        self.radius
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, radius)| {
                let position = self.position[i];

                let neighbour_indices = self.position_index.get_indices_within(
                    &self.position,
                    position,
                    NEIGHBOUR_RADIUS,
                );

                *radius = f(
                    (position, *radius),
                    neighbour_indices
                        .iter()
                        .map(|i| self.position[*i])
                        .collect(),
                )
            })
    }

    fn update_velocity(
        &mut self,
        f: impl Fn((Vector3, f32), Vec<(Vector3, f32)>) -> Vector3 + Send + Sync,
    ) {
        self.velocity
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, velocity)| {
                let position = self.position[i];
                let radius = self.radius[i];

                // TODO: Use particle radius*some constant?
                // let neighbours = self.position.get_items_in_radius(p, neighbour_radius);

                let neighbour_indices = self.position_index.get_indices_within(
                    &self.position,
                    position,
                    NEIGHBOUR_RADIUS,
                );

                *velocity = f(
                    (position, radius),
                    neighbour_indices
                        .iter()
                        .map(|i| (self.position[*i], self.radius[*i]))
                        .collect(),
                )
            });
    }

    fn update_positions(&mut self, f: impl Fn(Vector3, Vector3) -> Vector3 + Sync) {
        self.position
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, p)| *p = f(*p, self.velocity[i]));

        self.position_index.reindex(&self.position)
    }
}

// TODO: Particle store legacy
pub struct ParticleStore<T> {
    particles: KdContainer<Vector3>,
    attributes: Vec<T>,
}

impl<T> ParticleStore<T>
where
    T: Send + Sync,
{
    pub fn new(particles: Vec<Vector3>, f: impl Fn(Vector3) -> T) -> Self {
        let attributes = particles.iter().map(|p| f(*p)).collect();

        ParticleStore {
            particles: KdContainer::from_items(particles),
            attributes,
        }
    }

    pub fn update_attributes(
        &mut self,
        neighbour_radius: f32,
        f: impl Fn(&mut T, Vector3, Vec<&Vector3>) + Sync,
    ) {
        self.attributes
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, attribute)| {
                let particle = self.particles[i];

                let neighbours = self
                    .particles
                    .get_items_in_radius(particle, neighbour_radius);

                f(attribute, self.particles[i], neighbours)
            });
    }

    pub fn update_particles(&mut self, f: impl Fn(Vector3, &T) -> Vector3 + Sync) {
        self.particles
            .items
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, particle)| *particle = f(*particle, &self.attributes[i]));

        self.particles.reconstruct()
    }

    pub fn positions(&self) -> Vec<Vector3> {
        self.particles.items.clone()
    }
}
