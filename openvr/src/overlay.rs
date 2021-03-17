use openvr_sys as sys;

use std::{
    ffi::CStr,
    mem,
    ptr,
    slice,
};

use crate::{
    error_ext::*,
};

pub fn find_overlay<K: AsRef<CStr>>(k: K) -> Result<sys::VROverlayHandle_t, sys::EVROverlayError> {
    let k = k.as_ref();
    let (e, handle) = unsafe {
        let mut handle = mem::zeroed();
        let e = openvr_utils_find_overlay(k.as_ptr(), &mut handle as *mut _);
        (e, handle)
    };
    e.into_result().map(move |_| handle)
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
            openvr_utils_get_overlay_image_data(handle, &mut ret as *mut _).into_result()?;
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

extern "C" {
    fn openvr_utils_find_overlay(key: *const libc::c_char, handle: *mut sys::VROverlayHandle_t) -> sys::EVROverlayError;
    fn openvr_utils_get_overlay_image_data(handle: sys::VROverlayHandle_t, data: *mut OverlayImageData) -> sys::EVROverlayError;
    fn openvr_utils_overlay_image_data_destroy(data: *mut libc::c_void);
    fn openvr_utils_overlay_image_data_get_data(data: *mut libc::c_void) -> BufferData;
    fn openvr_utils_overlay_image_data_get_dimensions(data: *mut libc::c_void) -> Dimensions;
    fn openvr_utils_overlay_image_data_refill(data: *mut libc::c_void, handle: sys::VROverlayHandle_t) -> sys::EVROverlayError;
}
