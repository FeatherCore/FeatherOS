/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_clockconfig.c
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

#include <arch/board/board.h>

#include "arm_internal.h"
#include "chip.h"
#include "hardware/stm32n6_rcc.h"
#include "hardware/stm32n6_pwr.h"
#include "stm32n6_start.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define RCC_CR_HSIRDY       (1 << 2)
#define RCC_CR_HSION        (1 << 0)
#define RCC_CR_HSERDY       (1 << 17)
#define RCC_CR_HSEON        (1 << 16)
#define RCC_CR_PLL1RDY      (1 << 25)
#define RCC_CR_PLL1ON       (1 << 24)

#define RCC_CFGR_SW_HSI     0
#define RCC_CFGR_SW_HSE     1
#define RCC_CFGR_SW_PLL1    2
#define RCC_CFGR_SWS_SHIFT  2
#define RCC_CFGR_SWS_MASK   (3 << RCC_CFGR_SWS_SHIFT)

#define TIMEOUT_VALUE       0xfffff

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static inline void stm32n6_enable_hsi(void)
{
  uint32_t regval;
  int timeout;

  regval = getreg32(STM32_RCC_CR);
  if ((regval & RCC_CR_HSIRDY) == 0)
    {
      regval |= RCC_CR_HSION;
      putreg32(regval, STM32_RCC_CR);

      timeout = TIMEOUT_VALUE;
      while (((getreg32(STM32_RCC_CR) & RCC_CR_HSIRDY) == 0) &&
             (timeout-- > 0));
    }
}

#ifdef CONFIG_STM32N6_HSE_FREQUENCY
static inline void stm32n6_enable_hse(void)
{
  uint32_t regval;
  int timeout;

  regval = getreg32(STM32_RCC_CR);
  if ((regval & RCC_CR_HSERDY) == 0)
    {
      regval |= RCC_CR_HSEON;
#ifdef CONFIG_STM32N6_HSE_BYPASS
      regval |= (1 << 18);
#endif
      putreg32(regval, STM32_RCC_CR);

      timeout = TIMEOUT_VALUE;
      while (((getreg32(STM32_RCC_CR) & RCC_CR_HSERDY) == 0) &&
             (timeout-- > 0));
    }
}
#endif

static inline void stm32n6_set_voltage_scale(void)
{
  uint32_t regval;

  regval = getreg32(STM32_PWR_CR1);
  regval &= ~(3 << 14);
#if CONFIG_STM32N6_VOS_SCALE == 0
  regval |= (3 << 14);
#elif CONFIG_STM32N6_VOS_SCALE == 1
  regval |= (2 << 14);
#elif CONFIG_STM32N6_VOS_SCALE == 2
  regval |= (1 << 14);
#endif
  putreg32(regval, STM32_PWR_CR1);
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

void stm32n6_clockconfig(void)
{
  uint32_t regval;

  stm32n6_enable_hsi();

  regval = getreg32(STM32_RCC_AHB4ENR);
  regval |= (1 << 0);
  putreg32(regval, STM32_RCC_AHB4ENR);

  regval = getreg32(STM32_RCC_AHB4ENR);
  regval |= (1 << 28);
  putreg32(regval, STM32_RCC_AHB4ENR);

  stm32n6_set_voltage_scale();

#ifdef CONFIG_STM32N6_HSE_FREQUENCY
  stm32n6_enable_hse();
#endif

  regval = getreg32(STM32_RCC_CFGR1);
  regval &= ~RCC_CFGR_SWS_MASK;
  regval |= RCC_CFGR_SW_HSI;
  putreg32(regval, STM32_RCC_CFGR1);

  SystemCoreClock = STM32N6_SYSCLK_FREQUENCY;
}
