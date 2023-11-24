use metal::foreign_types::ForeignTypeRef;
use metal::{DeviceRef, MTLDevice, MTLRenderCommandEncoder, RenderCommandEncoderRef};

pub use lines::line_segments;
pub use pipeline::{LinePipeline, LineSegment};

mod lines;
mod pipeline;

// #[rustfmt::skip]
#[allow(clippy::all)]
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

    extern "Rust" {
        type SwiftLineSegments;
        #[swift_bridge(init)]
        fn new() -> SwiftLineSegments;

        fn push(&mut self, segment: LineSegment);

        fn clear(&mut self);
    }

    extern "Rust" {
        type SwiftLinePipeline;

        #[swift_bridge(init)]
        fn new(device: *mut MTLDevice) -> SwiftLinePipeline;

        fn draw(&mut self, encoder: *mut MTLRenderCommandEncoder, segments: &SwiftLineSegments);
    }
}

pub struct SwiftLineSegments {
    segments: Vec<LineSegment>,
}

impl SwiftLineSegments {
    pub fn new() -> Self {
        Self { segments: vec![] }
    }

    pub fn push(&mut self, segment: LineSegment) {
        self.segments.push(segment);
    }

    pub fn clear(&mut self) {
        self.segments.clear()
    }
}

pub struct SwiftLinePipeline {
    pipeline: LinePipeline,
}

impl SwiftLinePipeline {
    pub fn new(device: *mut MTLDevice) -> SwiftLinePipeline {
        let device_ref = unsafe { DeviceRef::from_ptr(device) };

        SwiftLinePipeline {
            pipeline: LinePipeline::new(device_ref),
        }
    }

    pub fn draw(&mut self, encoder: *mut MTLRenderCommandEncoder, segments: &SwiftLineSegments) {
        let encoder_ref = unsafe { RenderCommandEncoderRef::from_ptr(encoder) };

        self.pipeline.draw(encoder_ref, &segments.segments)
    }
}
