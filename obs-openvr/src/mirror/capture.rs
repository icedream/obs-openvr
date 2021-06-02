use crate::mirror::utils::{
    self,
    TextureFormat,
};

#[derive(Debug, thiserror::Error)]
pub enum CopyTextureError {
    #[error("OpenGL error: {0}")]
    Gl(u32),
}

fn required_buffer_size(dimensions: (i32, i32), format: TextureFormat) -> usize {
    dimensions.0 as usize * dimensions.1 as usize * format.bytes_per_pixel() as usize
}

#[derive(Debug)]
pub struct OpenVRMirrorCapture {
    eye: openvr::sys::EVREye,
    texture_info: openvr::compositor::MirrorTextureInfo,
    dimensions: (i32, i32),
    format: TextureFormat,
    buffer: Vec<u8>,
}

impl OpenVRMirrorCapture {
    pub fn new(eye: openvr::sys::EVREye) -> Result<Self, openvr::sys::EVRCompositorError> {
        trace!("Creating OpenVRMirrorCapture with eye: {:?}", &eye);
        let (texture_info, texture_size) = obs::graphics::with_graphics(|| {
            unsafe {
                openvr::compositor::get_mirror_texture_gl(eye)
                    .map(|info| {
                        let size = utils::get_gl_texture_size(info.id);
                        (info, size)
                    })
            }
        })?;
        let format = TextureFormat::Rgba;
        Ok(OpenVRMirrorCapture {
            eye: eye,
            texture_info: texture_info,
            dimensions: texture_size.into(),
            format: format,
            buffer: vec![0; required_buffer_size(texture_size.into(), format)],
        })
    }

    pub fn required_buffer_size(&self) -> usize {
        required_buffer_size(self.dimensions, self.format)
    }

    #[inline(always)]
    pub fn format(&self) -> TextureFormat {
        self.format
    }

    #[inline(always)]
    pub fn eye(&self) -> openvr::sys::EVREye {
        self.eye
    }

    pub fn copy_texture(&mut self) -> Result<(), CopyTextureError> {
        unsafe {
            utils::copy_gl_texture(self.texture_info.id, self.format.into(), self.buffer.as_mut_ptr())
                .map_err(CopyTextureError::Gl)
        }
    }
}
