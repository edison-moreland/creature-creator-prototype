use std::f32::consts::PI;
use std::mem::size_of;

use metal::{
    DepthStencilStateRef, DeviceRef, MTLPixelFormat, MTLPrimitiveType, MTLVertexFormat,
    MTLVertexStepFunction, NSUInteger, RenderCommandEncoderRef, RenderPipelineDescriptor,
    RenderPipelineState, VertexAttributeDescriptor, VertexBufferLayoutDescriptor, VertexDescriptor,
};

use crate::renderer::shared::Shared;
use crate::renderer::uniforms::Uniforms;

const SPHERE_SLICES: f32 = 16.0 / 2.0;
const SPHERE_RINGS: f32 = 16.0 / 2.0;
const SPHERE_VERTEX_COUNT: usize = (SPHERE_RINGS as usize + 2) * SPHERE_SLICES as usize * 6;
const MAX_INSTANCE_COUNT: usize = 20000;
const SPHERE_SHADER_LIBRARY: &[u8] = include_bytes!("sphere_shader.metallib");

#[derive(Copy, Clone)]
#[repr(C)]
struct Vertex {
    position: [f32; 3],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Sphere {
    pub center: [f32; 3],
    pub radius: f32,
    pub normal: [f32; 3],
}
pub struct SpherePipeline {
    pipeline: RenderPipelineState,

    instance_count: usize,
    instances: Shared<[Sphere; MAX_INSTANCE_COUNT]>,
    vertices: Shared<[Vertex; SPHERE_VERTEX_COUNT]>,
}

fn sphere_vertices(rings: f32, slices: f32) -> [Vertex; SPHERE_VERTEX_COUNT] {
    // This method of sphere vert generation was yoinked from raylib <3
    let mut data = [Vertex {
        position: [0.0, 0.0, 0.0],
    }; SPHERE_VERTEX_COUNT];

    let deg2rad = PI / 180.0;

    for i in 0..(rings as i32 + 2) {
        for j in 0..slices as i32 {
            let fi = i as f32;
            let fj = j as f32;

            let vertex = |i: f32, j: f32| Vertex {
                position: [
                    (deg2rad * (270.0 + (180.0 / (rings + 1.0)) * i)).cos()
                        * (deg2rad * (360.0 * j / slices)).sin(),
                    (deg2rad * (270.0 + (180.0 / (rings + 1.0)) * i)).sin(),
                    (deg2rad * (270.0 + (180.0 / (rings + 1.0)) * i)).cos()
                        * (deg2rad * (360.0 * j / slices)).cos(),
                ],
            };

            let idx = ((slices as i32 * 6 * i) + (j * 6)) as usize;

            data[idx] = vertex(fi, fj);
            data[idx + 1] = vertex(fi + 1.0, fj + 1.0);
            data[idx + 2] = vertex(fi + 1.0, fj);
            data[idx + 3] = vertex(fi, fj);
            data[idx + 4] = vertex(fi, fj + 1.0);
            data[idx + 5] = vertex(fi + 1.0, fj + 1.0);
        }
    }

    data
}

// Initialization
impl SpherePipeline {
    fn new_pipeline(device: &DeviceRef) -> RenderPipelineState {
        let library = device
            .new_library_with_data(SPHERE_SHADER_LIBRARY)
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

        let center_attribute = VertexAttributeDescriptor::new();
        center_attribute.set_format(MTLVertexFormat::Float3);
        center_attribute.set_buffer_index(1);
        center_attribute.set_offset(0);
        vertex_descriptor
            .attributes()
            .set_object_at(1, Some(&center_attribute));

        let radius_attribute = VertexAttributeDescriptor::new();
        radius_attribute.set_format(MTLVertexFormat::Float);
        radius_attribute.set_buffer_index(1);
        radius_attribute.set_offset((size_of::<f32>() * 3) as NSUInteger);
        vertex_descriptor
            .attributes()
            .set_object_at(2, Some(&radius_attribute));

        let color_attribute = VertexAttributeDescriptor::new();
        color_attribute.set_format(MTLVertexFormat::Float3);
        color_attribute.set_buffer_index(1);
        color_attribute.set_offset((size_of::<f32>() * 4) as NSUInteger);
        vertex_descriptor
            .attributes()
            .set_object_at(3, Some(&color_attribute));

        // Buffer layouts
        let vertex_buffer = VertexBufferLayoutDescriptor::new();
        vertex_buffer.set_stride((size_of::<f32>() * 3) as NSUInteger);
        vertex_buffer.set_step_function(MTLVertexStepFunction::PerVertex);
        vertex_buffer.set_step_rate(1);
        vertex_descriptor
            .layouts()
            .set_object_at(0, Some(&vertex_buffer));

        let instance_buffer = VertexBufferLayoutDescriptor::new();
        instance_buffer.set_stride((size_of::<f32>() * 7) as NSUInteger);
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

    fn new_instance_buffer(device: &DeviceRef) -> Shared<[Sphere; MAX_INSTANCE_COUNT]> {
        Shared::new(
            &device,
            [Sphere {
                center: [0.0, 0.0, 0.0],
                radius: 0.0,
                normal: [0.0, 0.0, 0.0],
            }; MAX_INSTANCE_COUNT],
        )
    }

    fn new_vertices_buffer(device: &DeviceRef) -> Shared<[Vertex; SPHERE_VERTEX_COUNT]> {
        Shared::new(device, sphere_vertices(SPHERE_RINGS, SPHERE_SLICES))
    }

    pub fn new(device: &DeviceRef) -> Self {
        Self {
            pipeline: Self::new_pipeline(device),
            instance_count: 0,
            instances: Self::new_instance_buffer(device),
            vertices: Self::new_vertices_buffer(device),
        }
    }
}

// Drawing
impl SpherePipeline {
    pub fn update_instance_buffer(&mut self, spheres: &[Sphere]) {
        let instance_count = spheres.len();
        if instance_count > MAX_INSTANCE_COUNT {
            panic!("HEY THAT:S TOO BIG!!! HEY !!")
        }

        self.instance_count = instance_count;
        self.instances[0..instance_count].copy_from_slice(spheres);
    }

    pub fn draw_spheres<'a>(
        &'a self,
        depth_stencil: &'a DepthStencilStateRef,
        uniforms: &'a Shared<Uniforms>,
    ) -> impl FnOnce(&RenderCommandEncoderRef) + 'a {
        move |encoder| {
            encoder.set_render_pipeline_state(&self.pipeline);
            encoder.set_depth_stencil_state(&depth_stencil);
            encoder.set_vertex_buffer(0, Some(self.vertices.buffer()), 0);
            encoder.set_vertex_buffer(1, Some(self.instances.buffer()), 0);
            encoder.set_vertex_buffer(2, Some(uniforms.buffer()), 0);

            encoder.draw_primitives_instanced(
                MTLPrimitiveType::Triangle,
                0,
                SPHERE_VERTEX_COUNT as NSUInteger,
                self.instance_count as NSUInteger,
            )
        }
    }
}
