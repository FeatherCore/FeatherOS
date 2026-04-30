/****************************************************************************
 * apps/wing/surface_demo/surface_demo_main.c
 *
 * Minimal independent task that renders into a Wing managed surface.
 ****************************************************************************/

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#include <wing/wing_surface.h>

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct wing_surface_bootstrap
{
  int id;
  int width;
  int height;
  int stride;
  int format;
  int buffers;
  int transport;
  int input;
  uint32_t token;
  uint32_t frame;
  uint32_t dirty;
  uint32_t input_handle;
};

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static int parse_surface_arg(FAR const char *arg,
                             FAR struct wing_surface_bootstrap *surface)
{
  int matched;

  if (arg == NULL || surface == NULL)
    {
      return -1;
    }

  if (strncmp(arg, "--wing-surface=", 15) != 0)
    {
      return -1;
    }

  matched = sscanf(arg + 15,
                   "%d:%d:%d:%d:%d:%d:%d:%d:%" SCNu32 ":%" SCNu32
                   ":%" SCNu32 ":%" SCNu32,
                   &surface->id,
                   &surface->width,
                   &surface->height,
                   &surface->stride,
                   &surface->format,
                   &surface->buffers,
                   &surface->transport,
                   &surface->input,
                   &surface->token,
                   &surface->frame,
                   &surface->dirty,
                   &surface->input_handle);

  return matched == 12 ? 0 : -1;
}

static uint16_t rgb565(uint8_t r, uint8_t g, uint8_t b)
{
  return (uint16_t)(((uint16_t)(r & 0xf8) << 8) |
                    ((uint16_t)(g & 0xfc) << 3) |
                    ((uint16_t)b >> 3));
}

static uint8_t triangle_wave(int value, int period)
{
  int phase = value % period;
  int half = period / 2;

  if (phase < 0)
    {
      phase += period;
    }

  if (phase > half)
    {
      phase = period - phase;
    }

  return (uint8_t)((phase * 255) / half);
}

static void draw_demo_frame(FAR uint16_t *pixels,
                            int width,
                            int height,
                            int stride_pixels,
                            int tick)
{
  int cx = width / 2;
  int cy = height / 2;
  int radius = width < height ? width / 4 : height / 4;
  int ring = radius + height / 12;
  int x;
  int y;

  for (y = 0; y < height; y++)
    {
      FAR uint16_t *row = pixels + y * stride_pixels;

      for (x = 0; x < width; x++)
        {
          int dx = x - cx;
          int dy = y - cy;
          int d2 = dx * dx + dy * dy;
          uint8_t base = triangle_wave(x + tick * 2, width * 2);
          uint8_t glow = triangle_wave(y - tick, height * 2);
          uint8_t r = (uint8_t)(18 + base / 5);
          uint8_t g = (uint8_t)(24 + glow / 4);
          uint8_t b = (uint8_t)(42 + base / 3);

          if (((x + tick) / 18 + (y / 18)) % 2 == 0)
            {
              b = (uint8_t)(b + 16);
            }

          if (d2 < radius * radius)
            {
              uint8_t shade = triangle_wave(dx + dy + tick * 4, 180);
              r = (uint8_t)(64 + shade / 2);
              g = (uint8_t)(188 + shade / 5);
              b = (uint8_t)(220 + shade / 8);
            }
          else if (d2 < ring * ring)
            {
              r = 240;
              g = (uint8_t)(160 + triangle_wave(tick * 3 + x, 120) / 4);
              b = 58;
            }

          row[x] = rgb565(r, g, b);
        }
    }
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

int main(int argc, FAR char *argv[])
{
  struct wing_surface_bootstrap surface;
  struct wing_surface_frame_info frame;
  int queue;
  int stride_pixels;
  int tick;
  int i;

  memset(&surface, 0, sizeof(surface));
  for (i = 1; i < argc; i++)
    {
      if (parse_surface_arg(argv[i], &surface) == 0)
        {
          break;
        }
    }

  if (i >= argc)
    {
      printf("wing_surface_demo: missing --wing-surface\n");
      return 1;
    }

  if (surface.format != WING_SURFACE_FORMAT_RGB565)
    {
      printf("wing_surface_demo: only RGB565 is supported\n");
      return 1;
    }

  if (wing_surface_frame_resolve(surface.frame, surface.token, &frame) < 0 ||
      frame.pixels == NULL ||
      frame.width <= 0 ||
      frame.height <= 0 ||
      frame.stride < frame.width * 2 ||
      frame.format != WING_SURFACE_FORMAT_RGB565)
    {
      printf("wing_surface_demo: failed to resolve surface frame\n");
      return 1;
    }

  queue = wing_surface_dirty_sender_open();
  if (queue < 0)
    {
      printf("wing_surface_demo: failed to open dirty queue\n");
      return 1;
    }

  stride_pixels = frame.stride / 2;
  for (tick = 0; ; tick = (tick + 1) & 0x7fff)
    {
      draw_demo_frame((FAR uint16_t *)frame.pixels,
                      frame.width,
                      frame.height,
                      stride_pixels,
                      tick);
      wing_surface_dirty_send(queue,
                              surface.dirty,
                              surface.token,
                              0,
                              0,
                              (uint16_t)frame.width,
                              (uint16_t)frame.height);
      usleep(33000);
    }

  wing_surface_dirty_close(queue);
  return 0;
}
