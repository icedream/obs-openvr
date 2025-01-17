use std::{
    borrow::Borrow,
    cell::UnsafeCell,
    ffi::{
        CStr,
        CString,
    },
    mem,
    num::NonZeroU64,
    ptr,
    sync::{
        Arc,
        atomic::{
            AtomicBool,
            Ordering,
        },
    },
    thread,
    time::{
        Duration,
        Instant,
    },
};
use obs::{
    OwnedPointerContainer,
    data::ObsData,
    source::AsyncVideoSource,
};
use openvr::{
    overlay::{
        OverlayImage,
        OverlayRef,
    },
};
use crate::{
    overlay::keys,
    thread_utils::JoinOnDrop,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceHandle(*mut obs::sys::obs_source_t);

impl SourceHandle {
    #[inline(always)]
    fn is_visible(&self) -> bool {
        unsafe {
            obs::sys::obs_source_showing(self.0)
        }
    }

    #[inline(always)]
    fn unwrap(self) -> *mut obs::sys::obs_source_t {
        self.0
    }

    #[inline(always)]
    fn handle(&self) -> *mut obs::sys::obs_source_t {
        self.0
    }

    #[inline]
    unsafe fn output_video(&self, frame: &obs::sys::obs_source_frame2) {
        obs::source::output_video2(self.handle(), frame);
    }
}

impl From<*mut obs::sys::obs_source_t> for SourceHandle {
    #[inline]
    fn from(p: *mut obs::sys::obs_source_t) -> Self {
        SourceHandle(p)
    }
}

unsafe impl Send for SourceHandle {}

pub struct OpenVRAsyncOverlaySource {
    handle: *mut obs::sys::obs_source_t,
    running: Arc<AtomicBool>,
    thread: UnsafeCell<Option<JoinOnDrop<()>>>,
}

fn spawn_overlay_thread(source: *mut obs::sys::obs_source_t, running: Arc<AtomicBool>, overlay: OverlayRef, sleep_time: Option<NonZeroU64>) -> thread::JoinHandle<()> {
    const BACKOFF_VISIBILITY: Duration = Duration::from_millis(500);

    let source = SourceHandle(source);
    running.store(true, Ordering::Relaxed);
    thread::spawn(move || {
        let start_time = Instant::now();
        let mut image = OverlayImage::new();
        while running.load(Ordering::Relaxed) {
            if !source.is_visible() || !overlay.is_visible() {
                thread::sleep(BACKOFF_VISIBILITY);
                continue;
            }
            let frame_time = Instant::now();
            if let Err(e) = image.fill(overlay.handle()) {
                error!("Error filling overlay image: {:?}", &e);
                return;
            }
            let (w, h) = image.dimensions();
            let mut frame_data: [*mut u8; 8] = [ptr::null_mut(); 8];
            frame_data[0] = {
                image.data().as_ptr() as *mut _
            };
            let linesize: [u32; 8] = [w * 4, 0, 0, 0, 0, 0, 0, 0];
            let ts = frame_time.duration_since(start_time).as_millis() as u64;
            unsafe {
                source.output_video(&obs::sys::obs_source_frame2 {
                    data: frame_data,
                    linesize: linesize,
                    width: w,
                    height: h,
                    timestamp: ts,
                    format: obs::sys::video_format_VIDEO_FORMAT_BGRA,
                    range: obs::sys::video_range_type_VIDEO_RANGE_DEFAULT,
                    color_matrix: [0.0; 16],
                    color_range_min: [0.0; 3],
                    color_range_max: [0.0; 3],
                    flip: false,
                    flags: 0,
                    trc: 0
                });
            }
            if let Some(t) = sleep_time {
                thread::sleep(Duration::from_millis(t.get()));
            }
        }
    })
}

fn try_init_openvr() -> bool {
    use crate::init_openvr;
    match init_openvr() {
        Ok(..) => true,
        Err(e) => {
            warn!("failed to initialize openvr: {}", &e);
            false
        },
    }
}

impl OpenVRAsyncOverlaySource {
    const NAME: &'static CStr = unsafe {
        CStr::from_bytes_with_nul_unchecked(b"OpenVR Overlay Source (async)\0")
    };
}

impl AsyncVideoSource for OpenVRAsyncOverlaySource {
    const ID: &'static CStr = unsafe {
        CStr::from_bytes_with_nul_unchecked(b"obs-openvr-overlay-async\0")
    };

    fn create(settings: &mut obs::sys::obs_data, source: *mut obs::sys::obs_source_t) -> Self {
        let ret = OpenVRAsyncOverlaySource {
            handle: source,
            running: Arc::new(AtomicBool::new(false)),
            thread: UnsafeCell::from(None),
        };
        ret.update(settings);
        ret
    }

    fn get_name() -> &'static CStr {
        Self::NAME
    }

    fn update(&self, data: &obs::sys::obs_data) {
        if !try_init_openvr() {
            self.running.store(false, Ordering::Relaxed);
            return;
        }
        let thread_handle: &mut Option<JoinOnDrop<()>> = {
            let p = self.thread.get();
            unsafe { p.as_mut().unwrap() }
        };
        if let Some(id) = data.get_cstr(keys::ID) {
            trace!("Updating overlay source with id: {:?}", &id);
            let overlay = match openvr::overlay::find_overlay(&id) {
                Ok(v) => {
                    trace!("Got overlay handle: {}", v);
                    Some(v)
                },
                Err(e) => {
                    warn!("Error finding overlay with id {:?}: {:?}", &id, &e);
                    None
                },
            };
            self.running.store(false, Ordering::Relaxed);
            mem::drop(thread_handle.take());
            *thread_handle = overlay
                .map(|overlay| spawn_overlay_thread(self.handle, self.running.clone(), overlay, None))
                .map(JoinOnDrop::from);
        }
    }

    fn get_properties(&self) -> obs::Properties {
        use obs::properties::{
            Properties,
            PropertiesExt,
        };
        let mut props = Properties::new();
        props.add_text(keys::ID, keys::ID, obs::sys::obs_text_type_OBS_TEXT_DEFAULT);
        props
    }
}

impl Drop for OpenVRAsyncOverlaySource {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}
