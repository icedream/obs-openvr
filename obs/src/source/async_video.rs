use obs_sys as sys;
use std::{
    mem,
    ffi::CStr,
};
use crate::{
    Properties,
    OwnedPointerContainer,
    source::{
        RawSourceInfo,
        empty_source_info,
    },
    ptr::*,
};

pub trait AsyncVideoSource: Sized {
    const ID: &'static [u8];
    const OUTPUT_FLAGS: Option<u32>;
    fn create(settings: &mut sys::obs_data, source: *mut sys::obs_source_t) -> Self;
    fn get_name() -> &'static CStr;
    fn update(&self, _settings: &sys::obs_data) {
    }
    fn get_properties(&self) -> *mut sys::obs_properties_t {
        let ret = Properties::new();
        unsafe { ret.leak() }
    }

    fn raw_source_info() -> RawSourceInfo<'static> {
        let id: &'static CStr = CStr::from_bytes_with_nul(Self::ID).unwrap();
        let mut info = empty_source_info(id, sys::obs_source_type::OBS_SOURCE_TYPE_INPUT, Some(async_video_source_output_flags::<Self>()));
        info.0.get_name = Some(async_video_source_get_name::<Self>);
        info.0.update = Some(async_video_source_update::<Self>);
        info.0.get_properties = Some(async_video_source_get_properties::<Self>);
        info.0.create = Some(async_video_source_create::<Self>);
        info.0.destroy = Some(async_video_source_destroy::<Self>);
        info
    }
}

unsafe extern "C" fn async_video_source_create<S: AsyncVideoSource>(settings: *mut sys::obs_data_t, source: *mut sys::obs_source_t) -> *mut libc::c_void {
    let source = Box::new(<S as AsyncVideoSource>::create(settings.as_mut().unwrap(), source));
    let ret: *mut S = Box::leak(source) as *mut S;
    mem::transmute(ret)
}

unsafe extern "C" fn async_video_source_destroy<T: AsyncVideoSource>(data: *mut libc::c_void) {
    let data: *mut T = mem::transmute(data);
    let ptr = Box::from_raw(data);
    mem::drop(ptr);
}

unsafe extern "C" fn async_video_source_get_name<T: AsyncVideoSource>(_data: *mut libc::c_void) -> *const libc::c_char {
    <T as AsyncVideoSource>::get_name().as_ptr()
}

unsafe extern "C" fn async_video_source_update<T: AsyncVideoSource>(data: *mut libc::c_void, settings: *mut sys::obs_data_t) {
    let data: &T = assert_ref(data);
    let settings = settings.as_mut().unwrap();
    data.update(settings);
}

unsafe extern "C" fn async_video_source_get_properties<T: AsyncVideoSource>(data: *mut libc::c_void) -> *mut sys::obs_properties_t {
    let data: &T = assert_ref(data);
    data.get_properties()
}

pub fn async_video_source_output_flags<S: AsyncVideoSource>() -> u32 {
    <S as AsyncVideoSource>::OUTPUT_FLAGS.unwrap_or(0) | sys::OBS_SOURCE_ASYNC_VIDEO
}
