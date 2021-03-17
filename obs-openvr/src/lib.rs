extern crate obs;
extern crate openvr;
extern crate glfw;
extern crate image;
#[macro_use] extern crate log;
extern crate env_logger;

pub use obs::sys as obs_sys;

mod logging;
pub mod module;
pub(crate) mod native_utils;
mod timing;
pub mod capture;
pub mod overlay;

pub use openvr::sys as openvr_sys;

use std::{
    io,
    ffi,
    fmt::Display,
    ops::Deref,
    sync::{
        RwLock,
    },
};
use image::{
    Rgba,
    ImageBuffer,
};
use capture::OpenVRCapture;
use obs::data::ObsData;

struct OpenVRMirrorSource {
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

fn obs_module_load_result() -> Result<(), impl Display + 'static> {
    use std::borrow::Cow;

    // Initialize logging
    logging::init();
    info!("logging initialized");

    // Initialize OpenVR
    let vr_initialized = openvr::init(openvr_sys::EVRApplicationType::EVRApplicationType_VRApplication_Background)
        .map(|result| result.value())
        .map_err(|e| Cow::Owned(format!("OpenVR failed to initialize: {:?}", &e)))?;
    if !vr_initialized {
        return Err(Cow::Borrowed("OpenVR failed to initialize, but with no error"));
    }

    // Create source info struct, and register it
    obs::register_video_source!(OpenVRMirrorSource);
    obs::register_video_source!(overlay::OpenVROverlaySource);

    trace!("loaded");
    Ok(())
}

#[no_mangle]
pub extern "C" fn obs_module_load() -> bool {
    match obs_module_load_result() {
        Ok(_) => {
            // use ffi::CStr;
            // use image::{ ImageBuffer, Bgra, buffer::ConvertBuffer };
            // let overlay_key = unsafe { CStr::from_bytes_with_nul_unchecked(b"StereoLayer:2\0") };
            // let overlay_image = openvr::overlay::OverlayImageData::find_overlay(overlay_key);
            // match overlay_image {
            //     Ok(data) => {
            //         info!("got overlay image with data length: {}", data.data().len());
            //         let image_buffer: ImageBuffer<Bgra<u8>, _> = ImageBuffer::from_raw(1920, 1080, data.data()).unwrap();
            //         let image_buffer: ImageBuffer<Rgba<u8>, _> = image_buffer.convert();
            //         if let Err(e) = image_buffer.save("/home/mcoffin/obs-openvr-overlay.png") {
            //             error!("Error saving overly image: {:?}", &e);
            //         }
            //     },
            //     Err(e) => error!("error getting overlay image: {:?}", &e),
            // }
            // info!("overlay \"{}\": {:?}", overlay_key.to_str().unwrap(), openvr::overlay::find_overlay(overlay_key));
            true
        },
        Err(e) => {
            use io::Write;
            let stderr = io::stderr();
            let mut stderr = stderr.lock();
            let _ = write!(&mut stderr, "error loading {}: {}", env!("CARGO_CRATE_NAME"), &e);
            false
        },
    }
}

#[no_mangle]
pub extern "C" fn obs_module_unload() {
    if !openvr::shutdown() {
        warn!("OpenVR was not actually shut down on obs_module_unload");
    }
    trace!("unloaded");
}
