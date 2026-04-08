/*
 * Feather Hybrid Render Engine (C version)
 *
 * Lightweight hybrid rendering engine supporting 2D, 2.5D, and 3D rendering
 */

#include "fhre.h"
#include <stdlib.h>

/* FHRE version */
#define FHRE_VERSION "1.0.0"

/* Initialize FHRE context */
fhre_context_t *fhre_init(int width, int height, enum fhre_render_mode mode)
{
    fhre_context_t *ctx = (fhre_context_t *)malloc(sizeof(fhre_context_t));
    if (ctx) {
        ctx->mode = mode;
        ctx->width = width;
        ctx->height = height;
        ctx->framebuffer = malloc(width * height * 4); // RGBA
    }
    return ctx;
}

/* Set render mode */
void fhre_set_mode(fhre_context_t *ctx, enum fhre_render_mode mode)
{
    if (ctx) {
        ctx->mode = mode;
    }
}

/* Clear framebuffer */
void fhre_clear(fhre_context_t *ctx, fhre_color_t color)
{
    if (ctx && ctx->framebuffer) {
        uint32_t *fb = (uint32_t *)ctx->framebuffer;
        uint32_t color_val = (color.a << 24) | (color.r << 16) | (color.g << 8) | color.b;
        int size = ctx->width * ctx->height;
        for (int i = 0; i < size; i++) {
            fb[i] = color_val;
        }
    }
}

/* Draw a 2D point */
void fhre_draw_point(fhre_context_t *ctx, fhre_vec2_t pos, fhre_color_t color)
{
    if (ctx && ctx->framebuffer) {
        int x = (int)pos.x;
        int y = (int)pos.y;
        if (x >= 0 && x < ctx->width && y >= 0 && y < ctx->height) {
            uint32_t *fb = (uint32_t *)ctx->framebuffer;
            uint32_t color_val = (color.a << 24) | (color.r << 16) | (color.g << 8) | color.b;
            fb[y * ctx->width + x] = color_val;
        }
    }
}

/* Draw a 2D line */
void fhre_draw_line(fhre_context_t *ctx, fhre_vec2_t start, fhre_vec2_t end, fhre_color_t color)
{
    // Bresenham's line algorithm
    int x0 = (int)start.x;
    int y0 = (int)start.y;
    int x1 = (int)end.x;
    int y1 = (int)end.y;
    
    int dx = abs(x1 - x0);
    int dy = abs(y1 - y0);
    int sx = x0 < x1 ? 1 : -1;
    int sy = y0 < y1 ? 1 : -1;
    int err = dx - dy;
    
    while (1) {
        fhre_vec2_t pos = { (float)x0, (float)y0 };
        fhre_draw_point(ctx, pos, color);
        
        if (x0 == x1 && y0 == y1) {
            break;
        }
        
        int e2 = 2 * err;
        if (e2 > -dy) {
            err -= dy;
            x0 += sx;
        }
        if (e2 < dx) {
            err += dx;
            y0 += sy;
        }
    }
}

/* Draw a 2D rectangle */
void fhre_draw_rect(fhre_context_t *ctx, fhre_vec2_t pos, float width, float height, fhre_color_t color)
{
    fhre_vec2_t start, end;
    
    // Top line
    start.x = pos.x;
    start.y = pos.y;
    end.x = pos.x + width;
    end.y = pos.y;
    fhre_draw_line(ctx, start, end, color);
    
    // Right line
    start.x = pos.x + width;
    start.y = pos.y;
    end.x = pos.x + width;
    end.y = pos.y + height;
    fhre_draw_line(ctx, start, end, color);
    
    // Bottom line
    start.x = pos.x + width;
    start.y = pos.y + height;
    end.x = pos.x;
    end.y = pos.y + height;
    fhre_draw_line(ctx, start, end, color);
    
    // Left line
    start.x = pos.x;
    start.y = pos.y + height;
    end.x = pos.x;
    end.y = pos.y;
    fhre_draw_line(ctx, start, end, color);
}

/* Get framebuffer */
void *fhre_get_framebuffer(fhre_context_t *ctx)
{
    return ctx ? ctx->framebuffer : NULL;
}

/* Cleanup FHRE context */
void fhre_cleanup(fhre_context_t *ctx)
{
    if (ctx) {
        if (ctx->framebuffer) {
            free(ctx->framebuffer);
        }
        free(ctx);
    }
}

/* Get FHRE version */
const char *fhre_version(void)
{
    return FHRE_VERSION;
}
