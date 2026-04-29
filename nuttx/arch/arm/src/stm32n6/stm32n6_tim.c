/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_tim.c
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

#include <sys/types.h>
#include <stdint.h>
#include <stdbool.h>
#include <debug.h>
#include <errno.h>

#include "arm_internal.h"
#include "chip.h"
#include "hardware/stm32n6_memorymap.h"
#include "stm32n6_tim.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_TIM_CR1_OFFSET        0x00
#define STM32_TIM_CR2_OFFSET        0x04
#define STM32_TIM_DIER_OFFSET       0x0C
#define STM32_TIM_SR_OFFSET         0x10
#define STM32_TIM_EGR_OFFSET        0x14
#define STM32_TIM_CCMR1_OFFSET      0x18
#define STM32_TIM_CCMR2_OFFSET      0x1C
#define STM32_TIM_CCER_OFFSET       0x20
#define STM32_TIM_CNT_OFFSET        0x24
#define STM32_TIM_PSC_OFFSET        0x28
#define STM32_TIM_ARR_OFFSET        0x2C
#define STM32_TIM_RCR_OFFSET        0x30
#define STM32_TIM_CCR1_OFFSET       0x34
#define STM32_TIM_CCR2_OFFSET       0x38
#define STM32_TIM_CCR3_OFFSET       0x3C
#define STM32_TIM_CCR4_OFFSET       0x40
#define STM32_TIM_BDTR_OFFSET       0x44
#define STM32_TIM_DCR_OFFSET        0x48
#define STM32_TIM_DMAR_OFFSET       0x4C

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static inline void stm32n6_tim_putreg(uintptr_t timbase,
                                      uint32_t offset, uint32_t value)
{
  putreg32(value, timbase + offset);
}

static inline uint32_t stm32n6_tim_getreg(uintptr_t timbase,
                                          uint32_t offset)
{
  return getreg32(timbase + offset);
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

int stm32n6_tim_initialize(uintptr_t timbase, uint32_t frequency,
                           uint32_t period)
{
  uint32_t prescaler;
  uint32_t clock;

  stm32n6_tim_disable(timbase);

  clock = STM32N6_APB1_TIM_FREQUENCY;
  prescaler = (clock / frequency) - 1;

  stm32n6_tim_putreg(timbase, STM32_TIM_PSC_OFFSET, prescaler);
  stm32n6_tim_putreg(timbase, STM32_TIM_ARR_OFFSET, period - 1);
  stm32n6_tim_putreg(timbase, STM32_TIM_CNT_OFFSET, 0);

  stm32n6_tim_putreg(timbase, STM32_TIM_EGR_OFFSET, TIM_EGR_UG);

  return OK;
}

void stm32n6_tim_enable(uintptr_t timbase)
{
  uint32_t regval;

  regval = stm32n6_tim_getreg(timbase, STM32_TIM_CR1_OFFSET);
  regval |= TIM_CR1_CEN;
  stm32n6_tim_putreg(timbase, STM32_TIM_CR1_OFFSET, regval);
}

void stm32n6_tim_disable(uintptr_t timbase)
{
  uint32_t regval;

  regval = stm32n6_tim_getreg(timbase, STM32_TIM_CR1_OFFSET);
  regval &= ~TIM_CR1_CEN;
  stm32n6_tim_putreg(timbase, STM32_TIM_CR1_OFFSET, regval);
}

void stm32n6_tim_set_period(uintptr_t timbase, uint32_t period)
{
  stm32n6_tim_putreg(timbase, STM32_TIM_ARR_OFFSET, period - 1);
}

uint32_t stm32n6_tim_get_counter(uintptr_t timbase)
{
  return stm32n6_tim_getreg(timbase, STM32_TIM_CNT_OFFSET);
}

void stm32n6_tim_clear_interrupt(uintptr_t timbase)
{
  stm32n6_tim_putreg(timbase, STM32_TIM_SR_OFFSET, 0);
}