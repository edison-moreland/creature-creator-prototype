use std::time::Instant;

use nalgebra::{point, vector, Transform3};
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

use crate::renderer::graph::{Kind, Node, RenderGraph};
use crate::renderer::surfaces::{Shape, Surface};
use crate::renderer::widgets::{cardinal_arrows, grid, Stroke, Style, Widget};
use crate::renderer::{Camera, Renderer};

mod geometry;
mod renderer;
mod spatial_indexer;

fn surface() -> Surface {
    let mut s = Surface::new();

    s.push(
        Transform3::identity(),
        Shape::Ellipsoid(vector![10.0, 10.0, 10.0]),
    );

    s
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

        let render_graph = RenderGraph::new();
        let mut root_node = render_graph.root();

        let mut ui_node = root_node.push_empty(Transform3::identity());
        ui_node.push_widget(Transform3::identity(), grid(100.0, 5.0));
        ui_node.push_widget(Transform3::identity(), cardinal_arrows(20.0));

        // TODO: How do I pass a god damn translation?
        let mut character_node = root_node.push_empty(Transform3::identity());
        character_node.push_shape(
            Transform3::identity(),
            Shape::Ellipsoid(vector![10.0, 10.0, 10.0]),
        );
        character_node.push_widget(
            Transform3::identity(),
            // TODO: \/ this api fucking sucks
            Widget::new_with(|w| {
                w.set_palette(vec![Style::new(vector![0.0, 0.0, 0.0], 0.5, 0.0)]);
                w.stroke(
                    0,
                    Stroke::Circle {
                        origin: point![0.0, 0.0, 0.0],
                        normal: vector![0.0, 1.0, 0.0],
                        radius: 10.5,
                    },
                )
            }),
        );

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
