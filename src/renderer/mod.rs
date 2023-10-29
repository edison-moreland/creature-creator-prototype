use cocoa::appkit::NSView;
use cocoa::base::id;
use core_graphics_types::geometry::CGSize;
use metal::objc::runtime::YES;
use metal::{
    CommandQueue, DepthStencilDescriptor, DepthStencilState, Device, DeviceRef, MTLClearColor,
    MTLCompareFunction, MTLLoadAction, MTLPixelFormat, MTLStorageMode, MTLStoreAction,
    MTLTextureUsage, MetalDrawableRef, MetalLayer, RenderCommandEncoderRef, Texture,
    TextureDescriptor,
};
use uniforms::Uniforms;
use winit::dpi::PhysicalSize;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::Window;

use crate::renderer::shared::Shared;
use crate::renderer::sphere_pass::SphereRenderPass;

pub use crate::renderer::sphere_pass::Sphere;
pub use crate::renderer::uniforms::Camera;

mod shared;
mod sphere_pass;
mod uniforms;

fn create_metal_layer(device: &DeviceRef, window: &Window) -> MetalLayer {
    let layer = MetalLayer::new();
    layer.set_device(device);
    layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
    layer.set_presents_with_transaction(false);

    let handle = window.window_handle().unwrap();
    if let RawWindowHandle::AppKit(handle) = handle.as_raw() {
        unsafe {
            let view = handle.ns_view.as_ptr() as id;
            view.setWantsLayer(YES);
            view.setLayer(
                layer.as_ref() as *const metal::MetalLayerRef as *mut metal::objc::runtime::Object
            );
        }
    }

    let draw_size = window.inner_size();
    layer.set_drawable_size(CGSize::new(draw_size.width as f64, draw_size.height as f64));

    let scale_factor = window.scale_factor();
    layer.set_contents_scale(scale_factor);

    layer
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

pub struct Renderer {
    device: Device,
    layer: MetalLayer,
    command_queue: CommandQueue,

    depth_state: DepthStencilState,
    depth_target: Texture,

    camera: Camera,
    uniforms: Shared<Uniforms>,

    sphere_render_pass: SphereRenderPass,
}

impl Renderer {
    pub fn new(window: &Window, mut camera: Camera) -> Self {
        let device = Device::system_default().expect("no device found");
        let command_queue = device.new_command_queue();

        let layer = create_metal_layer(&device, window);

        let size = window.inner_size();

        let depth_target = prepare_depth_target(&device, size);
        let depth_state = create_depth_state(&device);

        camera.aspect_ratio_updated(size.width as f32 / size.height as f32);
        let uniforms = Shared::new(&device, Uniforms::new(&camera));

        let sphere_render_pass = SphereRenderPass::new(&device);

        Renderer {
            device,
            layer,
            command_queue,
            depth_state,
            depth_target,
            camera,
            uniforms,
            sphere_render_pass,
        }
    }

    pub fn resized(&mut self, new_size: PhysicalSize<u32>) {
        self.layer
            .set_drawable_size(CGSize::new(new_size.width as f64, new_size.height as f64));

        self.depth_target = prepare_depth_target(&self.device, new_size);

        self.camera
            .aspect_ratio_updated(new_size.width as f32 / new_size.height as f32);
        self.uniforms.camera_updated(&self.camera)
    }

    pub fn rescaled(&self, scale_factor: f64) {
        self.layer.set_contents_scale(scale_factor);
    }

    pub fn draw(&mut self, instances: &[Sphere]) {
        let drawable = match self.layer.next_drawable() {
            Some(drawable) => drawable,
            None => return,
        };

        self.sphere_render_pass.update_instance_buffer(instances);

        self.render_pass(
            drawable,
            self.sphere_render_pass
                .draw_spheres(&self.depth_state, &self.uniforms),
        )
    }

    fn render_pass<F>(&self, drawable: &MetalDrawableRef, f: F)
    where
        F: FnOnce(&RenderCommandEncoderRef),
    {
        let render_pass = metal::RenderPassDescriptor::new();
        let color_attachment = render_pass.color_attachments().object_at(0).unwrap();
        color_attachment.set_texture(Some(drawable.texture()));
        color_attachment.set_load_action(MTLLoadAction::Clear);
        color_attachment.set_clear_color(MTLClearColor::new(1.0, 1.0, 1.0, 1.0));
        color_attachment.set_store_action(MTLStoreAction::Store);

        let depth_attachment = render_pass.depth_attachment().unwrap();
        depth_attachment.set_texture(Some(&self.depth_target));
        depth_attachment.set_clear_depth(1.0);
        depth_attachment.set_load_action(MTLLoadAction::Clear);
        depth_attachment.set_store_action(MTLStoreAction::DontCare);

        let command_buffer = self.command_queue.new_command_buffer();
        let encoder = command_buffer.new_render_command_encoder(render_pass);

        f(encoder);

        encoder.end_encoding();
        command_buffer.present_drawable(drawable);
        command_buffer.commit();
    }
}
