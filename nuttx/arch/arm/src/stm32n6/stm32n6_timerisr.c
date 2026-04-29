/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_timerisr.c
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

#include <stdint.h>

#include "arm_internal.h"
#include "chip.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define SYSTICK_CTRL_ENABLE    (1 << 0)
#define SYSTICK_CTRL_TICKINT   (1 << 1)
#define SYSTICK_CTRL_CLKSOURCE (1 << 2)
#define SYSTICK_CTRL_COUNTFLAG (1 << 16)

#define SYSTICK_LOAD_RELOAD    (1 << 0)

/****************************************************************************
 * Public Functions
 ****************************************************************************/

void arm_timer_initialize(void)
{
  uint32_t reload;

  putreg32(0, NVIC_SYSTICK_CTRL);

  reload = (STM32N6_HCLK_FREQUENCY / CONFIG_USEC_PER_TICK) - 1;
  putreg32(reload, NVIC_SYSTICK_RELOAD);

  putreg32(0, NVIC_SYSTICK_CURRENT);

  putreg32(SYSTICK_CTRL_CLKSOURCE | SYSTICK_CTRL_TICKINT | SYSTICK_CTRL_ENABLE,
           NVIC_SYSTICK_CTRL);
}

void up_timer_initialize(void)
{
  arm_timer_initialize();
}
