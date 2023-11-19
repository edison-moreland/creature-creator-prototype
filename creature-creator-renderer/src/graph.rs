use generational_arena::Arena;
use nalgebra::Matrix4;

use crate::lines::Line;
use crate::shapes::Shape;
use crate::transform::NodeTransform;

pub type NodeId = generational_arena::Index;

pub enum Kind {
    Line(Line),
    Shape(Shape),
}

pub struct Node {
    pub transform: NodeTransform,
    kind: Option<Kind>,
}

impl Node {
    pub fn new(kind: Option<Kind>) -> Self {
        Self {
            transform: NodeTransform::identity(),
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
    pub fn transform(&self) -> &NodeTransform {
        &self.node().0.transform
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
    pub fn transform(&mut self) -> &mut NodeTransform {
        &mut self.node().0.transform
    }
    pub fn with_transform<F>(&mut self, f: F)
    where
        F: FnOnce(&mut NodeTransform),
    {
        f(&mut self.node().0.transform)
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

    pub fn push_line(&mut self, line: Line) -> NodeMut {
        self.push(Node::new(Some(Kind::Line(line))))
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
        F: FnMut(Matrix4<f32>, &Kind),
    {
        let mut to_visit = vec![(Matrix4::identity(), self.root)];

        while let Some((previous_transform, index)) = to_visit.pop() {
            let (node, children) = &self.nodes[index];

            let transform = previous_transform * node.transform.to_homogeneous();

            if let Some(kind) = &node.kind {
                f(transform, kind);
            }

            for i in children {
                to_visit.push((transform, *i))
            }
        }
    }
}
