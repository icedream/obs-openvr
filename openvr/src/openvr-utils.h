#pragma once
#include <openvr/openvr.h>
#include <vector>

namespace openvr_utils {
	struct headset_view_size {
		headset_view_size(vr::IVRHeadsetView *headset_view);
		uint32_t m_width;
		uint32_t m_height;
	};

	class OverlayImageData {
	public:
		OverlayImageData();
		size_t required_size();
		vr::EVROverlayError fill_with(vr::VROverlayHandle_t handle);
		void *data();
		size_t data_size();
		uint32_t m_width;
		uint32_t m_height;
	private:
		std::vector<uint8_t> m_data;
	};
}

extern "C" {
	struct openvr_utils_buffer_data {
		size_t size;
		uint8_t *data;
	};

	struct openvr_utils_dimensions {
		uint32_t width;
		uint32_t height;
	};

	void obs_openvr_init_openvr(vr::EVRInitError *e, vr::EVRApplicationType application_type);
	void obs_openvr_shutdown_openvr();
	vr::EVRCompositorError obs_openvr_vrcompositor_getmirrortexturegl(vr::EVREye eye, vr::glUInt_t *tex_id, vr::glSharedTextureHandle_t *tex_handle);
	bool obs_openvr_vrcompositor_releasesharedgltexture(vr::glUInt_t id, vr::glSharedTextureHandle_t handle);
	void obs_openvr_vrcompositor_locksharedgltexture(vr::glSharedTextureHandle_t handle);
	void obs_openvr_vrcompositor_unlocksharedgltexture(vr::glSharedTextureHandle_t handle);

	vr::IVRHeadsetView *openvr_utils_get_headset_view();
	openvr_utils::headset_view_size openvr_utils_headset_view_get_size(vr::IVRHeadsetView *headset_view);
	float openvr_utils_headset_view_get_aspect_ratio(vr::IVRHeadsetView *headset_view);
	vr::HeadsetViewMode_t openvr_utils_headset_view_get_mode(vr::IVRHeadsetView *headset_view);
	vr::EVROverlayError openvr_utils_find_overlay(const char *key, vr::VROverlayHandle_t *handle);
	vr::EVROverlayError openvr_utils_get_overlay_image_data(vr::VROverlayHandle_t handle, openvr_utils::OverlayImageData **data);
	void openvr_utils_overlay_image_data_destroy(openvr_utils::OverlayImageData *data);
	openvr_utils_buffer_data openvr_utils_overlay_image_data_get_data(openvr_utils::OverlayImageData *data);
	openvr_utils_dimensions openvr_utils_overlay_image_data_get_dimensions(openvr_utils::OverlayImageData *data);
	vr::EVROverlayError openvr_utils_overlay_image_data_refill(openvr_utils::OverlayImageData *data, vr::VROverlayHandle_t handle);
}
