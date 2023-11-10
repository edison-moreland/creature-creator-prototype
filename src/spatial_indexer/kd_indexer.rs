use std::fmt::Debug;
use std::ops::Index;

use nalgebra::Point3;

use crate::spatial_indexer::{Positioned, SpatialIndexer};

// KD_LEAF_SIZE controls the max size of leaf nodes. 100 was chosen after some testing
const KD_LEAF_SIZE: usize = 100;

#[derive(Debug, Copy, Clone)]
enum SplitAxis {
    X,
    Y,
    Z,
}

impl SplitAxis {
    fn next(&self) -> SplitAxis {
        match self {
            SplitAxis::X => SplitAxis::Y,
            SplitAxis::Y => SplitAxis::Z,
            SplitAxis::Z => SplitAxis::X,
        }
    }

    fn component(&self, v: &Point3<f32>) -> f32 {
        match self {
            SplitAxis::X => v.x,
            SplitAxis::Y => v.y,
            SplitAxis::Z => v.z,
        }
    }
}

#[derive(Debug)]
enum KdTree {
    Leaf(Vec<usize>),
    Node(KdNode),
}

#[derive(Debug)]
struct KdNode {
    axis: SplitAxis,
    midpoint: f32,

    right: Box<KdTree>,
    left: Box<KdTree>,
}

fn _construct<T: Positioned + Debug + Sync>(
    item_arena: &[T],
    items: Vec<usize>,
    axis: SplitAxis,
) -> KdTree {
    if items.len() < KD_LEAF_SIZE {
        return KdTree::Leaf(items);
    }

    let (midpoint, left, right) = _split(item_arena, items, axis);

    let (left_node, right_node) = (
        _construct(item_arena, left, axis.next()),
        _construct(item_arena, right, axis.next()),
    );

    KdTree::Node(KdNode {
        axis,
        midpoint,
        right: Box::new(right_node),
        left: Box::new(left_node),
    })
}

fn _split<T: Positioned + Debug>(
    item_arena: &[T],
    mut items: Vec<usize>,
    axis: SplitAxis,
) -> (f32, Vec<usize>, Vec<usize>) {
    // TODO: This could probably be fast using select and split_off, but construction isn't a bottleneck
    let item_count = items.len();
    let midpoint = {
        let mut total = 0.0;
        for idx in &mut *items {
            total += axis.component(&item_arena[*idx].position())
        }

        total / (item_count as f32)
    };

    let mut left = vec![];
    let mut right = vec![];

    for idx in items.drain(..) {
        if axis.component(&item_arena[idx].position()) > midpoint {
            right.push(idx)
        } else {
            left.push(idx)
        }
    }

    (midpoint, left, right)
}

fn _insert_item_index<T: Positioned + Debug>(
    item_arena: &[T],
    tree: &mut KdTree,
    parent_axis: SplitAxis,
    index: usize,
) {
    match tree {
        KdTree::Leaf(l) => {
            l.push(index);

            let leaf_size = l.len();
            if leaf_size >= KD_LEAF_SIZE {
                // This leaf has gotten too large, we must split it into a node

                let axis = parent_axis.next();
                let (midpoint, left, right) = _split(item_arena, l.clone(), axis);

                *tree = KdTree::Node(KdNode {
                    axis,
                    midpoint,
                    right: Box::new(KdTree::Leaf(right)),
                    left: Box::new(KdTree::Leaf(left)),
                })
            }
        }
        KdTree::Node(n) => {
            // Point needs to be inserted into one side
            if n.axis.component(&item_arena[index].position()) > n.midpoint {
                _insert_item_index(item_arena, n.right.as_mut(), n.axis, index);
            } else {
                _insert_item_index(item_arena, n.left.as_mut(), n.axis, index);
            }
        }
    }
}

fn _remove_item_index<T: Positioned + Debug>(item_arena: &[T], tree: &mut KdTree, index: usize) {
    match tree {
        KdTree::Leaf(l) => {
            // Find the index of the index
            // TODO: If the index was sorted, we could use a binary_search
            let mut index_index: isize = -1;
            for (i, value) in l.iter().enumerate() {
                if *value == index {
                    index_index = i as isize;
                }
            }

            l.remove(index_index as usize);

            if l.is_empty() {
                panic!("handle empty leaf!")
            }
        }
        KdTree::Node(n) => {
            // Point needs to be inserted into one side
            if n.axis.component(&item_arena[index].position()) > n.midpoint {
                _remove_item_index(item_arena, n.right.as_mut(), index);
            } else {
                _remove_item_index(item_arena, n.left.as_mut(), index);
            }
        }
    }
}

fn _any_indices_within<T: Positioned + Debug>(
    item_arena: &[T],
    tree: &KdTree,
    origin: Point3<f32>,
    radius: f32,
) -> bool {
    match tree {
        KdTree::Leaf(l) => {
            for point in l {
                if (item_arena[*point].position() - origin).magnitude() <= radius {
                    return true;
                }
            }

            false
        }
        KdTree::Node(n) => {
            let component = n.axis.component(&origin);

            ((component - radius <= n.midpoint)
                && _any_indices_within(item_arena, n.left.as_ref(), origin, radius))
                || ((component + radius > n.midpoint)
                && _any_indices_within(item_arena, n.right.as_ref(), origin, radius))
        }
    }
}

fn _get_indices_within<T: Positioned + Debug>(
    item_arena: &[T],
    tree: &KdTree,
    origin: Point3<f32>,
    radius: f32,
    items: &mut Vec<usize>,
) {
    match tree {
        KdTree::Leaf(l) => {
            items.extend(
                l.iter()
                    .filter(|i| (item_arena[**i].position() - origin).magnitude() <= radius),
            );
        }
        KdTree::Node(n) => {
            let component = n.axis.component(&origin);

            if component - radius <= n.midpoint {
                _get_indices_within(item_arena, &n.left, origin, radius, items)
            }

            if component + radius > n.midpoint {
                _get_indices_within(item_arena, &n.right, origin, radius, items)
            }
        }
    }
}

// KdContainer is legacy, but SpatialIndexer interface doesn't work well when new points are being added
#[derive(Debug)]
pub struct KdContainer<T: Positioned + Debug> {
    pub items: Vec<T>,

    tree: KdTree,
}

impl<T> Index<usize> for KdContainer<T>
    where
        T: Positioned + Debug,
{
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

impl<T> KdContainer<T>
    where
        T: Positioned + Debug + Copy + Sync + Send,
{
    pub fn new() -> Self {
        KdContainer {
            items: vec![],
            tree: KdTree::Leaf(vec![]),
        }
    }

    pub fn push(&mut self, point: T) {
        self.items.push(point);

        let index = self.items.len() - 1;

        _insert_item_index(&self.items, &mut self.tree, SplitAxis::X, index)
    }

    pub fn append(&mut self, points: Vec<T>) {
        for point in points {
            self.push(point)
        }
    }

    pub fn any_items_in_radius(&self, point: Point3<f32>, radius: f32) -> bool {
        _any_indices_within(&self.items, &self.tree, point, radius)
    }
}

// KdIndexer uses a KdTree to provide spatial indexing
pub struct KdIndexer {
    root: KdTree,
}

impl KdIndexer {
    pub fn new() -> Self {
        KdIndexer {
            root: KdTree::Leaf(vec![]),
        }
    }
}

impl<P: Positioned + Debug + Sync> SpatialIndexer<P> for KdIndexer {
    fn reindex(&mut self, items: &[P], indices: Vec<usize>) {
        // (0..items.len()).collect()
        self.root = _construct(items, indices, SplitAxis::X)
    }

    fn insert_item_index(&mut self, items: &[P], index: usize) {
        _insert_item_index(items, &mut self.root, SplitAxis::X, index)
    }

    fn remove_item_index(&mut self, items: &[P], index: usize) {
        _remove_item_index(items, &mut self.root, index)
    }

    fn get_indices_within(&self, items: &[P], origin: Point3<f32>, radius: f32) -> Vec<usize> {
        let mut indicies = vec![];

        _get_indices_within(items, &self.root, origin, radius, &mut indicies);

        indicies
    }

    fn any_indices_within(&self, items: &[P], origin: Point3<f32>, radius: f32) -> bool {
        _any_indices_within(items, &self.root, origin, radius)
    }
}
