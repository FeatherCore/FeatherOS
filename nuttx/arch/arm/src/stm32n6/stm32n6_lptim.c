/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_lptim.c
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
#include "hardware/stm32n6_rcc.h"
#include "stm32n6_lptim.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_LPTIM_ISR_OFFSET       0x00
#define STM32_LPTIM_ICR_OFFSET       0x04
#define STM32_LPTIM_IER_OFFSET       0x08
#define STM32_LPTIM_CFGR_OFFSET     0x0C
#define STM32_LPTIM_CR_OFFSET        0x10
#define STM32_LPTIM_ARR_OFFSET      0x14
#define STM32_LPTIM_CNT_OFFSET      0x18
#define STM32_LPTIM_CMP_OFFSET      0x1C

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static inline void stm32n6_lptim_putreg(uintptr_t lptimbase,
                                      uint32_t offset, uint32_t value)
{
  putreg32(value, lptimbase + offset);
}

static inline uint32_t stm32n6_lptim_getreg(uintptr_t lptimbase,
                                          uint32_t offset)
{
  return getreg32(lptimbase + offset);
}

static void stm32n6_lptim_enable_clock(uintptr_t lptimbase)
{
  uint32_t regval;

  regval = getreg32(STM32_RCC_APB4ENR);
  if (lptimbase == STM32_LPTIM1_BASE)
    {
      regval |= (1 << 8);
    }
  else if (lptimbase == STM32_LPTIM2_BASE)
    {
      regval |= (1 << 9);
    }
  else if (lptimbase == STM32_LPTIM3_BASE)
    {
      regval |= (1 << 10);
    }
  putreg32(regval, STM32_RCC_APB4ENR);
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

int stm32n6_lptim_init(uintptr_t lptimbase, uint32_t frequency, uint8_t prescaler)
{
  uint32_t regval;

  stm32n6_lptim_enable_clock(lptimbase);

  stm32n6_lptim_disable(lptimbase);

  regval = LPTIM_CFGR_CKSEL;
  regval |= (prescaler << LPTIM_CFGR_PRESC_SHIFT) & LPTIM_CFGR_PRESC_MASK;
  stm32n6_lptim_putreg(lptimbase, STM32_LPTIM_CFGR_OFFSET, regval);

  return OK;
}

void stm32n6_lptim_enable(uintptr_t lptimbase)
{
  uint32_t regval;

  regval = stm32n6_lptim_getreg(lptimbase, STM32_LPTIM_CR_OFFSET);
  regval |= LPTIM_CR_ENABLE;
  stm32n6_lptim_putreg(lptimbase, STM32_LPTIM_CR_OFFSET, regval);
}

void stm32n6_lptim_disable(uintptr_t lptimbase)
{
  uint32_t regval;

  regval = stm32n6_lptim_getreg(lptimbase, STM32_LPTIM_CR_OFFSET);
  regval &= ~LPTIM_CR_ENABLE;
  stm32n6_lptim_putreg(lptimbase, STM32_LPTIM_CR_OFFSET, regval);
}

void stm32n6_lptim_setautoreload(uintptr_t lptimbase, uint32_t arr)
{
  stm32n6_lptim_putreg(lptimbase, STM32_LPTIM_ARR_OFFSET, arr);
}

uint32_t stm32n6_lptim_getautoreload(uintptr_t lptimbase)
{
  return stm32n6_lptim_getreg(lptimbase, STM32_LPTIM_ARR_OFFSET);
}

uint32_t stm32n6_lptim_getcounter(uintptr_t lptimbase)
{
  return stm32n6_lptim_getreg(lptimbase, STM32_LPTIM_CNT_OFFSET);
}

void stm32n6_lptim_start(uintptr_t lptimbase)
{
  uint32_t regval;

  regval = stm32n6_lptim_getreg(lptimbase, STM32_LPTIM_CR_OFFSET);
  regval |= LPTIM_CR_ENABLE;
  stm32n6_lptim_putreg(lptimbase, STM32_LPTIM_CR_OFFSET, regval);
}

void stm32n6_lptim_stop(uintptr_t lptimbase)
{
  uint32_t regval;

  regval = stm32n6_lptim_getreg(lptimbase, STM32_LPTIM_CR_OFFSET);
  regval &= ~LPTIM_CR_ENABLE;
  stm32n6_lptim_putreg(lptimbase, STM32_LPTIM_CR_OFFSET, regval);
}

void stm32n6_lptim_int_enable(uintptr_t lptimbase, uint32_t sources)
{
  uint32_t regval;

  regval = stm32n6_lptim_getreg(lptimbase, STM32_LPTIM_IER_OFFSET);
  regval |= sources;
  stm32n6_lptim_putreg(lptimbase, STM32_LPTIM_IER_OFFSET, regval);
}

void stm32n6_lptim_int_disable(uintptr_t lptimbase, uint32_t sources)
{
  uint32_t regval;

  regval = stm32n6_lptim_getreg(lptimbase, STM32_LPTIM_IER_OFFSET);
  regval &= ~sources;
  stm32n6_lptim_putreg(lptimbase, STM32_LPTIM_IER_OFFSET, regval);
}

bool stm32n6_lptim_int_status(uintptr_t lptimbase, uint32_t source)
{
  uint32_t regval;

  regval = stm32n6_lptim_getreg(lptimbase, STM32_LPTIM_ISR_OFFSET);
  return (regval & source) != 0;
}

void stm32n6_lptim_int_ack(uintptr_t lptimbase, uint32_t sources)
{
  stm32n6_lptim_putreg(lptimbase, STM32_LPTIM_ICR_OFFSET, sources);
}
