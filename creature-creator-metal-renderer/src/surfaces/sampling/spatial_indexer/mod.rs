use nalgebra::Point3;

pub mod kd_indexer;

pub trait Positioned {
    fn position(&self) -> Point3<f32>;
}

impl Positioned for Point3<f32> {
    fn position(&self) -> Point3<f32> {
        *self
    }
}

// SpatialIndexer is used to accelerate nearest neighbour searches. It doesn't own any data, just indices
pub trait SpatialIndexer<P: Positioned> {
    // reindex will rebuild the internal index with all items
    fn reindex(&mut self, items: &[P], indices: Vec<usize>);

    // insert_item_index will index the new item at items[index], allowing it to be queried later
    fn insert_item_index(&mut self, items: &[P], index: usize);

    // remove_item_index will remove the index for items[index], so it can no longer be queried for
    fn remove_item_index(&mut self, items: &[P], index: usize);

    // get_indices_within will return the index of all items within `radius` of `origin`
    fn get_indices_within(&self, items: &[P], origin: Point3<f32>, radius: f32) -> Vec<usize>;

    // any_indices_within will return true if there any items within `radius` of `origin`
    fn any_indices_within(&self, items: &[P], origin: Point3<f32>, radius: f32) -> bool;
}
