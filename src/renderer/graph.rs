use nalgebra::Transform3;

use crate::renderer::surfaces::Shape;
use crate::renderer::widgets::Widget;

pub enum Kind {
    Widget(Widget),
    Shape(Shape),
}

pub struct Node {
    transform: Transform3<f32>,
    kind: Option<Kind>,
}

impl Node {
    pub fn new(transform: Transform3<f32>, kind: Option<Kind>) -> Self {
        Self { transform, kind }
    }
}

pub struct RenderGraph {
    nodes: Vec<(Node, Vec<usize>)>,
    root: usize,
}

impl RenderGraph {
    pub fn new() -> Self {
        RenderGraph {
            nodes: vec![(
                Node {
                    transform: Transform3::identity(),
                    kind: None,
                },
                vec![],
            )],
            root: 0,
        }
    }

    pub fn insert_under(&mut self, parent: usize, child: Node) -> usize {
        let child_idx = self.nodes.len();
        self.nodes.push((child, vec![]));

        let (_, children) = &mut self.nodes[parent];
        children.push(child_idx);

        child_idx
    }

    pub fn walk<F>(&self, mut f: F)
    where
        F: FnMut(Transform3<f32>, &Kind),
    {
        let mut to_visit = vec![(Transform3::identity(), self.root)];

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
