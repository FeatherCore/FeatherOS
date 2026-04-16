/**
 * @file wing.h
 * @brief Wing Desktop Environment - C API Header
 *
 * Wing is a desktop shell for FeatherOS based on FHRE
 * (Feather Hybrid Rendering Engine).
 *
 * Features:
 * - Window management (create, move, resize, close)
 * - Desktop wallpaper and icon grid
 * - Taskbar with application launcher
 * - Multi-tasking support
 *
 * @author FeatherOS Team
 * @version 0.1.0
 */

#ifndef WING_H
#define WING_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Wing version information
 */
#define WING_VERSION_MAJOR 0
#define WING_VERSION_MINOR 1
#define WING_VERSION_PATCH 0
#define WING_VERSION_STRING "0.1.0"

/**
 * @brief Opaque handle to Wing desktop context
 */
typedef struct wing_context wing_context_t;

/**
 * @brief Window handle
 */
typedef uint32_t wing_window_id_t;

/**
 * @brief Window state
 */
typedef enum {
    WING_WINDOW_NORMAL = 0,
    WING_WINDOW_MINIMIZED,
    WING_WINDOW_MAXIMIZED,
    WING_WINDOW_CLOSED
} wing_window_state_t;

/**
 * @brief 2D vector for positions and sizes
 */
typedef struct {
    float x;
    float y;
} wing_vec2_t;

/**
 * @brief Rectangle
 */
typedef struct {
    float x;
    float y;
    float width;
    float height;
} wing_rect_t;

/**
 * @brief RGBA color
 */
typedef struct {
    uint8_t r;
    uint8_t g;
    uint8_t b;
    uint8_t a;
} wing_color_t;

/**
 * @brief Application information
 */
typedef struct {
    const char* name;
    const char* description;
    wing_vec2_t default_size;
    wing_vec2_t initial_position;
    wing_color_t icon_color;
} wing_app_info_t;

/**
 * @brief Window callbacks
 */
typedef struct {
    void (*on_create)(wing_window_id_t window_id, void* user_data);
    void (*on_close)(wing_window_id_t window_id, void* user_data);
    void (*on_focus)(wing_window_id_t window_id, void* user_data);
    void (*on_blur)(wing_window_id_t window_id, void* user_data);
    void (*on_resize)(wing_window_id_t window_id, wing_vec2_t new_size, void* user_data);
    void (*on_move)(wing_window_id_t window_id, wing_vec2_t new_position, void* user_data);
} wing_window_callbacks_t;

/**
 * @brief Error codes
 */
typedef enum {
    WING_OK = 0,
    WING_ERROR_INVALID_ARGUMENT,
    WING_ERROR_OUT_OF_MEMORY,
    WING_ERROR_WINDOW_NOT_FOUND,
    WING_ERROR_APP_NOT_FOUND,
    WING_ERROR_INVALID_OPERATION
} wing_error_t;

/* ==========================================================================
 * Context Management
 * ========================================================================== */

/**
 * @brief Create a new Wing desktop context
 *
 * @param width Screen width
 * @param height Screen height
 * @return wing_context_t* Context handle or NULL on error
 */
wing_context_t* wing_create(float width, float height);

/**
 * @brief Destroy Wing context and free resources
 *
 * @param ctx Context handle
 */
void wing_destroy(wing_context_t* ctx);

/**
 * @brief Initialize Wing desktop environment
 *
 * @param ctx Context handle
 * @return wing_error_t Error code
 */
wing_error_t wing_init(wing_context_t* ctx);

/**
 * @brief Update Wing desktop (call every frame)
 *
 * @param ctx Context handle
 * @param delta_time Time since last frame in seconds
 * @return wing_error_t Error code
 */
wing_error_t wing_update(wing_context_t* ctx, float delta_time);

/* ==========================================================================
 * Window Management
 * ========================================================================== */

/**
 * @brief Create a new window
 *
 * @param ctx Context handle
 * @param title Window title
 * @param size Window size
 * @param position Window position
 * @param callbacks Window callbacks (can be NULL)
 * @param user_data User data passed to callbacks
 * @return wing_window_id_t Window ID or 0 on error
 */
wing_window_id_t wing_window_create(
    wing_context_t* ctx,
    const char* title,
    wing_vec2_t size,
    wing_vec2_t position,
    const wing_window_callbacks_t* callbacks,
    void* user_data
);

/**
 * @brief Close a window
 *
 * @param ctx Context handle
 * @param window_id Window ID
 * @return wing_error_t Error code
 */
wing_error_t wing_window_close(wing_context_t* ctx, wing_window_id_t window_id);

/**
 * @brief Move a window
 *
 * @param ctx Context handle
 * @param window_id Window ID
 * @param delta Movement delta
 * @return wing_error_t Error code
 */
wing_error_t wing_window_move(wing_context_t* ctx, wing_window_id_t window_id, wing_vec2_t delta);

/**
 * @brief Resize a window
 *
 * @param ctx Context handle
 * @param window_id Window ID
 * @param new_size New size
 * @return wing_error_t Error code
 */
wing_error_t wing_window_resize(wing_context_t* ctx, wing_window_id_t window_id, wing_vec2_t new_size);

/**
 * @brief Focus a window
 *
 * @param ctx Context handle
 * @param window_id Window ID
 * @return wing_error_t Error code
 */
wing_error_t wing_window_focus(wing_context_t* ctx, wing_window_id_t window_id);

/**
 * @brief Get window state
 *
 * @param ctx Context handle
 * @param window_id Window ID
 * @param state Output state
 * @return wing_error_t Error code
 */
wing_error_t wing_window_get_state(
    wing_context_t* ctx,
    wing_window_id_t window_id,
    wing_window_state_t* state
);

/**
 * @brief Set window state
 *
 * @param ctx Context handle
 * @param window_id Window ID
 * @param state New state
 * @return wing_error_t Error code
 */
wing_error_t wing_window_set_state(
    wing_context_t* ctx,
    wing_window_id_t window_id,
    wing_window_state_t state
);

/**
 * @brief Get window at position
 *
 * @param ctx Context handle
 * @param position Position to check
 * @return wing_window_id_t Window ID or 0 if none
 */
wing_window_id_t wing_window_at_position(wing_context_t* ctx, wing_vec2_t position);

/* ==========================================================================
 * Application Launcher
 * ========================================================================== */

/**
 * @brief Launch an application
 *
 * @param ctx Context handle
 * @param app_info Application information
 * @return wing_window_id_t Window ID or 0 on error
 */
wing_window_id_t wing_launch_app(wing_context_t* ctx, const wing_app_info_t* app_info);

/**
 * @brief Toggle application launcher
 *
 * @param ctx Context handle
 */
void wing_launcher_toggle(wing_context_t* ctx);

/**
 * @brief Open application launcher
 *
 * @param ctx Context handle
 */
void wing_launcher_open(wing_context_t* ctx);

/**
 * @brief Close application launcher
 *
 * @param ctx Context handle
 */
void wing_launcher_close(wing_context_t* ctx);

/* ==========================================================================
 * Input Handling
 * ========================================================================== */

/**
 * @brief Handle mouse click
 *
 * @param ctx Context handle
 * @param position Click position
 */
void wing_handle_click(wing_context_t* ctx, wing_vec2_t position);

/**
 * @brief Handle mouse drag
 *
 * @param ctx Context handle
 * @param position Current position
 * @param delta Movement delta
 */
void wing_handle_drag(wing_context_t* ctx, wing_vec2_t position, wing_vec2_t delta);

/**
 * @brief Handle mouse release
 *
 * @param ctx Context handle
 * @param position Release position
 */
void wing_handle_release(wing_context_t* ctx, wing_vec2_t position);

/* ==========================================================================
 * Utility Functions
 * ========================================================================== */

/**
 * @brief Create a color from RGB values
 */
static inline wing_color_t wing_color_rgb(uint8_t r, uint8_t g, uint8_t b) {
    wing_color_t color = { r, g, b, 255 };
    return color;
}

/**
 * @brief Create a color from RGBA values
 */
static inline wing_color_t wing_color_rgba(uint8_t r, uint8_t g, uint8_t b, uint8_t a) {
    wing_color_t color = { r, g, b, a };
    return color;
}

/**
 * @brief Create a 2D vector
 */
static inline wing_vec2_t wing_vec2(float x, float y) {
    wing_vec2_t vec = { x, y };
    return vec;
}

/**
 * @brief Create a rectangle
 */
static inline wing_rect_t wing_rect(float x, float y, float width, float height) {
    wing_rect_t rect = { x, y, width, height };
    return rect;
}

/**
 * @brief Get Wing version string
 *
 * @return const char* Version string
 */
const char* wing_version_string(void);

/**
 * @brief Get Wing version number
 *
 * @return uint32_t Version as (major << 16) | (minor << 8) | patch
 */
uint32_t wing_version_number(void);

#ifdef __cplusplus
}
#endif

#endif /* WING_H */
