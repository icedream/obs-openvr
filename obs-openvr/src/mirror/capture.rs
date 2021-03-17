use std::{
    borrow::Cow,
    io::{
        self,
        ErrorKind,
    },
    ops::Deref,
    sync::{
        mpsc::Receiver,
        Arc,
        atomic::{
            AtomicBool,
            Ordering,
        },
        RwLock,
        Once,
        Mutex,
    },
    time::{
        Duration,
        Instant,
    },
    thread::{
        self,
        JoinHandle,
    },
};
use glfw::Context;
use openvr::compositor::MirrorTextureInfo;
use crate::{
    native_utils::{
        TextureFormat,
        CopyContext,
    },
};

fn init_glfw() -> Result<glfw::Glfw, io::Error> {
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

struct GlfwContext {
    _glfw: glfw::Glfw,
    window: glfw::Window,
    _events: Receiver<(f64, glfw::WindowEvent)>,
}

impl GlfwContext {
    fn new() -> Result<Self, io::Error> {
        let glfw = init_glfw()?;
        let (mut window, events) = glfw.create_window(HIDDEN_WINDOW_DIMENSIONS.0, HIDDEN_WINDOW_DIMENSIONS.1, "", glfw::WindowMode::Windowed)
            .map(Ok)
            .unwrap_or_else(|| Err(io::Error::new(ErrorKind::Other, "Failed to create GLFW offscreen window")))?;
        window.set_close_polling(true);
        Ok(GlfwContext {
            _glfw: glfw,
            window: window,
            _events: events,
        })
    }

    fn make_current(&mut self) {
        self.window.make_current();
    }
}

static INIT_GLFW_CONTEXT: Once = Once::new();
static mut GLFW_CONTEXT: Option<Mutex<GlfwContext>> = None;

fn init_glfw_context() {
    INIT_GLFW_CONTEXT.call_once(|| {
        let ctx = GlfwContext::new()
            .expect("Failed to initialize hidden GLFW window");
        unsafe {
            GLFW_CONTEXT = Some(Mutex::new(ctx));
        }
    });
}

fn with_context<Ret, F>(f: F) -> Ret where
    F: FnOnce() -> Ret,
{
    init_glfw_context();
    let ctx = unsafe {
        GLFW_CONTEXT.as_ref().unwrap()
    };
    let mut ctx = ctx.lock().unwrap();
    obs::graphics::isolate_context(move || {
        ctx.make_current();
        f()
    })
}

fn new_render_context() -> glfw::RenderContext {
    init_glfw_context();
    let ctx = unsafe {
        GLFW_CONTEXT.as_ref().unwrap()
    };
    let mut ctx = ctx.lock().unwrap();
    ctx.window.render_context()
}

struct JoinOnDrop<T>(Option<JoinHandle<T>>);

impl<T> From<JoinHandle<T>> for JoinOnDrop<T> {
    fn from(handle: JoinHandle<T>) -> Self {
        JoinOnDrop(Some(handle))
    }
}

impl<T> Drop for JoinOnDrop<T> {
    fn drop(&mut self) {
        trace!("OpenVRCapture::copy_thread::drop()");
        if let Some(Err(e)) = self.0.take().map(JoinHandle::join) {
            error!("failed to join capture thread: {:?}", &e);
        }
    }
}

pub struct OpenVRCapture {
    dimensions: (u32, u32),
    eye: openvr::sys::EVREye,
    _texture_info: Arc<RwLock<MirrorTextureInfo>>,
    running: Arc<AtomicBool>,
    _copy_thread: Option<JoinOnDrop<()>>,
    copy_context: Arc<RwLock<CopyContext>>,
    format: TextureFormat,
}

const HIDDEN_WINDOW_DIMENSIONS: (u32, u32) = (1920, 1080);
const ERROR_LOG_INTERVAL: Duration = Duration::from_secs(5);

impl OpenVRCapture {
    pub fn new(eye: openvr::sys::EVREye, interval: Option<Duration>) -> Result<Self, io::Error> {
        trace!("OpenVRCapture::new({:?})", &eye);
        let format = TextureFormat::Rgba;

        let texture_info = with_context(|| unsafe {
            openvr::compositor::get_mirror_texture_gl(eye)
        })
            .map_err(|e| io::Error::new(ErrorKind::Other, format!("Failed to get mirror texture: {:?}", &e)))?;
        trace!("got texture info: {:?}", &texture_info);

        let copy_context = Arc::new(RwLock::new(CopyContext::new(texture_info.id).unwrap()));
        let texture_info = Arc::new(RwLock::new(texture_info));

        let texture_size = with_context(|| {
            let copy_context = copy_context.read().unwrap();
            copy_context.get_size()
        });
        info!("OpenVR texture size: {:?}", &texture_size);

        let running = Arc::new(AtomicBool::new(true));
        let copy_thread = {
            let running = running.clone();
            let mut render_context = new_render_context();
            let texture_info = texture_info.clone();
            let copy_context = copy_context.clone();
            thread::spawn(move || {
                let mut last_error_log: Option<Instant> = None;

                let do_loop = || -> Result<(), Cow<'static, str>> {
                    obs::graphics::isolate_context(|| {
                        let mut ctx = copy_context.write().unwrap();
                        let texture_info = texture_info.read().unwrap();
                        let _copy_result = {
                            let _lock = unsafe { texture_info.lock() };
                            ctx.copy_texture(texture_size.width, texture_size.height, format)
                                .map_err(|e| format!("copy_texture({:?}) failed with error: {} (0x{:x})", eye, e, e))
                                .map_err(Cow::Owned)
                        }?;
                        Ok(())
                    })
                };
                while running.load(Ordering::Relaxed) {
                    render_context.make_current();
                    if let Err(e) = do_loop() {
                        if last_error_log.map(|t| Instant::now().duration_since(t) >= ERROR_LOG_INTERVAL).unwrap_or(true) {
                            last_error_log = Some(Instant::now());
                            error!("{}", e);
                        }
                    } else {
                        last_error_log = None;
                    }
                    interval.into_iter().for_each(thread::sleep);
                }
            })
        };

        Ok(OpenVRCapture {
            dimensions: (texture_size.width, texture_size.height),
            eye: eye,
            _texture_info: texture_info,
            running: running,
            _copy_thread: Some(JoinOnDrop::from(copy_thread)),
            copy_context: copy_context,
            format: format,
        })
    }

    #[inline(always)]
    pub fn eye(&self) -> openvr::sys::EVREye {
        self.eye
    }

    pub fn copy_context<'a>(&'a self) -> impl Deref<Target=CopyContext> + 'a {
        self.copy_context.read().unwrap()
    }

    #[inline(always)]
    pub fn format(&self) -> TextureFormat {
        self.format
    }

    pub fn obs_format(&self) -> Option<obs::sys::gs_color_format> {
        self.format.into()
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn width(&self) -> u32 {
        self.dimensions.0
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn height(&self) -> u32 {
        self.dimensions.1
    }

    #[inline(always)]
    pub fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }
}

impl Drop for OpenVRCapture {
    fn drop(&mut self) {
        trace!("OpenVRCapture::drop({:?})", self.eye());
        self.running.store(false, Ordering::Relaxed);
    }
}
