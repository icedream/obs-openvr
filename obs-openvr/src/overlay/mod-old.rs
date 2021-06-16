use std::{
    ffi::{
        CStr,
        CString,
    },
    mem,
    ops::Deref,
    time::{
        Duration,
        Instant,
    },
    sync::{
        Arc,
        RwLock,
        Mutex,
        atomic::{
            AtomicBool,
            Ordering,
        },
    },
    thread,
};
use obs::{
    data::ObsData,
    OwnedPointerContainer,
};
use openvr::overlay::OverlayImageData;
use crate::thread_utils::JoinOnDrop;

const MAX_ERROR_INTERVAL: Duration = Duration::from_millis(5000);

struct ImageDataBuffers {
    front: RwLock<Option<OverlayImageData>>,
    back: RwLock<Option<OverlayImageData>>,
    last_logged: Mutex<Option<Instant>>,
}

impl ImageDataBuffers {
    pub fn new() -> Self {
        ImageDataBuffers {
            front: RwLock::new(None),
            back: RwLock::new(None),
            last_logged: Mutex::new(None),
        }
    }

    pub fn front<'a>(&'a self) -> impl Deref<Target=Option<OverlayImageData>> + 'a {
        self.front.read().unwrap()
    }

    fn do_error_log(&self, prefix: &str, e: openvr::sys::EVROverlayError) {
        let mut last_logged = self.last_logged.lock().unwrap();
        let now = Instant::now();
        let should_log = last_logged
            .map(|t| now.duration_since(t) >= MAX_ERROR_INTERVAL)
            .unwrap_or(true);
        if e != openvr::sys::EVROverlayError::EVROverlayError_VROverlayError_UnknownOverlay || should_log {
            *last_logged = Some(now);
            error!("{}: {:?}", prefix, &e);
        } else {
            trace!("skipping logging due to rate limiting");
        }
    }

    pub fn render<K: AsRef<CStr>>(&self, key: K) {
        // Render to back buffer
        let mut image_data = self.back.write().unwrap();
        if let Some(image_data) = image_data.as_mut() {
            let handle = openvr::overlay::find_overlay(key).ok();
            if let Some(handle) = handle {
                if let Err(e) = image_data.refill(handle) {
                    self.do_error_log("error refilling overlay image", e);
                    // error!("error refilling overlay image: {:?}", &e);
                }
            }
        } else {
            *image_data = match openvr::overlay::OverlayImageData::find_overlay(key) {
                Ok(v) => Some(v),
                Err(e) => {
                    self.do_error_log("error refilling overlay image", e);
                    // error!("error getting overlay image: {:?}", &e);
                    return;
                },
            };
        }
        // Swap front/back buffers
        { 
            let mut front = self.front.write().unwrap();
            mem::swap::<Option<OverlayImageData>>(&mut *front, &mut *image_data);
        }
    }
}

unsafe impl Send for ImageDataBuffers {}
unsafe impl Sync for ImageDataBuffers {}

pub struct OpenVROverlaySource {
    _source_handle: *mut obs::sys::obs_source_t,
    key: Arc<RwLock<Option<CString>>>,
    dimensions: RwLock<Option<(u32, u32)>>,
    image_data: Arc<ImageDataBuffers>,
    _copy_thread: JoinOnDrop<()>,
    running: Arc<AtomicBool>,
    interval: Arc<Option<Duration>>,
}

impl OpenVROverlaySource {
    pub fn new(source: *mut obs::sys::obs_source_t, interval: Option<Duration>) -> Self {
        let image_data = Arc::new(ImageDataBuffers::new());
        let running = Arc::new(AtomicBool::new(true));
        let key = Arc::new(RwLock::new(None));
        let interval = Arc::new(interval);
        let copy_thread = {
            let image_data = image_data.clone();
            let running = running.clone();
            let key = key.clone();
            let interval = interval.clone();
            thread::spawn(move || {
                while running.load(Ordering::Relaxed) {
                    let start = Instant::now();
                    {
                        let key = key.read().unwrap();
                        key.as_ref()
                            .into_iter()
                            .for_each(|k| image_data.render(k));
                    }
                    if let Some(interval) = interval.as_ref().as_ref().map(|&v| v) {
                        let elapsed = Instant::now().duration_since(start);
                        if elapsed < interval {
                            let t = interval - elapsed;
                            info!("OpenVROverlaySource: sleeping for {}ms", t.as_millis());
                            thread::sleep(t);
                        }
                    }
                }
            })
        };
        OpenVROverlaySource {
            _source_handle: source,
            key: key,
            dimensions: RwLock::new(None),
            image_data: image_data,
            _copy_thread: JoinOnDrop::from(copy_thread),
            running: running,
            interval: interval,
        }
    }

    #[inline]
    pub fn interval(&self) -> Option<Duration> {
        *self.interval
    }
}

#[allow(dead_code)]
const DEFAULT_INTERVAL: Duration = Duration::from_millis(1000 / 60);

impl obs::source::VideoSource for OpenVROverlaySource {
    const ID: &'static [u8] = b"obs-openvr-overlay\0";
    const OUTPUT_FLAGS: Option<u32> = None;

    fn create(settings: &mut obs::sys::obs_data, source: *mut obs::sys::obs_source_t) -> Self {
        #[cfg(feature = "overlay-interval")]
        let ret = OpenVROverlaySource::new(source, Some(DEFAULT_INTERVAL));
        #[cfg(not(feature = "overlay-interval"))]
        let ret = OpenVROverlaySource::new(source, None);
        ret.update(settings);
        ret
    }

    fn get_name() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(b"OpenVR Overlay Source\0") }
    }

    fn get_properties(&self) -> *mut obs::sys::obs_properties {
        use obs::properties::{
            Properties,
        };

        let mut props = Properties::new();
        let id_key: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"id\0") };
        props.add_text(id_key, id_key, obs::sys::obs_text_type::OBS_TEXT_DEFAULT);
        unsafe { props.leak() }
    }

    fn video_render(&self, _effect: *mut obs::sys::gs_effect_t) {
        let image_data = self.image_data.front();
        let overlay_image = if let Some(v) = image_data.as_ref() {
            v
        } else {
            return;
        };
        let image_size = overlay_image.dimensions();
        {
            let mut dimensions = self.dimensions.write().unwrap();
            *dimensions = Some(image_size);
        }
        let (width, height) = image_size;
        let mut buffer = overlay_image.data().as_ptr();
        let mut texture = unsafe {
            if let Some(texture) = obs::graphics::Texture::new(width, height, obs::sys::gs_color_format::GS_BGRA, &mut [buffer], 0) {
                texture
            } else {
                error!("gs_texture_create failed");
                return;
            }
        };
        obs::source::draw(&mut texture, 0, 0, 0, 0, false);
    }

    fn get_dimensions(&self) -> (u32, u32) {
        let dimensions = self.dimensions.read().unwrap();
        let dimensions: &Option<(u32, u32)> = &*dimensions;
        dimensions.unwrap_or((0, 0))
    }

    fn update(&self, data: &obs::sys::obs_data) {
        trace!("OpenVROverlaySource::update()");
        let mut key = self.key.write().unwrap();
        let id_key: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"id\0") };
        *key = data.get_string(id_key).and_then(|s| CString::new(s).ok());
    }
}

impl Drop for OpenVROverlaySource {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}
