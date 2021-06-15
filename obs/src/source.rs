use obs_sys as sys;

use std::{
    mem,
    ffi::CStr,
    marker::PhantomData,
};
use crate::ptr::*;

pub struct RawSourceInfo<'a>(pub sys::obs_source_info, PhantomData<&'a ()>);

impl<'a> RawSourceInfo<'a> {
    unsafe fn from_raw(info: sys::obs_source_info) -> RawSourceInfo<'a> {
        RawSourceInfo(info, PhantomData {})
    }
    pub fn info_ref(&'a self) -> &'a sys::obs_source_info {
        &self.0
    }

    #[inline(always)]
    pub fn unwrap(&self) -> sys::obs_source_info {
        self.0
    }
}

impl<'a> Into<sys::obs_source_info> for RawSourceInfo<'a> {
    fn into(self) -> sys::obs_source_info {
        self.unwrap()
    }
}

pub fn empty_source_info<'a>(id: &'a CStr, ty: sys::obs_source_type, output_flags: Option<u32>) -> RawSourceInfo<'a> {
    let mut info = unsafe { mem::zeroed::<sys::obs_source_info>() };
    info.id = id.as_ptr();
    info.type_ = ty;
    info.output_flags = output_flags.unwrap_or(0);
    unsafe { RawSourceInfo::from_raw(info) }
}

#[inline(always)]
fn print_vs_stub(method: &str) {
    trace!("stub: {}::{}()", "VideoSource", method);
}

pub trait VideoSource: Sized {
    const ID: &'static [u8];
    const OUTPUT_FLAGS: Option<u32>;

    fn create(settings: &mut sys::obs_data, source: *mut sys::obs_source_t) -> Self;
    fn get_name() -> &'static CStr;
    fn update(&self, _settings: &sys::obs_data) {
        print_vs_stub("update");
    }
    fn get_properties(&self) -> *mut sys::obs_properties_t {
        print_vs_stub("get_properties");
        unsafe {
            sys::obs_properties_create()
        }
    }
    fn get_dimensions(&self) -> (u32, u32) {
        print_vs_stub("get_dimensions");
        (0, 0)
    }

    fn video_render(&self, _effect: *mut sys::gs_effect_t) {
        print_vs_stub("video_render");
    }
    fn video_tick(&self, _seconds: f32) {
        print_vs_stub("video_tick");
    }

    fn raw_source_info() -> RawSourceInfo<'static> {
        let id: &'static CStr = CStr::from_bytes_with_nul(Self::ID).unwrap();
        let mut info = empty_source_info(id, sys::obs_source_type::OBS_SOURCE_TYPE_INPUT, Some(video_source_output_flags::<Self>()));
        info.0.get_name = Some(video_source_get_name::<Self>);
        info.0.get_width = Some(video_source_get_width::<Self>);
        info.0.get_height = Some(video_source_get_height::<Self>);
        info.0.get_properties = Some(video_source_get_properties::<Self>);
        info.0.update = Some(video_source_update::<Self>);
        info.0.video_render = Some(video_source_video_render::<Self>);
        info.0.video_tick = Some(video_source_video_tick::<Self>);
        info.0.create = Some(video_source_create::<Self>);
        info.0.destroy = Some(video_source_destroy::<Self>);
        info
    }
}

trait VideoSourceExt {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
}

impl<T> VideoSourceExt for T where
    T: VideoSource,
{
    fn get_width(&self) -> u32 {
        self.get_dimensions().0
    }

    fn get_height(&self) -> u32 {
        self.get_dimensions().1
    }
}

fn video_source_output_flags<S: VideoSource>() -> u32 {
    <S as VideoSource>::OUTPUT_FLAGS.unwrap_or(0) | sys::OBS_SOURCE_VIDEO
}

unsafe extern "C" fn video_source_get_name<T: VideoSource + Sized>(_data: *mut libc::c_void) -> *const libc::c_char {
    <T as VideoSource>::get_name().as_ptr()
}

unsafe extern "C" fn video_source_get_width<T: VideoSource>(data: *mut libc::c_void) -> u32 {
    let data: &T = assert_ref(data);
    data.get_width()
}

unsafe extern "C" fn video_source_get_height<T: VideoSource>(data: *mut libc::c_void) -> u32 {
    let data: &T = assert_as_ref(data).unwrap();
    data.get_height()
}

unsafe extern "C" fn video_source_get_properties<T: VideoSource>(data: *mut libc::c_void) -> *mut sys::obs_properties_t {
    let data: &T = assert_ref(data);
    data.get_properties()
}

unsafe extern "C" fn video_source_update<T: VideoSource>(data: *mut libc::c_void, settings: *mut sys::obs_data_t) {
    let data: &T = assert_ref(data);
    let settings = settings.as_mut().unwrap();
    data.update(settings);
}

unsafe extern "C" fn video_source_video_render<T: VideoSource>(data: *mut libc::c_void, effect: *mut sys::gs_effect_t) {
    let data: &T = assert_ref(data);
    data.video_render(effect);
}

unsafe extern "C" fn video_source_video_tick<T: VideoSource>(data: *mut libc::c_void, seconds: f32) {
    let data: &T = assert_ref(data);
    data.video_tick(seconds);
}

unsafe extern "C" fn video_source_create<S: VideoSource>(settings: *mut sys::obs_data_t, source: *mut sys::obs_source_t) -> *mut libc::c_void {
    let source = Box::new(<S as VideoSource>::create(settings.as_mut().unwrap(), source));
    let ret: *mut S = Box::leak(source) as *mut S;
    mem::transmute(ret)
}

unsafe extern "C" fn video_source_destroy<S: VideoSource>(data: *mut libc::c_void) {
    let data: *mut S = mem::transmute(data);
    let ptr = Box::from_raw(data);
    mem::drop(ptr);
}

pub fn draw(image: &sys::gs_texture_t, x: libc::c_int, y: libc::c_int, cx: u32, cy: u32, flip: bool) {
    unsafe {
        let image = image as *const _;
        sys::obs_source_draw(image as *mut _, x, y, cx, cy, flip);
    }
}
