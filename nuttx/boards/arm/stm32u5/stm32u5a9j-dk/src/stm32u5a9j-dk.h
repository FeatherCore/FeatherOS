/****************************************************************************
 * boards/arm/stm32u5/stm32u5a9j-dk/src/stm32u5a9j-dk.h
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

#ifndef __BOARDS_ARM_STM32U5_STM32U5A9J_DK_SRC_STM32U5A9J_DK_H
#define __BOARDS_ARM_STM32U5_STM32U5A9J_DK_SRC_STM32U5A9J_DK_H_

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <nuttx/compiler.h>
#include <stdint.h>

#include "stm32_gpio.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Configuration ****************************************************/

#define HAVE_PROC             1
#define HAVE_RTC_DRIVER       1

#if !defined(CONFIG_FS_PROCFS)
#  undef HAVE_PROC
#endif

#if defined(HAVE_PROC) && defined(CONFIG_DISABLE_MOUNTPOINT)
#  warning Mountpoints disabled.  No procfs support
#  undef HAVE_PROC
#endif

/* Check if we can support the RTC driver */

#if !defined(CONFIG_RTC) || !defined(CONFIG_RTC_DRIVER)
#  undef HAVE_RTC_DRIVER
#endif

/* LCD and LTDC Configuration ************************************************/

/* STM32U5A9J-DK has a 5" LCD with MIPI DSI interface
 * Display: 720x1280 similar to STM32U5G9J-DK1
 */

#ifdef CONFIG_STM32U5_LTDC

/* LCD Timing Parameters (typical values, may need adjustment) */

#define BOARD_LTDC_WIDTH          720
#define BOARD_LTDC_HEIGHT         1280

/* HSYNC and VSYNC timing (typical values) */

#define BOARD_LTDC_HSYNC          5
#define BOARD_LTDC_VSYNC          5
#define BOARD_LTDC_HBP            10
#define BOARD_LTDC_HFP            10
#define BOARD_LTDC_VBP            10
#define BOARD_LTDC_VFP            10

/* LCD Pixel Clock (in Hz) - adjust based on display requirements */

#define BOARD_LTDC_PIXCLK         50000000

/* Frame buffer configuration */

#ifdef CONFIG_STM32U5_LTDC_FB_SIZE
#  define STM32U5_LTDC_FBSIZE     CONFIG_STM32U5_LTDC_FB_SIZE
#else
#  define STM32U5_LTDC_FBSIZE     (BOARD_LTDC_WIDTH * BOARD_LTDC_HEIGHT * 2)
#endif

#ifdef CONFIG_STM32U5_LTDC_FB_BASE
#  define STM32U5_LTDC_FBBAE     CONFIG_STM32U5_LTDC_FB_BASE
#else
/* Default to external SDRAM if available */
#  define STM32U5_LTDC_FBBAE     0x90000000
#endif

#endif /* CONFIG_STM32U5_LTDC */

/* Touchscreen Configuration *************************************************/

#ifdef CONFIG_INPUT_FT5X06

#define BOARD_FT5X06_I2C_BUS      5
#define BOARD_FT5X06_I2C_ADDR     0x38
#define BOARD_FT5X06_INT_PIN      (GPIO_PE8)
#define BOARD_FT5X06_RST_PIN      (GPIO_PD5)

#endif /* CONFIG_INPUT_FT5X06 */

/****************************************************************************
 * Public Types
 ****************************************************************************/

/****************************************************************************
 * Public Data
 ****************************************************************************/

#ifndef __ASSEMBLY__

/****************************************************************************
 * Public Function Declarations
 ****************************************************************************/

/****************************************************************************
 * Name: stm32_bringup
 *
 * Description:
 *   Perform architecture-specific initialization
 *
 *   CONFIG_BOARD_LATE_INITIALIZE=y :
 *     Called from board_late_initialize().
 *
 *   CONFIG_BOARD_LATE_INITIALIZE=n && CONFIG_BOARDCTL=y :
 *     Called from the NSH library
 *
 ****************************************************************************/

int stm32_bringup(void);

/****************************************************************************
 * Name: stm32_ltdcinitialize
 *
 * Description:
 *   Initialize the LTDC and LCD display
 *
 ****************************************************************************/

#ifdef CONFIG_STM32U5_LTDC
int stm32_ltdcinitialize(void);
#endif

/****************************************************************************
 * Name: stm32_touchscreen_initialize
 *
 * Description:
 *   Initialize the touchscreen
 *
 ****************************************************************************/

#ifdef CONFIG_INPUT_FT5X06
int stm32_touchscreen_initialize(void);
#endif

#endif /* __ASSEMBLY__ */
#endif /* __BOARDS_ARM_STM32U5_STM32U5A9J_DK_SRC_STM32U5A9J_DK_H */
