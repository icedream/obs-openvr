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
use thiserror::Error;

#[derive(Error, Debug)]
enum ObsOpenVRUnloadError {
    #[error("OpenVR was not actually shut down on obs_module_unload")]
    OpenVRShutdown,
}

struct ObsOpenVRModule {}

impl ObsOpenVRModule {
    fn unload_internal() -> Result<(), <Self as obs::ObsModule>::UnloadErr> {
        trace!("unloading");
        if !openvr::shutdown() {
            return Err(ObsOpenVRUnloadError::OpenVRShutdown);
        }
        trace!("unloaded");
        Ok(())
    }
}

static OPENVR_INIT_RESULT: RwLock<Result<(), Cow<'static, str>>> = RwLock::new(Err(Cow::Borrowed("")));
pub fn init_openvr() -> Result<(), Cow<'static, str>> {
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
    let do_init = || -> Result<(), Cow<'static, str>> {
        let vr_initialized = openvr::init(openvr_sys::EVRApplicationType::EVRApplicationType_VRApplication_Background)
            .map(|result| result.value())
            .map_err(|e| Cow::Owned(format!("OpenVR failed to initialize: {:?}", &e)))?;
        if !vr_initialized {
            return Err(Cow::Borrowed("OpenVR failed to initialize, but with no error"));
        }
        Ok(())
    };
    *init_result = do_init();
    init_result.clone()
}

impl obs::ObsModule for ObsOpenVRModule {
    const CRATE_NAME: &'static str = env!("CARGO_CRATE_NAME");
    type LoadErr = Cow<'static, str>;
    type UnloadErr = ObsOpenVRUnloadError;
    fn load() -> Result<(), Self::LoadErr> {
        // Initialize logging
        logging::init();
        info!("logging initialized");

        // Try to Initialize OpenVR
        if let Err(e) = init_openvr() {
            warn!("error initializing openvr: {}", &e);
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
