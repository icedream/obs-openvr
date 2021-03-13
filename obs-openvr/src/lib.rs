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
use native_utils::CopyContext;
use image::{
    Rgb,
    ImageBuffer,
};

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
    glfw: RwLock<glfw::Glfw>,
    window: Arc<RwLock<glfw::Window>>,
    events: Receiver<(f64, glfw::WindowEvent)>,
    handle: *mut obs::sys::obs_source,
    width: Option<u32>,
    height: Option<u32>,
    eye: openvr::sys::EVREye,
    texture_info: MirrorTextureInfo,
    copy_context: Option<RwLock<CopyContext>>,
    save_sender: Option<Sender<ImageBuffer<Rgb<u8>, Vec<u8>>>>,
    save_thread: Option<thread::JoinHandle<()>>,
}

type ObsOpenVRImageBuffer = ImageBuffer<Rgb<u8>, Vec<u8>>;

impl OpenVRMirrorSource {
    pub fn new(handle: *mut obs::sys::obs_source) -> Self {
        trace!("OpenVRMirrorSource::create()");
        let glfw = init_glfw().expect("Failed to initialize GLFW");
        let (mut window, events) = glfw.create_window(OpenVRMirrorSource::DEFAULT_DIMENSIONS.0, OpenVRMirrorSource::DEFAULT_DIMENSIONS.1, "", glfw::WindowMode::Windowed)
            .map(Ok)
            .unwrap_or_else(|| Err("Failed to create GLFW offscreen window"))
            .unwrap();
        window.set_close_polling(true);
        let (save_sender, save_receiver) = mpsc::channel::<ObsOpenVRImageBuffer>();
        let save_thread = thread::spawn(move || {
            use std::time::{Instant, Duration};
            let mut last_saved: Option<Instant> = None;
            let image_path: &'static Path = "/home/mcoffin/obs-openvr.png".as_ref();
            while let Ok(buffer) = save_receiver.recv() {
                let now = Instant::now();
                let should_skip = last_saved
                    .map(|t| now.duration_since(t) < Duration::from_secs(5))
                    .unwrap_or(false);
                if should_skip {
                    continue;
                }
                last_saved = Some(now);
                trace!("Saving image buffer to: {}", image_path.display());
                if let Err(e) = buffer.save(image_path) {
                    error!("Error saving image buffer: {:?}", &e);
                }
            }
        });
        let mut ret = OpenVRMirrorSource {
            glfw: RwLock::new(glfw),
            handle: handle,
            window: Arc::new(RwLock::new(window)),
            events: events,
            width: None,
            height: None,
            eye: openvr::sys::EVREye::EVREye_Eye_Left,
            texture_info: MirrorTextureInfo::empty(),
            copy_context: None,
            save_sender: Some(save_sender),
            save_thread: Some(save_thread),
        };
        ret.fill_mirror_texture();
        ret
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


    fn fill_mirror_texture(&mut self) {
        obs::graphics::isolate_context(|| {
            self.make_current();
            self.texture_info = unsafe { openvr::compositor::get_mirror_texture_gl(openvr::sys::EVREye::EVREye_Eye_Left).expect("Failed to get mirror texture") };
            trace!("creating copy context");
            self.copy_context = Some(RwLock::new(CopyContext::new(self.texture_info.id).unwrap()));
            trace!("texture_info: {:?}", &self.texture_info);
        });
    }

    fn make_current(&self) {
        use glfw::Context;
        let mut window = self.window.write().unwrap();
        window.make_current();
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width.unwrap_or(0), self.height.unwrap_or(0))
    }

    const DEFAULT_DIMENSIONS: (u32, u32) = (1920, 1080);
}

impl Drop for OpenVRMirrorSource {
    fn drop(&mut self) {
        trace!("OpenVRMirrorSource::drop()");
        mem::drop(self.save_sender.take());
        if let Some(t) = self.save_thread.take() {
            trace!("joining save_thread");
            if let Err(e) = t.join() {
                warn!("save thread failed with error: {:?}", &e);
            }
            trace!("save thread finished");
        } else {
            warn!("save thread didn't exist at drop time");
        }
    }
}

const MAX_SIZE: libc::c_int = 100000;

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
        self.width.unwrap_or(Self::DEFAULT_DIMENSIONS.0)
    }

    fn get_height(&self) -> u32 {
        self.height.unwrap_or(Self::DEFAULT_DIMENSIONS.1)
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

    fn video_tick(&self, _seconds: f32) {
        let window = self.window.read().unwrap();
        if !window.should_close() {
            let mut glfw = self.glfw.write().unwrap();
            glfw.poll_events();
        }
    }

    fn video_render(&self, effect: *mut obs::sys::gs_effect_t) {
        use openvr::headset_view::HeadsetView;
        obs::graphics::isolate_context(|| {
            self.make_current();
            if let Some(mut ctx) = self.copy_context.as_ref().map(|r| r.write().unwrap()) {
                if let Some(headset_view) = HeadsetView::global() {
                    trace!("headset view size: {:?}", headset_view.get_size());
                    trace!("headset view aspect ratio: {}", headset_view.get_aspect_ratio());
                    trace!("headset view mode: {:?}", headset_view.get_mode());
                } else {
                    warn!("failed to get headset view size");
                }
                {
                    let _lock = unsafe { self.texture_info.lock() };
                    ctx.copy_texture(2016, 2240, 0x1907)
                        .expect("Failed to copy texture to CPU memory");
                }
                let buffer = {
                    let buffer = ctx.image_buffer().unwrap();
                    Vec::from(buffer)
                };
                let img_size = ctx.img_size();
                trace!("creating ImageBuffer (lower size: {})", img_size);
                let buffer: ImageBuffer<Rgb<u8>, _> = ImageBuffer::from_raw(2016, 2240, buffer).unwrap();
                if let Err(e) = self.save_sender.as_ref().unwrap().send(buffer) {
                    warn!("save thread seems to be dead: {:?}", &e);
                }
            } else {
                error!("obs_openvr: we had no copy_context wtf");
            }
        });
    }
}

pub(crate) fn init_glfw() -> Result<glfw::Glfw, io::Error> {
    use io::ErrorKind;
    use glfw::WindowHint;

    trace!("initializing glfw");
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)
        .map_err(|e| io::Error::new(ErrorKind::Other, format!("Error initializing GLFW: {}", &e)))?;
    trace!("glfw initialized");
    // glfw.window_hint(WindowHint::Visible(false));
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
