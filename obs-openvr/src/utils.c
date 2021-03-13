#include "utils.h"
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

static GLFWglproc getProcAddressAndPrint(char *s) {
	GLFWglproc ret = glfwGetProcAddress(s);
	debug_printf("%s() = %p\n", s, ret);
	return ret;
}

void obs_openvr_utils_init() {
	printf("obs_openvr: obs_openvr_utils_init\n");
	fprintf(stderr, "obs_openvr: obs_openvr_utils_init\n");
	debug_printf("obs_openvr: obs_openvr_utils_init\n");
	debug_printf("obs_openvr: GL_NO_ERROR = %d\n", GL_NO_ERROR);
	glGenBuffers = (void (*)(GLsizei, GLuint*))getProcAddressAndPrint("glGenBuffers");
	glDeleteBuffers = (void (*)(GLsizei, GLuint*))getProcAddressAndPrint("glDeleteBuffers");
	glBindBuffer = (void (*)(GLenum, GLuint))getProcAddressAndPrint("glBindBuffer");
	// glBindTexture = (void (*)(GLenum, GLuint))getProcAddressAndPrint("glBindTexture");
	// glTexSubImage2D = (void (*)(GLuint, GLint, GLint, GLint, GLsizei, GLsizei, GLenum, GLenum, const void *))getProcAddressAndPrint("glTexSubImage2D");
	glMapBuffer = (void *(*)(GLenum, GLenum))getProcAddressAndPrint("glMapBuffer");
	glUnmapBuffer = (GLboolean (*)(GLenum))getProcAddressAndPrint("glUnmapBuffer");
	glBufferData = (void (*)(GLenum, GLsizeiptr, const void *, GLenum))getProcAddressAndPrint("glBufferData");
}

static void print_context_indented(struct obs_openvr_copy_context *ctx) {
	if (ctx == NULL) {
		return;
	}
	debug_printf("\ttexture: %d\n\tbuffer: %d\n", ctx->texture, ctx->buffer);
}

struct obs_openvr_copy_context *obs_openvr_copy_context_create(GLuint texture) {
	struct obs_openvr_copy_context *ctx = (struct obs_openvr_copy_context *)calloc(1, sizeof(struct obs_openvr_copy_context));
	if (ctx == NULL) {
		return ctx;
	}
	memset((void *)ctx, 0, sizeof(struct obs_openvr_copy_context));
	ctx->texture = texture;
	GLuint buffer;
	glGenBuffers(1, &buffer);
	if (buffer == 0) {
		return NULL;
	}
	ctx->buffer = buffer;
	debug_printf("copy_context_create():\n");
	print_context_indented(ctx);
	return ctx;
}

void obs_openvr_copy_context_destroy(struct obs_openvr_copy_context *ctx) {
	debug_printf("copy_context_destroy():\n");
	if (ctx == NULL) {
		return;
	}
	print_context_indented(ctx);
	if (ctx->img != NULL) {
		free(ctx->img);
		ctx->img = NULL;
		ctx->img_size = 0;
	}
	if (ctx->buffer != 0) {
		glDeleteBuffers(1, &ctx->buffer);
	}
	free(ctx);
}

static size_t get_bytes_per_pixel(GLenum format) {
	// TODO: actually implement
	return 3;
}

void obs_openvr_copy_context_ensure_size(struct obs_openvr_copy_context *ctx, GLsizei width, GLsizei height, GLenum format) {
	debug_printf("obs_openvr_copy_context_ensure_size(%p, %d, %d)\n", ctx, width, height);
	const size_t n = width * height * get_bytes_per_pixel(format);
	if (ctx->img == NULL || ctx->img_size < n) {
		if (ctx->img != NULL) {
			debug_printf("reallocating img with dimensions: (%d, %d)\n", width, height);
			debug_printf("\tprevious ptr: %p size: %lu\n", ctx->img, ctx->img_size);
			free(ctx->img);
		} else {
			debug_printf("allocating new img with dimensions: (%d, %d) size: %lu\n", width, height, n);
		}
		ctx->img = malloc(n * 10); // TODO: figure out why the size isn't right
		if (ctx->img == NULL) {
			debug_printf("allocation of img FAILED\n");
		}
		ctx->img_size = n;
	}
}

int obs_openvr_copy_texture(struct obs_openvr_copy_context *ctx, GLsizei width, GLsizei height, GLenum format) {
	debug_printf("obs_openvr_copy_texture(%p, %d, %d, %x)\n", ctx, width, height, format);
	GLenum status = glGetError();
	if (status != GL_NO_ERROR) {
		debug_printf("\tstarting with error: %x\n", status);
	}
	obs_openvr_copy_context_ensure_size(ctx, width, height, format);
	glBindTexture(GL_TEXTURE_2D, ctx->texture);
	if ((status = glGetError()) != GL_NO_ERROR) {
		debug_printf("glBindTexture failed with error: %x\n", status);
		return status;
	}
	glGetTexImage(GL_TEXTURE_2D, 0, format, GL_UNSIGNED_BYTE, ctx->img);
	if ((status = glGetError()) != GL_NO_ERROR) {
		debug_printf("glGetTexImage failed with error: %x\n", status);
		return status;
	}
	return GL_NO_ERROR;
	// obs_openvr_copy_context_ensure_size(ctx, width, height);
	// glBindTexture(GL_TEXTURE_2D, ctx->texture);
	// glBindBuffer(GL_PIXEL_UNPACK_BUFFER, ctx->buffer);
	// glTexSubImage2D(GL_TEXTURE_2D, 0, 0, 0, width, height, format, GL_UNSIGNED_BYTE, NULL);
	// if ((status = glGetError()) != GL_NO_ERROR) {
	// 	fprintf(stderr, "glTexSubImage2D failed with error: %x\n", status);
	// }
	// GLubyte *buf = (GLubyte *)glMapBuffer(GL_PIXEL_UNPACK_BUFFER, GL_READ_ONLY);
	// if (!buf) {
	// 	status = glGetError();
	// 	if (status != GL_NO_ERROR) {
	// 		fprintf(stderr, "glMapBuffer failed with error: %x\n", status);
	// 	}
	// 	return -1;
	// }
	// printf("obs_openvr: copied texture to buffer at %p\n", buf);
	// glUnmapBuffer(GL_PIXEL_UNPACK_BUFFER);
}
