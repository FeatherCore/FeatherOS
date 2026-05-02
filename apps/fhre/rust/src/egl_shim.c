#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#define FHRE_EGL_OK 0
#define FHRE_EGL_UNAVAILABLE 1
#define FHRE_EGL_UNSUPPORTED 2
#define FHRE_EGL_OVERFLOW 3
#define FHRE_EGL_SUBMIT_FAILED 4
#define FHRE_EGL_FENCE_FAILED 5

#define FHRE_EGL_COMMAND_SET_CLIP 1
#define FHRE_EGL_COMMAND_FILL 2
#define FHRE_EGL_COMMAND_IMAGE 3

#define FHRE_EGL_FORMAT_RGB565 1
#define FHRE_EGL_FORMAT_RGB888 2
#define FHRE_EGL_FORMAT_RGBA8888 3
#define FHRE_EGL_FORMAT_A8 4

#define FHRE_EGL_FIT_STRETCH 0

typedef struct {
    int32_t x;
    int32_t y;
    uint16_t w;
    uint16_t h;
} FhreEglRect;

typedef struct {
    uint8_t r;
    uint8_t g;
    uint8_t b;
    uint8_t a;
} FhreEglColor;

typedef struct {
    void *data;
    uintptr_t len;
    uint16_t width;
    uint16_t height;
    uintptr_t stride;
    uint8_t bpp;
    uint8_t format;
} FhreEglFramebuffer;

typedef struct {
    const void *data;
    uintptr_t len;
    uint16_t width;
    uint16_t height;
    uintptr_t stride;
    uint8_t format;
} FhreEglImageSource;

typedef struct {
    uint8_t kind;
    FhreEglRect rect;
    uint8_t has_clip;
    FhreEglRect clip;
    FhreEglColor color;
    FhreEglImageSource source;
    uint8_t opacity;
    uint8_t fit;
    uint8_t has_tint;
    FhreEglColor tint;
} FhreEglCommand;

typedef struct {
    FhreEglFramebuffer framebuffer;
    FhreEglRect run_bounds;
    const FhreEglCommand *commands;
    uintptr_t command_count;
} FhreEglPacket;

typedef struct {
    uint32_t token;
} FhreEglFence;

typedef struct {
    uint32_t queued;
    uint32_t submitted;
    uint32_t completed;
    uint32_t readbacks;
    uint32_t unsupported;
    uint32_t submit_failures;
    uint32_t fence_failures;
    uint32_t overflows;
} FhreEglWorkerStats;

#ifndef FHRE_EGL_REAL

int32_t fhre_egl2d_available(void) { return 0; }
int32_t fhre_egl2d_init(uint16_t width, uint16_t height) {
    (void)width;
    (void)height;
    return FHRE_EGL_UNAVAILABLE;
}
int32_t fhre_egl2d_shutdown(void) { return FHRE_EGL_OK; }
int32_t fhre_egl2d_submit(const FhreEglPacket *packet, FhreEglFence *fence) {
    (void)packet;
    if (fence != NULL) {
        fence->token = 0;
    }
    return FHRE_EGL_UNAVAILABLE;
}
int32_t fhre_egl2d_pending_bounds(FhreEglRect *out) {
    if (out != NULL) {
        out->x = 0;
        out->y = 0;
        out->w = 0;
        out->h = 0;
    }
    return FHRE_EGL_UNAVAILABLE;
}
int32_t fhre_egl2d_flush(void) { return FHRE_EGL_UNAVAILABLE; }
int32_t fhre_egl2d_stats(FhreEglWorkerStats *out) {
    if (out != NULL) {
        memset(out, 0, sizeof(*out));
    }
    return FHRE_EGL_UNAVAILABLE;
}
int32_t fhre_egl2d_last_error(void) { return FHRE_EGL_UNAVAILABLE; }

#else

#include <EGL/egl.h>
#include <GL/gl.h>
#include <pthread.h>

#define FHRE_EGL_MAX_PENDING_JOBS 16

typedef struct FhreEglJob {
    uint32_t token;
    FhreEglFramebuffer framebuffer;
    FhreEglRect bounds;
    FhreEglCommand *commands;
    uintptr_t command_count;
    uint8_t *snapshot_rgba;
    uint8_t *readback_rgba;
    int32_t status;
    struct FhreEglJob *next;
} FhreEglJob;

typedef struct {
    pthread_mutex_t mutex;
    pthread_cond_t cond;
    pthread_t thread;
    int thread_started;
    int stop;
    int active;
    uint32_t next_token;
    uint32_t pending_count;
    FhreEglRect pending_bounds;
    FhreEglJob *queue_head;
    FhreEglJob *queue_tail;
    FhreEglJob *done_head;
    FhreEglJob *done_tail;
    EGLDisplay display;
    EGLContext context;
    EGLSurface surface;
    EGLConfig config;
    uint16_t target_width;
    uint16_t target_height;
    int egl_ready;
    int32_t last_error;
    FhreEglWorkerStats stats;
} FhreEglWorker;

static FhreEglWorker g_worker = {
    PTHREAD_MUTEX_INITIALIZER,
    PTHREAD_COND_INITIALIZER,
    0,
    0,
    0,
    0,
    1,
    0,
    {0, 0, 0, 0},
    NULL,
    NULL,
    NULL,
    NULL,
    EGL_NO_DISPLAY,
    EGL_NO_CONTEXT,
    EGL_NO_SURFACE,
    0,
    0,
    0,
    0,
    FHRE_EGL_OK,
    {0, 0, 0, 0, 0, 0, 0, 0},
};

static int32_t set_error(int32_t error) {
    g_worker.last_error = error;
    return error;
}

static int rect_empty(FhreEglRect rect) {
    return rect.w == 0 || rect.h == 0;
}

static int32_t rect_right(FhreEglRect rect) {
    return rect.x + (int32_t)rect.w;
}

static int32_t rect_bottom(FhreEglRect rect) {
    return rect.y + (int32_t)rect.h;
}

static FhreEglRect rect_from_edges(int32_t x0, int32_t y0, int32_t x1, int32_t y1) {
    FhreEglRect rect;
    if (x1 <= x0 || y1 <= y0) {
        rect.x = 0;
        rect.y = 0;
        rect.w = 0;
        rect.h = 0;
        return rect;
    }
    rect.x = x0;
    rect.y = y0;
    rect.w = (uint16_t)((x1 - x0) > UINT16_MAX ? UINT16_MAX : (x1 - x0));
    rect.h = (uint16_t)((y1 - y0) > UINT16_MAX ? UINT16_MAX : (y1 - y0));
    return rect;
}

static FhreEglRect rect_clip(FhreEglRect rect, FhreEglRect bounds) {
    int32_t x0 = rect.x > bounds.x ? rect.x : bounds.x;
    int32_t y0 = rect.y > bounds.y ? rect.y : bounds.y;
    int32_t x1 = rect_right(rect) < rect_right(bounds) ? rect_right(rect) : rect_right(bounds);
    int32_t y1 =
        rect_bottom(rect) < rect_bottom(bounds) ? rect_bottom(rect) : rect_bottom(bounds);
    return rect_from_edges(x0, y0, x1, y1);
}

static FhreEglRect rect_union(FhreEglRect a, FhreEglRect b) {
    if (rect_empty(a)) {
        return b;
    }
    if (rect_empty(b)) {
        return a;
    }
    int32_t x0 = a.x < b.x ? a.x : b.x;
    int32_t y0 = a.y < b.y ? a.y : b.y;
    int32_t x1 = rect_right(a) > rect_right(b) ? rect_right(a) : rect_right(b);
    int32_t y1 = rect_bottom(a) > rect_bottom(b) ? rect_bottom(a) : rect_bottom(b);
    return rect_from_edges(x0, y0, x1, y1);
}

static uintptr_t bytes_per_pixel(uint8_t format, uint8_t bpp) {
    switch (format) {
    case FHRE_EGL_FORMAT_RGB565:
        return 2;
    case FHRE_EGL_FORMAT_RGB888:
        return 3;
    case FHRE_EGL_FORMAT_RGBA8888:
        return 4;
    case FHRE_EGL_FORMAT_A8:
        return 1;
    default:
        return (uintptr_t)(bpp / 8);
    }
}

static int framebuffer_valid(FhreEglFramebuffer framebuffer) {
    uintptr_t bpp = bytes_per_pixel(framebuffer.format, framebuffer.bpp);
    uintptr_t min_len = 0;
    if (framebuffer.data == NULL || framebuffer.width == 0 || framebuffer.height == 0 ||
        framebuffer.stride == 0 || bpp == 0) {
        return 0;
    }
    min_len = framebuffer.stride * (uintptr_t)framebuffer.height;
    return framebuffer.len >= min_len;
}

static int source_valid(FhreEglImageSource source) {
    uintptr_t bpp = bytes_per_pixel(source.format, 0);
    uintptr_t min_len = 0;
    if (source.data == NULL || source.width == 0 || source.height == 0 || source.stride == 0 ||
        bpp == 0) {
        return 0;
    }
    min_len = source.stride * (uintptr_t)source.height;
    return source.len >= min_len;
}

static FhreEglColor read_rgb565(const uint8_t *ptr) {
    uint16_t raw = (uint16_t)ptr[0] | ((uint16_t)ptr[1] << 8);
    FhreEglColor color;
    color.r = (uint8_t)((((raw >> 11) & 0x1f) * 255) / 31);
    color.g = (uint8_t)((((raw >> 5) & 0x3f) * 255) / 63);
    color.b = (uint8_t)(((raw & 0x1f) * 255) / 31);
    color.a = 255;
    return color;
}

static FhreEglColor read_pixel(FhreEglFramebuffer framebuffer, int32_t x, int32_t y) {
    FhreEglColor color = {0, 0, 0, 0};
    uintptr_t bpp = bytes_per_pixel(framebuffer.format, framebuffer.bpp);
    const uint8_t *ptr = (const uint8_t *)framebuffer.data +
                         (uintptr_t)y * framebuffer.stride + (uintptr_t)x * bpp;
    switch (framebuffer.format) {
    case FHRE_EGL_FORMAT_RGB565:
        return read_rgb565(ptr);
    case FHRE_EGL_FORMAT_RGB888:
        color.r = ptr[0];
        color.g = ptr[1];
        color.b = ptr[2];
        color.a = 255;
        return color;
    case FHRE_EGL_FORMAT_RGBA8888:
        color.r = ptr[0];
        color.g = ptr[1];
        color.b = ptr[2];
        color.a = ptr[3];
        return color;
    default:
        return color;
    }
}

static void write_pixel(FhreEglFramebuffer framebuffer, int32_t x, int32_t y, FhreEglColor color) {
    uintptr_t bpp = bytes_per_pixel(framebuffer.format, framebuffer.bpp);
    uint8_t *ptr =
        (uint8_t *)framebuffer.data + (uintptr_t)y * framebuffer.stride + (uintptr_t)x * bpp;
    switch (framebuffer.format) {
    case FHRE_EGL_FORMAT_RGB565: {
        uint16_t r = ((uint16_t)color.r >> 3) & 0x1f;
        uint16_t g = ((uint16_t)color.g >> 2) & 0x3f;
        uint16_t b = ((uint16_t)color.b >> 3) & 0x1f;
        uint16_t raw = (uint16_t)((r << 11) | (g << 5) | b);
        ptr[0] = (uint8_t)(raw & 0xff);
        ptr[1] = (uint8_t)(raw >> 8);
        break;
    }
    case FHRE_EGL_FORMAT_RGB888:
        ptr[0] = color.r;
        ptr[1] = color.g;
        ptr[2] = color.b;
        break;
    case FHRE_EGL_FORMAT_RGBA8888:
        ptr[0] = color.r;
        ptr[1] = color.g;
        ptr[2] = color.b;
        ptr[3] = color.a;
        break;
    default:
        break;
    }
}

static uint8_t *copy_framebuffer_region_rgba(FhreEglFramebuffer framebuffer, FhreEglRect bounds) {
    uintptr_t pixel_count = (uintptr_t)bounds.w * (uintptr_t)bounds.h;
    uint8_t *rgba = (uint8_t *)malloc(pixel_count * 4);
    if (rgba == NULL) {
        return NULL;
    }
    for (uint16_t y = 0; y < bounds.h; y++) {
        for (uint16_t x = 0; x < bounds.w; x++) {
            FhreEglColor color = read_pixel(framebuffer, bounds.x + (int32_t)x, bounds.y + (int32_t)y);
            uintptr_t out = ((uintptr_t)y * bounds.w + x) * 4;
            rgba[out + 0] = color.r;
            rgba[out + 1] = color.g;
            rgba[out + 2] = color.b;
            rgba[out + 3] = color.a;
        }
    }
    return rgba;
}

static void write_framebuffer_region_rgba(
    FhreEglFramebuffer framebuffer,
    FhreEglRect bounds,
    const uint8_t *rgba) {
    for (uint16_t y = 0; y < bounds.h; y++) {
        for (uint16_t x = 0; x < bounds.w; x++) {
            uintptr_t in = ((uintptr_t)y * bounds.w + x) * 4;
            FhreEglColor color = {rgba[in + 0], rgba[in + 1], rgba[in + 2], rgba[in + 3]};
            write_pixel(framebuffer, bounds.x + (int32_t)x, bounds.y + (int32_t)y, color);
        }
    }
}

static void free_job(FhreEglJob *job) {
    if (job == NULL) {
        return;
    }
    if (job->commands != NULL) {
        for (uintptr_t i = 0; i < job->command_count; i++) {
            if (job->commands[i].kind == FHRE_EGL_COMMAND_IMAGE) {
                free((void *)job->commands[i].source.data);
            }
        }
        free(job->commands);
    }
    free(job->snapshot_rgba);
    free(job->readback_rgba);
    free(job);
}

static int copy_command(FhreEglCommand *dst, const FhreEglCommand *src) {
    *dst = *src;
    if (dst->kind == FHRE_EGL_COMMAND_IMAGE) {
        if (!source_valid(dst->source) || dst->fit != FHRE_EGL_FIT_STRETCH) {
            return 0;
        }
        void *copy = malloc(dst->source.len);
        if (copy == NULL) {
            return 0;
        }
        memcpy(copy, dst->source.data, dst->source.len);
        dst->source.data = copy;
    }
    return 1;
}

static FhreEglJob *copy_packet(const FhreEglPacket *packet, uint32_t token) {
    FhreEglRect screen = {0, 0, packet->framebuffer.width, packet->framebuffer.height};
    FhreEglRect bounds = rect_clip(packet->run_bounds, screen);
    FhreEglJob *job = NULL;
    if (!framebuffer_valid(packet->framebuffer) || packet->commands == NULL ||
        packet->command_count == 0 || rect_empty(bounds)) {
        return NULL;
    }
    job = (FhreEglJob *)calloc(1, sizeof(*job));
    if (job == NULL) {
        return NULL;
    }
    job->token = token;
    job->framebuffer = packet->framebuffer;
    job->bounds = bounds;
    job->command_count = packet->command_count;
    job->commands = (FhreEglCommand *)calloc(packet->command_count, sizeof(FhreEglCommand));
    if (job->commands == NULL) {
        free_job(job);
        return NULL;
    }
    for (uintptr_t i = 0; i < packet->command_count; i++) {
        if (!copy_command(&job->commands[i], &packet->commands[i])) {
            free_job(job);
            return NULL;
        }
    }
    job->snapshot_rgba = copy_framebuffer_region_rgba(packet->framebuffer, bounds);
    if (job->snapshot_rgba == NULL) {
        free_job(job);
        return NULL;
    }
    job->readback_rgba = (uint8_t *)malloc((uintptr_t)bounds.w * (uintptr_t)bounds.h * 4);
    if (job->readback_rgba == NULL) {
        free_job(job);
        return NULL;
    }
    job->status = FHRE_EGL_OK;
    return job;
}

static void destroy_egl(void) {
    if (g_worker.display != EGL_NO_DISPLAY) {
        eglMakeCurrent(g_worker.display, EGL_NO_SURFACE, EGL_NO_SURFACE, EGL_NO_CONTEXT);
        if (g_worker.context != EGL_NO_CONTEXT) {
            eglDestroyContext(g_worker.display, g_worker.context);
        }
        if (g_worker.surface != EGL_NO_SURFACE) {
            eglDestroySurface(g_worker.display, g_worker.surface);
        }
        eglTerminate(g_worker.display);
    }
    g_worker.display = EGL_NO_DISPLAY;
    g_worker.context = EGL_NO_CONTEXT;
    g_worker.surface = EGL_NO_SURFACE;
    g_worker.config = 0;
    g_worker.target_width = 0;
    g_worker.target_height = 0;
    g_worker.egl_ready = 0;
}

static int ensure_egl(uint16_t width, uint16_t height) {
    static const EGLint config_attribs[] = {
        EGL_SURFACE_TYPE, EGL_PBUFFER_BIT,
        EGL_RENDERABLE_TYPE, EGL_OPENGL_BIT,
        EGL_RED_SIZE, 8,
        EGL_GREEN_SIZE, 8,
        EGL_BLUE_SIZE, 8,
        EGL_ALPHA_SIZE, 8,
        EGL_NONE,
    };
    EGLint pbuffer_attribs[] = {
        EGL_WIDTH, width,
        EGL_HEIGHT, height,
        EGL_NONE,
    };
    EGLint count = 0;
    if (g_worker.egl_ready && g_worker.target_width == width && g_worker.target_height == height) {
        return FHRE_EGL_OK;
    }
    destroy_egl();
    if (!eglBindAPI(EGL_OPENGL_API)) {
        return set_error(FHRE_EGL_UNAVAILABLE);
    }
    g_worker.display = eglGetDisplay(EGL_DEFAULT_DISPLAY);
    if (g_worker.display == EGL_NO_DISPLAY || !eglInitialize(g_worker.display, NULL, NULL)) {
        destroy_egl();
        return set_error(FHRE_EGL_UNAVAILABLE);
    }
    if (!eglChooseConfig(g_worker.display, config_attribs, &g_worker.config, 1, &count) ||
        count <= 0) {
        destroy_egl();
        return set_error(FHRE_EGL_UNAVAILABLE);
    }
    g_worker.surface = eglCreatePbufferSurface(g_worker.display, g_worker.config, pbuffer_attribs);
    if (g_worker.surface == EGL_NO_SURFACE) {
        destroy_egl();
        return set_error(FHRE_EGL_UNAVAILABLE);
    }
    g_worker.context =
        eglCreateContext(g_worker.display, g_worker.config, EGL_NO_CONTEXT, NULL);
    if (g_worker.context == EGL_NO_CONTEXT) {
        destroy_egl();
        return set_error(FHRE_EGL_UNAVAILABLE);
    }
    if (!eglMakeCurrent(g_worker.display, g_worker.surface, g_worker.surface, g_worker.context)) {
        destroy_egl();
        return set_error(FHRE_EGL_UNAVAILABLE);
    }
    g_worker.target_width = width;
    g_worker.target_height = height;
    g_worker.egl_ready = 1;
    return FHRE_EGL_OK;
}

static void set_scissor(FhreEglRect clip, uint16_t fb_height) {
    if (rect_empty(clip)) {
        glDisable(GL_SCISSOR_TEST);
        return;
    }
    glEnable(GL_SCISSOR_TEST);
    glScissor(
        clip.x,
        (GLint)fb_height - (clip.y + (GLint)clip.h),
        clip.w,
        clip.h);
}

static void draw_color_quad(FhreEglRect rect, FhreEglColor color) {
    glDisable(GL_TEXTURE_2D);
    if (color.a < 255) {
        glEnable(GL_BLEND);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
    } else {
        glDisable(GL_BLEND);
    }
    glColor4f(
        (GLfloat)color.r / 255.0f,
        (GLfloat)color.g / 255.0f,
        (GLfloat)color.b / 255.0f,
        (GLfloat)color.a / 255.0f);
    glBegin(GL_QUADS);
    glVertex2f((GLfloat)rect.x, (GLfloat)rect.y);
    glVertex2f((GLfloat)rect_right(rect), (GLfloat)rect.y);
    glVertex2f((GLfloat)rect_right(rect), (GLfloat)rect_bottom(rect));
    glVertex2f((GLfloat)rect.x, (GLfloat)rect_bottom(rect));
    glEnd();
}

static void draw_rgba_texture(
    FhreEglRect rect,
    const uint8_t *rgba,
    uint16_t width,
    uint16_t height,
    uint8_t opacity,
    int has_tint,
    FhreEglColor tint) {
    GLuint texture = 0;
    glGenTextures(1, &texture);
    glBindTexture(GL_TEXTURE_2D, texture);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP);
    glTexImage2D(
        GL_TEXTURE_2D,
        0,
        GL_RGBA,
        width,
        height,
        0,
        GL_RGBA,
        GL_UNSIGNED_BYTE,
        rgba);
    glEnable(GL_TEXTURE_2D);
    glEnable(GL_BLEND);
    glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
    if (has_tint) {
        glTexEnvi(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE, GL_MODULATE);
        glColor4f(
            (GLfloat)tint.r / 255.0f,
            (GLfloat)tint.g / 255.0f,
            (GLfloat)tint.b / 255.0f,
            ((GLfloat)tint.a / 255.0f) * ((GLfloat)opacity / 255.0f));
    } else {
        glTexEnvi(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE, GL_MODULATE);
        glColor4f(1.0f, 1.0f, 1.0f, (GLfloat)opacity / 255.0f);
    }
    glBegin(GL_QUADS);
    glTexCoord2f(0.0f, 1.0f);
    glVertex2f((GLfloat)rect.x, (GLfloat)rect.y);
    glTexCoord2f(1.0f, 1.0f);
    glVertex2f((GLfloat)rect_right(rect), (GLfloat)rect.y);
    glTexCoord2f(1.0f, 0.0f);
    glVertex2f((GLfloat)rect_right(rect), (GLfloat)rect_bottom(rect));
    glTexCoord2f(0.0f, 0.0f);
    glVertex2f((GLfloat)rect.x, (GLfloat)rect_bottom(rect));
    glEnd();
    glDisable(GL_TEXTURE_2D);
    glDeleteTextures(1, &texture);
}

static uint8_t *copy_source_rgba(FhreEglImageSource source, uint8_t opacity) {
    uintptr_t pixels = (uintptr_t)source.width * (uintptr_t)source.height;
    uint8_t *rgba = (uint8_t *)malloc(pixels * 4);
    if (rgba == NULL) {
        return NULL;
    }
    for (uint16_t y = 0; y < source.height; y++) {
        for (uint16_t x = 0; x < source.width; x++) {
            const uint8_t *ptr =
                (const uint8_t *)source.data + (uintptr_t)y * source.stride +
                (uintptr_t)x * bytes_per_pixel(source.format, 0);
            FhreEglColor color = {0, 0, 0, opacity};
            switch (source.format) {
            case FHRE_EGL_FORMAT_RGB565:
                color = read_rgb565(ptr);
                color.a = opacity;
                break;
            case FHRE_EGL_FORMAT_RGB888:
                color.r = ptr[0];
                color.g = ptr[1];
                color.b = ptr[2];
                color.a = opacity;
                break;
            case FHRE_EGL_FORMAT_RGBA8888:
                color.r = ptr[0];
                color.g = ptr[1];
                color.b = ptr[2];
                color.a = (uint8_t)(((uint16_t)ptr[3] * (uint16_t)opacity) / 255);
                break;
            case FHRE_EGL_FORMAT_A8:
                color.r = 255;
                color.g = 255;
                color.b = 255;
                color.a = (uint8_t)(((uint16_t)ptr[0] * (uint16_t)opacity) / 255);
                break;
            default:
                break;
            }
            uintptr_t out = ((uintptr_t)y * source.width + x) * 4;
            rgba[out + 0] = color.r;
            rgba[out + 1] = color.g;
            rgba[out + 2] = color.b;
            rgba[out + 3] = color.a;
        }
    }
    return rgba;
}

static int render_job(FhreEglJob *job) {
    FhreEglRect current_clip = {0, 0, job->framebuffer.width, job->framebuffer.height};
    FhreEglRect screen = current_clip;
    int status = ensure_egl(job->framebuffer.width, job->framebuffer.height);
    if (status != FHRE_EGL_OK) {
        return status;
    }

    glViewport(0, 0, job->framebuffer.width, job->framebuffer.height);
    glMatrixMode(GL_PROJECTION);
    glLoadIdentity();
    glOrtho(0.0, (GLdouble)job->framebuffer.width, (GLdouble)job->framebuffer.height, 0.0, -1.0, 1.0);
    glMatrixMode(GL_MODELVIEW);
    glLoadIdentity();
    glDisable(GL_DEPTH_TEST);
    glDisable(GL_CULL_FACE);
    glDisable(GL_DITHER);
    glDisable(GL_SCISSOR_TEST);
    glDisable(GL_BLEND);
    glClearColor(0.0f, 0.0f, 0.0f, 0.0f);
    glClear(GL_COLOR_BUFFER_BIT);

    set_scissor(job->bounds, job->framebuffer.height);
    {
        FhreEglColor no_tint = {255, 255, 255, 255};
        draw_rgba_texture(
            job->bounds,
            job->snapshot_rgba,
            job->bounds.w,
            job->bounds.h,
            255,
            0,
            no_tint);
    }

    for (uintptr_t i = 0; i < job->command_count; i++) {
        FhreEglCommand *command = &job->commands[i];
        if (command->kind == FHRE_EGL_COMMAND_SET_CLIP) {
            current_clip = command->has_clip ? rect_clip(command->clip, screen) : screen;
            continue;
        }

        FhreEglRect clip = command->has_clip ? rect_clip(command->clip, screen) : current_clip;
        clip = rect_clip(clip, job->bounds);
        if (rect_empty(clip)) {
            continue;
        }
        set_scissor(clip, job->framebuffer.height);

        if (command->kind == FHRE_EGL_COMMAND_FILL) {
            FhreEglRect rect = rect_clip(command->rect, job->bounds);
            if (!rect_empty(rect)) {
                if (command->color.a == 255) {
                    FhreEglRect clear_rect = rect_clip(rect, clip);
                    if (!rect_empty(clear_rect)) {
                        set_scissor(clear_rect, job->framebuffer.height);
                        glDisable(GL_TEXTURE_2D);
                        glDisable(GL_BLEND);
                        glClearColor(
                            (GLfloat)command->color.r / 255.0f,
                            (GLfloat)command->color.g / 255.0f,
                            (GLfloat)command->color.b / 255.0f,
                            1.0f);
                        glClear(GL_COLOR_BUFFER_BIT);
                    }
                } else {
                    draw_color_quad(rect, command->color);
                }
            }
        } else if (command->kind == FHRE_EGL_COMMAND_IMAGE) {
            uint8_t *source_rgba = NULL;
            if (command->fit != FHRE_EGL_FIT_STRETCH || !source_valid(command->source)) {
                return set_error(FHRE_EGL_UNSUPPORTED);
            }
            source_rgba = copy_source_rgba(command->source, command->opacity);
            if (source_rgba == NULL) {
                return set_error(FHRE_EGL_SUBMIT_FAILED);
            }
            draw_rgba_texture(
                command->rect,
                source_rgba,
                command->source.width,
                command->source.height,
                255,
                command->has_tint,
                command->tint);
            free(source_rgba);
        } else {
            return set_error(FHRE_EGL_UNSUPPORTED);
        }
    }

    if (glGetError() != GL_NO_ERROR) {
        return set_error(FHRE_EGL_FENCE_FAILED);
    }
    glFinish();
    {
        uintptr_t bytes = (uintptr_t)job->bounds.w * (uintptr_t)job->bounds.h * 4;
        uint8_t *gl_pixels = (uint8_t *)malloc(bytes);
        if (gl_pixels == NULL) {
            return set_error(FHRE_EGL_SUBMIT_FAILED);
        }
        glReadPixels(
            job->bounds.x,
            (GLint)job->framebuffer.height - (job->bounds.y + (GLint)job->bounds.h),
            job->bounds.w,
            job->bounds.h,
            GL_RGBA,
            GL_UNSIGNED_BYTE,
            gl_pixels);
        for (uint16_t y = 0; y < job->bounds.h; y++) {
            uint16_t src_y = (uint16_t)(job->bounds.h - 1 - y);
            memcpy(
                job->readback_rgba + (uintptr_t)y * job->bounds.w * 4,
                gl_pixels + (uintptr_t)src_y * job->bounds.w * 4,
                (uintptr_t)job->bounds.w * 4);
        }
        free(gl_pixels);
    }
    return FHRE_EGL_OK;
}

static void enqueue_done(FhreEglJob *job) {
    job->next = NULL;
    if (g_worker.done_tail != NULL) {
        g_worker.done_tail->next = job;
    } else {
        g_worker.done_head = job;
    }
    g_worker.done_tail = job;
}

static FhreEglJob *dequeue_job(void) {
    FhreEglJob *job = g_worker.queue_head;
    if (job != NULL) {
        g_worker.queue_head = job->next;
        if (g_worker.queue_head == NULL) {
            g_worker.queue_tail = NULL;
        }
        job->next = NULL;
    }
    return job;
}

static void *worker_main(void *arg) {
    (void)arg;
    for (;;) {
        pthread_mutex_lock(&g_worker.mutex);
        while (!g_worker.stop && g_worker.queue_head == NULL) {
            pthread_cond_wait(&g_worker.cond, &g_worker.mutex);
        }
        if (g_worker.stop && g_worker.queue_head == NULL) {
            pthread_mutex_unlock(&g_worker.mutex);
            break;
        }
        FhreEglJob *job = dequeue_job();
        g_worker.active = 1;
        pthread_mutex_unlock(&g_worker.mutex);

        job->status = render_job(job);

        pthread_mutex_lock(&g_worker.mutex);
        if (job->status == FHRE_EGL_OK) {
            g_worker.stats.submitted = g_worker.stats.submitted + 1;
            g_worker.stats.completed = g_worker.stats.completed + 1;
        } else {
            g_worker.stats.fence_failures = g_worker.stats.fence_failures + 1;
        }
        enqueue_done(job);
        g_worker.active = 0;
        pthread_cond_broadcast(&g_worker.cond);
        pthread_mutex_unlock(&g_worker.mutex);
    }
    destroy_egl();
    return NULL;
}

static int ensure_thread(void) {
    if (g_worker.thread_started) {
        return FHRE_EGL_OK;
    }
    g_worker.stop = 0;
    if (pthread_create(&g_worker.thread, NULL, worker_main, NULL) != 0) {
        return set_error(FHRE_EGL_UNAVAILABLE);
    }
    g_worker.thread_started = 1;
    return FHRE_EGL_OK;
}

int32_t fhre_egl2d_available(void) { return 1; }

int32_t fhre_egl2d_init(uint16_t width, uint16_t height) {
    (void)width;
    (void)height;
    pthread_mutex_lock(&g_worker.mutex);
    int status = ensure_thread();
    pthread_mutex_unlock(&g_worker.mutex);
    return status;
}

int32_t fhre_egl2d_shutdown(void) {
    pthread_mutex_lock(&g_worker.mutex);
    if (g_worker.thread_started) {
        g_worker.stop = 1;
        pthread_cond_broadcast(&g_worker.cond);
        pthread_mutex_unlock(&g_worker.mutex);
        pthread_join(g_worker.thread, NULL);
        pthread_mutex_lock(&g_worker.mutex);
        g_worker.thread_started = 0;
    }
    while (g_worker.queue_head != NULL) {
        FhreEglJob *job = dequeue_job();
        free_job(job);
    }
    while (g_worker.done_head != NULL) {
        FhreEglJob *job = g_worker.done_head;
        g_worker.done_head = job->next;
        free_job(job);
    }
    g_worker.done_tail = NULL;
    g_worker.pending_count = 0;
    g_worker.pending_bounds = (FhreEglRect){0, 0, 0, 0};
    pthread_mutex_unlock(&g_worker.mutex);
    return FHRE_EGL_OK;
}

int32_t fhre_egl2d_submit(const FhreEglPacket *packet, FhreEglFence *fence) {
    FhreEglJob *job = NULL;
    uint32_t token = 0;
    int32_t status = FHRE_EGL_OK;
    if (packet == NULL || fence == NULL) {
        return set_error(FHRE_EGL_SUBMIT_FAILED);
    }
    pthread_mutex_lock(&g_worker.mutex);
    if (g_worker.pending_count >= FHRE_EGL_MAX_PENDING_JOBS) {
        g_worker.stats.overflows = g_worker.stats.overflows + 1;
        pthread_mutex_unlock(&g_worker.mutex);
        return set_error(FHRE_EGL_OVERFLOW);
    }
    status = ensure_thread();
    if (status != FHRE_EGL_OK) {
        pthread_mutex_unlock(&g_worker.mutex);
        return status;
    }
    token = g_worker.next_token++;
    if (g_worker.next_token == 0) {
        g_worker.next_token = 1;
    }
    pthread_mutex_unlock(&g_worker.mutex);

    job = copy_packet(packet, token);
    if (job == NULL) {
        g_worker.stats.submit_failures = g_worker.stats.submit_failures + 1;
        return set_error(FHRE_EGL_SUBMIT_FAILED);
    }

    pthread_mutex_lock(&g_worker.mutex);
    if (g_worker.queue_tail != NULL) {
        g_worker.queue_tail->next = job;
    } else {
        g_worker.queue_head = job;
    }
    g_worker.queue_tail = job;
    g_worker.pending_count = g_worker.pending_count + 1;
    g_worker.pending_bounds = rect_union(g_worker.pending_bounds, job->bounds);
    g_worker.stats.queued = g_worker.stats.queued + 1;
    fence->token = token;
    pthread_cond_signal(&g_worker.cond);
    pthread_mutex_unlock(&g_worker.mutex);
    return FHRE_EGL_OK;
}

int32_t fhre_egl2d_pending_bounds(FhreEglRect *out) {
    if (out == NULL) {
        return FHRE_EGL_SUBMIT_FAILED;
    }
    pthread_mutex_lock(&g_worker.mutex);
    *out = g_worker.pending_bounds;
    pthread_mutex_unlock(&g_worker.mutex);
    return FHRE_EGL_OK;
}

int32_t fhre_egl2d_flush(void) {
    FhreEglJob *done = NULL;
    int32_t status = FHRE_EGL_OK;
    pthread_mutex_lock(&g_worker.mutex);
    while (g_worker.queue_head != NULL || g_worker.active) {
        pthread_cond_wait(&g_worker.cond, &g_worker.mutex);
    }
    done = g_worker.done_head;
    g_worker.done_head = NULL;
    g_worker.done_tail = NULL;
    g_worker.pending_count = 0;
    g_worker.pending_bounds = (FhreEglRect){0, 0, 0, 0};
    pthread_mutex_unlock(&g_worker.mutex);

    for (FhreEglJob *job = done; job != NULL; job = job->next) {
        if (job->status != FHRE_EGL_OK) {
            status = job->status;
            break;
        }
    }

    if (status == FHRE_EGL_OK) {
        for (FhreEglJob *job = done; job != NULL; job = job->next) {
            write_framebuffer_region_rgba(job->framebuffer, job->bounds, job->readback_rgba);
            pthread_mutex_lock(&g_worker.mutex);
            g_worker.stats.readbacks = g_worker.stats.readbacks + 1;
            pthread_mutex_unlock(&g_worker.mutex);
        }
    }

    while (done != NULL) {
        FhreEglJob *next = done->next;
        free_job(done);
        done = next;
    }
    if (status != FHRE_EGL_OK) {
        return set_error(status);
    }
    return FHRE_EGL_OK;
}

int32_t fhre_egl2d_stats(FhreEglWorkerStats *out) {
    if (out == NULL) {
        return FHRE_EGL_SUBMIT_FAILED;
    }
    pthread_mutex_lock(&g_worker.mutex);
    *out = g_worker.stats;
    pthread_mutex_unlock(&g_worker.mutex);
    return FHRE_EGL_OK;
}

int32_t fhre_egl2d_last_error(void) { return g_worker.last_error; }

#endif
