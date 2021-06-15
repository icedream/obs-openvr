use std::{
    fmt::{
        self,
        Debug,
    },
};
use crate::mirror::utils::{
    self,
    TextureFormat,
};
use obs::graphics::GsTexture;

#[derive(Debug, thiserror::Error)]
pub enum TextureCreationError {
    #[error("Error translating texture format to obs")]
    FormatTranslation(TextureFormat),
    #[error("Error allocating OBS texture")]
    TextureAllocation,
}

#[derive(Debug, thiserror::Error)]
pub enum CopyTextureError {
    #[error("OpenGL error: {0}")]
    Gl(u32),
    #[error("Error creating OBS texture: {0}")]
    TextureCreation(#[from] TextureCreationError)
}

fn required_buffer_size(dimensions: (i32, i32), format: TextureFormat) -> usize {
    dimensions.0 as usize * dimensions.1 as usize * format.bytes_per_pixel() as usize
}

pub struct OpenVRMirrorCapture {
    eye: openvr::sys::EVREye,
    texture_info: openvr::compositor::MirrorTextureInfo,
    dimensions: (i32, i32),
    format: TextureFormat,
    buffer: Vec<u8>,
    texture_flags: u32,
    texture: Option<obs::graphics::Texture>,
}

impl Debug for OpenVRMirrorCapture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenVRMirrorCapture")
            .field("eye", &self.eye)
            .field("texture_info", &self.texture_info)
            .field("dimensions", &self.dimensions)
            .field("format", &self.format)
            .finish()
    }
}

impl OpenVRMirrorCapture {
    pub fn new(eye: openvr::sys::EVREye, texture_flags: u32) -> Result<Self, openvr::sys::EVRCompositorError> {
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
        trace!("Created capture context with texture info: {:?}", &texture_info);
        let ret = OpenVRMirrorCapture {
            eye: eye,
            texture_info: texture_info,
            dimensions: texture_size.into(),
            format: format,
            buffer: vec![0; required_buffer_size(texture_size.into(), format)],
            texture_flags: texture_flags,
            texture: None,
        };
        trace!("Created capture context: {:?}", &ret);
        Ok(ret)
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

    pub unsafe fn copy_texture(&mut self) -> Result<(), CopyTextureError> {
        utils::copy_gl_texture(self.texture_info.id, self.format.into(), self.buffer.as_mut_ptr())
            .map_err(CopyTextureError::Gl)?;
        let linesize = self.linesize();
        if let Some(texture) = self.texture.as_mut() {
            texture.set_image_unchecked(self.buffer.as_slice(), linesize, false);
            Ok(())
        } else {
            self.texture = self.create_texture()
                .map(Some)
                .map_err(CopyTextureError::TextureCreation)?;
            Ok(())
        }
    }

    #[inline(always)]
    pub fn dimensions(&self) -> (i32, i32) {
        self.dimensions
    }

    #[inline(always)]
    pub fn width(&self) -> i32 {
        self.dimensions.0
    }

    #[inline(always)]
    pub fn height(&self) -> i32 {
        self.dimensions.1
    }

    fn linesize(&self) -> u32 {
        self.width() as u32 * self.format.bytes_per_pixel() as u32
    }

    #[inline(always)]
    pub fn texture<'a>(&'a self) -> Option<&'a obs::graphics::Texture> {
        self.texture.as_ref()
    }

    unsafe fn create_texture(&self) -> Result<obs::graphics::Texture, TextureCreationError> {
        let (w, h) = self.dimensions();
        let format: Option<obs::sys::gs_color_format> = self.format.into();
        let format = format
            .map(Ok)
            .unwrap_or_else(|| Err(TextureCreationError::FormatTranslation(self.format)))?;
        obs::graphics::Texture::new(w as u32, h as u32, format, &[self.buffer.as_slice().as_ptr()], self.texture_flags)
            .map(Ok)
            .unwrap_or(Err(TextureCreationError::TextureAllocation))
    }
}
