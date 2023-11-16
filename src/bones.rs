use crate::renderer::graph::{NodeId, NodeMut, RenderGraph};
use crate::renderer::lines::Line;
use nalgebra::point;

struct Bone {
    // This is the node that gets rotated to move the bone
    joint_id: NodeId,

    // This is the node that surfaces get put in
    bone_id: NodeId,

    size: f32,
}

impl Bone {
    fn new(mut joint_node: NodeMut, size: f32) -> Self {
        let joint_id = joint_node.node_id();
        let mut bone_node = joint_node.push_empty();
        bone_node.with_transform(|t| {
            t.set_position(point![0.0, size / 2.0, 0.0]);
            t.set_scaling(size)
        });

        bone_node.push_line(Line::new(1.0));

        Self {
            size,
            joint_id,
            bone_id: bone_node.node_id(),
        }
    }
}
