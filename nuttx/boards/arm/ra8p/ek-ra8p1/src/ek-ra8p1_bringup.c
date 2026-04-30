/****************************************************************************
 * boards/arm/ra8p/ek-ra8p1/src/ek-ra8p1_bringup.c
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

#include <stdio.h>
#include <syslog.h>
#include <errno.h>

#include <nuttx/board.h>
#include <nuttx/spinlock.h>
#include <nuttx/leds/userled.h>
#include <nuttx/drivers/userswitch.h>

#include "board_config.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Configuration ************************************************************/

#ifdef CONFIG_USERLED_LOWER
#  define HAVE_LEDS 1
#endif

#ifdef CONFIG_USERSWITCH
#  define HAVE_BUTTONS 1
#endif

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_bringup
 *
 * Description:
 *   Perform architecture-specific initialization
 *
 ****************************************************************************/

int ra8p_bringup(void)
{
  int ret = OK;

#ifdef HAVE_LEDS
  /* Initialize USER LED GPIOs */

  board_userled_initialize();

  /* Register the LED driver */

  ret = userled_lower_initialize(LED_DRIVER_PATH);
  if (ret < 0)
    {
      syslog(LOG_ERR, "ERROR: userled_lower_initialize() failed: %d\n", ret);
      return ret;
    }
#endif

#ifdef HAVE_BUTTONS
  /* Initialize button GPIOs */

  board_userswitch_initialize();

  /* Register the BUTTON driver */

  ret = userswitch_lower_initialize("/dev/buttons");
  if (ret < 0)
    {
      syslog(LOG_ERR, "ERROR: userswitch_lower_initialize() failed: %d\n", ret);
      return ret;
    }
#endif

  /* TODO: Initialize other board-specific peripherals (SPI, I2C, etc.) */

  UNUSED(ret);
  return OK;
}