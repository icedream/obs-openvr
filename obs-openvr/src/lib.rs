extern crate obs;
extern crate openvr;
#[cfg(feature = "mirror-source")]
extern crate image;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate thiserror;
extern crate mcoffin_option_ext;

pub use obs::sys as obs_sys;

mod logging;
pub mod module;
#[cfg(feature = "mirror-source")]
pub(crate) mod native_utils;
pub(crate) mod timing;
pub(crate) mod thread_utils;
#[cfg(feature = "overlay-source")]
pub mod overlay;
#[cfg(feature = "mirror-source")]
pub mod mirror;

pub use openvr::sys as openvr_sys;

use std::{
    borrow::Cow,
    sync::RwLock,
};

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum ObsOpenVRError {
    #[error("OpenVR initialization has not yet been attempted")]
    InitNotAttempted,
    #[error("OpenVR failed to initialize: {0:?}")]
    OpenVRInit(openvr_sys::EVRInitError),
    #[error("OpenVR failed to initialize, but with no error")]
    OpenVRInitNoError,
    #[error("OpenVR was not actually shut down on obs_module_unload")]
    OpenVRShutdown,
}

struct ObsOpenVRModule {}

impl ObsOpenVRModule {
    fn unload_internal() -> Result<(), <Self as obs::ObsModule>::UnloadErr> {
        trace!("unloading");
        if !openvr::shutdown() {
            return Err(ObsOpenVRError::OpenVRShutdown);
        }
        trace!("unloaded");
        Ok(())
    }
}

static OPENVR_INIT_RESULT: RwLock<Result<(), ObsOpenVRError>> = RwLock::new(Err(ObsOpenVRError::InitNotAttempted));
pub fn init_openvr() -> Result<(), ObsOpenVRError> {
    {
        let init_result = OPENVR_INIT_RESULT.read().unwrap();
        if init_result.is_ok() {
            return Ok(());
        }
    }
    let mut init_result = OPENVR_INIT_RESULT.write().unwrap();
    if init_result.is_ok() {
        return Ok(());
    }
    *init_result = {
        let vr_initialized = openvr::init(openvr_sys::EVRApplicationType::EVRApplicationType_VRApplication_Background)
            .map(|result| result.value())
            .map_err(ObsOpenVRError::OpenVRInit)?;
        if !vr_initialized {
            return Err(ObsOpenVRError::OpenVRInitNoError);
        }
        Ok(())
    };
    *init_result
}

impl obs::ObsModule for ObsOpenVRModule {
    const CRATE_NAME: &'static str = env!("CARGO_CRATE_NAME");
    type LoadErr = ObsOpenVRError;
    type UnloadErr = ObsOpenVRError;
    fn load() -> Result<(), Self::LoadErr> {
        // Initialize logging
        logging::init();
        info!("logging initialized");

        // Try to Initialize OpenVR
        if let Err(e) = init_openvr() {
            warn!("error initializing openvr on startup: {}", &e);
        }

        // Create source info struct, and register it
        #[cfg(feature = "mirror-source")]
        obs::register_video_source!(mirror::OpenVRMirrorSource);
        #[cfg(feature = "overlay-source")]
        obs::register_video_source!(overlay::OpenVROverlaySource);
        #[cfg(feature = "overlay-source")]
        obs::register_async_video_source!(overlay::OpenVRAsyncOverlaySource);

        trace!("loaded");
        Ok(())
    }
    fn unload() -> Result<(), Self::UnloadErr> {
        let ret = Self::unload_internal();
        if let Err(e) = ret.as_ref() {
            warn!("error unloading {}: {}", Self::CRATE_NAME, e);
        }
        ret
    }
}

obs::register_module!(ObsOpenVRModule);
