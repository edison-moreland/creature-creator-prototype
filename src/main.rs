use crate::particle_system::{Parameters, ParticleSystem};
use nalgebra::{point, vector, Vector3};
use rayon::prelude::{ParallelSlice, ParallelSliceMut};
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::StartCause;
use winit::event_loop::{ControlFlow, EventLoopWindowTarget};
use winit::window::Window;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

use crate::relaxation::RelaxationSystem;
use crate::renderer::{Camera, FastBallRenderer, Instance};
use crate::sampling::sample;
use crate::surfaces::{ellipsoid, gradient, rotate, smooth_union, sphere, translate, union};

mod particle_system;
mod pool;
mod relaxation;
mod renderer;
mod sampling;
mod spatial_indexer;
mod surfaces;

fn surface_at(t: f32) -> impl Fn(Vector3<f32>) -> f32 {
    smooth_union(
        sphere(10.0),
        union(
            rotate(
                vector![0.0, (t * 40.0) % 360.0, 0.0],
                smooth_union(
                    translate(vector![-10.0, 0.0, 0.0], ellipsoid(10.0, 5.0, 5.0)),
                    translate(vector![-20.0, 0.0, 0.0], sphere(10.0)),
                    0.5,
                ),
            ),
            translate(
                vector![0.0, (t).sin() * 10.0, 0.0],
                ellipsoid(5.0, 10.0, 5.0),
            ),
        ),
        0.5,
    )

    // smooth_union(
    //     sphere(10.0),
    //     union(
    //         translate(
    //             vector![(t).sin() * 10.0, 0.0, 0.0],
    //             ellipsoid(10.0, 5.0, 5.0),
    //         ),
    //         translate(
    //             vector![0.0, 0.0, (t).cos() * 10.0],
    //             ellipsoid(5.0, 5.0, 10.0),
    //         ),
    //     ),
    //     0.5,
    // )
}

struct App {
    window: Window,

    renderer: FastBallRenderer,

    particle_system: ParticleSystem,
}

impl App {
    fn init(event_loop: &EventLoopWindowTarget<()>) -> Self {
        let window = WindowBuilder::new()
            .with_title("Creature Creator")
            .with_inner_size(LogicalSize::new(600, 300))
            .build(&event_loop)
            .unwrap();

        let renderer = FastBallRenderer::new(
            &window,
            Camera::new(point![40.0, 40.0, 40.0], point![0.0, 0.0, 0.0], 60.0),
        );

        let sample_radius = 1.0;
        let points = sample(surface_at(0.0), vector![0.0, 0.0, 10.0], sample_radius);

        let mut parameters = Parameters::default();
        parameters.desired_repulsion_radius = sample_radius;

        let particle_system = ParticleSystem::new(parameters, points);

        App {
            window,
            renderer,
            particle_system,
        }
    }

    fn scale_factor_changed(&self, scale_factor: f64) {
        self.renderer.rescaled(scale_factor);
    }

    fn resized(&mut self, new_size: PhysicalSize<u32>) {
        self.renderer.resized(new_size);
    }

    fn draw(&mut self) {
        // let surface = surface_at(self.particle_system.time);
        let surface = surface_at(0.0);

        self.particle_system.advance_simulation(&surface);
        self.renderer
            .draw(self.particle_system.positions().map(|(point, radius)| {
                let normal = gradient(&surface, point).normalize();

                Instance {
                    center: point.data.0[0],
                    normal: normal.data.0[0],
                    radius,
                }
            }))
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let mut app = None;

    event_loop.run(move |event, event_loop, cf| match event {
        Event::NewEvents(StartCause::Init) => {
            app = Some(App::init(&event_loop));

            *cf = ControlFlow::Poll;
        }
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *cf = ControlFlow::Exit,
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                app.as_ref().unwrap().scale_factor_changed(scale_factor);
            }
            WindowEvent::Resized(size) => app.as_mut().unwrap().resized(size),
            _ => (),
        },
        Event::MainEventsCleared => app.as_mut().unwrap().draw(),
        _ => (),
    })
}
