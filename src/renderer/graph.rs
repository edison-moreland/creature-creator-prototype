use std::f32::consts::PI;
use std::ops::{Deref, DerefMut, Mul};

use generational_arena::Arena;
use nalgebra::{Point3, Similarity3, UnitQuaternion, Vector3};

use crate::renderer::surfaces::Shape;
use crate::renderer::lines::Widget;

type NodeId = generational_arena::Index;

pub enum Kind {
    Widget(Widget),
    Shape(Shape),
}

pub struct Node {
    pub transform: Transform,
    kind: Option<Kind>,
}

impl Node {
    pub fn new(kind: Option<Kind>) -> Self {
        Self {
            transform: Transform::identity(),
            kind,
        }
    }
}

type NodeStorage = Arena<(Node, Vec<NodeId>)>;

pub struct NodeRef<'a> {
    nodes: &'a NodeStorage,
    id: NodeId,
}

impl<'a> NodeRef<'a> {
    fn node(&self) -> &(Node, Vec<NodeId>) {
        &self.nodes[self.id]
    }
    pub fn node_id(&self) -> NodeId {
        self.id
    }
    pub fn children(&self) -> Vec<NodeRef> {
        self.node()
            .1
            .iter()
            .map(|i| NodeRef {
                nodes: self.nodes,
                id: *i,
            })
            .collect()
    }
}

pub struct NodeMut<'a> {
    nodes: &'a mut NodeStorage,
    id: NodeId,
}

impl<'a> NodeMut<'a> {
    fn node(&mut self) -> &mut (Node, Vec<NodeId>) {
        &mut self.nodes[self.id]
    }

    pub fn node_id(&self) -> NodeId {
        self.id
    }
    pub fn push(&mut self, node: Node) -> NodeMut {
        let child_index = self.nodes.insert((node, vec![]));

        self.node().1.push(child_index);

        NodeMut {
            nodes: self.nodes,
            id: child_index,
        }
    }

    pub fn push_empty(&mut self) -> NodeMut {
        self.push(Node::new(None))
    }

    pub fn push_widget(&mut self, widget: Widget) -> NodeMut {
        self.push(Node::new(Some(Kind::Widget(widget))))
    }

    pub fn push_shape(&mut self, shape: Shape) -> NodeMut {
        self.push(Node::new(Some(Kind::Shape(shape))))
    }
}

pub struct RenderGraph {
    nodes: NodeStorage,
    root: NodeId,
}

impl RenderGraph {
    pub fn new() -> Self {
        let mut node_storage: NodeStorage = Arena::new();
        let root_id = node_storage.insert((Node::new(None), vec![]));

        RenderGraph {
            nodes: node_storage,
            root: root_id,
        }
    }

    pub fn node(&self, id: NodeId) -> NodeRef {
        NodeRef {
            nodes: &self.nodes,
            id,
        }
    }

    pub fn root(&self) -> NodeRef {
        self.node(self.root)
    }

    pub fn node_mut(&mut self, id: NodeId) -> NodeMut {
        NodeMut {
            nodes: &mut self.nodes,
            id,
        }
    }

    pub fn root_mut(&mut self) -> NodeMut {
        self.node_mut(self.root)
    }

    pub fn walk<F>(&self, mut f: F)
    where
        F: FnMut(Transform, &Kind),
    {
        let mut to_visit = vec![(Transform::identity(), self.root)];

        while let Some((previous_transform, index)) = to_visit.pop() {
            let (node, children) = &self.nodes[index];

            let transform = previous_transform * node.transform;

            if let Some(kind) = &node.kind {
                f(transform, kind);
            }

            for i in children {
                to_visit.push((transform, *i))
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct Transform(Similarity3<f32>);

impl Transform {
    pub fn identity() -> Self {
        Self(Similarity3::identity())
    }

    pub fn apply_point(&self, point: &Point3<f32>) -> Point3<f32> {
        self.0.transform_point(point)
    }

    pub fn apply_vector(&self, vector: &Vector3<f32>) -> Vector3<f32> {
        self.transform_vector(vector)
    }

    pub fn position(&self) -> Point3<f32> {
        Point3::from(self.0.isometry.translation.vector)
    }

    fn set_position(&mut self, new_position: Point3<f32>) {
        self.isometry.translation.vector = new_position.coords
    }

    pub fn rotation(&self) -> Vector3<f32> {
        self.isometry.rotation.scaled_axis() * (180.0 / PI)
    }

    pub fn set_rotation(&mut self, rotation: Vector3<f32>) {
        self.isometry.rotation = UnitQuaternion::from_scaled_axis(rotation * (PI / 180.0))
    }
}

impl From<Similarity3<f32>> for Transform {
    fn from(value: Similarity3<f32>) -> Self {
        Self(value)
    }
}

impl Deref for Transform {
    type Target = Similarity3<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Transform {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Mul for Transform {
    type Output = Transform;

    fn mul(self, rhs: Self) -> Self::Output {
        (self.0 * rhs.0).into()
    }
}
impl Mul for &Transform {
    type Output = Transform;

    fn mul(self, rhs: Self) -> Self::Output {
        (self.0 * rhs.0).into()
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::{point, vector};

    use crate::renderer::graph::Transform;

    #[test]
    fn transform_position() {
        let mut transform = Transform::identity();

        assert_eq!(transform.position(), point![0.0, 0.0, 0.0]);

        transform.set_position(point![10.0, 0.0, 0.0]);

        assert_eq!(transform.position(), point![10.0, 0.0, 0.0]);
        assert_eq!(
            transform.apply_point(&point![0.0, 0.0, 0.0]),
            point![10.0, 0.0, 0.0]
        );
    }

    #[test]
    fn transform_rotation() {
        let mut transform = Transform::identity();

        assert_eq!(transform.rotation(), vector![0.0, 0.0, 0.0]);

        transform.set_rotation(vector![180.0, 0.0, 0.0]);

        assert!(
            dbg!((transform.rotation().abs() - vector![180.0, 0.0, 0.0]).magnitude()) <= 0.0001
        );
        assert!(
            (transform.apply_point(&point![0.0, 0.0, 1.0]) - point![0.0, 0.0, -1.0]).magnitude()
                <= 0.0001
        );
    }
}
