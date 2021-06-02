#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    Rgb = 0x1907,
    Rgba = 0x1908,
}

impl TextureFormat {
    pub fn bytes_per_pixel(self) -> u8 {
        match self {
            TextureFormat::Rgb => 3,
            TextureFormat::Rgba => 4,
        }
    }

    #[inline]
    pub fn to_gs_format(self) -> Option<obs::sys::gs_color_format> {
        self.into()
    }
}

impl Into<Option<obs::sys::gs_color_format>> for TextureFormat {
    fn into(self) -> Option<obs::sys::gs_color_format> {
        use obs::sys::gs_color_format::*;
        match self {
            TextureFormat::Rgb => None,
            TextureFormat::Rgba => Some(GS_RGBA),
        }
    }
}

impl Into<u32> for TextureFormat {
    #[inline]
    fn into(self) -> u32 {
        self as u32
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlTextureSize {
    pub width: i32,
    pub height: i32,
}

impl GlTextureSize {
    pub const fn empty() -> Self {
        GlTextureSize {
            width: 0,
            height: 0,
        }
    }
}

impl Into<(i32, i32)> for GlTextureSize {
    #[inline]
    fn into(self) -> (i32, i32) {
        (self.width, self.height)
    }
}

pub unsafe fn get_gl_texture_size(texture: u32) -> GlTextureSize {
    let mut ret = GlTextureSize::empty();
    obs_openvr_get_gl_texture_size(texture, &mut ret as *mut _);
    ret
}

#[inline]
fn gl_error_to_result(status: u32) -> Result<(), u32> {
    if status == 0 {
        Ok(())
    } else {
        Err(status)
    }
}

pub unsafe fn copy_gl_texture(texture: u32, format: u32, img: *mut u8) -> Result<(), u32> {
    gl_error_to_result(obs_openvr_copy_gl_texture(texture, format, img))
}

extern "C" {
    fn obs_openvr_get_gl_texture_size(texture: u32, out: *mut GlTextureSize);
    fn obs_openvr_copy_gl_texture(texture: u32, format: u32, img: *mut u8) -> u32;
}
