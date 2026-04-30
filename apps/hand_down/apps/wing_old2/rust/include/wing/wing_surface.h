/****************************************************************************
 * apps/wing/rust/include/wing/wing_surface.h
 *
 * Minimal Wing surface ABI for NuttX builtin applications.
 ****************************************************************************/

#ifndef __APPS_WING_RUST_INCLUDE_WING_WING_SURFACE_H
#define __APPS_WING_RUST_INCLUDE_WING_WING_SURFACE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C"
{
#endif

#define WING_SURFACE_FORMAT_RGB565 1
#define WING_SURFACE_FORMAT_ARGB8888 2

#define WING_SURFACE_TRANSPORT_SHARED_MEMORY 1
#define WING_SURFACE_TRANSPORT_STREAM_TEXTURE 2

#define WING_SURFACE_INPUT_NONE 0
#define WING_SURFACE_INPUT_POINTER 1
#define WING_SURFACE_INPUT_POINTER_KEYBOARD 2

#define WING_SURFACE_POINTER_MOVE 1
#define WING_SURFACE_POINTER_DOWN 2
#define WING_SURFACE_POINTER_UP 3
#define WING_SURFACE_POINTER_CANCEL 4

#define WING_SURFACE_ABI_VERSION 1

#define WING_SURFACE_CAP_FORMAT_RGB565    (1u << 0)
#define WING_SURFACE_CAP_FORMAT_ARGB8888  (1u << 1)

#define WING_SURFACE_CAP_TRANSPORT_SHARED_MEMORY  (1u << 0)
#define WING_SURFACE_CAP_TRANSPORT_STREAM_TEXTURE (1u << 1)

#define WING_SURFACE_CAP_INPUT_NONE             (1u << 0)
#define WING_SURFACE_CAP_INPUT_POINTER          (1u << 1)
#define WING_SURFACE_CAP_INPUT_POINTER_KEYBOARD (1u << 2)

struct wing_surface_capabilities
{
  uint32_t abi_version;
  uint32_t format_mask;
  uint32_t transport_mask;
  uint32_t input_mask;
  uint32_t max_surfaces;
  uint32_t max_width;
  uint32_t max_height;
  uint32_t max_frame_bytes;
};

struct wing_surface_frame_info
{
  uint32_t handle;
  uint32_t token;
  void *pixels;
  size_t bytes;
  int width;
  int height;
  int stride;
  int format;
  int buffers;
};

struct wing_surface_input_message
{
  uint32_t handle;
  uint32_t token;
  int16_t x;
  int16_t y;
  uint8_t event;
  uint8_t buttons;
  uint16_t reserved;
};

int wing_surface_query_capabilities(struct wing_surface_capabilities *caps);
int wing_surface_frame_resolve(uint32_t handle, uint32_t token,
                               struct wing_surface_frame_info *info);
int wing_surface_frame_unregister(uint32_t handle, uint32_t token);
int wing_task_close(int pid, uint32_t frame, uint32_t token);
int wing_surface_dirty_sender_open(void);
int wing_surface_dirty_send(int queue, uint32_t handle, uint32_t token,
                            int16_t x, int16_t y, uint16_t w, uint16_t h);
int wing_surface_dirty_close(int queue);
int wing_surface_input_receiver_open(void);
int wing_surface_input_receive(int queue,
                               struct wing_surface_input_message *message);
int wing_surface_input_close(int queue);
int wing_terminal_supported(void);
int wing_terminal_status(void);
int wing_terminal_open(void);
int wing_terminal_read(uint8_t *buffer, size_t buflen);
int wing_terminal_write(const uint8_t *buffer, size_t buflen);
int wing_terminal_close(void);

#ifdef __cplusplus
}
#endif

#endif /* __APPS_WING_RUST_INCLUDE_WING_WING_SURFACE_H */
