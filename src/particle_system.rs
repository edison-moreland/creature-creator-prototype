use crate::pool::{PoolAllocator, StackPoolAllocator};
use nalgebra::{vector, Vector3};
use rand::random;

use crate::relaxation::{
    constrain_to_surface, energy_contribution, should_die, should_fission_energy,
    should_fission_radius,
};

const MAX_PARTICLES: usize = 10000;

#[derive(Debug, Copy, Clone)]
struct Particle {
    position: Vector3<f32>,
    velocity: Vector3<f32>,
    radius: f32,
}

struct ParticleStorage<A> {
    index_allocator: A,

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

impl<A> ParticleStorage<A>
where
    A: PoolAllocator<MAX_PARTICLES>,
{
    fn init_with_samples(mut pa: A, samples: Vec<Vector3<f32>>, radius: f32) -> Self {
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
                pool_a[i].velocity = vector![random(), random(), random()];

                i
            })
            .collect();

        let pool_b = pool_a.clone();

        ParticleStorage {
            index_allocator: pa,
            active_particles,
            pool_a,
            pool_b,
            _active_pool: true,
        }
    }

    fn toggle_set(&mut self) {
        self._active_pool = !self._active_pool
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

            // TODO: Nearest neighbour :3
            let neighbours: Vec<Particle> = self
                .active_particles
                .iter()
                .map_while(|k| {
                    if *k != j {
                        Some(self.inactive_set()[*k])
                    } else {
                        None
                    }
                })
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
                    self.inactive_set_mut()[i] = p1;
                    self.inactive_set_mut()[sibling_i] = p2;
                }
            }
        }

        particles_killed.sort();
        for j in particles_killed.iter().rev() {
            self.index_allocator.remove(self.active_particles[*j]);
            self.active_particles.remove(*j);
        }

        self.toggle_set();
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
            equilibrium_speed: 100.0,
            fission_faction: 0.2,
            death_fraction: 0.7,
            iterations_per_step: 1,
            neighbour_radius: 3.0,
        }
    }
}

pub struct ParticleSystem {
    parameters: Parameters,
    storage: ParticleStorage<StackPoolAllocator<MAX_PARTICLES>>,
    pub time: f32,
}

impl ParticleSystem {
    pub fn new(parameters: Parameters, samples: Vec<Vector3<f32>>) -> Self {
        let storage = ParticleStorage::init_with_samples(
            StackPoolAllocator::new(),
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
                    if particle.velocity.magnitude()
                        < particle.radius * self.parameters.equilibrium_speed
                    {
                        if should_die(particle.radius, self.parameters.desired_repulsion_radius) {
                            return ParticleUpdate::Kill;
                        }

                        let re = repulsion_energy(&particle, neighbours.iter());
                        if should_fission_energy(
                            particle.radius,
                            re,
                            self.parameters.desired_repulsion_radius,
                        ) || should_fission_radius(
                            particle.radius,
                            self.parameters.desired_repulsion_radius,
                        ) {
                            let new_radius = particle.radius / (2.0_f32).sqrt();

                            let new_velocity = vector![random(), random(), random()]
                                .normalize()
                                .scale(particle.radius);

                            return ParticleUpdate::Fission(
                                Particle {
                                    radius: new_radius,
                                    velocity: new_velocity,
                                    position: particle.position,
                                },
                                Particle {
                                    radius: new_radius,
                                    velocity: -new_velocity,
                                    position: particle.position,
                                },
                            );
                        }
                    }

                    let velocity = constrain_to_surface(
                        &surface,
                        particle.position,
                        particle_velocity(&particle, neighbours.iter()),
                    );

                    let position =
                        particle.position + particle.velocity.scale(self.parameters.time_step);

                    let radius = particle_radius(&self.parameters, &particle, neighbours.iter());

                    ParticleUpdate::Live(Particle {
                        velocity,
                        position,
                        radius,
                    })
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
    p: &Particle,
    neighbours: impl Iterator<Item = &'a Particle> + Clone,
) -> f32 {
    let re = repulsion_energy(p, neighbours.clone());

    // desired change in energy
    let re_delta = -(parameters.feedback * (re - parameters.desired_energy()));

    // change in energy with respect to change in radius
    let di_ai = (1.0 / p.radius.powf(3.0))
        * neighbours.fold(0.0, |sum, n| {
            let dist = (p.position - n.position).magnitude().powf(2.0);

            sum + (dist * energy_contribution(p.radius, p.position, n.position))
        });

    // Radius change to bring us to desired energy
    let radius_delta = re_delta / (di_ai + 10.0);

    p.radius + (radius_delta * parameters.time_step)
}

fn repulsion_energy<'a>(
    p: &Particle,
    neighbours: impl Iterator<Item = &'a Particle> + Clone,
) -> f32 {
    neighbours.fold(0.0, |energy, n| {
        energy + energy_contribution(p.radius, p.position, n.position)
    })
}
