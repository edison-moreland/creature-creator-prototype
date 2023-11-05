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

use renderer::surfaces::Sphere;

use crate::renderer::widgets::{CardinalArrows, Grid};
use crate::renderer::{Camera, Renderer};
use crate::surfaces::live_sampling::SamplingSystem;
use crate::surfaces::{Shape, Surface};

mod geometry;
mod renderer;
mod spatial_indexer;
mod surfaces;

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

    desired_radius: f32,
    sampling_system: SamplingSystem,

    surface: Surface,

    grid: Grid,
    arrows: CardinalArrows,
}

impl App {
    fn init(event_loop: &EventLoopWindowTarget<()>, surface: Surface) -> Self {
        let window = WindowBuilder::new()
            .with_title("Creature Creator")
            .with_inner_size(LogicalSize::new(800, 600))
            .build(event_loop)
            .unwrap();

        let renderer = Renderer::new(
            &window,
            Camera::new(point![40.0, 40.0, 40.0], point![0.0, 0.0, 0.0], 60.0),
        );

        let sampling_system = SamplingSystem::new();

        let grid = Grid::new(100.0, 5.0);
        let arrows = CardinalArrows::new(point![0.0, 0.05, 0.0], 25.0);

        App {
            window,
            renderer,
            desired_radius: 0.5,
            sampling_system,
            grid,
            arrows,
            surface,
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
        self.sampling_system
            .update(self.desired_radius, &self.surface);
        let p_duration = start.elapsed();

        let start = Instant::now();

        self.renderer.draw_spheres(
            self.sampling_system
                .positions()
                .map(|(point, normal, radius)| Sphere {
                    center: point.coords.data.0[0],
                    normal: normal.data.0[0],
                    radius,
                })
                .collect::<Vec<Sphere>>()
                .as_slice(),
        );

        self.renderer.draw_widget(&self.grid);
        self.renderer.draw_widget(&self.arrows);
        // self.renderer.draw_widget(&self.surface);
        self.renderer.commit();

        let r_duration = start.elapsed();

        dbg!(p_duration, r_duration);
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = None;

    event_loop
        .run(move |event, event_loop| match event {
            Event::NewEvents(StartCause::Init) => {
                app.replace(App::init(event_loop, surface()));

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
