/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_irq.c
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
#include <debug.h>

#include <nuttx/irq.h>
#include <nuttx/arch.h>

#include "nvic.h"
#include "arm_internal.h"
#include "chip.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32N6_IRQ_NEXTINTS  196

/****************************************************************************
 * Public Data
 ****************************************************************************/

const uint8_t g_irqmap[STM32N6_IRQ_NEXTINTS] =
{
  0,   1,   2,   3,   4,   5,   6,   8,
  9,   10,  11,  12,  13,  14,  15,  16,
  17,  18,  19,  20,  21,  22,  23,  24,
  25,  26,  27,  28,  29,  30,  31,  32,
  33,  34,  35,  36,  37,  38,  39,  40,
  41,  42,  43,  44,  45,  46,  47,  48,
  49,  50,  51,  52,  53,  54,  55,  56,
  57,  58,  59,  60,  61,  62,  63,  64,
  65,  66,  67,  68,  69,  70,  71,  72,
  73,  74,  75,  76,  77,  78,  79,  80,
  81,  82,  83,  84,  85,  86,  87,  88,
  89,  90,  91,  92,  93,  94,  95,  96,
  97,  98,  99,  100, 101, 102, 103, 104,
  105, 106, 107, 108, 109, 110, 111, 112,
  113, 114, 115, 116, 117, 118, 119, 120,
  121, 122, 123, 124, 125, 126, 127, 128,
  129, 130, 131, 132, 133, 134, 135, 136,
  137, 138, 139, 140, 141, 142, 143, 144,
  145, 146, 147, 148, 149, 150, 151, 152,
  153, 154, 155, 156, 157, 158, 159, 160,
  161, 162, 163, 164, 165, 166, 167, 168,
  169, 170, 171, 172, 173, 174, 175, 176,
  177, 178, 179, 180, 181, 182, 183, 184,
  185, 186, 187, 188, 189, 190, 191, 192,
  193, 194, 195
};

/****************************************************************************
 * Public Functions
 ****************************************************************************/

void up_irqinitialize(void)
{
  int i;

  for (i = 0; i < STM32N6_IRQ_NEXTINTS; i++)
    {
      putreg32(0, NVIC_IRQ_ENABLE(i + 16));
      putreg32(0xff, NVIC_IRQ_CLEAR(i + 16));
    }

  putreg32(0, NVIC_SYSTICK_CTRL);
  putreg32(0, NVIC_SYSTICK_CURRENT);

  up_enable_irq(STM32N6_IRQ_SYSTICK);

#ifndef CONFIG_SUPPRESS_INTERRUPTS
  __asm__ volatile("cpsie i");
#endif
}
