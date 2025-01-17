#pragma once

#include <GLFW/glfw3.h>
#include <stdio.h>

#ifdef DEBUG
#define debug_printf(...) printf(__VA_ARGS__)
#else
#define debug_printf(...)
#endif // defined(DEBUG)

struct obs_openvr_copy_context {
	GLuint texture;
	size_t img_size;
	uint8_t *img;
};

struct obs_openvr_texture_size {
	GLint width;
	GLint height;
};

typedef struct obs_openvr_copy_context obs_openvr_copy_context_t;

// exported from C
extern struct obs_openvr_copy_context *obs_openvr_copy_context_create(GLuint texture);
extern void obs_openvr_copy_context_destroy(struct obs_openvr_copy_context *ctx);
extern void obs_openvr_copy_context_get_texture_size(struct obs_openvr_copy_context *ctx, struct obs_openvr_texture_size *out);
extern int obs_openvr_copy_texture(struct obs_openvr_copy_context *ctx, GLsizei width, GLsizei height, GLenum format);
extern void obs_openvr_copy_context_ensure_size(struct obs_openvr_copy_context *ctx, GLsizei width, GLsizei height, GLenum format);


// From rust
extern uint8_t obs_openvr_bytes_per_pixel(GLenum format);
extern void obs_openvr_copy_context_print(const struct obs_openvr_copy_context *ctx);
