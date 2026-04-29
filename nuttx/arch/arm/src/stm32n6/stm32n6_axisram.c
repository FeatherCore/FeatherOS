/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_axisram.c
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

#define RAMCFG_CR_OFFSET           0x00
#define RAMCFG_CR_EN               (1 << 0)

#define RAMCFG_SR_OFFSET           0x04

/****************************************************************************
 * Public Functions
 ****************************************************************************/

#if defined(CONFIG_STM32N6_AXISRAM3) || defined(CONFIG_STM32N6_AXISRAM4) || \
    defined(CONFIG_STM32N6_AXISRAM5) || defined(CONFIG_STM32N6_AXISRAM6)

void stm32n6_axisram_enable(void)
{
  uint32_t regval;

  regval = getreg32(STM32_RCC_AHB2ENR);
  regval |= RCC_AHB2ENR_RAMCFGEN;
  putreg32(regval, STM32_RCC_AHB2ENR);

#ifdef CONFIG_STM32N6_AXISRAM3
  putreg32(RAMCFG_CR_EN, STM32_RAMCFG_SRAM3_BASE + RAMCFG_CR_OFFSET);
#endif

#ifdef CONFIG_STM32N6_AXISRAM4
  putreg32(RAMCFG_CR_EN, STM32_RAMCFG_SRAM4_BASE + RAMCFG_CR_OFFSET);
#endif

#ifdef CONFIG_STM32N6_AXISRAM5
  putreg32(RAMCFG_CR_EN, STM32_RAMCFG_SRAM5_BASE + RAMCFG_CR_OFFSET);
#endif

#ifdef CONFIG_STM32N6_AXISRAM6
  putreg32(RAMCFG_CR_EN, STM32_RAMCFG_SRAM6_BASE + RAMCFG_CR_OFFSET);
#endif
}

#endif