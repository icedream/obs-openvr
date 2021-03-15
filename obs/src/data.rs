use obs_sys as sys;

use std::{
    ffi::CStr,
    mem,
    ops::{
        Deref,
        DerefMut,
    },
};
use crate::OwnedPointerContainer;

pub trait ObsData {
    fn get_string<'a, K: AsRef<CStr>>(&'a self, s: K) -> Option<&'a str>;
}

impl ObsData for sys::obs_data {
    fn get_string<'a, K: AsRef<CStr>>(&'a self, s: K) -> Option<&'a str> {
        let s = s.as_ref();
        let self_ptr: *mut sys::obs_data = unsafe {
            mem::transmute(self as *const _)
        };
        let ptr = unsafe {
            sys::obs_data_get_string(self_ptr, s.as_ptr())
        };
        let ptr = if ptr.is_null() {
            None
        } else {
            Some(ptr)
        };
        ptr
            .map(|ptr| unsafe { CStr::from_ptr(ptr) })
            .and_then(|s| s.to_str().ok())
    }
}

pub struct Data(*mut sys::obs_data);

impl Data {
    pub fn new() -> Option<Data> {
        let ptr = unsafe { 
            sys::obs_data_create()
        };
        if ptr.is_null() {
            None
        } else {
            Some(Data(ptr))
        }
    }

    pub unsafe fn from_raw(p: *mut sys::obs_data) -> Option<Data> {
        if p.is_null() {
            return None;
        }
        sys::obs_data_addref(p);
        Some(Data(p))
    }
}

impl Drop for Data {
    fn drop(&mut self) {
        Some(self.0).into_iter()
            .filter(|p| !p.is_null())
            .for_each(|p| unsafe {
                sys::obs_data_release(p);
            });
    }
}

impl OwnedPointerContainer<sys::obs_data> for Data {
    #[inline(always)]
    fn as_ptr(&self) -> *const sys::obs_data {
        self.0 as _
    }
    
    #[inline(always)]
    fn as_ptr_mut(&mut self) -> *mut sys::obs_data {
        self.0
    }
}

impl Deref for Data {
    type Target = sys::obs_data;

    fn deref(&self) -> &Self::Target {
        unsafe {
            self.0.as_ref().unwrap()
        }
    }
}

impl DerefMut for Data {
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        unsafe {
            self.0.as_mut().unwrap()
        }
    }
}
