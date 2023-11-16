use std::f32::consts::PI;
use std::time::Instant;

use nalgebra::{point, vector, Vector3};
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window, WindowBuilder};

use crate::renderer::graph::{NodeId, NodeMut, RenderGraph};
use crate::renderer::lines::Line;
use crate::renderer::surfaces::Shape;
use crate::renderer::{Camera, Renderer};

struct Character {
    root_id: NodeId,

    right_arm_id: NodeId,
}

impl Character {
    fn new(root_node: &mut NodeMut) -> Self {
        root_node.with_transform(|t| {
            t.set_position(point![0.0, 10.0, 0.0]);
            t.set_rotation(vector![0.0, 45.0, 0.0])
        });
        let root_id = root_node.node_id();

        let chest_size = 10.0;
        let chest_half_size = chest_size / 2.0;
        let mut chest_node = root_node.push_empty();
        chest_node
            .push_line(Line::new(chest_size))
            .with_transform(|t| t.set_position(point![chest_half_size, 0.0, 0.0]));
        chest_node
            .push_line(Line::new(chest_size))
            .with_transform(|t| t.set_position(point![-chest_half_size, 0.0, 0.0]));
        chest_node
            .push_line(Line::new(chest_size))
            .with_transform(|t| {
                t.set_rotation(vector![0.0, 0.0, 90.0]);
                t.set_position(point![0.0, chest_half_size, 0.0])
            });
        chest_node
            .push_line(Line::new(chest_size))
            .with_transform(|t| {
                t.set_rotation(vector![0.0, 0.0, 90.0]);
                t.set_position(point![0.0, -chest_half_size, 0.0])
            });

        // chest_node.push_shape(Shape::Quadratic(matrix![
        //     1.0, 0.0, 0.0, 0.0;
        //     0.0, 1.0, 0.0, 0.0;
        //     0.0, 0.0, 1.0, 0.0;
        //     0.0, 0.0, 0.0, -(3.0f32).powf(2.0);
        // ]));
        chest_node.push_shape(Shape::Ellipsoid(vector![
            chest_half_size * 1.25,
            chest_half_size * 1.25,
            chest_half_size / 2.0
        ]));
        chest_node
            .push_shape(Shape::Ellipsoid(vector![
                chest_half_size * 1.25,
                chest_half_size / 2.0,
                chest_half_size / 2.0
            ]))
            .with_transform(|t| t.set_position(point![0.0, chest_size / 2.0, 0.0]));

        let right_arm_length = 10.0;
        let mut right_arm_node = chest_node.push_empty();
        right_arm_node
            .transform()
            .set_position(point![chest_half_size, chest_half_size, 0.0]);

        let mut right_arm_bone_node = right_arm_node.push_empty();
        right_arm_bone_node
            .transform()
            .set_position(point![0.0, right_arm_length / 2.0, 0.0]);

        right_arm_bone_node.push_line(Line::new(right_arm_length));
        right_arm_bone_node.push_shape(Shape::Ellipsoid(vector![
            right_arm_length / 4.0,
            (right_arm_length / 2.0) * 1.25,
            right_arm_length / 4.0
        ]));

        Self {
            root_id,
            right_arm_id: right_arm_node.node_id(),
        }
    }

    fn update_animation(&self, render_graph: &mut RenderGraph, seconds: f32) {
        let wiggle = oscillation(seconds, 0.75, 0.0, 1.0);

        render_graph
            .node_mut(self.right_arm_id)
            .transform()
            .set_rotation(Vector3::lerp(
                &vector![0.0, 0.0, -5.0],
                &vector![0.0, 0.0, -45.0],
                wiggle,
            ))
    }
}

fn oscillation(seconds: f32, period: f32, min: f32, max: f32) -> f32 {
    assert!(min < max);

    let o = ((((seconds + (period / 4.0)) * (PI / period) * 2.0).sin() / 2.0) + 0.5) * (max - min)
        + min;

    assert!(o >= min);
    assert!(o <= max);

    return o;
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
        cardinal_arrows(ui_node.push_empty(), 5.0);

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
