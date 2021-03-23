use obs_sys as sys;

use std::{
    ffi::CStr,
    mem,
    ops::{
        Deref,
        DerefMut,
    },
    ptr,
    str::FromStr,
};
use crate::{
    OwnedPointerContainer,
    enums::ObsEnum,
};

/// Safe access functions for `sys::obs_data`
pub trait ObsData {
    fn get_string<'a, K: AsRef<CStr>>(&'a self, s: K) -> Option<&'a str>;
    fn get_int<K: AsRef<CStr>>(&self, k: K) -> libc::c_longlong;
    fn get_bool<K: AsRef<CStr>>(&self, k: K) -> bool;
    fn get_string_enum<T: ObsEnum, K: AsRef<CStr>>(&self, k: K) -> Option<Result<T, <T as FromStr>::Err>> {
        self.get_string(k)
            .map(|s| s.parse::<T>())
    }
    fn get_string_enum_opt<T: ObsEnum, K: AsRef<CStr>>(&self, k: K) -> Option<T> {
        self.get_string(k)
            .and_then(|s| s.parse::<T>().ok())
    }
    fn get_string_enum_default<T, K: AsRef<CStr>>(&self, k: K) -> T where
        T: ObsEnum + Default,
    {
        use crate::option_ext::*;
        self.get_string_enum_opt(k).or_default()
    }
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
    fn get_int<K: AsRef<CStr>>(&self, k: K) -> libc::c_longlong
    {
        let k = k.as_ref();
        let self_ptr: *mut sys::obs_data = unsafe {
            mem::transmute(self as *const _)
        };
        unsafe {
            sys::obs_data_get_int(self_ptr, k.as_ptr())
        }
    }
    fn get_bool<K: AsRef<CStr>>(&self, k: K) -> bool {
        let k = k.as_ref();
        unsafe {
            let self_ptr: *mut sys::obs_data = mem::transmute(self as *const _);
            sys::obs_data_get_bool(self_ptr, k.as_ptr())
        }
    }
}

/// Owned variant of `&sys::obs_data`
#[derive(Debug)]
pub struct Data(*mut sys::obs_data);

impl Data {
    /// Creates a new `sys::obs_data` object
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

    /// Takes ownership of a remote pointer, using `sys::obs_data_addref`
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

impl Clone for Data {
    fn clone(&self) -> Self {
        unsafe { Self::from_raw(self.0).unwrap() }
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

    unsafe fn leak(mut self) -> *mut sys::obs_data {
        let ret = self.0;
        self.0 = ptr::null_mut();
        ret
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
