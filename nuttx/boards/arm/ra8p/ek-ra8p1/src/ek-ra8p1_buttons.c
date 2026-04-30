/****************************************************************************
 * boards/arm/ra8p/ek-ra8p1/src/ek-ra8p1_buttons.c
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
#include <errno.h>

#include <nuttx/board.h>
#include <nuttx/irq.h>

#include "board_config.h"

#ifdef CONFIG_USERSWITCH

/****************************************************************************
 * Private Data
 ****************************************************************************/

/* Button GPIO configuration */

static const gpio_pinset_t g_button_pins[NUM_BUTTONS] =
{
  GPIO_SW1, GPIO_SW2
};

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: board_userswitch_initialize
 *
 * Description:
 *   Initialize the button GPIO pins
 *
 ****************************************************************************/

uint32_t board_userswitch_initialize(void)
{
  int i;

  /* Configure button GPIOs as inputs with pull-ups */

  for (i = 0; i < NUM_BUTTONS; i++)
    {
      ra8p_gpio_config(g_button_pins[i]);
    }

  return NUM_BUTTONS;
}

/****************************************************************************
 * Name: board_userswitch
 *
 * Description:
 *   Get the state of a single button
 *
 ****************************************************************************/

bool board_userswitch(int id)
{
  bool ret = false;

  if ((unsigned)id < NUM_BUTTONS)
    {
      /* Buttons are active low (pressed = 0) */

      ret = !ra8p_gpio_read(g_button_pins[id]);
    }

  return ret;
}

/****************************************************************************
 * Name: board_userswitch_all
 *
 * Description:
 *   Get the state of all buttons
 *
 ****************************************************************************/

uint32_t board_userswitch_all(void)
{
  uint32_t ret = 0;
  int i;

  for (i = 0; i < NUM_BUTTONS; i++)
    {
      /* Buttons are active low (pressed = 0) */

      if (!ra8p_gpio_read(g_button_pins[i]))
        {
          ret |= (1 << i);
        }
    }

  return ret;
}

#endif /* CONFIG_USERSWITCH */