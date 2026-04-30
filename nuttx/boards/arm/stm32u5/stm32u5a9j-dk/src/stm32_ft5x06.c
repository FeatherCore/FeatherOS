/****************************************************************************
 * boards/arm/stm32u5/stm32u5a9j-dk/src/stm32_ft5x06.c
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
#include <stdio.h>
#include <assert.h>
#include <debug.h>
#include <errno.h>

#include <nuttx/board.h>
#include <nuttx/i2c/i2c_master.h>
#include <nuttx/input/touchscreen.h>
#include <nuttx/input/ft5x06.h>

#include <nuttx/irq.h>

#include "stm32.h"
#include "stm32u5a9j-dk.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Configuration ************************************************************/

#ifdef CONFIG_INPUT_FT5X06

#ifndef CONFIG_INPUT
#  error "FT5X06 support requires CONFIG_INPUT"
#endif

#ifndef CONFIG_STM32U5_I2C5
#  error "FT5X06 support requires CONFIG_STM32U5_I2C5"
#endif

#ifndef CONFIG_FT5X06_I2C
#  error "Only the FT5X06 I2C interface is supported"
#endif

#ifndef CONFIG_FT5X06_FREQUENCY
#  define CONFIG_FT5X06_FREQUENCY 400000
#endif

#ifndef CONFIG_FT5X06_I2CDEV
#  define CONFIG_FT5X06_I2CDEV 5
#endif

#if CONFIG_FT5X06_I2CDEV != 5
#  error "CONFIG_FT5X06_I2CDEV must be five"
#endif

#ifndef CONFIG_FT5X06_DEVMINOR
#  define CONFIG_FT5X06_DEVMINOR 0
#endif

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ft5x06_config_s g_ft5x06_config =
{
  .frequency    = CONFIG_FT5X06_FREQUENCY,
  .address      = BOARD_FT5X06_I2C_ADDR,
  .int_pin      = BOARD_FT5X06_INT_PIN,
  .rst_pin      = BOARD_FT5X06_RST_PIN,
};

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32_touchscreen_initialize
 *
 * Description:
 *   Initialize and register the FT5X06 touchscreen driver.
 *
 ****************************************************************************/

int stm32_touchscreen_initialize(void)
{
  struct i2c_master_s *i2c;
  int ret;

  i2cinfo("Initializing FT5X06 touchscreen\n");

  /* Initialize I2C */

  i2c = stm32_i2cbus_initialize(BOARD_FT5X06_I2C_BUS);
  if (i2c == NULL)
    {
      i2cerr("ERROR: Failed to initialize I2C%d\n", BOARD_FT5X06_I2C_BUS);
      return -ENODEV;
    }

  /* Register the FT5X06 touchscreen driver */

  ret = ft5x06_register(i2c, &g_ft5x06_config, CONFIG_FT5X06_DEVMINOR);
  if (ret < 0)
    {
      i2cerr("ERROR: Failed to register FT5X06 driver: %d\n", ret);
      return ret;
    }

  i2cinfo("FT5X06 touchscreen initialized\n");
  return OK;
}

#endif /* CONFIG_INPUT_FT5X06 */