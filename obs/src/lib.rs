pub extern crate obs_sys as sys;
extern crate libc;
#[macro_use] extern crate log;

pub mod properties;
pub mod graphics;
pub mod source;
pub mod data;
pub(crate) mod ptr;

pub use data::Data;

use std::{
    mem,
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

pub trait OwnedPointerContainer<T> {
    fn as_ptr(&self) -> *const T;
    fn as_ptr_mut(&mut self) -> *mut T;
    unsafe fn leak(self) -> *mut T;
}
