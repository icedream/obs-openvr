#include "openvr-utils.h"
#include <openvr/openvr.h>
#include <iostream>
#include <vector>
#include <cstdlib>
#include <memory>

openvr_utils::headset_view_size::headset_view_size(vr::IVRHeadsetView *headset_view):
	m_width(0), m_height(0)
{
	std::cerr << "headset_view_size(" << headset_view << ")" << std::endl;
	uint32_t width, height;
	headset_view->GetHeadsetViewSize(&width, &height);
	std::cerr << "\t(" << width << ", " << height << ')' << std::endl;
	m_width = width;
	m_height = height;
}

vr::IVRHeadsetView *openvr_utils_get_headset_view() {
	return vr::VRHeadsetView();
}

void obs_openvr_init_openvr(vr::EVRInitError *e, vr::EVRApplicationType application_type)
{
	vr::VR_Init(e, application_type);
	if (*e != vr::VRInitError_None) {
		return;
	}
	uint32_t w, h;
	vr::VRSystem()->GetRecommendedRenderTargetSize(&w, &h);
	std::cout << "obs_openvr: render target size: (" << w << ", " << h << ")" << std::endl;
}

void obs_openvr_shutdown_openvr()
{
	vr::VR_Shutdown();
}

vr::EVRCompositorError obs_openvr_vrcompositor_getmirrortexturegl(vr::EVREye eye, vr::glUInt_t *tex_id, vr::glSharedTextureHandle_t *tex_handle)
{
	return vr::VRCompositor()->GetMirrorTextureGL(eye, tex_id, tex_handle);
}

bool obs_openvr_vrcompositor_releasesharedgltexture(vr::glUInt_t id, vr::glSharedTextureHandle_t handle)
{
	return vr::VRCompositor()->ReleaseSharedGLTexture(id, handle);
}
void obs_openvr_vrcompositor_locksharedgltexture(vr::glSharedTextureHandle_t handle)
{
	vr::VRCompositor()->LockGLSharedTextureForAccess(handle);
}
void obs_openvr_vrcompositor_unlocksharedgltexture(vr::glSharedTextureHandle_t handle)
{
	vr::VRCompositor()->UnlockGLSharedTextureForAccess(handle);
}

openvr_utils::headset_view_size openvr_utils_headset_view_get_size(vr::IVRHeadsetView *headset_view)
{
	auto ret = openvr_utils::headset_view_size(headset_view);
	return ret;
}

float openvr_utils_headset_view_get_aspect_ratio(vr::IVRHeadsetView *headset_view)
{
	return headset_view->GetHeadsetViewAspectRatio();
}

vr::HeadsetViewMode_t openvr_utils_headset_view_get_mode(vr::IVRHeadsetView *headset_view)
{
	return headset_view->GetHeadsetViewMode();
}

vr::EVROverlayError openvr_utils_find_overlay(const char *key, vr::VROverlayHandle_t *handle)
{
	return vr::VROverlay()->FindOverlay(key, handle);
}

openvr_utils::OverlayImageData::OverlayImageData():
	m_width(0), m_height(0), m_data()
{
}
size_t openvr_utils::OverlayImageData::required_size()
{
	return m_width * m_height * 4;
}
vr::EVROverlayError openvr_utils::OverlayImageData::fill_with(vr::VROverlayHandle_t handle)
{
	auto vroverlay = vr::VROverlay();
	vr::EVROverlayError status = vroverlay->GetOverlayImageData(handle, nullptr, 0, &m_width, &m_height);
	std::cerr << "OverlayImageData::fill_with: (" << m_width << ", " << m_height << ')' << std::endl;
	if (status != vr::VROverlayError_None && status != vr::VROverlayError_ArrayTooSmall) {
		return status;
	}
	std::vector<uint8_t> data(required_size(), 0);
	status = vroverlay->GetOverlayImageData(handle, static_cast<void*>(data.data()), data.size(), &m_width, &m_height);
	m_data = std::move(data);
	return status;
}
void *openvr_utils::OverlayImageData::data() {
	return static_cast<void*>(m_data.data());
}

vr::EVROverlayError openvr_utils_get_overlay_image_data(vr::VROverlayHandle_t handle, openvr_utils::OverlayImageData **data)
{
	std::unique_ptr<openvr_utils::OverlayImageData> image_data(new openvr_utils::OverlayImageData());
	auto ret = image_data->fill_with(handle);
	if (ret == vr::VROverlayError_None) {
		*data = image_data.release();
	}
	return ret;
}
void openvr_utils_overlay_image_data_destroy(openvr_utils::OverlayImageData *data)
{
	if (data == nullptr) {
		return;
	}
	delete data;
}
openvr_utils_buffer_data openvr_utils_overlay_image_data_get_data(openvr_utils::OverlayImageData *data)
{
	struct openvr_utils_buffer_data buf = {
		.size = data->required_size(),
		.data = static_cast<uint8_t*>(data->data()),
	};
	return buf;
}
openvr_utils_dimensions openvr_utils_overlay_image_data_get_dimensions(openvr_utils::OverlayImageData *data)
{
	struct openvr_utils_dimensions ret = {
		.width = data->m_width,
		.height = data->m_height,
	};
	return ret;
}
