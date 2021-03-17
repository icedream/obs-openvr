use std::{
    ffi::{
        CStr,
        CString,
    },
    sync::RwLock,
};
use obs::data::ObsData;

pub struct OpenVROverlaySource {
    _source_handle: *mut obs::sys::obs_source_t,
    key: RwLock<Option<CString>>,
    dimensions: RwLock<Option<(u32, u32)>>,
    image_data: RwLock<Option<openvr::overlay::OverlayImageData>>,
}

impl OpenVROverlaySource {
    pub fn new(source: *mut obs::sys::obs_source_t) -> Self {
        OpenVROverlaySource {
            _source_handle: source,
            key: RwLock::new(None),
            dimensions: RwLock::new(None),
            image_data: RwLock::new(None),
        }
    }
}

impl obs::source::VideoSource for OpenVROverlaySource {
    const ID: &'static [u8] = b"obs-openvr-overlay\0";
    const OUTPUT_FLAGS: Option<u32> = None;

    fn create(settings: &mut obs::sys::obs_data, source: *mut obs::sys::obs_source_t) -> Self {
        let ret = OpenVROverlaySource::new(source);
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
        let key = self.key.read().unwrap();
        if let Some(key) = key.as_ref() {
            let mut image_data = self.image_data.write().unwrap();
            if let Some(image_data) = image_data.as_mut() {
                let handle = openvr::overlay::find_overlay(key).ok();
                if let Some(handle) = handle {
                    if let Err(e) = image_data.refill(handle) {
                        error!("error refilling overlay image: {:?}", &e);
                    }
                }
            } else {
                *image_data = match openvr::overlay::OverlayImageData::find_overlay(key) {
                    Ok(v) => Some(v),
                    Err(e) => {
                        error!("error getting overlay image: {:?}", &e);
                        return;
                    },
                };
            }
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
                if let Some(texture) = obs::graphics::Texture::new(width, height, obs::sys::gs_color_format::GS_BGRA, 1, &mut buffer, 0) {
                    texture
                } else {
                    error!("gs_texture_create failed");
                    return;
                }
            };
            obs::source::draw(&mut texture, 0, 0, 0, 0, false);
        }
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
