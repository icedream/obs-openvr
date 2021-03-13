#![allow(non_snake_case)]

use openvr_sys as sys;

extern "C" {
    pub fn VR_InitInternal(e: *mut sys::EVRInitError, application_type: sys::EVRApplicationType);
    pub fn VR_ShutdownInternal();
}
