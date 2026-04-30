/****************************************************************************
 * boards/arm/ra8p/ek-ra8p1/src/ek-ra8p1_boot.c
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

#include <debug.h>

#include <nuttx/board.h>
#include <nuttx/arch.h>
#include <nuttx/clock.h>
#include <nuttx/spinlock.h>

#include <arch/board/board.h>

#include "board_config.h"

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_boardinitialize
 *
 * Description:
 *   This function is called by the architecture-specific up_boardinitialize()
 *   function.  It performs board-specific initialization.
 *
 ****************************************************************************/

void ra8p_boardinitialize(void)
{
  /* Configure GPIO pins for the board */

  /* Initialize the console UART (SCI2) pins */
  /* TODO: Implement GPIO pin configuration for UART */

#ifdef CONFIG_ARCH_LEDS
  /* Configure LEDs */

  board_userled_initialize();
#endif
}

/****************************************************************************
 * Name: board_late_initialize
 *
 * Description:
 *   If CONFIG_BOARD_LATE_INITIALIZE is selected, then an additional
 *   initialization call will be performed in the boot-up sequence to a
 *   function called board_late_initialize().  board_late_initialize() will
 *   be called immediately after up_initialize().  If CONFIG_BOARD_LATE_INITIALIZE
 *   is not selected, then the board_app_initialize() must be called from
 *   the IDLE thread at the beginning of the application.
 *
 ****************************************************************************/

#ifdef CONFIG_BOARD_LATE_INITIALIZE
void board_late_initialize(void)
{
  ra8p_bringup();
}
#endif