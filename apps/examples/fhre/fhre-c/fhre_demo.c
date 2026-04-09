/*
 * FHRE C language demo
 *
 * Demonstrates the basic usage of Feather Hybrid Render Engine
 */

#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>

/* Include FHRE header */
#include "../../../fhre/clang/fhre.h"

int main(int argc, char *argv[])
{
    printf("FHRE C language demo\n");
    printf("Version: %s\n", fhre_version());
    
    // Initialize FHRE context with 640x480 resolution and 2D mode
    fhre_context_t *ctx = fhre_init(640, 480, FHRE_MODE_2D);
    if (!ctx) {
        printf("Failed to initialize FHRE context\n");
        return 1;
    }
    
    printf("FHRE context initialized: %dx%d, mode: %d\n", 
           ctx->width, ctx->height, ctx->mode);
    
    // Clear framebuffer with black color
    fhre_color_t black = { 0, 0, 0, 255 };
    fhre_clear(ctx, black);
    printf("Framebuffer cleared with black\n");
    
    // Draw a red point
    fhre_color_t red = { 255, 0, 0, 255 };
    fhre_vec2_t point = { 100.0f, 100.0f };
    fhre_draw_point(ctx, point, red);
    printf("Drew red point at (100, 100)\n");
    
    // Draw a green line
    fhre_color_t green = { 0, 255, 0, 255 };
    fhre_vec2_t line_start = { 50.0f, 200.0f };
    fhre_vec2_t line_end = { 200.0f, 350.0f };
    fhre_draw_line(ctx, line_start, line_end, green);
    printf("Drew green line from (50, 200) to (200, 350)\n");
    
    // Draw a blue rectangle
    fhre_color_t blue = { 0, 0, 255, 255 };
    fhre_vec2_t rect_pos = { 300.0f, 150.0f };
    fhre_draw_rect(ctx, rect_pos, 150.0f, 100.0f, blue);
    printf("Drew blue rectangle at (300, 150) with size 150x100\n");
    
    // Get framebuffer address
    void *fb = fhre_get_framebuffer(ctx);
    if (fb) {
        printf("Framebuffer address: %p\n", fb);
        // Here you could write code to display the framebuffer
        // For example, copy it to a display device
    }
    
    // Cleanup
    fhre_cleanup(ctx);
    printf("FHRE context cleaned up\n");
    
    printf("FHRE C demo completed\n");
    return 0;
}
