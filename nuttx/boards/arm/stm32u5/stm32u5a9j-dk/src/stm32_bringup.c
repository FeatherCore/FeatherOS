/****************************************************************************
 * boards/arm/stm32u5/stm32u5a9j-dk/src/stm32_bringup.c
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

#include <sys/mount.h>
#include <sys/types.h>
#include <debug.h>

#include <nuttx/input/buttons.h>
#include <nuttx/leds/userled.h>
#include <nuttx/spi/spi_transfer.h>
#include <nuttx/board.h>
#include <nuttx/clock.h>

#include "stm32u5a9j-dk.h"

#include <arch/board/board.h>

#include <stm32_spi.h>

#if defined(CONFIG_I2C)

#include "stm32_i2c.h"
struct i2c_master_s *i2c1_m;
struct i2c_master_s *i2c2_m;
#  ifdef CONFIG_RTC_DSXXXX
#    include <nuttx/timers/rtc.h>
#    include <nuttx/timers/ds3231.h>
#  endif /* CONFIG_RTC_DSXXXX */

#endif /* CONFIG_I2C */

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/
#define DEVNO_ZERO   0
#define DEVNO_ONE    1
#define DEVNO_TWO    2

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Should not be here, but there is a bug in clock_initalize logic that
 * prevents external RTC to be used when no internal RTC !
 * **************************************************************************/
#if defined(CONFIG_RTC) && defined(CONFIG_RTC_EXTERNAL)
int up_rtc_initialize(void)
{
  return OK;
}
#endif

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

int stm32_bringup(void)
{
  int ret;

#ifdef CONFIG_FS_PROCFS
  /* Mount the procfs file system */

  ret = mount(NULL, "/proc", "procfs", 0, NULL);
  if (ret < 0)
    {
      ferr("ERROR: Failed to mount procfs at /proc: %d\n", ret);
    }
#endif

#ifdef CONFIG_STM32U5_LTDC
  /* Initialize the LCD */

  ret = stm32_ltdcinitialize();
  if (ret < 0)
    {
      ferr("ERROR: Failed to initialize LCD: %d\n", ret);
    }
#endif

#ifdef CONFIG_INPUT_FT5X06
  /* Initialize the touchscreen */

  ret = stm32_touchscreen_initialize();
  if (ret < 0)
    {
      ferr("ERROR: Failed to initialize touchscreen: %d\n", ret);
    }
#endif

  UNUSED(ret);
  return OK;
}
