pub extern crate obs_sys as sys;
extern crate libc;
#[macro_use] extern crate log;
extern crate mcoffin_option_ext as option_ext;

pub mod enums;
pub mod properties;
pub mod graphics;
pub mod source;
pub mod data;
pub(crate) mod ptr;

pub use data::Data;
pub use properties::Properties;

use std::{
    io,
    mem,
    fmt::Display,
};

pub unsafe fn register_source(info: &'static sys::obs_source_info, info_size: Option<usize>) {
    let info_size = info_size.unwrap_or(mem::size_of::<sys::obs_source_info>());
    sys::obs_register_source_s(info as *const _, info_size as u64);
}

#[macro_export]
macro_rules! register_video_source {
    ($t:ty) => {
        {
            static mut SOURCE_INFO: Option<$crate::sys::obs_source_info> = None;
            static FILL_INFO: std::sync::Once = std::sync::Once::new();
            FILL_INFO.call_once(|| {
                unsafe {
                    SOURCE_INFO = Some(<$t as $crate::source::VideoSource>::raw_source_info().unwrap());
                }
            });
            unsafe {
                $crate::register_source(SOURCE_INFO.as_ref().unwrap(), None)
            }
        }
    };
}

#[macro_export]
macro_rules! register_async_video_source {
    ($t:ty) => {
        {
            static mut SOURCE_INFO: Option<$crate::sys::obs_source_info> = None;
            static FILL_INFO: std::sync::Once = std::sync::Once::new();
            FILL_INFO.call_once(|| {
                unsafe {
                    SOURCE_INFO = Some(<$t as $crate::source::AsyncVideoSource>::raw_source_info().unwrap());
                }
            });
            unsafe {
                $crate::register_source(SOURCE_INFO.as_ref().unwrap(), None)
            }
        }
    };
}

pub trait ObsModule {
    type LoadErr;
    type UnloadErr;
    const CRATE_NAME: &'static str;
    fn load() -> Result<(), Self::LoadErr>;
    fn unload() -> Result<(), Self::UnloadErr>;
}

pub trait ObsModuleExt: ObsModule {
    fn do_load() -> bool;
    fn do_unload();
}

impl<T> ObsModuleExt for T where
    T: ObsModule,
    <T as ObsModule>::LoadErr: Display,
    <T as ObsModule>::UnloadErr: Display,
{
    fn do_load() -> bool {
        match <T as ObsModule>::load() {
            Ok(_) => true,
            Err(e) => {
                use std::io::Write;
                let stderr = io::stderr();
                let mut stderr = stderr.lock();
                let _ = write!(&mut stderr, "error loading {}: {}", <T as ObsModule>::CRATE_NAME, &e);
                false
            },
        }
    }

    fn do_unload() {
        if let Err(e) = <T as ObsModule>::unload() {
            use std::io::Write;
            let stderr = io::stderr();
            let mut stderr = stderr.lock();
            let _ = write!(&mut stderr, "error unloading: {}: {}", <T as ObsModule>::CRATE_NAME, &e);
        }
    }
}

#[macro_export]
macro_rules! register_module {
    ($t:ty) => {
        #[no_mangle]
        pub extern "C" fn obs_module_load() -> bool {
            <$t as $crate::ObsModuleExt>::do_load()
        }

        #[no_mangle]
        pub extern "C" fn obs_module_unload() {
            <$t as $crate::ObsModuleExt>::do_unload();
        }
    };
}

pub trait OwnedPointerContainer<T> {
    fn as_ptr(&self) -> *const T;
    fn as_ptr_mut(&mut self) -> *mut T;
    unsafe fn leak(self) -> *mut T;
}
