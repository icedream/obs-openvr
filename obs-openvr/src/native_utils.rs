use std::{
    ops::Deref,
    slice,
    sync::Once,
};

#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub struct CopyCtx {
    pub texture: u32,
    pub buffer: u32,
    img_size: libc::size_t,
    img: *mut u8
}

impl CopyCtx {
    #[inline(always)]
    pub fn img_size(&self) -> usize {
        self.img_size as usize
    }

    pub fn image_buffer<'a>(&'a self) -> Option<&'a [u8]> {
        if self.img.is_null() {
            None
        } else {
            unsafe {
                Some(slice::from_raw_parts(self.img as *const u8, self.img_size as usize))
            }
        }
    }
}

pub struct CopyContext(*mut CopyCtx);

impl Deref for CopyContext {
    type Target = CopyCtx;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref().unwrap() }
    }
}

static INITIALIZE: Once = Once::new();

fn init() {
    INITIALIZE.call_once(|| {
        unsafe {
            trace!("initializing native tools");
            obs_openvr_utils_init();
        }
    });
}

#[inline(always)]
fn status_to_result(status: i32) -> Result<(), i32> {
    if status == 0 {
        Ok(())
    } else {
        Err(status)
    }
}

impl CopyContext {
    pub fn new(texture: u32) -> Option<Self> {
        init();
        let ptr = unsafe { obs_openvr_copy_context_create(texture) };
        if ptr.is_null() {
            None
        } else {
            Some(CopyContext(ptr))
        }
    }

    pub fn copy_texture(&mut self, width: u32, height: u32, format: u32) -> Result<(), i32> {
        let status = unsafe {
            obs_openvr_copy_texture(self.0, width, height, format)
        };
        status_to_result(status)
    }
}

impl Drop for CopyContext {
    fn drop(&mut self) {
        unsafe {
            obs_openvr_copy_context_destroy(self.0);
        }
    }
}

extern "C" {
    fn obs_openvr_utils_init();
    fn obs_openvr_copy_context_create(texture: u32) -> *mut CopyCtx;
    fn obs_openvr_copy_context_destroy(ctx: *mut CopyCtx);
    fn obs_openvr_copy_texture(ctx: *mut CopyCtx, width: u32, height: u32, format: u32) -> i32;
}
