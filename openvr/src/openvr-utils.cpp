#include "openvr-utils.h"
#include <openvr/openvr.h>
#include <iostream>

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
	std::cout << "obs_openvr: " << "releasing shared GL texture" << std::endl;
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
