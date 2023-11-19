// Uniforms are shared between all render passes

use creature_creator_renderer::Camera;

#[repr(C)]
pub struct Uniforms {
    camera: [[f32; 4]; 4],
    camera_position: [f32; 3],
    _wtf: [f32; 4],
}

impl Uniforms {
    pub fn new(camera: &Camera) -> Self {
        Self {
            camera: camera.mvp_matrix(),
            camera_position: camera.position(),
            _wtf: [0.0; 4],
        }
    }

    pub fn camera_updated(&mut self, camera: &Camera) {
        self.camera = camera.mvp_matrix();
        self.camera_position = camera.position();
    }
}
