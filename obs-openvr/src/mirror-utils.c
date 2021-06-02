#include "mirror-utils.h"
#include "glad/glad/glad.h"
#include <stdio.h>

void obs_openvr_get_gl_texture_size(GLuint texture, struct obs_openvr_gl_texture_size *out) {
	glBindTexture(GL_TEXTURE_2D, texture);
	glGetTexLevelParameteriv(GL_TEXTURE_2D, 0, GL_TEXTURE_WIDTH, &out->width);
	glGetTexLevelParameteriv(GL_TEXTURE_2D, 0, GL_TEXTURE_HEIGHT, &out->height);
}

int obs_openvr_copy_gl_texture(GLuint texture, GLenum format, uint8_t *img) {
	GLenum status = glGetError();
	if (status != GL_NO_ERROR) {
		fprintf(stderr, "\tstarting with error: %x\n", status);
	}
	glBindTexture(GL_TEXTURE_2D, texture);
	if ((status = glGetError()) != GL_NO_ERROR) {
		fprintf(stderr, "glBindTexture failed with error: %x\n", status);
		return status;
	}
	glGetTexImage(GL_TEXTURE_2D, 0, format, GL_UNSIGNED_BYTE, img);
	if ((status = glGetError()) != GL_NO_ERROR) {
		fprintf(stderr, "glGetTexImage failed with error: %x\n", status);
		return status;
	}
	return GL_NO_ERROR;
}
