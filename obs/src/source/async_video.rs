use obs_sys as sys;
use std::{
    mem,
    num::NonZeroU32,
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
    const ID: &'static CStr;
    const OUTPUT_FLAGS: Option<NonZeroU32> = None;
    fn create(settings: &mut sys::obs_data, source: *mut sys::obs_source_t) -> Self;
    fn get_name() -> &'static CStr;
    fn update(&self, _settings: &sys::obs_data) {
    }
    fn get_properties(&self) -> Properties {
        Properties::new()
    }

    fn raw_source_info() -> RawSourceInfo<'static> {
        // let mut info = empty_source_info(Self::ID, sys::obs_source_type::OBS_SOURCE_TYPE_INPUT, Some(async_video_source_output_flags::<Self>()));
        // info.0.get_name = Some(async_video_source_get_name::<Self>);
        // info.0.update = Some(async_video_source_update::<Self>);
        // info.0.get_properties = Some(async_video_source_get_properties::<Self>);
        // info.0.create = Some(async_video_source_create::<Self>);
        // info.0.destroy = Some(async_video_source_destroy::<Self>);
        // info
        unsafe {
            let ret = sys::obs_source_info {
                id: Self::ID.as_ptr(),
                type_: sys::obs_source_type::OBS_SOURCE_TYPE_INPUT,
                output_flags: output_flags::<Self>(),
                get_name: Some(async_video_source_get_name::<Self>),
                update: Some(async_video_source_update::<Self>),
                get_properties: Some(async_video_source_get_properties::<Self>),
                create: Some(async_video_source_create::<Self>),
                destroy: Some(async_video_source_destroy::<Self>),
                ..mem::zeroed()
            };
            RawSourceInfo::from_raw(ret)
        }
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
    data.get_properties().leak()
}

const fn output_flags<S: AsyncVideoSource>() -> u32 {
    let custom = match <S as AsyncVideoSource>::OUTPUT_FLAGS {
        Some(f) => f.get(),
        None => 0u32,
    };
    custom | sys::OBS_SOURCE_ASYNC_VIDEO
}
