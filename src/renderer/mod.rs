use std::f32::consts::PI;
use std::mem;
use std::mem::size_of;
use std::pin::Pin;

use cocoa::appkit::NSView;
use cocoa::base::id;
use core_graphics_types::geometry::CGSize;
use metal::objc::runtime::YES;
use metal::{
    Buffer, CommandQueue, DepthStencilDescriptor, DepthStencilState, Device, DeviceRef, Function,
    MTLClearColor, MTLCompareFunction, MTLLoadAction, MTLPixelFormat, MTLPrimitiveType,
    MTLResourceOptions, MTLStorageMode, MTLStoreAction, MTLTextureUsage, MTLVertexFormat,
    MTLVertexStepFunction, MetalDrawableRef, MetalLayer, NSRange, NSUInteger,
    RenderCommandEncoderRef, RenderPipelineDescriptor,
    RenderPipelineState, Texture, TextureDescriptor, VertexAttributeDescriptor,
    VertexBufferLayoutDescriptor, VertexDescriptor,
};
use nalgebra::{vector, Matrix4, Vector3};
use winit::dpi::PhysicalSize;
use winit::platform::macos::WindowExtMacOS;
use winit::window::Window;

const SPHERE_SLICES: f32 = 16.0 / 2.0;
const SPHERE_RINGS: f32 = 16.0 / 2.0;
const MAX_INSTANCE_COUNT: usize = 10000;
const SHADER_LIBRARY: &[u8] = include_bytes!("shader.metallib");

#[repr(C)]
struct Vertex {
    position: [f32; 3],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Instance {
    pub center: [f32; 3],
    pub radius: f32,
    pub normal: [f32; 3],
}

#[repr(C)]
pub struct Uniforms {
    projection: [f32; 4 * 4],
    view: [f32; 4 * 4],
}

fn create_metal_layer(
    device: &DeviceRef,
    window: &Window,
) -> MetalLayer {
    let layer = MetalLayer::new();
    layer.set_device(&device);
    layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
    layer.set_presents_with_transaction(false);

    unsafe {
        let view = window.ns_view() as id;
        view.setWantsLayer(YES);
        view.setLayer(mem::transmute(layer.as_ref()));
    }

    let draw_size = window.inner_size();
    layer.set_drawable_size(CGSize::new(draw_size.width as f64, draw_size.height as f64));

    let scale_factor = window.scale_factor();
    layer.set_contents_scale(scale_factor);

    return layer;
}

fn create_pipeline(
    device: &DeviceRef,
    library: &[u8],
    vertex_shader: &str,
    fragment_shader: &str,
) -> RenderPipelineState {
    let library = device.new_library_with_data(library).unwrap();

    device
        .new_render_pipeline_state(
            create_pipeline_descriptor(
                library.get_function(vertex_shader, None).unwrap(),
                library.get_function(fragment_shader, None).unwrap(),
            )
                .as_ref(),
        )
        .unwrap()
}

fn create_pipeline_descriptor(
    vertex_shader: Function,
    fragment_shader: Function,
) -> RenderPipelineDescriptor {
    let pipeline_descriptor = RenderPipelineDescriptor::new();
    pipeline_descriptor.set_vertex_function(Some(&vertex_shader));
    pipeline_descriptor.set_fragment_function(Some(&fragment_shader));
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

    pipeline_descriptor.set_vertex_descriptor(Some(&vertex_descriptor));

    pipeline_descriptor
}


fn sphere_vertices(rings: f32, slices: f32) -> Vec<Vertex> {
    // This method of sphere vert generation was yoinked from raylib <3

    let mut data: Vec<Vertex> = vec![];
    data.reserve(((rings + 2.0) * slices * 6.0) as usize);

    let deg2rad = PI / 180.0;

    for i in 0..(rings as i32 + 2) {
        for j in 0..slices as i32 {
            let fi = i as f32;
            let fj = j as f32;

            let vertex = |i: f32, j: f32| {
                Vertex {
                    position: [
                        (deg2rad * (270.0 + (180.0 / (rings + 1.0)) * i)).cos()
                            * (deg2rad * (360.0 * j / slices)).sin(),
                        (deg2rad * (270.0 + (180.0 / (rings + 1.0)) * i)).sin(),
                        (deg2rad * (270.0 + (180.0 / (rings + 1.0)) * i)).cos()
                            * (deg2rad * (360.0 * j / slices)).cos(),
                    ],
                }
            };

            data.push(vertex(fi, fj));
            data.push(vertex(fi + 1.0, fj + 1.0));
            data.push(vertex(fi + 1.0, fj));
            data.push(vertex(fi, fj));
            data.push(vertex(fi, fj + 1.0));
            data.push(vertex(fi + 1.0, fj + 1.0));
        }
    }

    data
}

fn prepare_vertex_buffer(device: &DeviceRef) -> Buffer {
    let data = sphere_vertices(SPHERE_RINGS, SPHERE_SLICES);

    device.new_buffer_with_data(
        data.as_ptr() as *const _,
        (data.len() * size_of::<Vertex>()) as NSUInteger,
        MTLResourceOptions::StorageModeShared,
    )
}

fn prepare_instance_buffer(device: &DeviceRef) -> (Buffer, Pin<Box<[Instance; MAX_INSTANCE_COUNT]>>) {
    let instance_array = Box::pin([Instance{
        center: [0.0, 0.0, 0.0],
        radius: 0.0,
        normal: [0.0, 0.0, 0.0],
    }; MAX_INSTANCE_COUNT]);

    let buffer = device.new_buffer_with_bytes_no_copy(
        instance_array.as_ptr() as *const _,
        (size_of::<Instance>() * MAX_INSTANCE_COUNT) as u64,
        MTLResourceOptions::StorageModeShared,
        None
    );

    (buffer, instance_array)
}

fn prepare_depth_target(device: &DeviceRef, size: PhysicalSize<u32>) -> Texture {
    let texture_descriptor = TextureDescriptor::new();
    texture_descriptor.set_width(size.width as u64);
    texture_descriptor.set_height(size.height as u64);
    texture_descriptor.set_pixel_format(MTLPixelFormat::Depth32Float);
    texture_descriptor.set_storage_mode(MTLStorageMode::Memoryless);
    texture_descriptor.set_usage(MTLTextureUsage::RenderTarget);

    device.new_texture(&texture_descriptor)
}

fn create_depth_state(device: &DeviceRef) -> DepthStencilState {
    let depth_stencil_descriptor = DepthStencilDescriptor::new();
    depth_stencil_descriptor.set_depth_compare_function(MTLCompareFunction::LessEqual);
    depth_stencil_descriptor.set_depth_write_enabled(true);

    device.new_depth_stencil_state(&depth_stencil_descriptor)
}

fn prepare_uniforms(aspect_ratio: f32, camera_position: Vector3<f32>, camera_rotation: Vector3<f32>) -> Uniforms {
    // TODO: Am I doing any of this right??

    // Projection matrix
    let proj = Matrix4::new_perspective(aspect_ratio, 60.0 * (PI / 180.0), 1.0, 1000.0);

    // View matrix
    let view = Matrix4::new_translation(&-camera_position)
        * Matrix4::new_rotation(vector![camera_rotation.x *(PI / 180.0), 0.0, 0.0])
        * Matrix4::new_rotation(vector![0.0, camera_rotation.y * (PI / 180.0), 0.0])
        * Matrix4::new_rotation(vector![0.0, 0.0, camera_rotation.z * (PI / 180.0)]);

    Uniforms {
        projection: proj.as_slice().to_vec().try_into().unwrap(),
        view: view.as_slice().to_vec().try_into().unwrap(),
    }
}

fn prepare_uniform_buffer(device: &DeviceRef, aspect_ratio: f32, camera_position: Vector3<f32>, camera_rotation: Vector3<f32>) -> Buffer {
    let data = [prepare_uniforms(aspect_ratio, camera_position, camera_rotation)];

    device.new_buffer_with_data(
        data.as_ptr() as *const _,
        size_of::<Uniforms>() as NSUInteger,
        MTLResourceOptions::StorageModeShared,
    )
}

pub struct FastBallRenderer {
    device: Device,
    layer: MetalLayer,
    command_queue: CommandQueue,
    pipeline: RenderPipelineState,
    depth_state: DepthStencilState,
    depth_target: Texture,
    vertex_buffer: Buffer,
    instance_buffer: Buffer,
    instance_array: Pin<Box<[Instance; MAX_INSTANCE_COUNT]>>,
    uniform_buffer: Buffer,

    camera_position: Vector3<f32>,
    camera_rotation: Vector3<f32>,
}

impl FastBallRenderer {
    pub fn new(window: &Window, camera_position: Vector3<f32>, camera_rotation: Vector3<f32>) -> Self {
        let device = Device::system_default().expect("no device found");
        let command_queue = device.new_command_queue();

        let layer = create_metal_layer(&device, &window);

        let pipeline = create_pipeline(&device, SHADER_LIBRARY, "vertex_main", "fragment_main");

        let size = window.inner_size();

        let depth_target = prepare_depth_target(&device, size);
        let depth_state = create_depth_state(&device);

        let vertex_buffer = prepare_vertex_buffer(&device);
        let (instance_buffer, instance_array) = prepare_instance_buffer(&device);
        let uniform_buffer = prepare_uniform_buffer(&device, size.width as f32 / size.height as f32, camera_position, camera_rotation);

        FastBallRenderer {
            device,
            layer,
            command_queue,
            depth_state,
            depth_target,
            pipeline,
            vertex_buffer,
            instance_buffer,
            instance_array,
            uniform_buffer,
            camera_position,
            camera_rotation,
        }
    }

    pub fn resized(&mut self, new_size: PhysicalSize<u32>) {
        self.layer
            .set_drawable_size(CGSize::new(new_size.width as f64, new_size.height as f64));

        self.depth_target = prepare_depth_target(&self.device, new_size);
        self.uniform_buffer = prepare_uniform_buffer(&self.device, new_size.width as f32 / new_size.height as f32, self.camera_position, self.camera_rotation);
    }

    pub fn rescaled(&self, scale_factor: f64) {
        self.layer.set_contents_scale(scale_factor);
    }

    fn update_instance_buffer(&mut self, instances: impl Iterator<Item=Instance>) {
        for (i, instance) in instances.enumerate() {
            self.instance_array[i] = instance
        }
    }

    pub fn draw(&mut self, instances: impl Iterator<Item=Instance> + ExactSizeIterator) {
        let instance_count = instances.len();
        if instance_count > MAX_INSTANCE_COUNT {
            panic!("HEY THAT:S TOO BIG!!! HEY !!")
        }
        self.update_instance_buffer(instances);

        let drawable = match self.layer.next_drawable() {
            Some(drawable) => drawable,
            None => return,
        };

        self.render_pass(drawable, |encoder| {
            encoder.set_render_pipeline_state(&self.pipeline);
            encoder.set_depth_stencil_state(&self.depth_state);
            encoder.set_vertex_buffer(0, Some(&self.vertex_buffer), 0);
            encoder.set_vertex_buffer(1, Some(&self.instance_buffer), 0);
            encoder.set_vertex_buffer(2, Some(&self.uniform_buffer), 0);

            encoder.draw_primitives_instanced(
                MTLPrimitiveType::Triangle,
                0,
                (SPHERE_RINGS as u64 + 2) * SPHERE_SLICES as u64 * 6,
                instance_count as NSUInteger,
            )
        })
    }

    fn render_pass<F>(&self, drawable: &MetalDrawableRef, f: F)
        where
            F: Fn(&RenderCommandEncoderRef),
    {
        let render_pass = metal::RenderPassDescriptor::new();
        let color_attachment = render_pass.color_attachments().object_at(0).unwrap();
        color_attachment.set_texture(Some(drawable.texture()));
        color_attachment.set_load_action(MTLLoadAction::Clear);
        color_attachment.set_clear_color(MTLClearColor::new(0.0, 0.0, 0.0, 1.0));
        color_attachment.set_store_action(MTLStoreAction::Store);

        let depth_attachment = render_pass.depth_attachment().unwrap();
        depth_attachment.set_texture(Some(&self.depth_target));
        depth_attachment.set_clear_depth(1.0);
        depth_attachment.set_load_action(MTLLoadAction::Clear);
        depth_attachment.set_store_action(MTLStoreAction::DontCare);

        let command_buffer = self.command_queue.new_command_buffer();
        let encoder = command_buffer.new_render_command_encoder(&render_pass);

        f(encoder);

        encoder.end_encoding();
        command_buffer.present_drawable(&drawable);
        command_buffer.commit();
    }
}
