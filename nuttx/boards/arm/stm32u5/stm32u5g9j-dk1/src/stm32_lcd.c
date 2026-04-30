/****************************************************************************
 * boards/arm/stm32u5/stm32u5g9j-dk1/src/stm32_lcd.c
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.  The
 * ASF licenses this file to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance with the
 * License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
 * WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.  See the
 * License for the specific language governing permissions and limitations
 * under the License.
 *
 ****************************************************************************/

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

#include <stdbool.h>
#include <errno.h>
#include <debug.h>

#include <nuttx/arch.h>
#include <nuttx/board.h>
#include <nuttx/lcd/lcd.h>
#include <nuttx/video/fb.h>

#include <arch/board/board.h>

#include "arm_internal.h"
#include "stm32_ltdc.h"
#include "stm32u5g9j-dk1.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Configuration ***********************************************************/

#ifdef CONFIG_STM32U5_LTDC

/* Display settings for typical MIPI DSI LCD panels on STM32U5G9J-DK1 (480x480) */

#ifndef BOARD_LTDC_WIDTH
#  define BOARD_LTDC_WIDTH          480
#endif

#ifndef BOARD_LTDC_HEIGHT
#  define BOARD_LTDC_HEIGHT         480
#endif

#ifndef BOARD_LTDC_HSYNC
#  define BOARD_LTDC_HSYNC          10
#endif

#ifndef BOARD_LTDC_VSYNC
#  define BOARD_LTDC_VSYNC          2
#endif

#ifndef BOARD_LTDC_HBP
#  define BOARD_LTDC_HBP            20
#endif

#ifndef BOARD_LTDC_HFP
#  define BOARD_LTDC_HFP            20
#endif

#ifndef BOARD_LTDC_VBP
#  define BOARD_LTDC_VBP            10
#endif

#ifndef BOARD_LTDC_VFP
#  define BOARD_LTDC_VFP            10
#endif

/* Display format and color depth */

#ifdef CONFIG_STM32U5_LTDC_FMT_16RGB
#  define FB_FMT              FB_FMT_RGB16_565
#  define LCD_PFM             LCD_PFM_RGB565
#elif defined(CONFIG_STM32U5_LTDC_FMT_24RGB)
#  define FB_FMT              FB_FMT_RGB24
#  define LCD_PFM             LCD_PFM_RGB888
#else
#  define FB_FMT              FB_FMT_RGB16_565
#  define LCD_PFM             LCD_PFM_RGB565
#endif

/* Display initialization structure */

static const struct stm32_ltdc_s g_lcdconfig =
{
  .width        = BOARD_LTDC_WIDTH,
  .height       = BOARD_LTDC_HEIGHT,
  .hsw          = BOARD_LTDC_HSYNC,
  .hbp          = BOARD_LTDC_HBP,
  .hfp          = BOARD_LTDC_HFP,
  .vsw          = BOARD_LTDC_VSYNC,
  .vbp          = BOARD_LTDC_VBP,
  .vfp          = BOARD_LTDC_VFP,
  .format       = LCD_PFM,
  .bpp          = 16,
  .fbsize       = STM32U5_LTDC_FBSIZE,
  .fbbase       = STM32U5_LTDC_FBBASE,
  .pwren        = 0,     /* No separate power enable pin on discovery kit */
  .vsynen       = 0,     /* VSYNC not connected separately */
  .hsyen        = 0,     /* HSYNC not connected separately */
  .denen        = 0,     /* DEN not connected separately */
  .pixclk       = BOARD_LTDC_PIXCLK,
  .pcpol        = 0,     /* Pixel clock polarity: falling edge */
  .depol        = 0,     /* Data enable polarity: active high */
  .vspol        = 0,     /* VSYNC polarity: active high */
  .hspol        = 0      /* HSYNC polarity: active high */
};

/* LCD driver structure */

static struct lcd_dev_s *g_lcddev = NULL;

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32_ltdcinitialize
 *
 * Description:
 *   Initialize the LCD driver and register the LCD device
 *
 ****************************************************************************/

int stm32_ltdcinitialize(void)
{
  int ret;

  /* Only initialize the LCD if it is not already initialized */

  if (g_lcddev == NULL)
    {
      /* Initialize the LTDC hardware */

      ret = stm32_ltdc_setup(&g_lcdconfig);
      if (ret < 0)
        {
          lcerr("ERROR: stm32_ltdc_setup failed: %d\n", ret);
          return ret;
        }

      /* Initialize the LCD based on the LTDC */

      g_lcddev = stm32_ltdc_lcdinitialize(0);
      if (g_lcddev == NULL)
        {
          lcerr("ERROR: stm32_ltdc_lcdinitialize failed\n");
          return -ENODEV;
        }

      /* Clear the display */

      g_lcddev->clear(g_lcddev);

      linfo("LCD initialized\n");
    }

  return OK;
}

/****************************************************************************
 * Name: up_fbinitialize
 *
 * Description:
 *   Initialize the framebuffer
 *
 ****************************************************************************/

int up_fbinitialize(int display)
{
  int ret;

  /* Initialize the LTDC */

  ret = stm32_ltdcinitialize();
  if (ret < 0)
    {
      return ret;
    }

  return OK;
}

/****************************************************************************
 * Name: up_fbgetvplane
 *
 * Description:
 *   Get the video plane structure
 *
 ****************************************************************************/

struct fb_vtable_s *up_fbgetvplane(int display, int vplane)
{
  /* Call the LTDC driver implementation */

  return stm32_ltdcgetvplane(vplane);
}

/****************************************************************************
 * Name: up_fbuninitialize
 *
 * Description:
 *   Uninitialize the framebuffer
 *
 ****************************************************************************/

void up_fbuninitialize(int display)
{
  /* Uninitialize the LTDC */

  stm32_ltdcuninitialize();
}

#endif /* CONFIG_STM32U5_LTDC */