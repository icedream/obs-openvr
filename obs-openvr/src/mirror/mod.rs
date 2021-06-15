pub mod utils;
mod capture;

use capture::OpenVRMirrorCapture;
use std::{
    convert::TryFrom,
    ffi::{
        self,
        CStr,
    },
    io,
    sync::{
        mpsc::Receiver,
        RwLock,
    },
};
use obs::{
    graphics::with_graphics,
    data::ObsData,
    OwnedPointerContainer,
};

const DEFAULT_EYE: openvr::sys::EVREye = openvr::sys::EVREye::EVREye_Eye_Left;

const OBS_TEXTURE_FLAGS: u32 = obs::sys::GS_DYNAMIC;

#[derive(Debug)]
struct OpenVRMirrorSourceSettings {
    eye: openvr::sys::EVREye,
}

impl OpenVRMirrorSourceSettings {
    fn update<D: obs::data::ObsData>(&mut self, data: &D) {
        self.eye = data.get_eye();
        trace!("OpenVRMirrorSourceSettings::update: {:?}", self);
    }

    #[inline(always)]
    pub fn eye(&self) -> openvr::sys::EVREye {
        self.eye
    }
}

impl<'a, T: obs::data::ObsData> From<&'a T> for OpenVRMirrorSourceSettings {
    fn from(data: &'a T) -> Self {
        OpenVRMirrorSourceSettings {
            eye: data.get_eye(),
        }
    }
}

impl<'a> TryFrom<&'a OpenVRMirrorSourceSettings> for OpenVRMirrorCapture {
    type Error = openvr::sys::EVRCompositorError;

    #[inline]
    fn try_from(settings: &'a OpenVRMirrorSourceSettings) -> Result<Self, Self::Error> {
        OpenVRMirrorCapture::new(settings.eye())
    }
}

pub struct OpenVRMirrorSource {
    handle: *mut obs::sys::obs_source,
    settings: RwLock<OpenVRMirrorSourceSettings>,
    capture_context: RwLock<Option<OpenVRMirrorCapture>>,
}

impl OpenVRMirrorSource {
    pub fn new(settings: &mut obs::sys::obs_data, handle: *mut obs::sys::obs_source) -> Self {
        let ret = OpenVRMirrorSource {
            handle: handle,
            settings: RwLock::new(OpenVRMirrorSourceSettings::from(settings as &_)),
            capture_context: RwLock::new(None),
        };
        {
            let settings = ret.settings.read().unwrap();
            trace!("Creating capture context with settings: {:?}", &*settings);
            ret.recreate_capture_context(&*settings);
        }
        ret
    }

    fn recreate_capture_context(&self, settings: &OpenVRMirrorSourceSettings) {
        let new_context = match OpenVRMirrorCapture::try_from(settings) {
            Ok(v) => Some(v),
            Err(e) => {
                error!("Error creating mirror capture: {:?}", &e);
                None
            },
        };
        let mut capture_context = self.capture_context.write().unwrap();
        *capture_context = new_context;
    }

    #[inline(always)]
    pub fn is_showing(&self) -> bool {
        unsafe {
            obs::sys::obs_source_showing(self.handle)
        }
    }
}

trait MirrorSourceSettings {
    fn get_eye(&self) -> openvr::sys::EVREye;
}

impl<T> MirrorSourceSettings for T where
    T: obs::data::ObsData,
{
    fn get_eye(&self) -> openvr::sys::EVREye {
        use openvr::sys::EVREye::*;
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

impl obs::source::VideoSource for OpenVRMirrorSource {
    const ID: &'static [u8] = b"obs-openvr-mirror\0";
    const OUTPUT_FLAGS: Option<u32> = None;

    fn create(settings: &mut obs::sys::obs_data, source: *mut obs::sys::obs_source_t) -> Self {
        OpenVRMirrorSource::new(settings, source)
    }

    fn get_name() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(b"OpenVR Mirror Source\0") }
    }

    fn get_dimensions(&self) -> (u32, u32) {
        let capture_context = self.capture_context.read().unwrap();
        capture_context.as_ref()
            .map(|ctx| {
                let (w, h) = ctx.dimensions();
                (w as u32, h as u32)
            })
            .unwrap_or((0, 0))
    }

    fn get_properties(&self) -> *mut obs::sys::obs_properties {
        use obs::properties::{
            Properties,
            PropertiesExt,
            PropertyDescription,
        };

        let mut props = Properties::new();

        let eye_name: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"eye\0") };
        let left_eye: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"left\0") };
        let right_eye: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"right\0") };
        props.add_string_list_complete(PropertyDescription::new(eye_name, None), [(left_eye, left_eye), (right_eye, right_eye)].iter().map(|&v| v));

        unsafe { props.leak() }
    }

    fn update(&self, data: &obs::sys::obs_data) {
        let mut settings = self.settings.write().unwrap();
        settings.update(data);
        self.recreate_capture_context(&*settings);
    }

    fn video_tick(&self, seconds: f32) {
        if !self.is_showing() {
            return;
        }
        let mut capture_context = self.capture_context.write().unwrap();
        if let Some(capture_context) = capture_context.as_mut() {
            if let Err(e) = with_graphics(|| unsafe { capture_context.copy_texture() }) {
                error!("Error copying texture data from OpenVR mirror: {:?}", &e);
                return;
            }
        }
    }

    fn video_render(&self, _effect: *mut obs::sys::gs_effect_t) {
        let capture_context = self.capture_context.read().unwrap();
        if let Some(capture_context) = capture_context.as_ref() {
            let mut texture = match with_graphics(|| unsafe { capture_context.create_texture(OBS_TEXTURE_FLAGS) }) {
                Ok(v) => v,
                Err(e) => {
                    error!("Error creating OBS texture: {}", &e);
                    return;
                },
            };
            obs::source::draw(&mut texture, 0, 0, 0, 0, false);
        }
    }
}
