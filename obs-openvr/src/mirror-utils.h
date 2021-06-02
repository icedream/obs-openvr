#pragma once

#include "glad/glad/glad.h"

struct obs_openvr_gl_texture_size {
	GLint width;
	GLint height;
};

extern void obs_openvr_get_gl_texture_size(GLuint texture, struct obs_openvr_gl_texture_size *out);
extern int obs_openvr_copy_gl_texture(GLuint texture, GLenum format, uint8_t *img);
