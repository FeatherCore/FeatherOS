/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_npu.c
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
#include "hardware/stm32n6_memorymap.h"
#include "hardware/stm32n6_rcc.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define NPU_CR_OFFSET              0x00
#define NPU_CR_ENABLE              (1 << 0)
#define NPU_CR_RESET               (1 << 1)

#define NPU_CACHE_CR_OFFSET        0x00
#define NPU_CACHE_CR_EN            (1 << 0)

/****************************************************************************
 * Public Functions
 ****************************************************************************/

#ifdef CONFIG_STM32N6_NPU

void stm32n6_npu_enable(void)
{
  uint32_t regval;

  regval = getreg32(STM32_RCC_AHB5ENR);
  regval |= RCC_AHB5ENR_NPUEN | RCC_AHB5ENR_NPUCACHEEN;
  putreg32(regval, STM32_RCC_AHB5ENR);

#ifdef CONFIG_STM32N6_NPU_CACHE
  putreg32(NPU_CACHE_CR_EN, STM32_NPU_CACHE_BASE + NPU_CACHE_CR_OFFSET);
#endif

  putreg32(NPU_CR_ENABLE, STM32_NPU_BASE + NPU_CR_OFFSET);
}

void stm32n6_npu_reset(void)
{
  uint32_t regval;

  regval = getreg32(STM32_NPU_BASE + NPU_CR_OFFSET);
  regval |= NPU_CR_RESET;
  putreg32(regval, STM32_NPU_BASE + NPU_CR_OFFSET);

  while ((getreg32(STM32_NPU_BASE + NPU_CR_OFFSET) & NPU_CR_RESET) != 0);
}

void stm32n6_npu_disable(void)
{
  uint32_t regval;

  regval = getreg32(STM32_NPU_BASE + NPU_CR_OFFSET);
  regval &= ~NPU_CR_ENABLE;
  putreg32(regval, STM32_NPU_BASE + NPU_CR_OFFSET);

  regval = getreg32(STM32_RCC_AHB5ENR);
  regval &= ~(RCC_AHB5ENR_NPUEN | RCC_AHB5ENR_NPUCACHEEN);
  putreg32(regval, STM32_RCC_AHB5ENR);
}

#endif /* CONFIG_STM32N6_NPU */