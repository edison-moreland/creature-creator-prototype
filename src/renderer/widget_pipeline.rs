use crate::renderer::shared::Shared;
use crate::renderer::uniforms::Uniforms;
use metal::{
    DepthStencilStateRef, DeviceRef, MTLPixelFormat, MTLPrimitiveType, MTLVertexFormat,
    MTLVertexStepFunction, NSUInteger, RenderCommandEncoderRef, RenderPipelineDescriptor,
    RenderPipelineState, VertexAttributeDescriptor, VertexBufferLayoutDescriptor, VertexDescriptor,
};
use std::mem::size_of;

const MAX_LINE_SEGMENTS: usize = 1000;
const WIDGET_SHADER_LIBRARY: &[u8] = include_bytes!("widget_shader.metallib");

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

pub struct WidgetPipeline {
    pipeline: RenderPipelineState,

    vertex_count: usize,
    vertices: Shared<[Vertex; MAX_LINE_SEGMENTS]>,
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
        let position_attribute = VertexAttributeDescriptor::new();
        position_attribute.set_format(MTLVertexFormat::Float3);
        position_attribute.set_buffer_index(0);
        position_attribute.set_offset(0);
        vertex_descriptor
            .attributes()
            .set_object_at(0, Some(&position_attribute));

        let color_attribute = VertexAttributeDescriptor::new();
        color_attribute.set_format(MTLVertexFormat::Float3);
        color_attribute.set_buffer_index(0);
        color_attribute.set_offset((size_of::<[f32; 3]>()) as NSUInteger);
        vertex_descriptor
            .attributes()
            .set_object_at(1, Some(&color_attribute));

        // Buffer layouts
        let vertex_buffer = VertexBufferLayoutDescriptor::new();
        vertex_buffer.set_stride((size_of::<Vertex>()) as NSUInteger);
        vertex_buffer.set_step_function(MTLVertexStepFunction::PerVertex);
        vertex_buffer.set_step_rate(1);
        vertex_descriptor
            .layouts()
            .set_object_at(0, Some(&vertex_buffer));

        pipeline_descriptor.set_vertex_descriptor(Some(vertex_descriptor));

        device
            .new_render_pipeline_state(pipeline_descriptor.as_ref())
            .unwrap()
    }

    fn new_vertex_buffer(device: &DeviceRef) -> Shared<[Vertex; MAX_LINE_SEGMENTS]> {
        Shared::new(
            device,
            [Vertex {
                position: [0.0, 0.0, 0.0],
                color: [0.0, 0.0, 0.0],
            }; MAX_LINE_SEGMENTS],
        )
    }

    pub fn new(device: &DeviceRef) -> Self {
        Self {
            pipeline: Self::new_pipeline(device),
            vertex_count: 0,
            vertices: Self::new_vertex_buffer(device),
        }
    }
}
// Drawing
impl WidgetPipeline {
    pub fn update_widgets(&mut self, vertices: &[Vertex]) {
        // TODO: Support actual primitives

        let vert_count = vertices.len();
        if vert_count > MAX_LINE_SEGMENTS {
            panic!("HEY THAT:S TOO BIG!!! HEY !!")
        }

        self.vertex_count = vert_count;
        self.vertices[0..vert_count].copy_from_slice(vertices);
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
            encoder.set_vertex_buffer(1, Some(uniforms.buffer()), 0);

            encoder.draw_primitives(MTLPrimitiveType::Line, 0, self.vertex_count as NSUInteger)
        }
    }
}
