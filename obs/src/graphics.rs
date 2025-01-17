use obs_sys as sys;

use std::{
    ops::{
        Deref,
        DerefMut,
    },
    ptr,
};

/// Saves the obs graphics context, runs the provided function, then restores the original graphics
/// context
pub fn isolate_context<Ret, F>(f: F) -> Ret where
    F: FnOnce() -> Ret,
{
    let previous_ctx = unsafe {
        let ctx = sys::gs_get_context();
        sys::gs_leave_context();
        ctx
    };
    let ret = f();
    unsafe {
        sys::gs_enter_context(previous_ctx);
    }
    ret
}

unsafe fn enter_graphics() {
    // trace!("obs_enter_graphics");
    sys::obs_enter_graphics();
}

unsafe fn leave_graphics() {
    // trace!("obs_leave_graphics");
    sys::obs_leave_graphics();
}

/// Enters the obs graphics context, runs the provided function, then leaves the obs graphics
/// context. see: `obs_enter_graphics` and `obs_leave_graphics` in the obs-studio API
pub fn with_graphics<Ret, F: FnMut() -> Ret>(mut f: F) -> Ret {
    unsafe { enter_graphics(); }
    let ret = f();
    unsafe { leave_graphics(); }
    ret
}

pub trait GsTexture {
    unsafe fn set_image_unchecked(&mut self, data: &[u8], linesize: u32, inverted: bool);
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn get_color_format(&self) -> sys::gs_color_format;

    fn get_dimensions(&self) -> (u32, u32) {
        (self.get_width(), self.get_height())
    }
}

impl GsTexture for sys::gs_texture_t {
    unsafe fn set_image_unchecked(&mut self, data: &[u8], linesize: u32, inverted: bool) {
        sys::gs_texture_set_image(self as *mut _, data.as_ptr(), linesize, inverted);
    }

    fn get_width(&self) -> u32 {
        unsafe {
            sys::gs_texture_get_width(self as *const _)
        }
    }

    fn get_height(&self) -> u32 {
        unsafe {
            sys::gs_texture_get_height(self as *const _)
        }
    }

    fn get_color_format(&self) -> sys::gs_color_format {
        unsafe {
            sys::gs_texture_get_color_format(self as *const _)
        }
    }
}

pub struct Texture(*mut sys::gs_texture_t);

impl Texture {
    pub unsafe fn new(width: u32, height: u32, format: sys::gs_color_format, data: &[*const u8], flags: u32) -> Option<Self> {
        let datap = data.as_ptr() as *mut _;
        let p = sys::gs_texture_create(width, height, format, data.len() as u32, datap, flags);
        if p.is_null() {
            None
        } else {
            Some(Texture(p))
        }
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *const sys::gs_texture_t {
        self.0 as *const _
    }

    #[inline(always)]
    pub fn as_mut(&mut self) -> *mut sys::gs_texture_t {
        self.0
    }

    pub unsafe fn leak(&mut self) -> *mut sys::gs_texture_t {
        let ret = self.0;
        self.0 = ptr::null_mut();
        ret
    }
}

impl Deref for Texture {
    type Target = sys::gs_texture_t;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe {
            self.0.as_ref().unwrap()
        }
    }
}

impl DerefMut for Texture {
    #[inline]
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        unsafe {
            self.0.as_mut().unwrap()
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        with_graphics(|| {
            unsafe {
                let p = self.leak();
                if !p.is_null() {
                    sys::gs_texture_destroy(p);
                }
            }
        });
    }
}
