use std::f32::consts::PI;
use std::time::Instant;

use nalgebra::{point, vector, Vector3};
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window, WindowBuilder};

use crate::bones::Bone;
use crate::renderer::graph::{NodeId, NodeMut, RenderGraph};
use crate::renderer::lines::Line;
use crate::renderer::surfaces::Shape;
use crate::renderer::{Camera, Renderer};

struct Character {
    root_id: NodeId,

    arm: Bone,
    forearm: Bone,
}

impl Character {
    fn new(render_graph: &mut RenderGraph, root_id: NodeId) -> Self {
        let mut root_node = render_graph.node_mut(root_id);

        root_node.with_transform(|t| {
            t.position = point![0.0, 0.0, 0.0];
            t.rotation = vector![0.0, 45.0, 0.0];
        });
        let root_id = root_node.node_id();

        let arm = Bone::new(root_node.push_empty(), 10.0, |mut s| {
            s.push_shape(Shape::Sphere(0.5));
        });
        let forearm = Bone::new(render_graph.node_mut(arm.next_joint_id), 10.0, |mut s| {
            s.push_shape(Shape::Sphere(0.5));
        });

        Self {
            root_id,
            arm,
            forearm,
        }
    }

    fn update_animation(&self, render_graph: &mut RenderGraph, seconds: f32) {
        let wiggle = oscillation(seconds, 0.75, 0.0, 1.0);

        let mut elbow_node = render_graph.node_mut(self.forearm.joint_id);

        elbow_node.with_transform(|t| {
            t.rotation = Vector3::lerp(&vector![0.0, 0.0, 0.0], &vector![0.0, 0.0, 90.0], wiggle)
        })
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
        x_line.with_transform(|t| {
            t.position = point![grid_line_position, 0.0, 0.0];
            t.rotation = vector![90.0, 0.0, 0.0];
        });

        let mut y_line = root.push_line(Line::new(size));
        y_line.with_transform(|t| {
            t.position = point![0.0, 0.0, grid_line_position];
            t.rotation = vector![0.0, 0.0, 90.0];
        });

        grid_line_position += step
    }
}

fn cardinal_arrows(mut root: NodeMut, magnitude: f32) {
    root.push_line(
        Line::new_arrow(magnitude)
            .thickness(0.2)
            .color(vector![1.0, 0.0, 0.0]),
    )
    .with_transform(|t| {
        t.rotation = vector![0.0, 0.0, -90.0];
    });

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
    .with_transform(|t| {
        t.rotation = vector![90.0, 0.0, 0.0];
    })
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

        let character_node_id = root_node.push_empty().node_id();

        let character = Character::new(&mut render_graph, character_node_id);

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
        self.renderer.draw_graph(0.4, &self.render_graph);
        self.renderer.commit();
        let draw_duration = start.elapsed();
        dbg!(draw_duration);
    }
}
