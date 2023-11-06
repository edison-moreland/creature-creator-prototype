use std::cell::RefCell;
use std::rc::Rc;

use nalgebra::Transform3;

use crate::renderer::surfaces::{Shape, Surface};
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

pub struct NodeRef {
    nodes: Rc<RefCell<Vec<(Node, Vec<usize>)>>>,
    index: usize,
}

impl NodeRef {
    fn with_nodes<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Vec<(Node, Vec<usize>)>) -> R,
    {
        f(self.nodes.borrow().as_ref())
    }

    fn with_nodes_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Vec<(Node, Vec<usize>)>) -> R,
    {
        f(self.nodes.borrow_mut().as_mut())
    }

    pub fn children(&self) -> Vec<NodeRef> {
        self.with_nodes(|nodes| {
            nodes[self.index]
                .1
                .iter()
                .map(|i| NodeRef {
                    nodes: self.nodes.clone(),
                    index: *i,
                })
                .collect()
        })
    }

    pub fn push(&mut self, node: Node) -> NodeRef {
        let child_index = self.with_nodes_mut(|nodes| {
            let child_index = nodes.len();
            nodes.push((node, vec![]));

            child_index
        });

        self.nodes.borrow_mut()[self.index].1.push(child_index);

        NodeRef {
            nodes: self.nodes.clone(),
            index: child_index,
        }
    }

    pub fn push_empty(&mut self, transform: Transform3<f32>) -> NodeRef {
        self.push(Node {
            transform,
            kind: None,
        })
    }

    pub fn push_widget(&mut self, transform: Transform3<f32>, widget: Widget) -> NodeRef {
        self.push(Node {
            transform,
            kind: Some(Kind::Widget(widget)),
        })
    }

    pub fn push_shape(&mut self, transform: Transform3<f32>, shape: Shape) -> NodeRef {
        self.push(Node {
            transform,
            kind: Some(Kind::Shape(shape)),
        })
    }
}

pub struct RenderGraph {
    nodes: Rc<RefCell<Vec<(Node, Vec<usize>)>>>,
    root: usize,
}

impl RenderGraph {
    pub fn new() -> Self {
        RenderGraph {
            nodes: Rc::new(RefCell::new(vec![(
                Node {
                    transform: Transform3::identity(),
                    kind: None,
                },
                vec![],
            )])),
            root: 0,
        }
    }

    pub fn root(&self) -> NodeRef {
        NodeRef {
            nodes: self.nodes.clone(),
            index: self.root,
        }
    }

    pub fn walk<F>(&self, mut f: F)
    where
        F: FnMut(Transform3<f32>, &Kind),
    {
        let mut to_visit = vec![(Transform3::identity(), self.root)];

        while let Some((previous_transform, index)) = to_visit.pop() {
            let (node, children) = &self.nodes.borrow()[index];

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
