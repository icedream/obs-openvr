mod async_source;

use std::{
    cell::Cell,
    ffi::{
        CStr,
        CString,
    },
    sync::{
        RwLock,
    },
};
use openvr::{
    overlay::{
        OverlayImage,
        OverlayRef,
    },
};
use obs::{
    OwnedPointerContainer,
    data::ObsData,
    graphics::{
        GsTexture,
        with_graphics,
    },
};

pub use async_source::OpenVRAsyncOverlaySource;

pub struct OpenVROverlaySource {
    handle: *mut obs::sys::obs_source_t,
    image: RwLock<OverlayImage>,
    texture: RwLock<Option<obs::graphics::Texture>>,
    overlay_handle: Cell<Option<OverlayRef>>,
    dimensions: Cell<(u32, u32)>,
}

impl OpenVROverlaySource {
    #[inline(always)]
    pub fn is_showing(&self) -> bool {
        unsafe {
            obs::sys::obs_source_showing(self.handle)
        }
    }

    fn linesize(&self) -> u32 {
        use obs::source::VideoSource;
        let (w, _h) = self.get_dimensions();
        w * 4
    }
}

impl obs::source::VideoSource for OpenVROverlaySource {
    const ID: &'static [u8] = b"obs-openvr-overlay\0";
    const OUTPUT_FLAGS: Option<u32> = None;

    fn create(settings: &mut obs::sys::obs_data, source: *mut obs::sys::obs_source_t) -> Self {
        let ret = OpenVROverlaySource {
            handle: source,
            image: RwLock::new(OverlayImage::new()),
            texture: RwLock::new(None),
            overlay_handle: Cell::new(None),
            dimensions: Cell::new((0, 0)),
        };
        ret.update(settings);
        ret
    }

    fn get_name() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(b"OpenVR Overlay Source\0") }
    }

    #[inline]
    fn get_dimensions(&self) -> (u32, u32) {
        self.dimensions.get()
    }

    fn get_properties(&self) -> *mut obs::sys::obs_properties {
        use obs::properties::{
            Properties,
            PropertiesExt,
            PropertyDescription,
        };

        let mut props = Properties::new();

        let overlay_id_name = keys::ID;
        props.add_text(overlay_id_name, overlay_id_name, obs::sys::obs_text_type::OBS_TEXT_DEFAULT);

        unsafe { props.leak() }
    }

    fn update(&self, data: &obs::sys::obs_data) {
        let id_key = keys::ID;
        if let Some(id) = data.get_string(id_key).and_then(|s| CString::new(s).ok()) {
            trace!("Updating overlay source with id: {:?}", &id);
            let new_handle = match openvr::overlay::find_overlay(&id) {
                Ok(v) => {
                    trace!("Got overlay handle: {}", v);
                    Some(v)
                },
                Err(e) => {
                    warn!("Error finding overlay with id {:?}: {:?}", &id, &e);
                    None
                },
            };
            self.overlay_handle.set(new_handle);
        }
    }

    fn video_tick(&self, _seconds: f32) {
        if let Some(overlay) = self.overlay_handle.get() {
            if !self.is_showing() || !overlay.is_visible() {
                return;
            }
            let overlay_handle = overlay.handle();
            let mut image = self.image.write().unwrap();
            if let Err(e) = image.fill(overlay_handle) {
                error!("Error filling overlay image: {:?}", &e);
                return;
            }
            self.dimensions.set(image.dimensions());
            let mut texture = self.texture.write().unwrap();
            with_graphics(|| match &mut *texture {
                &mut Some(ref mut texture) if texture.get_dimensions() == self.dimensions.get() => unsafe {
                    texture.set_image_unchecked(image.data(), self.linesize(), false);
                },
                texture => unsafe {
                    let (w, h) = self.dimensions.get();
                    *texture = obs::graphics::Texture::new(w, h, obs::sys::gs_color_format::GS_BGRA, &[image.data().as_ptr()], obs::sys::GS_DYNAMIC);
                    if texture.is_none() {
                        error!("Error creating obs texture from image data");
                    }
                },
            });
        }
    }

    fn video_render(&self, _effect: *mut obs::sys::gs_effect_t) {
        let texture = self.texture.read().unwrap();
        texture.as_ref().into_iter().for_each(|texture| {
            obs::source::draw(texture, 0, 0, 0, 0, false);
        });
    }
}

pub(crate) mod keys {
    use std::ffi::CStr;

    pub const ID: &'static CStr = unsafe {
        CStr::from_bytes_with_nul_unchecked(b"id\0")
    };
}
