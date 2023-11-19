use winit::dpi::PhysicalSize;

// These are the only exports
pub use camera::Camera;
pub use graph::{Kind, NodeId, NodeMut, NodeRef, RenderGraph};
pub use transform::NodeTransform;

mod camera;
mod graph;
pub mod lines;
pub mod shapes;
mod transform;

pub trait Renderer {
    // fn new(window: &Window, camera: Camera) -> Self;
    fn resized(&mut self, new_size: PhysicalSize<u32>);
    fn rescaled(&mut self, new_scale_factor: f64);
    fn draw(&mut self, graph: &RenderGraph);
}
