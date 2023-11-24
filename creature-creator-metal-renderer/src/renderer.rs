use cocoa::appkit::NSView;
use cocoa::base::id;
use core_graphics_types::geometry::CGSize;
use metal::objc::runtime::YES;
use metal::{
    CommandQueue, DepthStencilDescriptor, DepthStencilState, Device, DeviceRef, MTLClearColor,
    MTLCompareFunction, MTLLoadAction, MTLPixelFormat, MTLStorageMode, MTLStoreAction,
    MTLTextureUsage, MetalLayer, Texture, TextureDescriptor,
};
use raw_window_handle::{RawWindowHandle, WindowHandle};

use creature_creator_renderer::{Camera, Kind, RenderGraph, Renderer};

use crate::lines::{line_segments, LinePipeline};
use crate::shared::Shared;
use crate::surfaces::{Surface, SurfacePipeline};
use crate::uniforms::Uniforms;

fn create_metal_layer(device: &DeviceRef, window_handle: &WindowHandle) -> MetalLayer {
    let layer = MetalLayer::new();
    layer.set_device(device);
    layer.set_pixel_format(MTLPixelFormat::RGBA8Unorm);
    layer.set_presents_with_transaction(false);

    if let RawWindowHandle::AppKit(handle) = window_handle.as_raw() {
        unsafe {
            let view = handle.ns_view.as_ptr() as id;
            view.setWantsLayer(YES);
            view.setLayer(
                layer.as_ref() as *const metal::MetalLayerRef as *mut metal::objc::runtime::Object
            );
        }
    }

    layer
}

fn prepare_depth_target(device: &DeviceRef, size: (u32, u32)) -> Texture {
    let texture_descriptor = TextureDescriptor::new();
    texture_descriptor.set_width(size.0 as u64);
    texture_descriptor.set_height(size.1 as u64);
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

pub struct MetalRenderer {
    device: Device,
    layer: MetalLayer,
    command_queue: CommandQueue,

    depth_state: DepthStencilState,
    depth_target: Texture,

    camera: Camera,
    uniforms: Shared<Uniforms>,

    sphere_pipeline: SurfacePipeline,
    line_pipeline: LinePipeline,
}

impl MetalRenderer {
    pub fn new(window: &WindowHandle, mut camera: Camera) -> Self {
        let device = Device::system_default().expect("no device found");
        let command_queue = device.new_command_queue();

        let layer = create_metal_layer(&device, window);

        let depth_target = prepare_depth_target(&device, (10, 10));
        let depth_state = create_depth_state(&device);

        camera.aspect_ratio_updated(1.0);
        let uniforms = Shared::new(&device, Uniforms::new(&camera));

        let sphere_pipeline = SurfacePipeline::new(&device);

        let widget_pipeline = LinePipeline::new(&device);

        MetalRenderer {
            device,
            layer,
            command_queue,
            depth_state,
            depth_target,
            camera,
            uniforms,
            sphere_pipeline,
            line_pipeline: widget_pipeline,
        }
    }
}

impl Renderer for MetalRenderer {
    fn resized(&mut self, new_size: (u32, u32)) {
        self.layer
            .set_drawable_size(CGSize::new(new_size.0 as f64, new_size.1 as f64));

        self.depth_target = prepare_depth_target(&self.device, new_size);

        self.camera
            .aspect_ratio_updated(new_size.0 as f32 / new_size.1 as f32);
        self.uniforms.camera_updated(&self.camera)
    }

    fn rescaled(&mut self, new_scale_factor: f64) {
        self.layer.set_contents_scale(new_scale_factor);
    }

    fn draw(&mut self, graph: &RenderGraph) {
        let mut surface = Surface::new();
        let mut segments = vec![];

        graph.walk(|transform, kind| match kind {
            Kind::Line(l) => line_segments(l, &mut segments, &transform),
            Kind::Shape(s) => surface.push(
                transform.try_inverse().expect("transform can be inverted"),
                *s,
            ),
        });

        let drawable = match self.layer.next_drawable() {
            Some(drawable) => drawable,
            None => return,
        };

        let render_pass = metal::RenderPassDescriptor::new();
        let color_attachment = render_pass.color_attachments().object_at(0).unwrap();
        color_attachment.set_texture(Some(drawable.texture()));
        color_attachment.set_load_action(MTLLoadAction::Clear);
        color_attachment.set_clear_color(MTLClearColor::new(0.960, 0.991, 0.960, 1.0));
        color_attachment.set_store_action(MTLStoreAction::Store);

        let depth_attachment = render_pass.depth_attachment().unwrap();
        depth_attachment.set_texture(Some(&self.depth_target));
        depth_attachment.set_clear_depth(1.0);
        depth_attachment.set_load_action(MTLLoadAction::Clear);
        depth_attachment.set_store_action(MTLStoreAction::DontCare);

        let command_buffer = self.command_queue.new_command_buffer();
        let encoder = command_buffer.new_render_command_encoder(render_pass);

        if !surface.empty() {
            encoder.set_depth_stencil_state(&self.depth_state);
            encoder.set_vertex_buffer(2, Some(self.uniforms.buffer()), 0);
            self.sphere_pipeline.draw(encoder, &surface, 0.4);
        }
        encoder.set_depth_stencil_state(&self.depth_state);
        encoder.set_vertex_buffer(2, Some(self.uniforms.buffer()), 0);
        self.line_pipeline.draw(encoder, &segments);

        encoder.end_encoding();
        command_buffer.present_drawable(drawable);
        command_buffer.commit();
    }
}
