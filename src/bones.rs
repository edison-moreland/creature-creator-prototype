use nalgebra::vector;

use crate::renderer::graph::{NodeId, NodeMut};
use crate::renderer::lines::Line;
use crate::renderer::surfaces::Shape;

pub struct Bone {
    // This is the node that gets rotated to move the bone
    pub joint_id: NodeId,

    // This is the node that child bones attach to
    pub next_joint_id: NodeId,

    // This is the node that surfaces get put in
    bone_id: NodeId,

    size: f32,
}

impl Bone {
    pub fn new<S>(mut joint_node: NodeMut, size: f32, skin: S) -> Self
    where
        S: FnOnce(NodeMut),
    {
        let joint_id = joint_node.node_id();

        let mut next_joint = joint_node.push_empty();
        next_joint.with_transform(|t| t.position.y = size);
        let next_joint_id = next_joint.node_id();

        // Scaling the bone node means the surface and any debug lines will move as size changes
        let mut bone_node = joint_node.push_empty();
        bone_node.with_transform(|t| {
            t.position.y = size / 2.0;
            t.scale = vector![size / 2.0, size, size / 2.0];
        });

        // Debug lines for the bones
        bone_node
            .push_line(Line::new_circle(0.5).color(vector![1.0, 0.0, 0.0]))
            .with_transform(|t| t.position.y += 0.5);
        bone_node
            .push_line(Line::new_circle(0.5).color(vector![0.0, 1.0, 0.0]))
            .with_transform(|t| t.position.y -= 0.5);
        bone_node.push_line(Line::new(1.0));

        let mut skin_node = bone_node.push_empty();
        skin_node.with_transform(|t| {
            // Y is scaled slightly so the skin from two bones connects
            // This scaling isn't applied on the bone node because we don't want
            // to affect debug markers
            t.scale.y += 0.2
        });

        skin(skin_node);

        Self {
            size,
            joint_id,
            next_joint_id,
            bone_id: bone_node.node_id(),
        }
    }
}
