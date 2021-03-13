#pragma once
#include <openvr/openvr.h>

namespace openvr_utils {
	struct headset_view_size {
		headset_view_size(vr::IVRHeadsetView *headset_view);
		uint32_t m_width;
		uint32_t m_height;
	};
}

extern "C" {
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
}
