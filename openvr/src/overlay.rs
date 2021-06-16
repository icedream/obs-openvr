use openvr_sys as sys;

use std::{
    ffi::CStr,
    fmt::{
        self,
        Display,
    },
    mem,
    ptr,
    slice,
};

use crate::{
    error_ext::*,
};

#[derive(Debug, Clone, Copy)]
pub struct OverlayRef(sys::VROverlayHandle_t);

impl Display for OverlayRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.handle())
    }
}

impl OverlayRef {
    #[inline(always)]
    pub fn handle(&self) -> sys::VROverlayHandle_t {
        self.0
    }

    #[inline]
    pub fn is_visible(&self) -> bool {
        unsafe { openvrs_is_overlay_visible(self.handle()) }
    }
}

impl From<sys::VROverlayHandle_t> for OverlayRef {
    #[inline]
    fn from(handle: sys::VROverlayHandle_t) -> Self {
        OverlayRef(handle)
    }
}

pub fn find_overlay<K: AsRef<CStr>>(k: K) -> Result<OverlayRef, sys::EVROverlayError> {
    let k = k.as_ref();
    let (e, handle) = unsafe {
        let mut handle = mem::zeroed();
        let e = openvr_utils_find_overlay(k.as_ptr(), &mut handle as *mut _);
        (e, handle)
    };
    e.into_result().map(move |_| OverlayRef::from(handle))
}

#[repr(C)]
struct OverlayImageInfo {
    width: u32,
    height: u32,
    data: *mut u8,
    length: usize,
}

#[repr(C)]
struct Dimensions {
    width: u32,
    height: u32,
}

#[repr(C)]
struct BufferData {
    size: libc::size_t,
    data: *mut u8,
}

impl BufferData {
    unsafe fn as_slice<'a>(&self) -> &'a [u8] {
        slice::from_raw_parts(self.data, self.size as usize)
    }
}

#[repr(C)]
pub struct OverlayImageData(*mut libc::c_void);

impl OverlayImageData {
    pub fn find_overlay<K: AsRef<CStr>>(k: K) -> Result<Self, sys::EVROverlayError> {
        let handle = find_overlay(k)?;
        let mut ret = OverlayImageData(ptr::null_mut());
        unsafe {
            openvr_utils_get_overlay_image_data(handle.handle(), &mut ret as *mut _).into_result()?;
        }
        Ok(ret)
    }

    pub fn data<'a>(&'a self) -> &'a [u8] {
        unsafe {
            let buffer_data = openvr_utils_overlay_image_data_get_data(self.0);
            buffer_data.as_slice()
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        let size = unsafe {
            openvr_utils_overlay_image_data_get_dimensions(self.0)
        };
        (size.width, size.height)
    }

    pub fn refill(&mut self, handle: sys::VROverlayHandle_t) -> Result<(), sys::EVROverlayError> {
        unsafe {
            openvr_utils_overlay_image_data_refill(self.0, handle).into_empty_result()
        }
    }
}

impl Drop for OverlayImageData {
    fn drop(&mut self) {
        unsafe {
            if let Some(p) = self.0.as_mut() {
                openvr_utils_overlay_image_data_destroy(p as *mut _);
            }
        }
    }
}

pub struct OverlayImage(*mut libc::c_void);

impl OverlayImage {
    #[inline(always)]
    pub fn new() -> OverlayImage {
        let p = unsafe {
            openvrs_overlay_image_create()
        };
        OverlayImage(p)
    }

    pub fn fill(&mut self, handle: sys::VROverlayHandle_t) -> Result<(), sys::EVROverlayError> {
        let status = unsafe {
            openvrs_overlay_image_fill(self.0, handle)
        };
        status.into_empty_result()
    }

    #[inline(always)]
    fn info(&self) -> OverlayImageInfo {
        unsafe {
            openvrs_overlay_image_get_data(self.0)
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        let info = self.info();
        (info.width, info.height)
    } 

    pub fn data<'a>(&'a self) -> &'a [u8] {
        let info = self.info();
        unsafe {
            slice::from_raw_parts(info.data, info.length)
        }
    }
}

impl Drop for OverlayImage {
    fn drop(&mut self) {
        if self.0.is_null() {
            return;
        }
        unsafe {
            openvrs_overlay_image_destroy(self.0);
        }
    }
}

extern "C" {
    fn openvrs_overlay_image_create() -> *mut libc::c_void;
    fn openvrs_overlay_image_destroy(image: *mut libc::c_void);
    fn openvrs_overlay_image_fill(image: *mut libc::c_void, handle: sys::VROverlayHandle_t) -> sys::EVROverlayError;
    fn openvrs_overlay_image_get_data(image: *mut libc::c_void) -> OverlayImageInfo;
    fn openvrs_is_overlay_visible(handle: sys::VROverlayHandle_t) -> bool;
    fn openvr_utils_find_overlay(key: *const libc::c_char, handle: *mut sys::VROverlayHandle_t) -> sys::EVROverlayError;
    fn openvr_utils_get_overlay_image_data(handle: sys::VROverlayHandle_t, data: *mut OverlayImageData) -> sys::EVROverlayError;
    fn openvr_utils_overlay_image_data_destroy(data: *mut libc::c_void);
    fn openvr_utils_overlay_image_data_get_data(data: *mut libc::c_void) -> BufferData;
    fn openvr_utils_overlay_image_data_get_dimensions(data: *mut libc::c_void) -> Dimensions;
    fn openvr_utils_overlay_image_data_refill(data: *mut libc::c_void, handle: sys::VROverlayHandle_t) -> sys::EVROverlayError;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overlay_image_starts_with_empty_data() {
        let image = OverlayImage::new();
        assert_eq!(image.data().len(), 0);
    }
}
