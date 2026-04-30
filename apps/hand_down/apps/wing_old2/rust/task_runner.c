/****************************************************************************
 * apps/wing/rust/task_runner.c
 *
 * Thin NuttX task launch shim for the Rust Wing runtime.
 ****************************************************************************/

#include <nuttx/config.h>

#include <errno.h>
#include <fcntl.h>
#include <mqueue.h>
#include <sched.h>
#include <spawn.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <termios.h>
#include <unistd.h>

#include <nuttx/lib/builtin.h>

#if defined(CONFIG_PSEUDOTERM) && defined(CONFIG_PSEUDOTERM_SUSV1) && \
    defined(CONFIG_NSH_LIBRARY)
#  include <nuttx/serial/pty.h>
#  include <nshlib/nshlib.h>
#  define WING_TERMINAL_HAVE_PTY 1
#else
#  define WING_TERMINAL_HAVE_PTY 0
#endif

#define WING_TASK_STDIO_INHERIT 0
#define WING_TASK_STDIO_NULL    1

#define WING_TASK_DEFAULT_PRIORITY (-1)
#define WING_TASK_DEFAULT_STACK    0
#define WING_TASK_DETACHED_SURFACE 0
#define WING_TASK_MANAGED_SURFACE  1

#define WING_TASK_ARGV_CAPACITY    8
#define WING_SURFACE_ARG_CAPACITY  160
#define WING_SETTINGS_ARG_CAPACITY 96
#define WING_SURFACE_SLOT_CAPACITY 8
#define WING_SURFACE_FRAME_MAX_BYTES (640 * 640 * 4 * 2)
#define WING_SURFACE_DIRTY_MAXMSG  4
#define WING_SURFACE_DIRTY_QUEUE   "/wing_surface_dirty"
#define WING_SURFACE_INPUT_MAXMSG  8
#define WING_SURFACE_INPUT_QUEUE   "/wing_surface_input"
#define WING_SETTINGS_MAXMSG       8
#define WING_SETTINGS_QUEUE        "/wing_settings"
#define WING_TERMINAL_NSH_PRIORITY 100
#define WING_TERMINAL_NSH_STACK    8192

#define WING_SURFACE_ABI_VERSION 1

#define WING_SURFACE_CAP_FORMAT_RGB565    (1u << 0)
#define WING_SURFACE_CAP_FORMAT_ARGB8888  (1u << 1)

#define WING_SURFACE_CAP_TRANSPORT_SHARED_MEMORY  (1u << 0)
#define WING_SURFACE_CAP_TRANSPORT_STREAM_TEXTURE (1u << 1)

#define WING_SURFACE_CAP_INPUT_NONE             (1u << 0)
#define WING_SURFACE_CAP_INPUT_POINTER          (1u << 1)
#define WING_SURFACE_CAP_INPUT_POINTER_KEYBOARD (1u << 2)

#define WING_SURFACE_POINTER_MOVE   1
#define WING_SURFACE_POINTER_DOWN   2
#define WING_SURFACE_POINTER_UP     3
#define WING_SURFACE_POINTER_CANCEL 4

struct wing_surface_dirty_message
{
  uint32_t handle;
  uint32_t token;
  int16_t x;
  int16_t y;
  uint16_t w;
  uint16_t h;
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

struct wing_settings_message
{
  uint16_t key;
  uint16_t reserved;
  uint32_t value;
};

struct wing_surface_frame_info
{
  uint32_t handle;
  uint32_t token;
  FAR void *pixels;
  size_t bytes;
  int width;
  int height;
  int stride;
  int format;
  int buffers;
};

struct wing_surface_frame_slot
{
  int active;
  uint32_t handle;
  uint32_t token;
  FAR uint8_t *pixels;
  size_t bytes;
  int width;
  int height;
  int stride;
  int format;
  int buffers;
};

struct wing_task_launch_config
{
  FAR const char *program;
  FAR char * const *argv;
  int stdio;
  int priority;
  int stack_size;
  int surface;
  int surface_id;
  int surface_width;
  int surface_height;
  int surface_stride;
  int surface_format;
  int surface_buffers;
  int surface_transport;
  int surface_input;
  int surface_token;
  int surface_frame;
  int surface_dirty;
  int surface_input_handle;
  int settings_valid;
  int settings_theme;
  int settings_preview;
  int settings_brightness;
  int settings_haptic;
  int settings_reduce_motion;
};

static struct wing_surface_frame_slot
  g_wing_surface_frames[WING_SURFACE_SLOT_CAPACITY];

struct wing_terminal_session
{
  int master;
  pid_t pid;
  int last_error;
};

static struct wing_terminal_session g_wing_terminal =
{
  -1,
  -1,
  0
};

static int wing_surface_frame_bytes(
    FAR const struct wing_task_launch_config *config,
    FAR size_t *bytes)
{
  size_t stride;
  size_t height;
  size_t buffers;

  if (config == NULL || bytes == NULL ||
      config->surface_width <= 0 ||
      config->surface_height <= 0 ||
      config->surface_stride <= 0 ||
      config->surface_buffers <= 0)
    {
      return EINVAL;
    }

  stride = (size_t)config->surface_stride;
  height = (size_t)config->surface_height;
  buffers = (size_t)config->surface_buffers;

  if (height != 0 && stride > SIZE_MAX / height)
    {
      return EOVERFLOW;
    }

  stride *= height;
  if (buffers != 0 && stride > SIZE_MAX / buffers)
    {
      return EOVERFLOW;
    }

  *bytes = stride * buffers;
  if (*bytes == 0 || *bytes > WING_SURFACE_FRAME_MAX_BYTES)
    {
      return EINVAL;
    }

  return 0;
}

static int wing_surface_frame_find(uint32_t handle, uint32_t token)
{
  int index;

  for (index = 0; index < WING_SURFACE_SLOT_CAPACITY; index++)
    {
      if (g_wing_surface_frames[index].active &&
          g_wing_surface_frames[index].handle == handle &&
          g_wing_surface_frames[index].token == token)
        {
          return index;
        }
    }

  return -1;
}

int wing_surface_query_capabilities(
    FAR struct wing_surface_capabilities *caps)
{
  if (caps == NULL)
    {
      return -EINVAL;
    }

  memset(caps, 0, sizeof(*caps));
  caps->abi_version = WING_SURFACE_ABI_VERSION;
  caps->format_mask = WING_SURFACE_CAP_FORMAT_RGB565;
  caps->transport_mask = WING_SURFACE_CAP_TRANSPORT_SHARED_MEMORY;
  caps->input_mask = WING_SURFACE_CAP_INPUT_NONE |
                     WING_SURFACE_CAP_INPUT_POINTER;
  caps->max_surfaces = WING_SURFACE_SLOT_CAPACITY;
  caps->max_width = 640;
  caps->max_height = 640;
  caps->max_frame_bytes = WING_SURFACE_FRAME_MAX_BYTES;
  return 0;
}

static int wing_surface_frame_register(
    FAR const struct wing_task_launch_config *config)
{
  FAR struct wing_surface_frame_slot *slot;
  size_t bytes;
  int index;
  int ret;

  if (config->surface == WING_TASK_DETACHED_SURFACE)
    {
      return 0;
    }

  if (config->surface != WING_TASK_MANAGED_SURFACE ||
      config->surface_frame <= 0 ||
      config->surface_token <= 0)
    {
      return EINVAL;
    }

  if (wing_surface_frame_find((uint32_t)config->surface_frame,
                              (uint32_t)config->surface_token) >= 0)
    {
      return EEXIST;
    }

  ret = wing_surface_frame_bytes(config, &bytes);
  if (ret != 0)
    {
      return ret;
    }

  for (index = 0; index < WING_SURFACE_SLOT_CAPACITY; index++)
    {
      slot = &g_wing_surface_frames[index];
      if (!slot->active)
        {
          slot->pixels = (FAR uint8_t *)calloc(1, bytes);
          if (slot->pixels == NULL)
            {
              return ENOMEM;
            }

          slot->active = 1;
          slot->handle = (uint32_t)config->surface_frame;
          slot->token = (uint32_t)config->surface_token;
          slot->bytes = bytes;
          slot->width = config->surface_width;
          slot->height = config->surface_height;
          slot->stride = config->surface_stride;
          slot->format = config->surface_format;
          slot->buffers = config->surface_buffers;
          return 0;
        }
    }

  return ENOSPC;
}

int wing_surface_frame_unregister(uint32_t handle, uint32_t token)
{
  int index;

  index = wing_surface_frame_find(handle, token);
  if (index < 0)
    {
      return -ENOENT;
    }

  free(g_wing_surface_frames[index].pixels);
  memset(&g_wing_surface_frames[index], 0,
         sizeof(g_wing_surface_frames[index]));
  return 0;
}

int wing_surface_frame_reset(void)
{
  int index;

  for (index = 0; index < WING_SURFACE_SLOT_CAPACITY; index++)
    {
      if (g_wing_surface_frames[index].active)
        {
          free(g_wing_surface_frames[index].pixels);
        }

      memset(&g_wing_surface_frames[index], 0,
             sizeof(g_wing_surface_frames[index]));
    }

  return 0;
}

int wing_surface_frame_resolve(uint32_t handle, uint32_t token,
                               FAR struct wing_surface_frame_info *info)
{
  FAR struct wing_surface_frame_slot *slot;
  int index;

  if (info == NULL)
    {
      return -EINVAL;
    }

  index = wing_surface_frame_find(handle, token);
  if (index < 0)
    {
      return -ENOENT;
    }

  slot = &g_wing_surface_frames[index];
  info->handle = slot->handle;
  info->token = slot->token;
  info->pixels = slot->pixels;
  info->bytes = slot->bytes;
  info->width = slot->width;
  info->height = slot->height;
  info->stride = slot->stride;
  info->format = slot->format;
  info->buffers = slot->buffers;
  return 0;
}

int wing_surface_dirty_open(void)
{
#ifdef CONFIG_DISABLE_MQUEUE
  return -ENOSYS;
#else
  struct mq_attr attr;
  mqd_t queue;

  memset(&attr, 0, sizeof(attr));
  attr.mq_maxmsg = WING_SURFACE_DIRTY_MAXMSG;
  attr.mq_msgsize = sizeof(struct wing_surface_dirty_message);

  mq_unlink(WING_SURFACE_DIRTY_QUEUE);
  queue = mq_open(WING_SURFACE_DIRTY_QUEUE,
                  O_RDONLY | O_CREAT | O_NONBLOCK,
                  0666, &attr);
  if (queue < 0)
    {
      return -errno;
    }

  return queue;
#endif
}

int wing_surface_dirty_receive(int queue,
                               FAR struct wing_surface_dirty_message *message)
{
#ifdef CONFIG_DISABLE_MQUEUE
  return -ENOSYS;
#else
  ssize_t received;

  if (queue < 0 || message == NULL)
    {
      return -EINVAL;
    }

  received = mq_receive((mqd_t)queue, (FAR char *)message,
                        sizeof(struct wing_surface_dirty_message), NULL);
  if (received < 0)
    {
      if (errno == EAGAIN)
        {
          return 0;
        }

      return -errno;
    }

  if (received != sizeof(struct wing_surface_dirty_message))
    {
      return -EMSGSIZE;
    }

  return 1;
#endif
}

int wing_surface_dirty_sender_open(void)
{
#ifdef CONFIG_DISABLE_MQUEUE
  return -ENOSYS;
#else
  mqd_t queue;

  queue = mq_open(WING_SURFACE_DIRTY_QUEUE, O_WRONLY | O_NONBLOCK);
  if (queue < 0)
    {
      return -errno;
    }

  return queue;
#endif
}

int wing_surface_dirty_send(int queue, uint32_t handle, uint32_t token,
                            int16_t x, int16_t y, uint16_t w, uint16_t h)
{
#ifdef CONFIG_DISABLE_MQUEUE
  return -ENOSYS;
#else
  struct wing_surface_dirty_message message;
  int ret;

  if (queue < 0)
    {
      return -EINVAL;
    }

  message.handle = handle;
  message.token = token;
  message.x = x;
  message.y = y;
  message.w = w;
  message.h = h;

  ret = mq_send((mqd_t)queue, (FAR const char *)&message,
                sizeof(message), 0);
  if (ret < 0)
    {
      return -errno;
    }

  return 0;
#endif
}

int wing_surface_dirty_close(int queue)
{
#ifdef CONFIG_DISABLE_MQUEUE
  return -ENOSYS;
#else
  if (queue < 0)
    {
      return -EINVAL;
    }

  return mq_close((mqd_t)queue);
#endif
}

int wing_surface_input_open(void)
{
#ifdef CONFIG_DISABLE_MQUEUE
  return -ENOSYS;
#else
  struct mq_attr attr;
  mqd_t queue;

  memset(&attr, 0, sizeof(attr));
  attr.mq_maxmsg = WING_SURFACE_INPUT_MAXMSG;
  attr.mq_msgsize = sizeof(struct wing_surface_input_message);

  mq_unlink(WING_SURFACE_INPUT_QUEUE);
  queue = mq_open(WING_SURFACE_INPUT_QUEUE,
                  O_WRONLY | O_CREAT | O_NONBLOCK,
                  0666, &attr);
  if (queue < 0)
    {
      return -errno;
    }

  return queue;
#endif
}

int wing_surface_input_receiver_open(void)
{
#ifdef CONFIG_DISABLE_MQUEUE
  return -ENOSYS;
#else
  mqd_t queue;

  queue = mq_open(WING_SURFACE_INPUT_QUEUE, O_RDONLY | O_NONBLOCK);
  if (queue < 0)
    {
      return -errno;
    }

  return queue;
#endif
}

int wing_surface_input_send(int queue, uint32_t handle, uint32_t token,
                            int16_t x, int16_t y, uint8_t event,
                            uint8_t buttons)
{
#ifdef CONFIG_DISABLE_MQUEUE
  return -ENOSYS;
#else
  struct wing_surface_input_message message;
  int ret;

  if (queue < 0)
    {
      return -EINVAL;
    }

  if (event < WING_SURFACE_POINTER_MOVE ||
      event > WING_SURFACE_POINTER_CANCEL)
    {
      return -EINVAL;
    }

  message.handle = handle;
  message.token = token;
  message.x = x;
  message.y = y;
  message.event = event;
  message.buttons = buttons;
  message.reserved = 0;

  ret = mq_send((mqd_t)queue, (FAR const char *)&message,
                sizeof(message), 0);
  if (ret < 0)
    {
      return -errno;
    }

  return 0;
#endif
}

int wing_surface_input_receive(int queue,
                               FAR struct wing_surface_input_message *message)
{
#ifdef CONFIG_DISABLE_MQUEUE
  return -ENOSYS;
#else
  ssize_t received;

  if (queue < 0 || message == NULL)
    {
      return -EINVAL;
    }

  received = mq_receive((mqd_t)queue, (FAR char *)message,
                        sizeof(struct wing_surface_input_message), NULL);
  if (received < 0)
    {
      if (errno == EAGAIN)
        {
          return 0;
        }

      return -errno;
    }

  if (received != sizeof(struct wing_surface_input_message))
    {
      return -EMSGSIZE;
    }

  return 1;
#endif
}

int wing_surface_input_close(int queue)
{
#ifdef CONFIG_DISABLE_MQUEUE
  return -ENOSYS;
#else
  if (queue < 0)
    {
      return -EINVAL;
    }

  return mq_close((mqd_t)queue);
#endif
}

int wing_settings_open(void)
{
#ifdef CONFIG_DISABLE_MQUEUE
  return -ENOSYS;
#else
  struct mq_attr attr;
  mqd_t queue;

  memset(&attr, 0, sizeof(attr));
  attr.mq_maxmsg = WING_SETTINGS_MAXMSG;
  attr.mq_msgsize = sizeof(struct wing_settings_message);

  mq_unlink(WING_SETTINGS_QUEUE);
  queue = mq_open(WING_SETTINGS_QUEUE,
                  O_RDONLY | O_CREAT | O_NONBLOCK,
                  0666, &attr);
  if (queue < 0)
    {
      return -errno;
    }

  return queue;
#endif
}

int wing_settings_sender_open(void)
{
#ifdef CONFIG_DISABLE_MQUEUE
  return -ENOSYS;
#else
  mqd_t queue;

  queue = mq_open(WING_SETTINGS_QUEUE, O_WRONLY | O_NONBLOCK);
  if (queue < 0)
    {
      return -errno;
    }

  return queue;
#endif
}

int wing_settings_send(int queue, uint16_t key, uint32_t value)
{
#ifdef CONFIG_DISABLE_MQUEUE
  return -ENOSYS;
#else
  struct wing_settings_message message;
  int ret;

  if (queue < 0 || key == 0)
    {
      return -EINVAL;
    }

  message.key = key;
  message.reserved = 0;
  message.value = value;

  ret = mq_send((mqd_t)queue, (FAR const char *)&message,
                sizeof(message), 0);
  if (ret < 0)
    {
      return -errno;
    }

  return 0;
#endif
}

int wing_settings_receive(int queue,
                          FAR struct wing_settings_message *message)
{
#ifdef CONFIG_DISABLE_MQUEUE
  return -ENOSYS;
#else
  ssize_t received;

  if (queue < 0 || message == NULL)
    {
      return -EINVAL;
    }

  received = mq_receive((mqd_t)queue, (FAR char *)message,
                        sizeof(struct wing_settings_message), NULL);
  if (received < 0)
    {
      if (errno == EAGAIN)
        {
          return 0;
        }

      return -errno;
    }

  if (received != sizeof(struct wing_settings_message))
    {
      return -EMSGSIZE;
    }

  return 1;
#endif
}

int wing_settings_close(int queue)
{
#ifdef CONFIG_DISABLE_MQUEUE
  return -ENOSYS;
#else
  if (queue < 0)
    {
      return -EINVAL;
    }

  return mq_close((mqd_t)queue);
#endif
}

int wing_terminal_supported(void)
{
  return WING_TERMINAL_HAVE_PTY;
}

int wing_terminal_status(void)
{
#if WING_TERMINAL_HAVE_PTY
  if (g_wing_terminal.master >= 0)
    {
      return 1;
    }

  return g_wing_terminal.last_error;
#else
  return -ENOSYS;
#endif
}

int wing_terminal_open(void)
{
#if WING_TERMINAL_HAVE_PTY
  posix_spawn_file_actions_t actions;
  posix_spawnattr_t attr;
  struct sched_param sched;
  struct termios tio;
  char pts_name[24];
  int master;
  int slave;
  pid_t pid;
  int ret;

  if (g_wing_terminal.master >= 0)
    {
      return 0;
    }

  master = open("/dev/ptmx", O_RDWR | O_NOCTTY | O_NONBLOCK);
  if (master < 0)
    {
      g_wing_terminal.last_error = -errno;
      return g_wing_terminal.last_error;
    }

  ret = grantpt(master);
  if (ret < 0)
    {
      ret = errno;
      goto errout_with_master;
    }

  ret = unlockpt(master);
  if (ret < 0)
    {
      ret = errno;
      goto errout_with_master;
    }

  ret = ptsname_r(master, pts_name, sizeof(pts_name));
  if (ret < 0)
    {
      ret = errno;
      goto errout_with_master;
    }

  slave = open(pts_name, O_RDWR | O_NOCTTY);
  if (slave < 0)
    {
      ret = errno;
      goto errout_with_master;
    }

  if (tcgetattr(slave, &tio) == 0)
    {
      tio.c_oflag |= OPOST | ONLCR;
      (void)tcsetattr(slave, TCSANOW, &tio);
    }

  ret = posix_spawnattr_init(&attr);
  if (ret != 0)
    {
      goto errout_with_slave;
    }

  ret = posix_spawn_file_actions_init(&actions);
  if (ret != 0)
    {
      goto errout_with_attr;
    }

  sched.sched_priority = WING_TERMINAL_NSH_PRIORITY;
  ret = posix_spawnattr_setschedparam(&attr, &sched);
  if (ret != 0)
    {
      goto errout_with_actions;
    }

  ret = posix_spawnattr_setstacksize(&attr, WING_TERMINAL_NSH_STACK);
  if (ret != 0)
    {
      goto errout_with_actions;
    }

#  if CONFIG_RR_INTERVAL > 0
  ret = posix_spawnattr_setschedpolicy(&attr, SCHED_RR);
  if (ret != 0)
    {
      goto errout_with_actions;
    }

  ret = posix_spawnattr_setflags(&attr,
                                 POSIX_SPAWN_SETSCHEDPARAM |
                                 POSIX_SPAWN_SETSCHEDULER);
#  else
  ret = posix_spawnattr_setflags(&attr, POSIX_SPAWN_SETSCHEDPARAM);
#  endif
  if (ret != 0)
    {
      goto errout_with_actions;
    }

  ret = posix_spawn_file_actions_adddup2(&actions, slave, 0);
  if (ret != 0)
    {
      goto errout_with_actions;
    }

  ret = posix_spawn_file_actions_adddup2(&actions, slave, 1);
  if (ret != 0)
    {
      goto errout_with_actions;
    }

  ret = posix_spawn_file_actions_adddup2(&actions, slave, 2);
  if (ret != 0)
    {
      goto errout_with_actions;
    }

  (void)posix_spawn_file_actions_addclose(&actions, master);
  (void)posix_spawn_file_actions_addclose(&actions, slave);

  pid = task_spawn("wing_nsh", nsh_consolemain, &actions, &attr, NULL, NULL);
  ret = pid < 0 ? -pid : 0;
  if (ret != 0)
    {
      goto errout_with_actions;
    }

  posix_spawn_file_actions_destroy(&actions);
  posix_spawnattr_destroy(&attr);
  close(slave);

  g_wing_terminal.master = master;
  g_wing_terminal.pid = pid;
  g_wing_terminal.last_error = 0;
  return 0;

errout_with_actions:
  posix_spawn_file_actions_destroy(&actions);
errout_with_attr:
  posix_spawnattr_destroy(&attr);
errout_with_slave:
  close(slave);
errout_with_master:
  close(master);
  g_wing_terminal.master = -1;
  g_wing_terminal.pid = -1;
  g_wing_terminal.last_error = -ret;
  return g_wing_terminal.last_error;
#else
  g_wing_terminal.last_error = -ENOSYS;
  return -ENOSYS;
#endif
}

int wing_terminal_read(FAR uint8_t *buffer, size_t buflen)
{
#if WING_TERMINAL_HAVE_PTY
  ssize_t ret;

  if (buffer == NULL || buflen == 0)
    {
      return -EINVAL;
    }

  if (g_wing_terminal.master < 0)
    {
      return -ENOTCONN;
    }

  ret = read(g_wing_terminal.master, buffer, buflen);
  if (ret < 0)
    {
      if (errno == EAGAIN || errno == EWOULDBLOCK)
        {
          return 0;
        }

      g_wing_terminal.last_error = -errno;
      return g_wing_terminal.last_error;
    }

  return (int)ret;
#else
  return -ENOSYS;
#endif
}

int wing_terminal_write(FAR const uint8_t *buffer, size_t buflen)
{
#if WING_TERMINAL_HAVE_PTY
  ssize_t ret;

  if (buffer == NULL || buflen == 0)
    {
      return -EINVAL;
    }

  if (g_wing_terminal.master < 0)
    {
      return -ENOTCONN;
    }

  ret = write(g_wing_terminal.master, buffer, buflen);
  if (ret < 0)
    {
      if (errno == EAGAIN || errno == EWOULDBLOCK)
        {
          return 0;
        }

      g_wing_terminal.last_error = -errno;
      return g_wing_terminal.last_error;
    }

  return (int)ret;
#else
  return -ENOSYS;
#endif
}

int wing_terminal_close(void)
{
#if WING_TERMINAL_HAVE_PTY
  if (g_wing_terminal.pid > 0)
    {
      (void)task_delete(g_wing_terminal.pid);
    }

  if (g_wing_terminal.master >= 0)
    {
      close(g_wing_terminal.master);
    }

  g_wing_terminal.master = -1;
  g_wing_terminal.pid = -1;
  g_wing_terminal.last_error = 0;
  return 0;
#else
  g_wing_terminal.last_error = -ENOSYS;
  return -ENOSYS;
#endif
}

int wing_task_close(pid_t pid, uint32_t frame, uint32_t token)
{
  int task_ret = 0;
  int surface_ret = 0;

  if (pid > 0)
    {
      task_ret = task_delete(pid);
    }

  if (frame != 0 && token != 0)
    {
      surface_ret = wing_surface_frame_unregister(frame, token);
      if (surface_ret == -ENOENT)
        {
          surface_ret = 0;
        }
    }

  return surface_ret != 0 ? surface_ret : task_ret;
}

static int wing_task_configure_stdio(FAR posix_spawn_file_actions_t *actions,
                                     int stdio)
{
  int ret;

  if (stdio == WING_TASK_STDIO_INHERIT)
    {
      return 0;
    }

  if (stdio != WING_TASK_STDIO_NULL)
    {
      return EINVAL;
    }

  ret = posix_spawn_file_actions_addopen(actions, 0, "/dev/null", O_RDWR, 0);
  if (ret != 0)
    {
      return ret;
    }

  ret = posix_spawn_file_actions_addopen(actions, 1, "/dev/null", O_RDWR, 0);
  if (ret != 0)
    {
      return ret;
    }

  return posix_spawn_file_actions_addopen(actions, 2, "/dev/null", O_RDWR, 0);
}

static int wing_task_append_surface_arg(
    FAR const struct wing_task_launch_config *config,
    FAR char *argv[WING_TASK_ARGV_CAPACITY],
    FAR char *surface_arg)
{
  int index;
  int ret;

  if (config->argv == NULL || config->argv[0] == NULL)
    {
      argv[0] = (FAR char *)config->program;
      argv[1] = NULL;
    }
  else
    {
      for (index = 0; index < WING_TASK_ARGV_CAPACITY - 1; index++)
        {
          argv[index] = config->argv[index];
          if (argv[index] == NULL)
            {
              break;
            }
        }

      if (index >= WING_TASK_ARGV_CAPACITY - 1)
        {
          return E2BIG;
        }
    }

  if (config->surface == WING_TASK_DETACHED_SURFACE)
    {
      return 0;
    }

  if (config->surface != WING_TASK_MANAGED_SURFACE ||
      config->surface_id <= 0 ||
      config->surface_width <= 0 ||
      config->surface_height <= 0 ||
      config->surface_stride <= 0 ||
      config->surface_buffers <= 0 ||
      config->surface_token <= 0 ||
      config->surface_frame <= 0 ||
      config->surface_dirty <= 0)
    {
      return EINVAL;
    }

  ret = snprintf(surface_arg, WING_SURFACE_ARG_CAPACITY,
                 "--wing-surface=%d:%d:%d:%d:%d:%d:%d:%d:%d:%d:%d:%d",
                 config->surface_id,
                 config->surface_width,
                 config->surface_height,
                 config->surface_stride,
                 config->surface_format,
                 config->surface_buffers,
                 config->surface_transport,
                 config->surface_input,
                 config->surface_token,
                 config->surface_frame,
                 config->surface_dirty,
                 config->surface_input_handle);
  if (ret < 0 || ret >= WING_SURFACE_ARG_CAPACITY)
    {
      return E2BIG;
    }

  for (index = 0; index < WING_TASK_ARGV_CAPACITY; index++)
    {
      if (argv[index] == NULL)
        {
          if (index >= WING_TASK_ARGV_CAPACITY - 1)
            {
              return E2BIG;
            }

          argv[index] = surface_arg;
          argv[index + 1] = NULL;
          return 0;
        }
    }

  return E2BIG;
}

static int wing_task_append_settings_arg(
    FAR const struct wing_task_launch_config *config,
    FAR char *argv[WING_TASK_ARGV_CAPACITY],
    FAR char *settings_arg)
{
  int index;
  int ret;

  if (config->settings_valid == 0)
    {
      return 0;
    }

  ret = snprintf(settings_arg, WING_SETTINGS_ARG_CAPACITY,
                 "--wing-settings=%d:%d:%d:%d:%d",
                 config->settings_theme,
                 config->settings_preview,
                 config->settings_brightness,
                 config->settings_haptic,
                 config->settings_reduce_motion);
  if (ret < 0 || ret >= WING_SETTINGS_ARG_CAPACITY)
    {
      return E2BIG;
    }

  for (index = 0; index < WING_TASK_ARGV_CAPACITY; index++)
    {
      if (argv[index] == NULL)
        {
          if (index >= WING_TASK_ARGV_CAPACITY - 1)
            {
              return E2BIG;
            }

          argv[index] = settings_arg;
          argv[index + 1] = NULL;
          return 0;
        }
    }

  return E2BIG;
}

int wing_task_spawn(FAR const struct wing_task_launch_config *config)
{
  FAR const struct builtin_s *builtin;
  posix_spawn_file_actions_t actions;
  posix_spawnattr_t attr;
  FAR char *argv[WING_TASK_ARGV_CAPACITY];
  char surface_arg[WING_SURFACE_ARG_CAPACITY];
  char settings_arg[WING_SETTINGS_ARG_CAPACITY];
  struct sched_param sched;
  pid_t pid;
  int stack_size;
  int priority;
  int index;
  int ret;
  int frame_registered = 0;

  if (config == NULL || config->program == NULL)
    {
      return -EINVAL;
    }

  ret = wing_task_append_surface_arg(config, argv, surface_arg);
  if (ret != 0)
    {
      return -ret;
    }

  ret = wing_task_append_settings_arg(config, argv, settings_arg);
  if (ret != 0)
    {
      return -ret;
    }

  index = builtin_isavail(config->program);
  if (index < 0)
    {
      return -ENOENT;
    }

  builtin = builtin_for_index(index);
  if (builtin == NULL || builtin->main == NULL)
    {
      return -ENOENT;
    }

  frame_registered = config->surface == WING_TASK_MANAGED_SURFACE;
  if (frame_registered)
    {
      ret = wing_surface_frame_register(config);
      if (ret != 0)
        {
          return -ret;
        }
    }

  ret = posix_spawnattr_init(&attr);
  if (ret != 0)
    {
      goto errout_with_frame;
    }

  ret = posix_spawn_file_actions_init(&actions);
  if (ret != 0)
    {
      goto errout_with_attr;
    }

  priority = config->priority == WING_TASK_DEFAULT_PRIORITY ?
             builtin->priority : config->priority;
  stack_size = config->stack_size == WING_TASK_DEFAULT_STACK ?
               builtin->stacksize : config->stack_size;

  if (priority < 0 || stack_size <= 0)
    {
      ret = EINVAL;
      goto errout_with_actions;
    }

  sched.sched_priority = priority;
  ret = posix_spawnattr_setschedparam(&attr, &sched);
  if (ret != 0)
    {
      goto errout_with_actions;
    }

  ret = posix_spawnattr_setstacksize(&attr, stack_size);
  if (ret != 0)
    {
      goto errout_with_actions;
    }

#if CONFIG_RR_INTERVAL > 0
  ret = posix_spawnattr_setschedpolicy(&attr, SCHED_RR);
  if (ret != 0)
    {
      goto errout_with_actions;
    }

  ret = posix_spawnattr_setflags(&attr,
                                 POSIX_SPAWN_SETSCHEDPARAM |
                                 POSIX_SPAWN_SETSCHEDULER);
  if (ret != 0)
    {
      goto errout_with_actions;
    }
#else
  ret = posix_spawnattr_setflags(&attr, POSIX_SPAWN_SETSCHEDPARAM);
  if (ret != 0)
    {
      goto errout_with_actions;
    }
#endif

  ret = wing_task_configure_stdio(&actions, config->stdio);
  if (ret != 0)
    {
      goto errout_with_actions;
    }

#ifdef CONFIG_LIBC_EXECFUNCS
  ret = posix_spawn(&pid, builtin->name, &actions, &attr,
                    argv, NULL);
  if (ret != 0)
#endif
    {
      pid = task_spawn(builtin->name, builtin->main, &actions, &attr,
                       argv[0] ? &argv[1] : NULL, NULL);
      ret = pid < 0 ? -pid : 0;
    }

  if (ret != 0)
    {
      goto errout_with_actions;
    }

  posix_spawn_file_actions_destroy(&actions);
  posix_spawnattr_destroy(&attr);
  return pid;

errout_with_actions:
  posix_spawn_file_actions_destroy(&actions);

errout_with_attr:
  posix_spawnattr_destroy(&attr);

errout_with_frame:
  if (frame_registered)
    {
      wing_surface_frame_unregister((uint32_t)config->surface_frame,
                                    (uint32_t)config->surface_token);
    }

  return -ret;
}
