/*
 * Feather Hybrid Render Engine (C version) header file
 *
 * Lightweight hybrid rendering engine supporting 2D, 2.5D, and 3D rendering
 */

#ifndef FHRE_H
#define FHRE_H

#include <stdint.h>
#include <stdbool.h>

/* FHRE version */
#define FHRE_VERSION "1.0.0"

/* Render modes */
enum fhre_render_mode {
    FHRE_MODE_2D,
    FHRE_MODE_25D,
    FHRE_MODE_3D
};

/* Color structure */
typedef struct {
    uint8_t r;
    uint8_t g;
    uint8_t b;
    uint8_t a;
} fhre_color_t;

/* Vector 2D structure */
typedef struct {
    float x;
    float y;
} fhre_vec2_t;

/* Vector 3D structure */
typedef struct {
    float x;
    float y;
    float z;
} fhre_vec3_t;

/* Render context */
typedef struct {
    enum fhre_render_mode mode;
    int width;
    int height;
    void *framebuffer;
} fhre_context_t;

/* Function prototypes */
fhre_context_t *fhre_init(int width, int height, enum fhre_render_mode mode);
void fhre_set_mode(fhre_context_t *ctx, enum fhre_render_mode mode);
void fhre_clear(fhre_context_t *ctx, fhre_color_t color);
void fhre_draw_point(fhre_context_t *ctx, fhre_vec2_t pos, fhre_color_t color);
void fhre_draw_line(fhre_context_t *ctx, fhre_vec2_t start, fhre_vec2_t end, fhre_color_t color);
void fhre_draw_rect(fhre_context_t *ctx, fhre_vec2_t pos, float width, float height, fhre_color_t color);
void *fhre_get_framebuffer(fhre_context_t *ctx);
void fhre_cleanup(fhre_context_t *ctx);
const char *fhre_version(void);

#endif /* FHRE_H */
