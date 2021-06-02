pub mod capture;

use std::{
    ffi,
    ops::Deref,
    sync::RwLock,
};
use image::{
    ImageBuffer,
    Rgba,
};
use capture::OpenVRCapture;
use obs::{
    data::ObsData,
    OwnedPointerContainer,
};

pub struct OpenVRMirrorSource {
    _handle: *mut obs::sys::obs_source,
    capture_context: RwLock<OpenVRCapture>,
}

const DEFAULT_EYE: openvr::sys::EVREye = openvr::sys::EVREye::EVREye_Eye_Left;

trait MirrorSourceSettings {
    fn get_eye(&self) -> openvr::sys::EVREye;
}

impl MirrorSourceSettings for obs::sys::obs_data {
    fn get_eye(&self) -> openvr::sys::EVREye {
        use openvr::sys::EVREye::*;
        use ffi::CStr;
        let eye_key: &'static CStr = unsafe {
            CStr::from_bytes_with_nul_unchecked(b"eye\0")
        };
        self.get_string(eye_key)
            .and_then(|s| match s {
                "left" => Some(EVREye_Eye_Left),
                "right" => Some(EVREye_Eye_Right),
                _ => None,
            })
            .unwrap_or(DEFAULT_EYE)
    }
}

impl OpenVRMirrorSource {
    pub fn new(settings: &mut obs::sys::obs_data, handle: *mut obs::sys::obs_source) -> Self {
        let eye = settings.get_eye();
        let capture_context = OpenVRCapture::new(eye, None)
            .expect("failed to create capture context");
        trace!("OpenVRMirrorSource::create()");
        OpenVRMirrorSource {
            _handle: handle,
            capture_context: RwLock::new(capture_context),
        }
    }

    fn capture_context<'a>(&'a self) -> impl Deref<Target=OpenVRCapture> + 'a {
        self.capture_context.read().unwrap()
    }

    #[inline(always)]
    pub fn eye(&self) -> openvr::sys::EVREye {
        self.capture_context().eye()
    }

    pub fn set_eye(&self, eye: openvr::sys::EVREye) {
        if eye == self.eye() {
            return;
        }
        trace!("OpenVRMirrorSource: changing eye value {:?} -> {:?}", self.eye(), eye);
        let mut capture_context = self.capture_context.write().unwrap();
        *capture_context = OpenVRCapture::new(eye, None)
            .expect("failed to create capture context");
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.capture_context().dimensions()
    }
}

impl Drop for OpenVRMirrorSource {
    fn drop(&mut self) {
        trace!("OpenVRMirrorSource::drop()");
    }
}

impl obs::source::VideoSource for OpenVRMirrorSource {
    const ID: &'static [u8] = b"obs-openvr-mirror\0";
    const OUTPUT_FLAGS: Option<u32> = None;

    fn create(settings: &mut obs::sys::obs_data, source: *mut obs::sys::obs_source_t) -> Self {
        OpenVRMirrorSource::new(settings, source)
    }

    fn get_name() -> &'static ffi::CStr {
        unsafe { ffi::CStr::from_bytes_with_nul_unchecked(b"OpenVR Mirror Source\0") }
    }

    fn get_dimensions(&self) -> (u32, u32) {
        self.dimensions()
    }

    fn get_properties(&self) -> *mut obs::sys::obs_properties {
        use obs::properties::{
            Properties,
            PropertiesExt,
            PropertyDescription,
        };
        use ffi::CStr;

        let mut props = Properties::new();

        let eye_name: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"eye\0") };
        let left_eye: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"left\0") };
        let right_eye: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"right\0") };
        props.add_string_list_complete(PropertyDescription::new(eye_name, None), [(left_eye, left_eye), (right_eye, right_eye)].iter().map(|&v| v));
        unsafe { props.leak() }
    }

    fn update(&self, data: &obs::sys::obs_data) {
        trace!("OpenVRMirrorSource::update()");
        self.set_eye(data.get_eye());
    }

    fn video_render(&self, _effect: *mut obs::sys::gs_effect_t) {
        let capture_context = self.capture_context.read().unwrap();
        #[cfg(feature = "log-render-time")]
        use timing::{
            Timer,
            TimerExt,
        };
        #[cfg(feature = "log-render-time")]
        let mut timer = Timer::new();
        let (width, height) = capture_context.dimensions();
        let copy_context = capture_context.copy_context();
        let format = capture_context.obs_format().unwrap();
        let buffer = if let Some(buf) = copy_context.image_buffer() {
            buf
        } else {
            error!("image buffer doesn't exist");
            return;
        };
        let dimensions = capture_context.dimensions();
        let buffer: ImageBuffer<Rgba<u8>, _> = match ImageBuffer::from_raw(dimensions.0, dimensions.1, buffer) {
            Some(v) => v,
            None =>  {
                error!("image buffer wasn't big enough to fit {}x{} image", dimensions.0, dimensions.1);
                return;
            },
        };
        let mut buffer = buffer.as_raw().as_ptr();
        let mut texture = unsafe {
            if let Some(texture) = obs::graphics::Texture::new(width, height, format, 1, &mut buffer, 0) {
                texture
            } else {
                error!("gs_texture_create failed");
                return;
            }
        };
        obs::source::draw(&mut texture, 0, 0, 0, 0, false);
        #[cfg(feature = "log-render-time")]
        timer.log_checkpoint_ms("video_render");
    }
}
