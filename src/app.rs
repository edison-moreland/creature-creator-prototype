use std::time::Instant;

use nalgebra::{point, vector};
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window, WindowBuilder};

use crate::renderer::graph::{NodeId, NodeMut, RenderGraph};
use crate::renderer::lines::{Fill, Line};
use crate::renderer::surfaces::Shape;
use crate::renderer::{Camera, Renderer};

struct Character {
    root_id: NodeId,

    bouncing_id: NodeId,
    rotating_id: NodeId,
}

impl Character {
    fn new(root_node: &mut NodeMut) -> Self {
        let root_id = root_node.node_id();
        root_node.push_shape(Shape::Ellipsoid(vector![10.0, 10.0, 10.0]));
        root_node.push_line(
            Line::new_circle(10.2)
                .fill(Fill::Dashed(0.8))
                .thickness(0.4),
        );

        let mut bouncing_node = root_node.push_empty();
        let bouncing_id = bouncing_node.node_id();
        bouncing_node.push_shape(Shape::Ellipsoid(vector![5.0, 10.0, 5.0]));

        let mut rotating_node = root_node.push_empty();
        let rotating_id = rotating_node.node_id();
        rotating_node.push_line(
            Line::new_circle(20.0)
                .fill(Fill::Dashed(0.8))
                .thickness(0.4),
        );
        rotating_node
            .push_shape(Shape::Ellipsoid(vector![10.0, 5.0, 5.0]))
            .transform()
            .set_position(point![10.0, 0.0, 0.0]);

        let mut bauble_node = rotating_node.push_shape(Shape::Ellipsoid(vector![10.0, 10.0, 10.0]));
        bauble_node.transform().set_position(point![20.0, 0.0, 0.0]);
        bauble_node.push_line(
            Line::new_circle(10.2)
                .fill(Fill::Dashed(0.8))
                .thickness(0.4),
        );
        let mut bauble_arrow = bauble_node.push_line(Line::new_arrow(5.0).thickness(0.4));
        bauble_arrow
            .transform()
            .set_position(point![0.0, 0.0, -10.0]);
        bauble_arrow
            .transform()
            .set_rotation(vector![-90.0, 0.0, 0.0]);

        Self {
            root_id,
            bouncing_id,
            rotating_id,
        }
    }

    fn update_animation(&self, render_graph: &mut RenderGraph, seconds: f32) {
        render_graph
            .node_mut(self.rotating_id)
            .transform()
            .set_rotation(vector![0.0, (seconds * 40.0) % 360.0, 0.0]);

        render_graph
            .node_mut(self.bouncing_id)
            .transform()
            .set_position(point![0.0, seconds.sin() * 10.0, 0.0])
    }
}

fn grid(mut root: NodeMut, size: f32, step: f32) {
    let start = -(size / 2.0);

    let mut grid_line_position = start;
    while grid_line_position <= -start {
        let mut x_line = root.push_line(Line::new(size));
        x_line
            .transform()
            .set_position(point![grid_line_position, 0.0, 0.0]);
        x_line.transform().set_rotation(vector![90.0, 0.0, 0.0]);

        let mut y_line = root.push_line(Line::new(size));
        y_line
            .transform()
            .set_position(point![0.0, 0.0, grid_line_position]);
        y_line.transform().set_rotation(vector![0.0, 0.0, 90.0]);

        grid_line_position += step
    }
}

fn cardinal_arrows(mut root: NodeMut, magnitude: f32) {
    root.push_line(
        Line::new_arrow(magnitude)
            .thickness(0.2)
            .color(vector![1.0, 0.0, 0.0]),
    )
    .transform()
    .set_rotation(vector![0.0, 0.0, -90.0]);

    root.push_line(
        Line::new_arrow(magnitude)
            .thickness(0.2)
            .color(vector![0.0, 1.0, 0.0]),
    );

    root.push_line(
        Line::new_arrow(magnitude)
            .thickness(0.2)
            .color(vector![0.0, 0.0, 1.0]),
    )
    .transform()
    .set_rotation(vector![90.0, 0.0, 0.0]);
}

pub struct App {
    #[allow(dead_code)] // Window is never used after initialization but it can't be dropped
    window: Window,

    start: Instant,
    character: Character,

    renderer: Renderer,
    render_graph: RenderGraph,
}

impl App {
    pub fn init(event_loop: &EventLoopWindowTarget<()>) -> Self {
        let window = WindowBuilder::new()
            .with_title("Creature Creator")
            .with_inner_size(LogicalSize::new(800, 600))
            .build(event_loop)
            .unwrap();

        let renderer = Renderer::new(
            &window,
            Camera::new(point![40.0, 40.0, 40.0], point![0.0, 0.0, 0.0], 60.0),
        );

        let mut render_graph = RenderGraph::new();
        let mut root_node = render_graph.root_mut();

        let mut ui_node = root_node.push_empty();
        grid(ui_node.push_empty(), 100.0, 5.0);
        cardinal_arrows(ui_node.push_empty(), 20.0);

        let character = Character::new(&mut root_node.push_empty());

        App {
            window,
            start: Instant::now(),
            character,
            renderer,
            render_graph,
        }
    }

    pub fn scale_factor_changed(&self, scale_factor: f64) {
        self.renderer.rescaled(scale_factor);
    }

    pub fn resized(&mut self, new_size: PhysicalSize<u32>) {
        self.renderer.resized(new_size);
    }

    fn update(&mut self) {
        let seconds = self.start.elapsed().as_secs_f32();

        self.character
            .update_animation(&mut self.render_graph, seconds);
    }

    pub fn draw(&mut self) {
        self.update();

        let start = Instant::now();
        self.renderer.draw_graph(0.52, &self.render_graph);
        self.renderer.commit();
        let draw_duration = start.elapsed();
        dbg!(draw_duration);
    }
}
