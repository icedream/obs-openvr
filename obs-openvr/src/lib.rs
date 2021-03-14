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
mod capture;

pub use openvr::sys as openvr_sys;

use std::{
    io,
    ptr,
    ffi,
    fmt::Display,
    mem,
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

pub(crate) const CRATE_NAME: &'static str = env!("CARGO_CRATE_NAME");

macro_rules! debug_print {
    ($args:tt) => {
        print!("{}: ", crate::CRATE_NAME);
        println!($args);
    }
}

static FILL_INFO: Once = Once::new();
static mut SOURCE_INFO: Option<obs_sys::obs_source_info> = None;

struct OpenVRMirrorSource {
    handle: *mut obs::sys::obs_source,
    width: Option<u32>,
    height: Option<u32>,
    eye: openvr::sys::EVREye,
    capture_context: OpenVRCapture,
}

type ObsOpenVRImageBuffer = ImageBuffer<Rgba<u8>, Vec<u8>>;

const DEFAULT_EYE: openvr::sys::EVREye = openvr::sys::EVREye::EVREye_Eye_Left;

impl OpenVRMirrorSource {
    pub fn new(handle: *mut obs::sys::obs_source) -> Self {
        let eye = DEFAULT_EYE;
        let capture_context = OpenVRCapture::new(eye, HEADSET_DIMENSIONS.0, HEADSET_DIMENSIONS.1, None)
            .expect("failed to create capture context");
        trace!("OpenVRMirrorSource::create()");
        OpenVRMirrorSource {
            handle: handle,
            width: None,
            height: None,
            eye: eye,
            capture_context: capture_context,
        }
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
        (self.width.unwrap_or(0), self.height.unwrap_or(0))
    }

    const DEFAULT_DIMENSIONS: (u32, u32) = (1920, 1080);
}

impl Drop for OpenVRMirrorSource {
    fn drop(&mut self) {
        trace!("OpenVRMirrorSource::drop()");
    }
}

const MAX_SIZE: libc::c_int = 100000;
const HEADSET_DIMENSIONS: (u32, u32) = (2468, 2740);

impl obs::source::VideoSource for OpenVRMirrorSource {
    const ID: &'static [u8] = b"obs-openvr-mirror\0";
    const OUTPUT_FLAGS: Option<u32> = None;

    fn create(settings: *mut obs::sys::obs_data, source: *mut obs::sys::obs_source_t) -> Self {
        OpenVRMirrorSource::new(source)
    }

    fn get_name() -> &'static ffi::CStr {
        unsafe { ffi::CStr::from_bytes_with_nul_unchecked(b"OpenVR Mirror Source\0") }
    }

    fn get_width(&self) -> u32 {
        // self.width.unwrap_or(Self::DEFAULT_DIMENSIONS.0)
        HEADSET_DIMENSIONS.0
    }

    fn get_height(&self) -> u32 {
        // self.height.unwrap_or(Self::DEFAULT_DIMENSIONS.1)
        HEADSET_DIMENSIONS.1
    }

    fn get_properties(&self) -> *mut obs::sys::obs_properties {
        use obs::properties::PropertyDescription;
        unsafe {
            use obs::sys as sys;
            use ffi::CStr;
            let mut props = obs::properties::Properties::new();
            let width_name: &'static CStr = CStr::from_bytes_with_nul_unchecked(b"width\0");
            let height_name: &'static CStr = CStr::from_bytes_with_nul_unchecked(b"height\0");
            let eye_name: &'static CStr = CStr::from_bytes_with_nul_unchecked(b"eye\0");
            let left_eye: &'static CStr = CStr::from_bytes_with_nul_unchecked(b"left\0");
            let right_eye: &'static CStr = CStr::from_bytes_with_nul_unchecked(b"right\0");
            props.add_int(width_name, width_name, 0, MAX_SIZE, 1);
            props.add_int(height_name, height_name, 0, MAX_SIZE, 1);
            {
                let mut eye_list = props.add_string_list(PropertyDescription::new(eye_name, None), false);
                eye_list.add_string(left_eye, left_eye);
                eye_list.add_string(right_eye, right_eye);
            }
            props.leak()
        }
    }

    fn video_render(&self, effect: *mut obs::sys::gs_effect_t) {
        #[cfg(feature = "log-render-time")]
        use timing::{
            Timer,
            TimerExt,
        };
        #[cfg(feature = "log-render-time")]
        let mut timer = Timer::new();
        let (width, height) = self.capture_context.dimensions();
        let copy_context = self.capture_context.copy_context();
        let format = self.capture_context.obs_format().unwrap();
        let buffer = if let Some(buf) = copy_context.image_buffer() {
            buf
        } else {
            error!("image buffer doesn't exist");
            return;
        };
        let buffer: ImageBuffer<Rgba<u8>, _> = match ImageBuffer::from_raw(HEADSET_DIMENSIONS.0, HEADSET_DIMENSIONS.1, buffer) {
            Some(v) => v,
            None =>  {
                error!("image buffer wasn't big enough to fit {}x{} image", HEADSET_DIMENSIONS.0, HEADSET_DIMENSIONS.1);
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

pub(crate) fn init_glfw() -> Result<glfw::Glfw, io::Error> {
    use io::ErrorKind;
    use glfw::WindowHint;

    trace!("initializing glfw");
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)
        .map_err(|e| io::Error::new(ErrorKind::Other, format!("Error initializing GLFW: {}", &e)))?;
    trace!("glfw initialized");
    #[cfg(not(feature = "show-context-window"))]
    glfw.window_hint(WindowHint::Visible(false));
    glfw.window_hint(WindowHint::ContextVersion(3, 2));
    Ok(glfw)
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
