#pragma once
#include <openvr/openvr.h>
#include <vector>

namespace openvrs {
	class OverlayImage {
	public:
		OverlayImage();
		size_t required_size();
		vr::EVROverlayError fill_with(vr::VROverlayHandle_t handle);
		void *data();
		inline uint32_t width() {
			return m_width;
		}
		inline uint32_t height() {
			return m_height;
		}
		size_t data_size();
	private:
		uint32_t m_width;
		uint32_t m_height;
		std::vector<uint8_t> m_data;
	};
}

extern "C" {
	struct openvrs_overlay_image_data {
		uint32_t width;
		uint32_t height;
		void *data;
		size_t length;
	};

	openvrs::OverlayImage *openvrs_overlay_image_create();
	void openvrs_overlay_image_destroy(openvrs::OverlayImage *image);
	vr::EVROverlayError openvrs_overlay_image_fill(openvrs::OverlayImage *image, vr::VROverlayHandle_t handle);
	openvrs_overlay_image_data openvrs_overlay_image_get_data(openvrs::OverlayImage *image);
	bool openvrs_is_overlay_visible(vr::VROverlayHandle_t handle);
}
