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

pub use openvr::sys as openvr_sys;

use std::{
    io,
    ptr,
    ffi,
    fmt::Display,
    mem,
    ops::Deref,
    sync::{
        atomic::AtomicBool,
        Arc,
        Once,
        RwLock,
        mpsc::{
            self,
            Sender,
            Receiver,
        },
    },
    thread,
    path::Path,
};
use openvr::compositor::MirrorTextureInfo;
use native_utils::{
    CopyContext,
    TextureFormat,
};
use image::{
    Rgba,
    ImageBuffer,
};
use capture::OpenVRCapture;
use obs::data::ObsData;

static FILL_INFO: Once = Once::new();
static mut SOURCE_INFO: Option<obs_sys::obs_source_info> = None;

struct OpenVRMirrorSource {
    handle: *mut obs::sys::obs_source,
    capture_context: RwLock<OpenVRCapture>,
}

type ObsOpenVRImageBuffer = ImageBuffer<Rgba<u8>, Vec<u8>>;

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
            handle: handle,
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

    #[inline(always)]
    pub fn handle(&self) -> *mut obs::sys::obs_source {
        self.handle
    }

    // #[allow(dead_code)]
    // fn with_context<Ret, F>(&self, f: F) -> Ret where
    //     F: FnOnce(&OpenVRMirrorSource) -> Ret,
    // {
    //     obs::graphics::isolate_context(move || {
    //         self.make_current();
    //         f(self)
    //     })
    // }

    // fn with_context_mut<Ret, F>(&mut self, f: F) -> Ret where
    //     F: FnOnce(&mut OpenVRMirrorSource) -> Ret,
    // {
    //     obs::graphics::isolate_context(move || {
    //         self.make_current();
    //         f(self)
    //     })
    // }

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
        unsafe {
            use obs::sys as sys;
            use ffi::CStr;

            let mut props = Properties::new();

            let eye_name: &'static CStr = CStr::from_bytes_with_nul_unchecked(b"eye\0");
            let left_eye: &'static CStr = CStr::from_bytes_with_nul_unchecked(b"left\0");
            let right_eye: &'static CStr = CStr::from_bytes_with_nul_unchecked(b"right\0");
            props.add_string_list_complete(PropertyDescription::new(eye_name, None), [(left_eye, left_eye), (right_eye, right_eye)].iter().map(|&v| v));
            props.leak()
        }
    }

    fn update(&self, data: &obs::sys::obs_data) {
        trace!("OpenVRMirrorSource::update()");
        use openvr::sys::EVREye::*;
        use ffi::CStr;
        let eye_key: &'static CStr = unsafe {
            CStr::from_bytes_with_nul_unchecked(b"eye\0")
        };
        self.set_eye(data.get_eye());
    }

    fn video_render(&self, effect: *mut obs::sys::gs_effect_t) {
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
    use glfw::Context;
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

    // Initialize glfw off-screen

    // Create source info struct, and register it
    FILL_INFO.call_once(|| {
        use obs::source::VideoSource;
        unsafe {
            SOURCE_INFO = Some(<OpenVRMirrorSource as VideoSource>::raw_source_info().unwrap());

        }
    });
    unsafe {
        obs::register_source(SOURCE_INFO.as_ref().unwrap(), None);
    }

    trace!("loaded");
    Ok(())
}

#[no_mangle]
pub extern "C" fn obs_module_load() -> bool {
    match obs_module_load_result() {
        Ok(_) => true,
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
