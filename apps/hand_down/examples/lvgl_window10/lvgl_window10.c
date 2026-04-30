/****************************************************************************
 * apps/examples/lvgl_window10/lvgl_window10.c
 *
 * NuttX entry point for the Windows 10 Mobile LVGL demo migrated from:
 *   /home/uan-gpd/codes/windows-10-mobile-lvgl
 *
 * The migrated UI itself lives under ui/ and keeps the original MIT license
 * notice in LICENSE.windows-10-mobile-lvgl.
 ****************************************************************************/

#include <nuttx/config.h>

#include <stdbool.h>
#include <stdint.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#include <sys/boardctl.h>

#include <lvgl/lvgl.h>

#include "ui/ui.h"

#ifdef CONFIG_LV_USE_NUTTX_LIBUV
#include <uv.h>
#endif

#undef NEED_BOARDINIT

#if defined(CONFIG_BOARDCTL) && !defined(CONFIG_NSH_ARCHINIT)
#  define NEED_BOARDINIT 1
#endif

#define PREF_SLOTS 24
#define PREF_KEY_LEN 24

struct pref_slot_s
{
  char key[PREF_KEY_LEN];
  uint32_t value;
  bool used;
};

static struct pref_slot_s g_prefs[PREF_SLOTS];

static const char *const g_days[7] =
{
  "Sunday",
  "Monday",
  "Tuesday",
  "Wednesday",
  "Thursday",
  "Friday",
  "Saturday"
};

static const char *const g_months[12] =
{
  "January",
  "February",
  "March",
  "April",
  "May",
  "June",
  "July",
  "August",
  "September",
  "October",
  "November",
  "December"
};

static int pref_find(const char *key)
{
  int free_slot = -1;

  for (int i = 0; i < PREF_SLOTS; i++)
    {
      if (g_prefs[i].used && strncmp(g_prefs[i].key, key, PREF_KEY_LEN) == 0)
        {
          return i;
        }

      if (!g_prefs[i].used && free_slot < 0)
        {
          free_slot = i;
        }
    }

  return free_slot;
}

void setPrefInt(const char *key, uint32_t value)
{
  int slot = pref_find(key);

  if (slot < 0)
    {
      return;
    }

  strlcpy(g_prefs[slot].key, key, sizeof(g_prefs[slot].key));
  g_prefs[slot].value = value;
  g_prefs[slot].used = true;
}

uint32_t getPrefInt(const char *key, uint32_t def)
{
  int slot = pref_find(key);

  if (slot < 0 || !g_prefs[slot].used)
    {
      return def;
    }

  return g_prefs[slot].value;
}

void setPrefBool(const char *key, bool value)
{
  setPrefInt(key, value ? 1 : 0);
}

bool getPrefBool(const char *key, bool def)
{
  return getPrefInt(key, def ? 1 : 0) != 0;
}

void onBrightnessChange(int32_t value)
{
  setPrefInt("brightness", value);
}

void onTimeoutChange(int16_t selected)
{
  setPrefInt("timeout", selected);
}

void onWifiStateChange(int state)
{
  setPrefInt("wifi_state", state ? 1 : 0);
}

static void lvgl_window10_update_time(void)
{
  struct tm *tm;
  time_t now = time(NULL);

  tm = localtime(&now);
  if (tm == NULL)
    {
      return;
    }

  if (ui_statusPanelTime != NULL)
    {
      lv_label_set_text_fmt(ui_statusPanelTime, "%02d:%02d",
                            tm->tm_hour, tm->tm_min);
    }

  if (ui_Label41 != NULL)
    {
      lv_label_set_text_fmt(ui_Label41, "%02d:%02d",
                            tm->tm_hour, tm->tm_min);
    }

  if (ui_Label39 != NULL)
    {
      lv_label_set_text_fmt(ui_Label39, "%s, %s %d",
                            g_days[tm->tm_wday % 7],
                            g_months[tm->tm_mon % 12],
                            tm->tm_mday);
    }
}

#ifdef CONFIG_LV_USE_NUTTX_LIBUV
static void lvgl_window10_uv_loop(uv_loop_t *loop,
                                  lv_nuttx_result_t *result)
{
  lv_nuttx_uv_t uv_info;
  void *data;

  uv_loop_init(loop);

  lv_memzero(&uv_info, sizeof(uv_info));
  uv_info.loop = loop;
  uv_info.disp = result->disp;
  uv_info.indev = result->indev;
#ifdef CONFIG_UINPUT_TOUCH
  uv_info.uindev = result->utouch_indev;
#endif

  data = lv_nuttx_uv_init(&uv_info);
  uv_run(loop, UV_RUN_DEFAULT);
  lv_nuttx_uv_deinit(&data);
}
#endif

int main(int argc, FAR char *argv[])
{
  lv_nuttx_dsc_t info;
  lv_nuttx_result_t result;

#ifdef CONFIG_LV_USE_NUTTX_LIBUV
  uv_loop_t ui_loop;
  lv_memzero(&ui_loop, sizeof(ui_loop));
#endif

  LV_UNUSED(argc);
  LV_UNUSED(argv);

  if (lv_is_initialized())
    {
      LV_LOG_ERROR("LVGL already initialized! aborting.");
      return -1;
    }

#ifdef NEED_BOARDINIT
  boardctl(BOARDIOC_INIT, 0);
#endif

  lv_init();

  lv_nuttx_dsc_init(&info);

#ifdef CONFIG_LV_USE_NUTTX_LCD
  info.fb_path = "/dev/lcd0";
#endif

#ifdef CONFIG_INPUT_TOUCHSCREEN
  info.input_path = CONFIG_EXAMPLES_LVGL_WINDOW10_INPUT_DEVPATH;
#endif

  lv_nuttx_init(&info, &result);

  if (result.disp == NULL)
    {
      LV_LOG_ERROR("lvgl_window10 initialization failure!");
      return 1;
    }

  setPrefInt("bg_type", 1);
  setPrefInt("bg_img", 5);
  setPrefInt("lock_img", 2);
  setPrefInt("start_opa", 200);
  setPrefInt("nav_opa", 200);
  ui_init();
  lvgl_window10_update_time();

#ifdef CONFIG_LV_USE_NUTTX_LIBUV
  lvgl_window10_uv_loop(&ui_loop, &result);
#else
  while (1)
    {
      uint32_t idle;

      lvgl_window10_update_time();
      idle = lv_timer_handler();
      idle = idle ? idle : 1;
      usleep(idle * 1000);
    }
#endif

  lv_nuttx_deinit(&result);
  lv_deinit();

  return 0;
}
