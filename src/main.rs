use nalgebra::{vector, Vector3};
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

use crate::renderer::{FastBallRenderer, Instance};

mod relaxation;
mod renderer;
mod sampling;
mod spatial_indexer;
mod surfaces;
use crate::relaxation::RelaxationSystem;
use crate::sampling::sample;
use crate::surfaces::{ellipsoid, gradient, smooth_union, sphere, translate, union};

fn surface_at(t: f32) -> impl Fn(Vector3<f32>) -> f32 {
    smooth_union(
        sphere(10.0),
        union(
            translate(
                vector![(t).sin() * 10.0, 0.0, 0.0],
                ellipsoid(10.0, 5.0, 5.0),
            ),
            translate(
                vector![0.0, 0.0, (t).cos() * 10.0],
                ellipsoid(5.0, 5.0, 10.0),
            ),
        ),
        0.5,
    )
}

// fn main() {
//     let (mut rl, thread) = raylib::init().size(640, 480).title("Hello, World").build();
//
//     let camera = Camera3D::perspective(
//         Vector3::new(25.0, 25.0, 25.0),
//         Vector3::new(0.0, 0.0, 0.0),
//         Vector3::up(),
//         40.0,
//     );
//
//     let seed = rvec3(0.0, 10.0, 0.0);
//     let sample_radius = 0.5;
//
//     let mut t = 0.0;
//     let surface = surface_at(t);
//     let points = sample(surface, seed, sample_radius);
//
//     let mut particles = RelaxationSystem::new(points, sample_radius);
//
//     while !rl.window_should_close() {
//         let mut d = rl.begin_drawing(&thread);
//
//         d.clear_background(Color::WHITE);
//
//         t += 0.03;
//         let surface = surface_at(t);
//
//         {
//             let mut d3d = d.begin_mode3D(camera);
//             for (point, radius) in particles.positions() {
//                 let normal = gradient(&surface, point).normalized();
//
//                 let point_color = Color::color_from_normalized(Vector4::new(
//                     normal.x.abs(),
//                     normal.y.abs(),
//                     normal.z.abs(),
//                     1.0,
//                 ));
//
//                 d3d.draw_sphere(
//                     point - normal.scale_by(radius * 2.0),
//                     radius * 2.0,
//                     point_color,
//                 )
//             }
//         }
//
//         particles.update(sample_radius, &surface);
//
//         d.draw_fps(0, 0);
//     }
// }

struct App {
    window: Window,

    renderer: FastBallRenderer,

    t: f32,
    desired_radius: f32,
    particle_system: RelaxationSystem,
}

impl App {
    fn init(event_loop: &EventLoopWindowTarget<()>) -> Self {
        let window = WindowBuilder::new()
            .with_title("Creature Creator")
            .with_inner_size(LogicalSize::new(600, 300))
            .build(&event_loop)
            .unwrap();

        let renderer = FastBallRenderer::new(&window);

        let seed = vector![0.0, 10.0, 0.0];
        let sample_radius = 0.5;

        let surface = surface_at(0.0);
        let points = sample(surface, seed, sample_radius);

        let particle_system = RelaxationSystem::new(points, sample_radius);

        App {
            window,
            renderer,
            t: 0.0,
            desired_radius: 0.3,
            particle_system,
        }
    }

    fn scale_factor_changed(&self, scale_factor: f64) {
        self.renderer.rescaled(scale_factor);
    }

    fn resized(&self, new_size: PhysicalSize<u32>) {
        self.renderer.resized(new_size);
    }

    // fn update(&mut self) {
    //
    // }

    fn draw(&mut self) {
        self.t += 0.03;
        let surface = surface_at(self.t);

        self.particle_system.update(self.desired_radius, &surface);

        let mut instances: Vec<Instance> = self
            .particle_system
            .positions()
            .map(|(point, radius)| {
                let normal = gradient(&surface, point).normalize();

                let origin = point - normal.scale(radius * 2.0);

                Instance {
                    center: [origin.x, origin.y, origin.z],
                    radius: radius,
                    color: [normal.x.abs(), normal.y.abs(), normal.z.abs()],
                }
            })
            .collect();

        instances.par_sort_unstable_by(|r, l| r.center[2].total_cmp(&l.center[2]).reverse());

        self.renderer.draw(instances)
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
            WindowEvent::Resized(size) => app.as_ref().unwrap().resized(size),
            _ => (),
        },
        Event::MainEventsCleared => app.as_mut().unwrap().draw(),
        _ => (),
    })
}
