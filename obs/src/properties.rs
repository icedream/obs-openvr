use obs_sys as sys;

use std::{
    ffi::CStr,
    ops::{
        Deref,
        DerefMut,
    },
    ptr,
};
use crate::OwnedPointerContainer;

pub struct StringPropertyList<'a>(&'a mut sys::obs_property);

impl<'a> StringPropertyList<'a> {
    unsafe fn from_ptr(ptr: *mut sys::obs_property) -> Option<Self> {
        ptr.as_mut().map(|r| StringPropertyList(r))
    }

    #[inline(always)]
    fn as_ptr_mut(&mut self) -> *mut sys::obs_property {
        self.0 as _
    }

    pub fn add_int(&mut self, name: &'static CStr, value: libc::c_longlong) -> usize {
        unsafe {
            sys::obs_property_list_add_int(self.as_ptr_mut(), name.as_ptr(), value)
        }
    }

    pub fn add_string(&mut self, name: &'static CStr, value: &'static CStr) -> usize {
        unsafe {
            sys::obs_property_list_add_string(self.as_ptr_mut(), name.as_ptr(), value.as_ptr())
        }
    }
}

impl<'a> Deref for StringPropertyList<'a> {
    type Target = sys::obs_property;

    fn deref(&self) -> &Self::Target {
        self.0 as _
    }
}

pub struct PropertyDescription<'a> {
    pub name: &'a CStr,
    pub description: &'a CStr,
}

impl<'a> PropertyDescription<'a> {
    #[inline(always)]
    pub fn new(name: &'a CStr, description: Option<&'a CStr>) -> Self {
        PropertyDescription {
            name: name,
            description: description.unwrap_or(name),
        }
    }
}

/// Safe interface to `obs_sys::obs_properties`
pub struct Properties(*mut sys::obs_properties);

impl Properties {
    /// Creates a new obs properties object
    pub fn new() -> Properties {
        let ptr = unsafe {
            sys::obs_properties_create()
        };
        unsafe { ptr.as_ref().unwrap(); }
        Properties(ptr)
    }

    pub fn add_bool(&mut self, name: &'static CStr, description: &'static CStr) -> &mut sys::obs_property {
        unsafe {
            sys::obs_properties_add_bool(self.as_ptr_mut(), name.as_ptr(), description.as_ptr()).as_mut().unwrap()
        }
    }

    pub fn add_int(&mut self, name: &'static CStr, description: &'static CStr, min: libc::c_int, max: libc::c_int, step: libc::c_int) -> &mut sys::obs_property {
        unsafe {
            sys::obs_properties_add_int(self.deref_mut() as _, name.as_ptr(), description.as_ptr(), min, max, step).as_mut().unwrap()
        }
    }

    pub fn add_text(&mut self, name: &'static CStr, description: &'static CStr, ty: sys::obs_text_type) -> &mut sys::obs_property {
        unsafe {
            sys::obs_properties_add_text(self.as_ptr_mut(), name.as_ptr(), description.as_ptr(), ty).as_mut().unwrap()
        }
    }

    pub fn add_string_list<'a>(&'a mut self, header: PropertyDescription<'static>, editable: bool) -> StringPropertyList<'a> {
        let combo_type = if editable {
            sys::obs_combo_type_OBS_COMBO_TYPE_EDITABLE
        } else {
            sys::obs_combo_type_OBS_COMBO_TYPE_LIST
        };
        unsafe {
            let ptr = sys::obs_properties_add_list(self.deref_mut() as _, header.name.as_ptr(), header.description.as_ptr(), combo_type, sys::obs_combo_format_OBS_COMBO_FORMAT_STRING);
            StringPropertyList::from_ptr(ptr).unwrap()
        }
    }
}

impl OwnedPointerContainer<sys::obs_properties> for Properties {
    #[inline(always)]
    fn as_ptr(&self) -> *const sys::obs_properties {
        self.0 as _
    }

    #[inline(always)]
    fn as_ptr_mut(&mut self) -> *mut sys::obs_properties {
        self.0
    }

    unsafe fn leak(mut self) -> *mut sys::obs_properties {
        let ret = self.0;
        self.0 = ptr::null_mut();
        ret
    }
}

impl Drop for Properties {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                sys::obs_properties_destroy(self.0);
            }
        }
    }
}

impl Deref for Properties {
    type Target = sys::obs_properties;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref().unwrap() }
    }
}

impl DerefMut for Properties {
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        unsafe { self.0.as_mut().unwrap() }
    }
}

pub trait PropertiesExt {
    fn add_string_list_complete<'a, It>(&'a mut self, header: PropertyDescription<'static>, it: It) where
        It: Iterator<Item=(&'static CStr, &'static CStr)>;
}

impl PropertiesExt for Properties {
    fn add_string_list_complete<'a, It>(&'a mut self, header: PropertyDescription<'static>, it: It) where
        It: Iterator<Item=(&'static CStr, &'static CStr)>,
    {
        let mut prop = self.add_string_list(header, false);
        it.for_each(|(k, v)| {
            prop.add_string(k, v);
        });
    }
}
