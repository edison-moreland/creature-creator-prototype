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

struct App {
    window: Window,

    renderer: FastBallRenderer,
}

impl App {
    fn init(event_loop: &EventLoopWindowTarget<()>) -> Self {
        let window = WindowBuilder::new()
            .with_title("Creature Creator")
            .with_inner_size(LogicalSize::new(600, 300))
            .build(&event_loop)
            .unwrap();

        let renderer = FastBallRenderer::new(&window);

        App { window, renderer }
    }

    fn scale_factor_changed(&self, scale_factor: f64) {
        self.renderer.rescaled(scale_factor);
    }

    fn resized(&self, new_size: PhysicalSize<u32>) {
        self.renderer.resized(new_size);
    }

    fn draw(&self) {
        let instances = vec![
            Instance {
                center: [0.0, 0.0, 0.0],
                radius: 5.0,
                color: [1.0, 0.0, 0.0],
            },
            Instance {
                center: [10.0, 0.0, 0.0],
                radius: 5.0,
                color: [0.0, 1.0, 0.0],
            },
            Instance {
                center: [-10.0, 0.0, 0.0],
                radius: 5.0,
                color: [0.0, 0.0, 1.0],
            },
        ];

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
        Event::MainEventsCleared => app.as_ref().unwrap().draw(),
        _ => (),
    })
}
