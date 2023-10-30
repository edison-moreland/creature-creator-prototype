use crate::plane::Plane;
use crate::renderer::shared::Shared;
use crate::renderer::uniforms::Uniforms;
use metal::{
    DepthStencilStateRef, DeviceRef, MTLPixelFormat, MTLPrimitiveType, MTLVertexFormat,
    MTLVertexStepFunction, NSUInteger, RenderCommandEncoderRef, RenderPipelineDescriptor,
    RenderPipelineState, VertexAttributeDescriptor, VertexBufferLayoutDescriptor, VertexDescriptor,
};
use nalgebra::Vector3;
use std::mem::size_of;

const VERTEX_COUNT: usize = 4; // Just a quad
const STYLE_COUNT: usize = 2;
const MAX_LINE_SEGMENTS: usize = 1000;
const WIDGET_SHADER_LIBRARY: &[u8] = include_bytes!("widget_shader.metallib");

pub enum Widget {
    Line {
        start: Vector3<f32>,
        end: Vector3<f32>,
        color: Vector3<f32>,
    },
    Circle {
        origin: Vector3<f32>,
        normal: Vector3<f32>,
        radius: f32,
        color: Vector3<f32>,
    },
    Arrow {
        origin: Vector3<f32>,
        direction: Vector3<f32>,
        magnitude: f32,
        color: Vector3<f32>,
    },
}

#[derive(Copy, Clone)]
#[repr(C)]
struct Vertex {
    position: [f32; 2],
}

#[derive(Copy, Clone)]
#[repr(C)]
struct LineSegment {
    start: [f32; 3],
    end: [f32; 3],
    color: [f32; 3],
    thickness: f32,
    style: u32,
}

pub struct WidgetPipeline {
    pipeline: RenderPipelineState,

    vertices: Shared<[Vertex; VERTEX_COUNT * STYLE_COUNT]>,

    segment_count: usize,
    segments: Shared<[LineSegment; MAX_LINE_SEGMENTS]>,
}

// Initialization
impl WidgetPipeline {
    fn new_pipeline(device: &DeviceRef) -> RenderPipelineState {
        let library = device
            .new_library_with_data(WIDGET_SHADER_LIBRARY)
            .expect("sphere should load without error");
        let vertex_function = library
            .get_function("vertex_main", None)
            .expect("function `vertex_main` to exist");
        let frag_function = library
            .get_function("fragment_main", None)
            .expect("function `fragment_main` to exist");

        let pipeline_descriptor = RenderPipelineDescriptor::new();
        pipeline_descriptor.set_vertex_function(Some(&vertex_function));
        pipeline_descriptor.set_fragment_function(Some(&frag_function));
        pipeline_descriptor.set_depth_attachment_pixel_format(MTLPixelFormat::Depth32Float);

        let attachment = pipeline_descriptor
            .color_attachments()
            .object_at(0)
            .unwrap();

        attachment.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        attachment.set_blending_enabled(true);
        attachment.set_rgb_blend_operation(metal::MTLBlendOperation::Add);
        attachment.set_alpha_blend_operation(metal::MTLBlendOperation::Add);
        attachment.set_source_rgb_blend_factor(metal::MTLBlendFactor::SourceAlpha);
        attachment.set_source_alpha_blend_factor(metal::MTLBlendFactor::SourceAlpha);
        attachment.set_destination_rgb_blend_factor(metal::MTLBlendFactor::OneMinusSourceAlpha);
        attachment.set_destination_alpha_blend_factor(metal::MTLBlendFactor::OneMinusSourceAlpha);

        let vertex_descriptor = VertexDescriptor::new();

        // Vertex attributes
        let start_attribute = VertexAttributeDescriptor::new();
        start_attribute.set_format(MTLVertexFormat::Float3);
        start_attribute.set_buffer_index(1);
        start_attribute.set_offset(0);
        vertex_descriptor
            .attributes()
            .set_object_at(0, Some(&start_attribute));

        let end_attribute = VertexAttributeDescriptor::new();
        end_attribute.set_format(MTLVertexFormat::Float3);
        end_attribute.set_buffer_index(1);
        end_attribute.set_offset(size_of::<[f32; 3]>() as NSUInteger);
        vertex_descriptor
            .attributes()
            .set_object_at(1, Some(&end_attribute));

        let color_attribute = VertexAttributeDescriptor::new();
        color_attribute.set_format(MTLVertexFormat::Float3);
        color_attribute.set_buffer_index(1);
        color_attribute.set_offset(size_of::<[f32; 6]>() as NSUInteger);
        vertex_descriptor
            .attributes()
            .set_object_at(2, Some(&color_attribute));

        let thickness_attribute = VertexAttributeDescriptor::new();
        thickness_attribute.set_format(MTLVertexFormat::Float);
        thickness_attribute.set_buffer_index(1);
        thickness_attribute.set_offset((size_of::<[f32; 9]>()) as NSUInteger);
        vertex_descriptor
            .attributes()
            .set_object_at(3, Some(&thickness_attribute));

        let style_attribute = VertexAttributeDescriptor::new();
        style_attribute.set_format(MTLVertexFormat::UInt);
        style_attribute.set_buffer_index(1);
        style_attribute.set_offset((size_of::<[f32; 10]>()) as NSUInteger);
        vertex_descriptor
            .attributes()
            .set_object_at(4, Some(&style_attribute));

        // Buffer layouts
        let instance_buffer = VertexBufferLayoutDescriptor::new();
        instance_buffer.set_stride(size_of::<LineSegment>() as NSUInteger);
        instance_buffer.set_step_function(MTLVertexStepFunction::PerInstance);
        instance_buffer.set_step_rate(1);
        vertex_descriptor
            .layouts()
            .set_object_at(1, Some(&instance_buffer));

        pipeline_descriptor.set_vertex_descriptor(Some(vertex_descriptor));

        device
            .new_render_pipeline_state(pipeline_descriptor.as_ref())
            .unwrap()
    }

    fn new_vertex_buffer(device: &DeviceRef) -> Shared<[Vertex; VERTEX_COUNT * STYLE_COUNT]> {
        let vertices = [
            // Regular line style
            Vertex {
                position: [-1.0, -1.0],
            },
            Vertex {
                position: [-1.0, 1.0],
            },
            Vertex {
                position: [1.0, -1.0],
            },
            Vertex {
                position: [1.0, 1.0],
            },
            // Arrow style
            Vertex {
                position: [1.0, -1.0],
            },
            Vertex {
                position: [1.0, 0.0],
            },
            Vertex {
                position: [-1.0, 0.0],
            },
            Vertex {
                position: [1.0, 1.0],
            },
        ];

        Shared::new(device, vertices)
    }

    fn new_segment_buffer(device: &DeviceRef) -> Shared<[LineSegment; MAX_LINE_SEGMENTS]> {
        Shared::new(
            device,
            [LineSegment {
                start: [0.0, 0.0, 0.0],
                end: [0.0, 0.0, 0.0],
                color: [0.0, 0.0, 0.0],
                thickness: 0.0,
                style: 0,
            }; MAX_LINE_SEGMENTS],
        )
    }

    pub fn new(device: &DeviceRef) -> Self {
        Self {
            pipeline: Self::new_pipeline(device),
            vertices: Self::new_vertex_buffer(device),
            segment_count: 0,
            segments: Self::new_segment_buffer(device),
        }
    }
}
// Drawing
impl WidgetPipeline {
    fn segment(
        a: Vector3<f32>,
        b: Vector3<f32>,
        color: Vector3<f32>,
        thickness: f32,
        style: u32,
    ) -> LineSegment {
        LineSegment {
            start: a.data.0[0],
            end: b.data.0[0],
            color: color.data.0[0],
            thickness,
            style,
        }
    }

    pub fn update_widgets(&mut self, widgets: &[Widget]) {
        let mut segment_count = 0;
        for widget in widgets {
            match widget {
                &Widget::Line { start, end, color } => {
                    self.segments[segment_count] = Self::segment(start, end, color, 0.1, 0);
                    segment_count += 1;
                }
                &Widget::Circle {
                    origin,
                    color,
                    normal,
                    radius,
                } => {
                    let segments = 24;
                    let points =
                        Plane::from_origin_normal(origin, normal).circle_points(segments, radius);

                    for i in 0..segments {
                        let last_i = if i == 0 { segments - 1 } else { i - 1 };

                        self.segments[segment_count] =
                            Self::segment(points[last_i], points[i], color, 0.1, 0);
                        segment_count += 1;
                    }
                }
                &Widget::Arrow {
                    origin,
                    direction,
                    magnitude,
                    color,
                } => {
                    let start = origin;
                    let end = start + (direction * magnitude);

                    let stem_thickness = 0.2;
                    let arrow_thickness = stem_thickness * 4.0;
                    let arrow_head_length = arrow_thickness * 1.5;

                    if magnitude <= arrow_head_length {
                        self.segments[segment_count] =
                            Self::segment(start, end, color, arrow_thickness, 1);
                        segment_count += 1;
                    } else {
                        let stem_length = magnitude - arrow_head_length;
                        let stem_end = start + (direction * stem_length);

                        self.segments[segment_count] =
                            Self::segment(start, stem_end, color, stem_thickness, 0);
                        segment_count += 1;

                        self.segments[segment_count] =
                            Self::segment(stem_end, end, color, arrow_thickness, 1);
                        segment_count += 1;
                    }
                }
            }
        }

        self.segment_count = segment_count;
    }

    pub fn draw_widgets<'a>(
        &'a self,
        depth_stencil: &'a DepthStencilStateRef,
        uniforms: &'a Shared<Uniforms>,
    ) -> impl FnOnce(&RenderCommandEncoderRef) + 'a {
        move |encoder| {
            encoder.set_render_pipeline_state(&self.pipeline);
            encoder.set_depth_stencil_state(&depth_stencil);
            encoder.set_vertex_buffer(0, Some(self.vertices.buffer()), 0);
            encoder.set_vertex_buffer(1, Some(self.segments.buffer()), 0);
            encoder.set_vertex_buffer(2, Some(uniforms.buffer()), 0);

            encoder.draw_primitives_instanced(
                MTLPrimitiveType::TriangleStrip,
                0,
                VERTEX_COUNT as NSUInteger,
                self.segment_count as NSUInteger,
            )
        }
    }
}
