use std::time::Instant;

use nalgebra::{point, vector, Vector3};
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
use crate::renderer::{Camera, Renderer, Sphere, Widget};
use crate::sampling::sample;
use crate::surfaces::primitives::{ellipsoid, rotate, smooth_union, sphere, translate, union};
use crate::surfaces::{Surface, SurfaceFn};

mod buffer_allocator;
mod plane;
mod relaxation;
mod renderer;
mod sampling;
mod spatial_indexer;
mod surfaces;

fn surface() -> impl Surface {
    SurfaceFn::new(vector![0.0, 0.0, 10.0], |t: f32, p: Vector3<f32>| -> f32 {
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
        )(p)
    })
}

struct App<S> {
    window: Window,

    renderer: Renderer,
    // surface: S,
    t: f32,
    desired_radius: f32,
    particle_system: RelaxationSystem<S>,
}

impl<S: Surface> App<S> {
    fn init(event_loop: &EventLoopWindowTarget<()>, surface: S) -> Self {
        let window = WindowBuilder::new()
            .with_title("Creature Creator")
            .with_inner_size(LogicalSize::new(600, 300))
            .build(event_loop)
            .unwrap();

        let renderer = Renderer::new(
            &window,
            Camera::new(point![40.0, 40.0, 40.0], point![0.0, 0.0, 0.0], 60.0),
        );

        let sample_radius = 0.5;

        let points = sample(&surface, surface.sample_point(), sample_radius);
        let particle_system = RelaxationSystem::new(points, sample_radius, surface);

        App {
            window,
            renderer,
            t: 0.0,
            desired_radius: 0.4,
            particle_system,
            // surface,
        }
    }

    fn scale_factor_changed(&self, scale_factor: f64) {
        self.renderer.rescaled(scale_factor);
    }

    fn resized(&mut self, new_size: PhysicalSize<u32>) {
        self.renderer.resized(new_size);
    }

    fn draw(&mut self) {
        // self.t += 0.03;
        // let surface = surface_at(self.t);

        let start = Instant::now();
        self.particle_system.update(self.desired_radius);
        let p_duration = start.elapsed();

        let start = Instant::now();
        let particles: Vec<Sphere> = self
            .particle_system
            .positions()
            .map(|(point, normal, radius)| Sphere {
                center: point.data.0[0],
                normal: normal.data.0[0],
                radius,
            })
            .collect();

        let mut widgets = self.grid_widgets();
        widgets.push(Widget::Circle {
            origin: vector![0.0, 0.0, 0.0],
            normal: vector![0.0, 1.0, 0.0],
            color: vector![1.0, 0.0, 0.0],
            radius: 30.0,
        });

        self.renderer.draw(&particles, &widgets);
        let r_duration = start.elapsed();

        dbg!(p_duration, r_duration);
    }

    fn grid_widgets(&self) -> Vec<Widget> {
        let grid_size = 100.0f32;
        let grid_step = 5.0f32;

        let mut grid_widgets = vec![];
        grid_widgets.reserve(((grid_size / grid_step) * 2.0) as usize);

        let start = -(grid_size / 2.0);

        let mut grid_line_position = start;
        while grid_line_position <= -start {
            grid_widgets.push(Widget::Line {
                start: vector![grid_line_position, 0.0, -start],
                end: vector![grid_line_position, 0.0, start],
                color: vector![0.0, 0.0, 0.0],
            });
            grid_widgets.push(Widget::Line {
                start: vector![-start, 0.0, grid_line_position],
                end: vector![start, 0.0, grid_line_position],
                color: vector![0.0, 0.0, 0.0],
            });

            grid_line_position += grid_step
        }

        grid_widgets
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
                _ => (),
            },
            Event::AboutToWait => app.as_mut().unwrap().draw(),
            _ => (),
        })
        .unwrap()
}
