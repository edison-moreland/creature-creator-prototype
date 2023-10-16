use crate::spatial_indexer::SpatialIndexer;
use raylib::prelude::*;
use std::fmt::Debug;
use std::ops::Index;

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

    fn component(&self, v: &Vector3) -> f32 {
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

pub trait Positioned {
    fn position(&self) -> Vector3;
}

impl Positioned for Vector3 {
    fn position(&self) -> Vector3 {
        *self
    }
}

fn _construct<T: Positioned + Debug + Sync>(
    item_arena: &Vec<T>,
    items: Vec<usize>,
    axis: SplitAxis,
) -> KdTree {
    if items.len() < KD_LEAF_SIZE {
        return KdTree::Leaf(items);
    }

    let (midpoint, left, right) = _split(item_arena, items, axis);

    // let (left_node, right_node) = rayon::join(
    //     || _construct(item_arena, left, axis.next()),
    //     || _construct(item_arena, right, axis.next()),
    // );
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
    item_arena: &Vec<T>,
    mut items: Vec<usize>,
    axis: SplitAxis,
) -> (f32, Vec<usize>, Vec<usize>) {
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
        if axis.component(&item_arena[idx].position()) < midpoint {
            right.push(idx)
        } else {
            left.push(idx)
        }
    }

    (midpoint, left, right)
}

fn _push<T: Positioned + Debug>(
    item_arena: &mut Vec<T>,
    tree: &mut KdTree,
    parent_axis: SplitAxis,
    item: T,
) {
    match tree {
        KdTree::Leaf(l) => {
            item_arena.push(item);
            l.push(item_arena.len() - 1);

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
            if n.axis.component(&item.position()) < n.midpoint {
                _push(item_arena, n.right.as_mut(), n.axis, item);
            } else {
                _push(item_arena, n.left.as_mut(), n.axis, item);
            }
        }
    }
}

fn _any_overlap<T: Positioned + Debug>(
    item_arena: &Vec<T>,
    tree: &KdTree,
    origin: Vector3,
    radius: f32,
) -> bool {
    match tree {
        KdTree::Leaf(l) => {
            for point in l {
                if item_arena[*point].position().distance_to(origin) <= radius {
                    return true;
                }
            }

            false
        }
        KdTree::Node(n) => {
            let component = n.axis.component(&origin);

            ((component + radius >= n.midpoint)
                && _any_overlap(item_arena, n.left.as_ref(), origin, radius))
                || ((component - radius < n.midpoint)
                    && _any_overlap(item_arena, n.right.as_ref(), origin, radius))
        }
    }
}

fn _get_items_in_radius<T: Positioned + Debug>(
    item_arena: &Vec<T>,
    tree: &KdTree,
    origin: Vector3,
    radius: f32,
    items: &mut Vec<usize>,
) {
    match tree {
        KdTree::Leaf(l) => {
            for item_idx in l {
                let item = &item_arena[*item_idx];

                if item.position().distance_to(origin) <= radius {
                    items.push(*item_idx)
                }
            }
        }
        KdTree::Node(n) => {
            let component = n.axis.component(&origin);

            if component + radius >= n.midpoint {
                _get_items_in_radius(item_arena, &n.left, origin, radius, items)
            }

            if component - radius < n.midpoint {
                _get_items_in_radius(item_arena, &n.right, origin, radius, items)
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

    pub fn from_items(items: Vec<T>) -> Self {
        let mut container = KdContainer {
            items,
            tree: KdTree::Leaf(vec![]),
        };

        container.reconstruct();

        return container;
    }

    pub fn push(&mut self, point: T) {
        _push(&mut self.items, &mut self.tree, SplitAxis::X, point)
    }

    pub fn append(&mut self, points: Vec<T>) {
        for point in points {
            self.push(point)
        }
    }

    pub fn any_items_in_radius(&self, point: Vector3, radius: f32) -> bool {
        _any_overlap(&self.items, &self.tree, point, radius)
    }

    pub fn get_items_in_radius(&self, point: Vector3, radius: f32) -> Vec<&T> {
        let mut items = vec![];

        _get_items_in_radius(&self.items, &self.tree, point, radius, &mut items);

        items.iter().map(|i| &self.items[*i]).collect()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn reconstruct(&mut self) {
        self.tree = _construct(&self.items, (0..self.items.len()).collect(), SplitAxis::X)
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

impl SpatialIndexer for KdIndexer {
    fn reindex(&mut self, items: &Vec<Vector3>) {
        self.root = _construct(&items, (0..items.len()).collect(), SplitAxis::X)
    }

    fn get_indices_within(&self, items: &Vec<Vector3>, origin: Vector3, radius: f32) -> Vec<usize> {
        let mut indicies = vec![];

        _get_items_in_radius(items, &self.root, origin, radius, &mut indicies);

        indicies
    }

    fn any_indices_within(&self, items: &Vec<Vector3>, origin: Vector3, radius: f32) -> bool {
        _any_overlap(items, &self.root, origin, radius)
    }
}
