use crate::kdtree::KdContainer;
use raylib::prelude::*;
use rayon::prelude::*;

// pub struct RelaxationSystem {
//     position: KdContainer<Vector3>,
//     velocity: Vec<Vector3>,
//     radius: Vec<f32>,
// }
//
// impl RelaxationSystem {
//     pub fn new(particles: Vec<Vector3>, f: impl Fn(Vector3) -> (Vector3, f32)) -> RelaxationSystem {
//         let mut velocity = vec![];
//         velocity.reserve(particles.len());
//
//         let mut radius = vec![];
//         radius.reserve(particles.len());
//
//         for position in &particles {
//             let (p_velocity, p_radius) = f(*position);
//             velocity.push(p_velocity);
//             radius.push(p_radius);
//         }
//
//         RelaxationSystem {
//             position: KdContainer::from_items(particles),
//             velocity,
//             radius,
//         }
//     }
//
//     pub fn update_velocity(
//         &mut self,
//         neighbour_radius: f32,
//         f: impl Fn(Vector3, Vec<&Vector3>) -> Vector3 + Send + Sync,
//     ) {
//         self.velocity
//             .par_iter_mut()
//             .enumerate()
//             .for_each(|(i, velocity)| {
//                 let p = self.position[i];
//
//                 // TODO: Use particle radius*some constant?
//                 let neighbours = self.position.get_items_in_radius(p, neighbour_radius);
//
//                 *velocity = f(p, neighbours)
//             });
//     }
//
//     pub fn update_radius(
//         &mut self,
//         neighbour_radius: f32,
//         f: impl Fn(Vector3, Vec<&Vector3>) -> f32 + Send + Sync
//     ) {
//         self.radius.par_iter_mut().en
//     }
//
//     pub fn update_positions(&mut self, f: impl Fn(Vector3, Vector3, f32) -> Vector3 + Sync) {
//         self.position
//             .items
//             .par_iter_mut()
//             .enumerate()
//             .for_each(|(i, p)| *p = f(*p, self.velocity[i], self.radius[i]));
//
//         self.position.reconstruct()
//     }
//
//     pub fn positions(&self) -> Vec<Vector3> {
//         self.position.items.clone()
//     }
// }

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
