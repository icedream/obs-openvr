pub extern crate obs_sys as sys;
extern crate libc;
#[macro_use] extern crate log;

pub mod properties;
pub mod graphics;
pub mod source;

use std::{
    mem,
};

pub unsafe fn register_source(info: &'static sys::obs_source_info, info_size: Option<usize>) {
    let info_size = info_size.unwrap_or(mem::size_of::<sys::obs_source_info>());
    sys::obs_register_source_s(info as *const _, info_size as u64);
}
