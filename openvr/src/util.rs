use openvr_sys as sys;

extern "C" {
    pub fn obs_openvr_init_openvr(e: *mut sys::EVRInitError, application_type: sys::EVRApplicationType);
    pub fn obs_openvr_shutdown_openvr();
    pub fn obs_openvr_vrcompositor_releasesharedgltexture(id: sys::glUInt_t, handle: sys::glSharedTextureHandle_t) -> bool;
}
