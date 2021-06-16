#include <openvr/openvr.h>
#include <vector>
#include <memory>
#include <iostream>
#include "overlay-utils.h"

openvrs::OverlayImage::OverlayImage():
	m_width(0), m_height(0), m_data()
{
}

size_t openvrs::OverlayImage::required_size()
{
	return m_width * m_height * 4;
}

vr::EVROverlayError openvrs::OverlayImage::fill_with(vr::VROverlayHandle_t handle)
{
	auto vroverlay = vr::VROverlay();
	auto status = vroverlay->GetOverlayImageData(handle, data(), m_data.size(), &m_width, &m_height);
	if (status == vr::VROverlayError_ArrayTooSmall) {
		std::cerr << "reallocating buffer for overlay handle: 0x" << std::hex << handle << std::endl;
		m_data.resize(required_size(), 0);
		status = vroverlay->GetOverlayImageData(handle, data(), m_data.size(), &m_width, &m_height);
	}
	return status;
}

void *openvrs::OverlayImage::data()
{
	return static_cast<void *>(m_data.data());
}

size_t openvrs::OverlayImage::data_size()
{
	return m_data.size();
}

openvrs::OverlayImage *openvrs_overlay_image_create()
{
	std::unique_ptr<openvrs::OverlayImage> ret(new openvrs::OverlayImage());
	return ret.release();
}

void openvrs_overlay_image_destroy(openvrs::OverlayImage *image)
{
	if (image == nullptr) {
		return;
	}
	delete image;
}
vr::EVROverlayError openvrs_overlay_image_fill(openvrs::OverlayImage *image, vr::VROverlayHandle_t handle)
{
	return image->fill_with(handle);
}
openvrs_overlay_image_data openvrs_overlay_image_get_data(openvrs::OverlayImage *image)
{
	struct openvrs_overlay_image_data ret = {
		.width = image->width(),
		.height = image->height(),
		.data = image->data(),
		.length = image->data_size(),
	};
	return ret;
}
bool openvrs_is_overlay_visible(vr::VROverlayHandle_t handle)
{
	auto vroverlay = vr::VROverlay();
	if (vroverlay == nullptr) {
		return false;
	}
	return vroverlay->IsOverlayVisible(handle);
}
