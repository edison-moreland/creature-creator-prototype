use crate::pool::{PoolAllocator, StackPoolAllocator};
use nalgebra::{vector, Vector3};
use rand::random;

use crate::relaxation::{
    constrain_to_surface, energy_contribution, should_die, should_fission_energy,
    should_fission_radius,
};
use crate::spatial_indexer::kd_indexer::KdIndexer;
use crate::spatial_indexer::{Positioned, SpatialIndexer};

const MAX_PARTICLES: usize = 10000;

#[derive(Debug, Copy, Clone)]
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

struct ParticleStorage<A, S> {
    index_allocator: A,
    spatial_indexer: S,

    active_particles: Vec<usize>,

    pool_a: [Particle; MAX_PARTICLES],
    pool_b: [Particle; MAX_PARTICLES],
    _active_pool: bool,
}

enum ParticleUpdate {
    Kill,
    Live(Particle),
    Fission(Particle, Particle),
}

impl<A, S> ParticleStorage<A, S>
where
    A: PoolAllocator<MAX_PARTICLES>,
    S: SpatialIndexer<Particle>,
{
    fn init_with_samples(mut pa: A, mut si: S, samples: Vec<Vector3<f32>>, radius: f32) -> Self {
        let mut pool_a = [Particle {
            position: Default::default(),
            velocity: Default::default(),
            radius,
        }; MAX_PARTICLES];

        let active_particles: Vec<usize> = samples
            .iter()
            .map(|position| {
                let i = pa.insert();

                pool_a[i].position = *position;
                pool_a[i].velocity = vector![random(), random(), random()]
                    .normalize()
                    .scale(radius);

                i
            })
            .collect();

        si.reindex(&pool_a[..], &active_particles);

        let pool_b = pool_a.clone();

        ParticleStorage {
            index_allocator: pa,
            spatial_indexer: si,
            active_particles,
            pool_a,
            pool_b,
            _active_pool: true,
        }
    }

    fn toggle_set(&mut self) {
        self._active_pool = !self._active_pool
    }

    fn active_set(&self) -> &[Particle; MAX_PARTICLES] {
        if self._active_pool {
            &self.pool_a
        } else {
            &self.pool_b
        }
    }
    fn active_set_mut(&mut self) -> &mut [Particle; MAX_PARTICLES] {
        if self._active_pool {
            &mut self.pool_a
        } else {
            &mut self.pool_b
        }
    }

    fn inactive_set(&self) -> &[Particle; MAX_PARTICLES] {
        if self._active_pool {
            &self.pool_b
        } else {
            &self.pool_a
        }
    }

    fn inactive_set_mut(&mut self) -> &mut [Particle; MAX_PARTICLES] {
        if self._active_pool {
            &mut self.pool_b
        } else {
            &mut self.pool_a
        }
    }

    // TODO: Legacy
    fn positions(&self) -> impl Iterator<Item = (Vector3<f32>, f32)> + ExactSizeIterator + '_ {
        self.active_particles.iter().map(|i| {
            let p = self.inactive_set()[*i];

            (p.position, p.radius)
        })
    }

    fn update_particles(
        &mut self,
        neighbour_radius: f32,
        f: impl Fn(Particle, Vec<Particle>) -> ParticleUpdate,
    ) {
        let mut particles_killed = vec![];

        let count = self.active_particles.len();

        for j in 0..count {
            let i = self.active_particles[j];

            let particle = self.inactive_set()[i];

            let neighbours: Vec<Particle> = self
                .spatial_indexer
                .get_indices_within(
                    self.inactive_set(),
                    particle.position,
                    particle.radius * neighbour_radius,
                )
                .iter()
                .filter(|k| **k != j)
                .map(|k| self.inactive_set()[*k])
                .collect();

            match f(particle, neighbours) {
                ParticleUpdate::Kill => {
                    particles_killed.push(j);
                }
                ParticleUpdate::Live(p) => self.active_set_mut()[i] = p,
                ParticleUpdate::Fission(p1, p2) => {
                    let sibling_i = self.index_allocator.insert();
                    self.active_particles.push(sibling_i);

                    self.active_set_mut()[i] = p1;
                    self.active_set_mut()[sibling_i] = p2;
                    // self.inactive_set_mut()[i] = p1;
                    self.inactive_set_mut()[sibling_i] = p2;
                }
            }
        }

        if particles_killed.len() > (count / 2) {
            panic!("Half all particles killed?")
        }

        particles_killed.sort();
        for j in particles_killed.iter().rev() {
            self.index_allocator.remove(self.active_particles[*j]);
            self.active_particles.remove(*j);
        }

        self.toggle_set();
        self.spatial_indexer
            .reindex(self.inactive_set(), &self.active_particles);
    }
}

pub struct Parameters {
    pub time_step: f32,

    pub iterations_per_step: usize,

    // feedback keeps particles on the surface and energizes the system
    pub feedback: f32,

    pub repulsion_amplitude: f32,

    // desired_energy = repulsion_amplitude * desired_energy_fraction
    pub desired_energy_fraction: f32,

    pub desired_repulsion_radius: f32,

    // max_repulsion_radius = desired_repulsion_radius * max_repulsion_radius_fraction
    pub max_repulsion_radius_fraction: f32,

    pub equilibrium_speed: f32,

    pub fission_faction: f32,

    pub death_fraction: f32,

    // The maximum radius that particles can effect each other. Scaled by radius
    pub neighbour_radius: f32,
}

impl Parameters {
    fn max_repulsion_radius(&self) -> f32 {
        self.desired_repulsion_radius * self.max_repulsion_radius_fraction
    }

    fn desired_energy(&self) -> f32 {
        self.repulsion_amplitude * self.desired_energy_fraction
    }
}

impl Default for Parameters {
    fn default() -> Self {
        Parameters {
            time_step: 0.03,
            feedback: 15.0,
            repulsion_amplitude: 6.0,
            desired_energy_fraction: 0.8,
            desired_repulsion_radius: 0.0,
            max_repulsion_radius_fraction: 1.2,
            equilibrium_speed: 6.0,
            fission_faction: 0.2,
            death_fraction: 0.7,
            iterations_per_step: 1,
            neighbour_radius: 3.0,
        }
    }
}

pub struct ParticleSystem {
    parameters: Parameters,
    storage: ParticleStorage<StackPoolAllocator<MAX_PARTICLES>, KdIndexer>,
    pub time: f32,
}

impl ParticleSystem {
    pub fn new(parameters: Parameters, samples: Vec<Vector3<f32>>) -> Self {
        let storage = ParticleStorage::init_with_samples(
            StackPoolAllocator::new(),
            KdIndexer::new(),
            samples,
            parameters.desired_repulsion_radius,
        );

        ParticleSystem {
            parameters,
            storage,
            time: 0.0,
        }
    }

    pub fn advance_simulation(&mut self, surface: impl Fn(Vector3<f32>) -> f32) {
        for _ in 0..self.parameters.iterations_per_step {
            self.storage.update_particles(
                self.parameters.neighbour_radius,
                |particle, neighbours| {
                    let vm = particle.velocity.magnitude();
                    if vm < particle.radius * self.parameters.equilibrium_speed {
                        if should_die(particle.radius, self.parameters.desired_repulsion_radius) {
                            return ParticleUpdate::Kill;
                        }

                        let re =
                            repulsion_energy(particle.position, particle.radius, neighbours.iter());
                        if should_fission_energy(
                            particle.radius,
                            re,
                            self.parameters.desired_repulsion_radius,
                        ) || should_fission_radius(
                            particle.radius,
                            self.parameters.desired_repulsion_radius,
                        ) {
                            let new_radius = particle.radius / (2.0_f32).sqrt();

                            let offset = vector![random(), random(), random()]
                                .normalize()
                                .scale(particle.radius);

                            return ParticleUpdate::Fission(
                                Particle {
                                    radius: new_radius,
                                    velocity: offset.scale(self.parameters.equilibrium_speed * 2.0),
                                    position: particle.position + offset,
                                },
                                Particle {
                                    radius: new_radius,
                                    velocity: -offset
                                        .scale(self.parameters.equilibrium_speed * 2.0),
                                    position: particle.position - offset,
                                },
                            );
                        }
                    }
                    let new_particle = {
                        let velocity = particle_velocity(&particle, neighbours.iter());

                        let position = particle.position
                            + constrain_to_surface(&surface, particle.position, velocity)
                                .scale(self.parameters.time_step);

                        let radius = particle_radius(
                            &self.parameters,
                            position,
                            particle.radius,
                            neighbours.iter(),
                        );

                        Particle {
                            velocity,
                            position,
                            radius,
                        }
                    };

                    ParticleUpdate::Live(new_particle)
                },
            );

            // Time isn't actually used by the simulation, but it let's other keep in sync
            self.time += self.parameters.time_step;
        }
    }

    pub fn positions(&self) -> impl Iterator<Item = (Vector3<f32>, f32)> + ExactSizeIterator + '_ {
        self.storage.positions()
    }
}

fn particle_velocity<'a>(
    p: &Particle,
    neighbours: impl Iterator<Item = &'a Particle>,
) -> Vector3<f32> {
    neighbours
        .fold(vector![0.0, 0.0, 0.0], |dv: Vector3<f32>, (n)| {
            let rij = p.position - n.position;

            let rei = (rij / p.radius.powf(2.0))
                .scale(energy_contribution(p.radius, p.position, n.position));

            let rej = (rij / n.radius.powf(2.0))
                .scale(energy_contribution(n.radius, n.position, p.position));

            dv + (rei + rej)
        })
        .scale(p.radius.powf(2.0))
}

fn particle_radius<'a>(
    parameters: &Parameters,
    p_position: Vector3<f32>,
    p_radius: f32,
    neighbours: impl Iterator<Item = &'a Particle> + Clone,
) -> f32 {
    let re = repulsion_energy(p_position, p_radius, neighbours.clone());

    // desired change in energy
    let re_delta = -(parameters.feedback * (re - parameters.desired_energy()));

    // change in energy with respect to change in radius
    let di_ai = (1.0 / p_radius.powf(3.0))
        * neighbours.fold(0.0, |sum, n| {
            let dist = (p_position - n.position).magnitude().powf(2.0);

            sum + (dist * energy_contribution(p_radius, p_position, n.position))
        });

    // Radius change to bring us to desired energy
    let radius_delta = re_delta / (di_ai + 10.0);

    p_radius + (radius_delta * parameters.time_step)
}

fn repulsion_energy<'a>(
    p_position: Vector3<f32>,
    p_radius: f32,
    neighbours: impl Iterator<Item = &'a Particle> + Clone,
) -> f32 {
    neighbours.fold(0.0, |energy, n| {
        energy + energy_contribution(p_radius, p_position, n.position)
    })
}
