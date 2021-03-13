use openvr_sys as sys;

pub struct HeadsetView(*mut libc::c_void);

impl HeadsetView {
    pub fn global() -> Option<Self> {
        unsafe {
            let ptr = openvr_utils_get_headset_view();
            ptr.as_mut().map(|ptr| HeadsetView(ptr as *mut libc::c_void))
        }
    }

    pub fn get_size(&self) -> HeadsetViewSize {
        unsafe {
            openvr_utils_headset_view_get_size(self.0)
        }
    }

    pub fn get_aspect_ratio(&self) -> f32 {
        unsafe {
            openvr_utils_headset_view_get_aspect_ratio(self.0)
        }
    }

    pub fn get_mode(&self) -> sys::HeadsetViewMode_t {
        unsafe {
            openvr_utils_headset_view_get_mode(self.0)
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeadsetViewSize {
    pub width: u32,
    pub height: u32,
}

extern "C" {
    fn openvr_utils_get_headset_view() -> *mut libc::c_void;
    fn openvr_utils_headset_view_get_size(headset_view: *mut libc::c_void) -> HeadsetViewSize;
    fn openvr_utils_headset_view_get_aspect_ratio(headset_view: *mut libc::c_void) -> f32;
    fn openvr_utils_headset_view_get_mode(headset_view: *mut libc::c_void) -> sys::HeadsetViewMode_t;
}
