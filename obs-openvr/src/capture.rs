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
    },
    time::Duration,
    thread::{
        self,
        JoinHandle,
    },
};
use glfw::Context;
use openvr::compositor::MirrorTextureInfo;
use crate::{
    init_glfw,
    native_utils::{
        TextureFormat,
        CopyContext,
    },
};

struct JoinOnDrop<T>(Option<JoinHandle<T>>);

impl<T> From<JoinHandle<T>> for JoinOnDrop<T> {
    fn from(handle: JoinHandle<T>) -> Self {
        JoinOnDrop(Some(handle))
    }
}

impl<T> Drop for JoinOnDrop<T> {
    fn drop(&mut self) {
        if let Some(Err(e)) = self.0.take().map(JoinHandle::join) {
            error!("failed to join capture thread: {:?}", &e);
        }
    }
}

pub struct OpenVRCapture {
    glfw: glfw::Glfw,
    window: glfw::Window,
    events: Receiver<(f64, glfw::WindowEvent)>,
    dimensions: (u32, u32),
    eye: openvr::sys::EVREye,
    texture_info: Arc<RwLock<MirrorTextureInfo>>,
    interval: Option<Duration>,
    running: Arc<AtomicBool>,
    copy_thread: JoinOnDrop<()>,
    copy_context: Arc<RwLock<CopyContext>>,
    format: TextureFormat,
}

impl OpenVRCapture {
    pub fn new(eye: openvr::sys::EVREye, width: u32, height: u32, interval: Option<Duration>) -> Result<Self, io::Error> {
        let format = TextureFormat::Rgba;
        let glfw = init_glfw()
            .map_err(|e| io::Error::new(ErrorKind::Other, format!("Failed to initialize GLFW: {}", &e)))?;
        let (mut window, events) = glfw.create_window(width, height, "", glfw::WindowMode::Windowed)
            .map(Ok)
            .unwrap_or_else(|| Err(io::Error::new(ErrorKind::Other, "Failed to create GLFW offscreen window")))?;
        window.set_close_polling(true);

        let texture_info = obs::graphics::isolate_context(|| {
            window.make_current();
            unsafe {
                openvr::compositor::get_mirror_texture_gl(openvr::sys::EVREye::EVREye_Eye_Left)
            }
        })
            .map_err(|e| io::Error::new(ErrorKind::Other, format!("Failed to get mirror texture: {:?}", &e)))?;

        let copy_context = Arc::new(RwLock::new(CopyContext::new(texture_info.id).unwrap()));
        let texture_info = Arc::new(RwLock::new(texture_info));

        let running = Arc::new(AtomicBool::new(true));
        let copy_thread = {
            let running = running.clone();
            let mut render_context = window.render_context();
            let texture_info = texture_info.clone();
            let copy_context = copy_context.clone();
            thread::spawn(move || {
                let obs_format: Option<obs::sys::gs_color_format> = format.into();
                let obs_format = obs_format.unwrap();

                let do_loop = || {
                    let result = obs::graphics::isolate_context(|| {
                        let mut ctx = copy_context.write().unwrap();
                        let texture_info = texture_info.read().unwrap();
                        let _copy_result = {
                            let _lock = unsafe { texture_info.lock() };
                            ctx.copy_texture(width, height, format)
                                .map_err(|e| format!("copy_texture failed with error: {}", e))
                                .map_err(Cow::Owned)
                        }?;
                        Ok(ctx)
                    });
                    result.and_then(|ctx| -> Result<(), Cow<'static, str>> {
                        let buffer = ctx.image_buffer()
                            .map(Ok)
                            .unwrap_or(Err(Cow::Borrowed("image buffer doesn't exist anymore wtf")))?;
                        let mut buffer = buffer.as_ptr();
                        let mut texture = unsafe {
                            obs::graphics::Texture::new(width, height, obs_format, 1, &mut buffer, 0)
                                .map(Ok)
                                .unwrap_or(Err(Cow::Borrowed("gs_create_texture failed")))
                        }?;
                        obs::source::draw(&mut texture, 0, 0, 0, 0, false);
                        Ok(())
                    })
                };
                while running.load(Ordering::Relaxed) {
                    render_context.make_current();
                    if let Err(e) = do_loop() {
                        error!("{}", e);
                    }
                    interval.into_iter().for_each(thread::sleep);
                }
            })
        };

        Ok(OpenVRCapture {
            glfw: glfw,
            window: window,
            events: events,
            dimensions: (width, height),
            eye: eye,
            texture_info: texture_info,
            interval: interval,
            running: running,
            copy_thread: JoinOnDrop::from(copy_thread),
            copy_context: copy_context,
            format: format,
        })
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

    #[inline(always)]
    pub fn width(&self) -> u32 {
        self.dimensions.0
    }

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
        self.running.store(false, Ordering::Relaxed);
    }
}
