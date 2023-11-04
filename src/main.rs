use std::time::Instant;

use nalgebra::{point, vector, Rotation3};
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

use crate::relaxation::RelaxationSystem;
use crate::renderer::widgets::{CardinalArrows, Grid, Widget};
use crate::renderer::{Camera, Renderer, Sphere};
use crate::sampling::sample;
use crate::stick_man::{Limb, StickMan};
use crate::surfaces::{seed, Surface};

mod buffer_allocator;
mod plane;
mod relaxation;
mod renderer;
mod sampling;
mod spatial_indexer;
mod stick_man;
mod surfaces;

fn surface() -> impl Surface + Widget {
    let mut stick_man = StickMan::new(
        vector![0.0, 0.0, 0.0],
        vector![1.0, 0.0, 0.0],
        vector![10.0, 15.0],
    );

    stick_man
        .head_mut()
        .attach(Limb::new(Rotation3::new(vector![0.0, 0.0, 1.0]), 5.0))
        .attach(Limb::new(Rotation3::new(vector![0.0, 0.0, -2.0]), 5.0));

    stick_man
        .right_arm_mut()
        .attach(Limb::new(Rotation3::new(vector![1.0, 0.0, 0.0]), 5.0))
        .attach(Limb::new(Rotation3::new(vector![1.0, 0.0, 1.0]), 5.0));

    stick_man
        .left_arm_mut()
        .attach(Limb::new(Rotation3::new(vector![1.0, 0.0, 0.0]), 5.0))
        .attach(Limb::new(Rotation3::new(vector![1.0, 0.0, 1.0]), 5.0));

    stick_man
        .right_leg_mut()
        .attach(Limb::new(Rotation3::new(vector![0.0, 0.0, 1.0]), 7.0))
        .attach(Limb::new(Rotation3::new(vector![0.0, 0.0, -2.0]), 7.0));

    stick_man
        .left_leg_mut()
        .attach(Limb::new(Rotation3::new(vector![0.0, 0.0, 1.0]), 7.0))
        .attach(Limb::new(Rotation3::new(vector![0.0, 0.0, -2.0]), 7.0));

    stick_man.update_debug_strokes();

    stick_man
}

struct App<S> {
    #[allow(dead_code)] // Window is never used after initialization but it can't be dropped
    window: Window,

    renderer: Renderer,

    desired_radius: f32,
    particle_system: RelaxationSystem,

    surface: S,

    grid: Grid,
    arrows: CardinalArrows,
}

impl<S: Surface + Widget> App<S> {
    fn init(event_loop: &EventLoopWindowTarget<()>, surface: S) -> Self {
        let window = WindowBuilder::new()
            .with_title("Creature Creator")
            .with_inner_size(LogicalSize::new(800, 600))
            .build(event_loop)
            .unwrap();

        let renderer = Renderer::new(
            &window,
            Camera::new(point![40.0, 40.0, 40.0], point![0.0, 0.0, 0.0], 60.0),
        );

        let sample_radius = 0.5;

        println!("Initial sampling...");
        let seed = match surface.sample_point() {
            Some(p) => p,
            None => seed(&surface, 0.0),
        };

        let points = sample(&surface, seed, sample_radius);

        println!("Done! Initializing particle system...");
        let particle_system = RelaxationSystem::new(points, sample_radius, &surface);

        let grid = Grid::new(100.0, 5.0);
        let arrows = CardinalArrows::new(vector![0.0, 0.05, 0.0], 25.0);

        App {
            window,
            renderer,
            desired_radius: 0.5,
            particle_system,
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
        self.particle_system
            .update(self.desired_radius, &self.surface);
        let p_duration = start.elapsed();

        let start = Instant::now();

        self.renderer.draw_spheres(
            self.particle_system
                .positions()
                .map(|(point, normal, radius)| Sphere {
                    center: point.data.0[0],
                    normal: normal.data.0[0],
                    radius,
                })
                .collect::<Vec<Sphere>>()
                .as_slice(),
        );

        self.renderer.draw_widget(&self.grid);
        self.renderer.draw_widget(&self.arrows);
        self.renderer.draw_widget(&self.surface);
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
