/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_wdg.c
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
#include <errno.h>

#include "arm_internal.h"
#include "chip.h"
#include "hardware/stm32n6_memorymap.h"
#include "stm32n6_wdg.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define LSI_FREQUENCY           32000

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static inline void stm32n6_wdg_putreg(uintptr_t base, uint32_t offset, uint32_t value)
{
  putreg32(value, base + offset);
}

static inline uint32_t stm32n6_wdg_getreg(uintptr_t base, uint32_t offset)
{
  return getreg32(base + offset);
}

/****************************************************************************
 * Public Functions - IWDG
 ****************************************************************************/

int stm32n6_iwdg_init(uint32_t timeout_ms)
{
  uint32_t reload;
  uint8_t prescaler;
  uint32_t timeout;

  reload = (LSI_FREQUENCY * timeout_ms) / 1000;

  prescaler = IWDG_PR_DIV_256;
  timeout = (reload * 256) / (LSI_FREQUENCY / 1000);

  while (timeout > timeout_ms && prescaler > IWDG_PR_DIV_4)
    {
      prescaler--;
      timeout = (reload * (1 << (prescaler + 2)) / (LSI_FREQUENCY / 1000);
    }

  stm32n6_wdg_putreg(STM32_IWDG_BASE, IWDG_KR_OFFSET, IWDG_KR_UNLOCK);
  stm32n6_wdg_putreg(STM32_IWDG_BASE, IWDG_PR_OFFSET, prescaler);
  stm32n6_wdg_putreg(STM32_IWDG_BASE, IWDG_RLR_OFFSET, reload & 0xfff);

  while ((stm32n6_wdg_getreg(STM32_IWDG_BASE, IWDG_SR_OFFSET) & IWDG_SR_PVU) != 0);

  return OK;
}

void stm32n6_iwdg_enable(void)
{
  stm32n6_wdg_putreg(STM32_IWDG_BASE, IWDG_KR_OFFSET, IWDG_KR_START);
}

void stm32n6_iwdg_feed(void)
{
  stm32n6_wdg_putreg(STM32_IWDG_BASE, IWDG_KR_OFFSET, IWDG_KR_UNLOCK);
  stm32n6_wdg_putreg(STM32_IWDG_BASE, IWDG_KR_OFFSET, IWDG_KR_RELOAD);
}

void stm32n6_iwdg_get_timeout(uint32_t *timeout_ms)
{
  uint8_t prescaler;
  uint32_t reload;

  prescaler = stm32n6_wdg_getreg(STM32_IWDG_BASE, IWDG_PR_OFFSET) & 0x7;
  reload = stm32n6_wdg_getreg(STM32_IWDG_BASE, IWDG_RLR_OFFSET) & 0xfff;

  *timeout_ms = (reload * (1 << (prescaler + 2)) * 1000 / LSI_FREQUENCY);
}

/****************************************************************************
 * Public Functions - WWDG
 ****************************************************************************/

int stm32n6_wwdg_init(uint32_t timeout_ms, uint32_t window)
{
  uint32_t pclk1;
  uint32_t prescaler;
  uint32_t reload;
  uint32_t counter;
  uint32_t window_reg;

  pclk1 = STM32N6_PCLK1_FREQUENCY;

  prescaler = 0;
  reload = (pclk1 * timeout_ms) / (1000 * 8);

  while (reload > 0x40 && prescaler < 3)
    {
      prescaler++;
      reload = (pclk1 * timeout_ms) / (1000 * (8 << prescaler));
    }

  counter = reload & WWDG_CR_T;

  window_reg = (window * (reload + 1)) / timeout_ms;
  window_reg &= WWDG_CFR_W;

  stm32n6_wdg_putreg(STM32_WWDG_BASE, WWDG_CFR_OFFSET,
                     (prescaler << 7) | window_reg | 0x60);

  stm32n6_wdg_putreg(STM32_WWDG_BASE, WWDG_CR_OFFSET, counter);

  return OK;
}

void stm32n6_wwdg_enable(void)
{
  uint32_t regval;

  regval = stm32n6_wdg_getreg(STM32_WWDG_BASE, WWDG_CR_OFFSET);
  regval |= WWDG_CR_WDGA;
  stm32n6_wdg_putreg(STM32_WWDG_BASE, WWDG_CR_OFFSET, regval);
}

void stm32n6_wwdg_feed(void)
{
  uint32_t regval;

  regval = stm32n6_wdg_getreg(STM32_WWDG_BASE, WWDG_CR_OFFSET);
  regval &= WWDG_CR_T;
  stm32n6_wdg_putreg(STM32_WWDG_BASE, WWDG_CR_OFFSET, regval);
}
