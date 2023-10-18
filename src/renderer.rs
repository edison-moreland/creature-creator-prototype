use std::f32::consts::PI;
use std::mem;
use std::mem::size_of;

use cocoa::appkit::NSView;
use cocoa::base::id;
use core_graphics_types::geometry::CGSize;
use metal::objc::runtime::YES;
use metal::{
    Buffer, CommandQueue, Device, DeviceRef, Function, MTLPixelFormat, MTLPrimitiveType,
    MTLResourceOptions, MTLVertexFormat, MTLVertexStepFunction, MetalDrawableRef, MetalLayer,
    NSUInteger, RenderCommandEncoderRef, RenderPipelineDescriptor, RenderPipelineState,
    VertexAttributeDescriptor, VertexBufferLayoutDescriptor, VertexDescriptor,
};
use nalgebra::{vector, Matrix4};
use winit::dpi::PhysicalSize;
use winit::platform::macos::WindowExtMacOS;
use winit::window::Window;

const SPHERE_SLICES: f32 = 16.0;
const SPHERE_RINGS: f32 = 16.0;

const SHADER_METALLIB: &[u8] = include_bytes!("shader.metallib");

fn create_metal_layer(
    device: &DeviceRef,
    window: &Window,
    // raw_window: &AppKitWindowHandle,
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

#[repr(C)]
struct Vertex {
    position: [f32; 3],
}

#[repr(C)]
pub struct Instance {
    pub center: [f32; 3],
    pub radius: f32,
    pub color: [f32; 3],
}
// // Draw sphere with extended parameters
//    #define DEG2RAD (PI/180.0f)
// void DrawSphereEx(Vector3 centerPos, float radius, int rings, int slices, Color color)
// {
//     rlPushMatrix();
//         // NOTE: Transformation is applied in inverse order (scale -> translate)
//         rlTranslatef(centerPos.x, centerPos.y, centerPos.z);
//         rlScalef(radius, radius, radius);
//
//         rlBegin(RL_TRIANGLES);
//             rlColor4ub(color.r, color.g, color.b, color.a);
//
//             for (int i = 0; i < (rings + 2); i++)
//             {
//                 for (int j = 0; j < slices; j++)
//                 {
//                     rlVertex3f(cosf(DEG2RAD*(270 + (180.0f/(rings + 1))*i))*sinf(DEG2RAD*(360.0f*j/slices)),
//                                sinf(DEG2RAD*(270 + (180.0f/(rings + 1))*i)),
//                                cosf(DEG2RAD*(270 + (180.0f/(rings + 1))*i))*cosf(DEG2RAD*(360.0f*j/slices)));
//                     rlVertex3f(cosf(DEG2RAD*(270 + (180.0f/(rings + 1))*(i + 1)))*sinf(DEG2RAD*(360.0f*(j + 1)/slices)),
//                                sinf(DEG2RAD*(270 + (180.0f/(rings + 1))*(i + 1))),
//                                cosf(DEG2RAD*(270 + (180.0f/(rings + 1))*(i + 1)))*cosf(DEG2RAD*(360.0f*(j + 1)/slices)));
//                     rlVertex3f(cosf(DEG2RAD*(270 + (180.0f/(rings + 1))*(i + 1)))*sinf(DEG2RAD*(360.0f*j/slices)),
//                                sinf(DEG2RAD*(270 + (180.0f/(rings + 1))*(i + 1))),
//                                cosf(DEG2RAD*(270 + (180.0f/(rings + 1))*(i + 1)))*cosf(DEG2RAD*(360.0f*j/slices)));
//
//                     rlVertex3f(cosf(DEG2RAD*(270 + (180.0f/(rings + 1))*i))*sinf(DEG2RAD*(360.0f*j/slices)),
//                                sinf(DEG2RAD*(270 + (180.0f/(rings + 1))*i)),
//                                cosf(DEG2RAD*(270 + (180.0f/(rings + 1))*i))*cosf(DEG2RAD*(360.0f*j/slices)));
//                     rlVertex3f(cosf(DEG2RAD*(270 + (180.0f/(rings + 1))*(i)))*sinf(DEG2RAD*(360.0f*(j + 1)/slices)),
//                                sinf(DEG2RAD*(270 + (180.0f/(rings + 1))*(i))),
//                                cosf(DEG2RAD*(270 + (180.0f/(rings + 1))*(i)))*cosf(DEG2RAD*(360.0f*(j + 1)/slices)));
//                     rlVertex3f(cosf(DEG2RAD*(270 + (180.0f/(rings + 1))*(i + 1)))*sinf(DEG2RAD*(360.0f*(j + 1)/slices)),
//                                sinf(DEG2RAD*(270 + (180.0f/(rings + 1))*(i + 1))),
//                                cosf(DEG2RAD*(270 + (180.0f/(rings + 1))*(i + 1)))*cosf(DEG2RAD*(360.0f*(j + 1)/slices)));
//                 }
//             }
//         rlEnd();
//     rlPopMatrix();
// }

fn prepare_vertex_buffer(device: &DeviceRef) -> Buffer {
    let mut data: Vec<Vertex> = vec![];

    let rings = SPHERE_RINGS;
    let slices = SPHERE_SLICES;

    let deg2rad = PI / 180.0;

    for i in 0..(rings as i32 + 2) {
        for j in 0..slices as i32 {
            let fi = i as f32;
            let fj = j as f32;

            data.push(Vertex {
                position: [
                    (deg2rad * (270.0 + (180.0 / (rings + 1.0)) * fi)).cos()
                        * (deg2rad * (360.0 * fj / slices)).sin(),
                    (deg2rad * (270.0 + (180.0 / (rings + 1.0)) * fi)).sin(),
                    (deg2rad * (270.0 + (180.0 / (rings + 1.0)) * fi)).cos()
                        * (deg2rad * (360.0 * fj / slices)).cos(),
                ],
            });
            data.push(Vertex {
                position: [
                    (deg2rad * (270.0 + (180.0 / (rings + 1.0)) * (fi + 1.0))).cos()
                        * (deg2rad * (360.0 * (fj + 1.0) / slices)).sin(),
                    (deg2rad * (270.0 + (180.0 / (rings + 1.0)) * (fi + 1.0))).sin(),
                    (deg2rad * (270.0 + (180.0 / (rings + 1.0)) * (fi + 1.0))).cos()
                        * (deg2rad * (360.0 * (fj + 1.0) / slices)).cos(),
                ],
            });
            data.push(Vertex {
                position: [
                    (deg2rad * (270.0 + (180.0 / (rings + 1.0)) * (fi + 1.0))).cos()
                        * (deg2rad * (360.0 * fj / slices)).sin(),
                    (deg2rad * (270.0 + (180.0 / (rings + 1.0)) * (fi + 1.0))).sin(),
                    (deg2rad * (270.0 + (180.0 / (rings + 1.0)) * (fi + 1.0))).cos()
                        * (deg2rad * (360.0 * fj / slices)).cos(),
                ],
            });

            data.push(Vertex {
                position: [
                    (deg2rad * (270.0 + (180.0 / (rings + 1.0)) * fi)).cos()
                        * (deg2rad * (360.0 * fj / slices)).sin(),
                    (deg2rad * (270.0 + (180.0 / (rings + 1.0)) * fi)).sin(),
                    (deg2rad * (270.0 + (180.0 / (rings + 1.0)) * fi)).cos()
                        * (deg2rad * (360.0 * fj / slices)).cos(),
                ],
            });
            data.push(Vertex {
                position: [
                    (deg2rad * (270.0 + (180.0 / (rings + 1.0)) * fi)).cos()
                        * (deg2rad * (360.0 * (fj + 1.0) / slices)).sin(),
                    (deg2rad * (270.0 + (180.0 / (rings + 1.0)) * fi)).sin(),
                    (deg2rad * (270.0 + (180.0 / (rings + 1.0)) * fi)).cos()
                        * (deg2rad * (360.0 * (fj + 1.0) / slices)).cos(),
                ],
            });
            data.push(Vertex {
                position: [
                    (deg2rad * (270.0 + (180.0 / (rings + 1.0)) * (fi + 1.0))).cos()
                        * (deg2rad * (360.0 * (fj + 1.0) / slices)).sin(),
                    (deg2rad * (270.0 + (180.0 / (rings + 1.0)) * (fi + 1.0))).sin(),
                    (deg2rad * (270.0 + (180.0 / (rings + 1.0)) * (fi + 1.0))).cos()
                        * (deg2rad * (360.0 * (fj + 1.0) / slices)).cos(),
                ],
            });
        }
    }

    device.new_buffer_with_data(
        data.as_ptr() as *const _,
        (data.len() * size_of::<Vertex>()) as NSUInteger,
        MTLResourceOptions::StorageModeShared,
    )
}

fn prepare_camera_matrices(aspect_ratio: f32) -> (Vec<f32>, Vec<f32>) {
    // TODO: Something something uniforms struct, make it one buffer
    // Projection matrix
    let proj = Matrix4::new_perspective(aspect_ratio, 30.0 * (PI / 180.0), 1.0, 1000.0);

    // View matrix
    let view = Matrix4::new_translation(&vector![0.0, 0.0, -30.0]);

    return (proj.as_slice().to_vec(), view.as_slice().to_vec());
}

pub struct FastBallRenderer {
    device: Device,
    layer: MetalLayer,
    command_queue: CommandQueue,
    pipeline: RenderPipelineState,
    vertex_buffer: Buffer,
}

impl FastBallRenderer {
    pub fn new(window: &Window) -> Self {
        let device = Device::system_default().expect("no device found");
        let command_queue = device.new_command_queue();

        let layer = create_metal_layer(&device, &window);

        let pipeline = create_pipeline(&device, SHADER_METALLIB, "vertex_main", "fragment_main");

        let vertex_buffer = prepare_vertex_buffer(&device);

        FastBallRenderer {
            device,
            layer,
            command_queue,
            pipeline,
            vertex_buffer,
        }
    }

    pub fn resized(&self, new_size: PhysicalSize<u32>) {
        self.layer
            .set_drawable_size(CGSize::new(new_size.width as f64, new_size.height as f64))
    }

    pub fn rescaled(&self, scale_factor: f64) {
        self.layer.set_contents_scale(scale_factor);
    }

    pub fn draw(&self, instances: Vec<Instance>) {
        let drawable = match self.layer.next_drawable() {
            Some(drawable) => drawable,
            None => return,
        };

        let size = self.layer.drawable_size();

        let (proj, view) = prepare_camera_matrices((size.width / size.height) as f32);

        self.render_pass(drawable, |encoder| {
            encoder.set_render_pipeline_state(&self.pipeline);
            encoder.set_vertex_buffer(0, Some(&self.vertex_buffer), 0);
            encoder.set_vertex_bytes(
                1,
                (instances.len() * size_of::<Instance>()) as NSUInteger,
                instances.as_ptr() as *const _,
            );
            encoder.set_vertex_bytes(
                2,
                (proj.len() * size_of::<f32>()) as NSUInteger,
                proj.as_ptr() as *const _,
            );
            encoder.set_vertex_bytes(
                3,
                (view.len() * size_of::<f32>()) as NSUInteger,
                view.as_ptr() as *const _,
            );

            encoder.draw_primitives_instanced(
                MTLPrimitiveType::Triangle,
                0,
                (SPHERE_RINGS as u64 + 2) * SPHERE_SLICES as u64 * 6,
                instances.len() as NSUInteger,
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
        color_attachment.set_load_action(metal::MTLLoadAction::Clear);
        color_attachment.set_clear_color(metal::MTLClearColor::new(0.0, 0.0, 0.0, 1.0));
        color_attachment.set_store_action(metal::MTLStoreAction::Store);

        let command_buffer = self.command_queue.new_command_buffer();
        let encoder = command_buffer.new_render_command_encoder(&render_pass);

        f(encoder);

        encoder.end_encoding();
        command_buffer.present_drawable(&drawable);
        command_buffer.commit();
    }
}
