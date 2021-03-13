#[macro_use] extern crate log;
extern crate libc;
pub extern crate openvr_sys as sys;

pub mod sys_expose;
pub mod error_ext;
pub mod util;
pub mod compositor;
pub mod headset_view;

use error_ext::{
    ErrorType,
    ErrorTypeExt,
};

use std::{
    sync::atomic::{
        AtomicBool,
        Ordering,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitResult {
    OtherInitializer(bool),
    Initialized(bool),
}

impl InitResult {
    #[inline(always)]
    pub fn new(is_other: bool, value: bool) -> InitResult {
        use InitResult::*;
        if is_other {
            OtherInitializer(value)
        } else {
            Initialized(value)
        }
    }

    #[inline(always)]
    pub fn value(&self) -> bool {
        use InitResult::*;
        match *self {
            OtherInitializer(v) => v,
            Initialized(v) => v,
        }
    }

    #[inline(always)]
    pub fn is_other(&self) -> bool {
        if let &InitResult::OtherInitializer(_) = self {
            true
        } else {
            false
        }
    }
}

static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initializes the openvr system, returning a result indicating that another initialization was
/// already ocurring, or that we successfully initialized
pub fn init(application_type: sys::EVRApplicationType) -> Result<InitResult, sys::EVRInitError> {
    if INITIALIZED.fetch_or(true, Ordering::SeqCst) {
        println!("obs-openvr: OpenVR was already initialized?");
        Ok(InitResult::new(true, true))
    } else {
        // println!("obs-openvr: Initializing OpenVR");
        let mut e = sys::EVRInitError::non_error();
        unsafe { util::obs_openvr_init_openvr(&mut e as *mut sys::EVRInitError, application_type); }
        // unsafe { sys2::VR_InitInternal(&mut e as *mut sys::EVRInitError, application_type); }
        e.into_empty_result()?;
        Ok(InitResult::new(false, true))
    }
}

/// Shuts down openvr, returning true if openvr was initialized, and shutdown was actually called
pub fn shutdown() -> bool {
    if INITIALIZED.fetch_and(false, Ordering::SeqCst) {
        unsafe { util::obs_openvr_shutdown_openvr(); }
        // unsafe { sys2::VR_ShutdownInternal(); }
        true
    } else {
        false
    }
}
