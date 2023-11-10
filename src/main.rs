use std::time::Instant;

use nalgebra::{point, vector};
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::StartCause;
use winit::event_loop::{ControlFlow, EventLoopWindowTarget};
use winit::keyboard::{Key, NamedKey};
use winit::window::Window;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

use crate::renderer::graph::{NodeMut, RenderGraph};
use crate::renderer::lines::{Fill, Line};
use crate::renderer::surfaces::Shape;
use crate::renderer::{Camera, Renderer};

mod geometry;
mod renderer;
mod spatial_indexer;

fn character(mut character_node: NodeMut) {
    character_node
        .transform()
        .set_position(point![0.0, 10.0, 0.0]);
    character_node.push_shape(Shape::Ellipsoid(vector![10.0, 10.0, 10.0]));
    character_node.push_line(Line::new_circle(
        10.1,
        Fill::Dashed(0.4),
        0.2,
        vector![0.0, 0.0, 0.0],
    ));
}

fn grid(mut root: NodeMut, size: f32, step: f32) {
    let start = -(size / 2.0);

    let mut grid_line_position = start;
    while grid_line_position <= -start {
        let mut x_line = root.push_line(Line::new(size, Fill::Solid, 0.1, vector![0.0, 0.0, 0.0]));
        x_line
            .transform()
            .set_position(point![grid_line_position, 0.0, 0.0]);
        x_line.transform().set_rotation(vector![90.0, 0.0, 0.0]);

        let mut y_line = root.push_line(Line::new(size, Fill::Solid, 0.1, vector![0.0, 0.0, 0.0]));
        y_line
            .transform()
            .set_position(point![0.0, 0.0, grid_line_position]);
        y_line.transform().set_rotation(vector![0.0, 0.0, 90.0]);

        grid_line_position += step
    }
}

fn cardinal_arrows(mut root: NodeMut, magnitude: f32) {
    root.push_line(Line::new_arrow(
        magnitude,
        Fill::Solid,
        0.2,
        vector![1.0, 0.0, 0.0],
    ))
    .transform()
    .set_rotation(vector![0.0, 0.0, -90.0]);

    root.push_line(Line::new_arrow(
        magnitude,
        Fill::Solid,
        0.2,
        vector![0.0, 1.0, 0.0],
    ));

    root.push_line(Line::new_arrow(
        magnitude,
        Fill::Solid,
        0.2,
        vector![0.0, 0.0, 1.0],
    ))
    .transform()
    .set_rotation(vector![90.0, 0.0, 0.0]);
}

struct App {
    #[allow(dead_code)] // Window is never used after initialization but it can't be dropped
    window: Window,

    renderer: Renderer,
    render_graph: RenderGraph,
}

impl App {
    fn init(event_loop: &EventLoopWindowTarget<()>) -> Self {
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

        character(root_node.push_empty());

        App {
            window,
            renderer,
            render_graph,
        }
    }

    fn scale_factor_changed(&self, scale_factor: f64) {
        self.renderer.rescaled(scale_factor);
    }

    fn resized(&mut self, new_size: PhysicalSize<u32>) {
        self.renderer.resized(new_size);
    }

    fn draw(&mut self) {
        let start = Instant::now();

        self.renderer.draw_graph(0.5, &self.render_graph);
        self.renderer.commit();

        let draw_duration = start.elapsed();
        dbg!(draw_duration);
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = None;

    event_loop
        .run(move |event, event_loop| match event {
            Event::NewEvents(StartCause::Init) => {
                app.replace(App::init(event_loop));

                event_loop.set_control_flow(ControlFlow::Poll)
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    app.as_ref().unwrap().scale_factor_changed(scale_factor);
                }
                WindowEvent::Resized(size) => app.as_mut().unwrap().resized(size),
                WindowEvent::KeyboardInput { event, .. } => {
                    if event.logical_key == Key::Named(NamedKey::Escape) {
                        event_loop.exit()
                    }
                }
                _ => (),
            },
            Event::AboutToWait => app.as_mut().unwrap().draw(),
            _ => (),
        })
        .unwrap()
}
