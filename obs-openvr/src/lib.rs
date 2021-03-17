extern crate obs;
extern crate openvr;
#[cfg(feature = "mirror-source")]
extern crate glfw;
extern crate image;
#[macro_use] extern crate log;
extern crate env_logger;

pub use obs::sys as obs_sys;

mod logging;
pub mod module;
#[cfg(feature = "mirror-source")]
pub(crate) mod native_utils;
pub(crate) mod timing;
#[cfg(feature = "overlay-source")]
pub mod overlay;
#[cfg(feature = "mirror-source")]
pub mod mirror;

pub use openvr::sys as openvr_sys;

use std::{
    io,
    fmt::Display,
};

fn obs_module_load_result() -> Result<(), impl Display + 'static> {
    use std::borrow::Cow;

    // Initialize logging
    logging::init();
    info!("logging initialized");

    // Initialize OpenVR
    let vr_initialized = openvr::init(openvr_sys::EVRApplicationType::EVRApplicationType_VRApplication_Background)
        .map(|result| result.value())
        .map_err(|e| Cow::Owned(format!("OpenVR failed to initialize: {:?}", &e)))?;
    if !vr_initialized {
        return Err(Cow::Borrowed("OpenVR failed to initialize, but with no error"));
    }

    // Create source info struct, and register it
    #[cfg(feature = "mirror-source")]
    obs::register_video_source!(mirror::OpenVRMirrorSource);
    #[cfg(feature = "overlay-source")]
    obs::register_video_source!(overlay::OpenVROverlaySource);

    trace!("loaded");
    Ok(())
}

#[no_mangle]
pub extern "C" fn obs_module_load() -> bool {
    match obs_module_load_result() {
        Ok(_) => true,
        Err(e) => {
            use io::Write;
            let stderr = io::stderr();
            let mut stderr = stderr.lock();
            let _ = write!(&mut stderr, "error loading {}: {}", env!("CARGO_CRATE_NAME"), &e);
            false
        },
    }
}

#[no_mangle]
pub extern "C" fn obs_module_unload() {
    if !openvr::shutdown() {
        warn!("OpenVR was not actually shut down on obs_module_unload");
    }
    trace!("unloaded");
}
