use metal::foreign_types::ForeignTypeRef;
use metal::{DeviceRef, MTLDevice, MTLRenderCommandEncoder, RenderCommandEncoderRef};

pub use lines::line_segments;
pub use pipeline::{LinePipeline, LineSegment};

mod lines;
mod pipeline;

#[swift_bridge::bridge]
mod ffi {
    extern "Rust" {
        type LineSegment;

        #[swift_bridge(init)]
        fn new_ugly(
            a_x: f32,
            a_y: f32,
            a_z: f32,
            b_x: f32,
            b_y: f32,
            b_z: f32,
            color_r: f32,
            color_g: f32,
            color_b: f32,
            thickness: f32,
            segment_size: f32,
            style: u32,
            t_offset: f32,
        ) -> LineSegment;
    }

    // extern "Rust" {
    //     type SwiftLinePipeline;
    //
    //     #[swift_bridge(init)]
    //     fn new(config: AppConfig) -> SwiftLinePipeline;
    //
    //     fn get_user(&self, lookup: UserLookup) -> Option<&User>;
    // }
}

struct SwiftLinePipeline {
    pipeline: LinePipeline,
}

impl SwiftLinePipeline {
    fn new(device: *mut MTLDevice) -> SwiftLinePipeline {
        let device_ref = unsafe { DeviceRef::from_ptr(device) };

        SwiftLinePipeline {
            pipeline: LinePipeline::new(&device_ref),
        }
    }

    fn draw(&mut self, encoder: *mut MTLRenderCommandEncoder, segments: Vec<LineSegment>) {
        let encoder_ref = unsafe { RenderCommandEncoderRef::from_ptr(encoder) };

        self.pipeline.draw(&encoder_ref, segments)
    }
}
