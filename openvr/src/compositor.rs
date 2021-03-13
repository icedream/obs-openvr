use openvr_sys as sys;
use crate::util;
use crate::error_ext::{
    ErrorType,
    ErrorTypeExt,
};

use std::{
    ptr,
    marker::PhantomData,
};

pub struct MirrorTextureLock<'a>(sys::glSharedTextureHandle_t, PhantomData<&'a MirrorTextureInfo>);

impl<'a> MirrorTextureLock<'a> {
    unsafe fn new(handle: sys::glSharedTextureHandle_t) -> Self {
        trace!("locking shared gl texture: {:x}", handle as usize);
        obs_openvr_vrcompositor_locksharedgltexture(handle);
        MirrorTextureLock(handle, PhantomData {})
    }
}

impl<'a> Drop for MirrorTextureLock<'a> {
    fn drop(&mut self) {
        unsafe {
            trace!("unlocking shared gl texture: {:x}", self.0 as usize);
            obs_openvr_vrcompositor_unlocksharedgltexture(self.0);
        }
    }
}

#[derive(Debug)]
pub struct MirrorTextureInfo {
    pub id: sys::glUInt_t,
    pub handle: sys::glSharedTextureHandle_t,
}

impl MirrorTextureInfo {
    pub const fn empty() -> Self {
        MirrorTextureInfo {
            id: 0,
            handle: ptr::null_mut(),
        }
    }

    pub unsafe fn lock<'a>(&'a self) -> MirrorTextureLock<'a> {
        MirrorTextureLock::new(self.handle)
    }
}

impl Drop for MirrorTextureInfo {
    fn drop(&mut self) {
        if (self.id <= 0 || self.handle.is_null()) {
            return;
        }
        unsafe {
            util::obs_openvr_vrcompositor_releasesharedgltexture(self.id, self.handle);
        }
        self.id = 0;
        self.handle = ptr::null_mut();
    }
}

pub unsafe fn get_mirror_texture_gl(eye: sys::EVREye) -> Result<MirrorTextureInfo, sys::EVRCompositorError> {
    let mut info = MirrorTextureInfo::empty();
    let e = obs_openvr_vrcompositor_getmirrortexturegl(eye, &mut info.id as *mut _, &mut info.handle as *mut _);
    e.into_result()
        .map(|_| info)
}

extern "C" {
    pub fn obs_openvr_vrcompositor_getmirrortexturegl(eye: sys::EVREye, tex_id: *mut sys::glUInt_t, tex_handle: *mut sys::glSharedTextureHandle_t) -> sys::EVRCompositorError;
    fn obs_openvr_vrcompositor_locksharedgltexture(handle: sys::glSharedTextureHandle_t);
    fn obs_openvr_vrcompositor_unlocksharedgltexture(handle: sys::glSharedTextureHandle_t);
}
