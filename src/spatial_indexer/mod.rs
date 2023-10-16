pub mod kd_indexer;

use raylib::prelude::*;

// SpatialIndexer is used to accelerate nearest neighbour searches. It doesn't own any data, just indices
pub trait SpatialIndexer {
    fn reindex(&mut self, items: &Vec<Vector3>);

    fn get_indices_within(&self, items: &Vec<Vector3>, origin: Vector3, radius: f32) -> Vec<usize>;

    fn any_indices_within(&self, items: &Vec<Vector3>, origin: Vector3, radius: f32) -> bool;
}
