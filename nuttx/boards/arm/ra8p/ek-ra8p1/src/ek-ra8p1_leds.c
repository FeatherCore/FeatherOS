/****************************************************************************
 * boards/arm/ra8p/ek-ra8p1/src/ek-ra8p1_leds.c
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

#include <stdint.h>
#include <stdbool.h>
#include <debug.h>

#include <nuttx/board.h>
#include <nuttx/leds/userled.h>

#include "board_config.h"

#ifdef CONFIG_USERLED_LOWER

/****************************************************************************
 * Private Data
 ****************************************************************************/

/* LED GPIO configuration */

static const gpio_pinset_t g_led_pins[BOARD_NLEDS] =
{
  GPIO_LED1, GPIO_LED2, GPIO_LED3
};

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: board_userled_initialize
 *
 * Description:
 *   Initialize the LED GPIO pins
 *
 ****************************************************************************/

uint32_t board_userled_initialize(void)
{
  int i;

  /* Configure LED GPIOs as outputs */

  for (i = 0; i < BOARD_NLEDS; i++)
    {
      ra8p_gpio_config(g_led_pins[i]);
    }

  return BOARD_NLEDS;
}

/****************************************************************************
 * Name: board_userled
 *
 * Description:
 *   Set the state of a single LED
 *
 ****************************************************************************/

void board_userled(int led, bool ledon)
{
  if ((unsigned)led < BOARD_NLEDS)
    {
      ra8p_gpio_write(g_led_pins[led], ledon);
    }
}

/****************************************************************************
 * Name: board_userled_all
 *
 * Description:
 *   Set the state of all LEDs
 *
 ****************************************************************************/

void board_userled_all(uint32_t ledset)
{
  int i;

  for (i = 0; i < BOARD_NLEDS; i++)
    {
      ra8p_gpio_write(g_led_pins[i], (ledset & (1 << i)) != 0);
    }
}

#endif /* CONFIG_USERLED_LOWER */