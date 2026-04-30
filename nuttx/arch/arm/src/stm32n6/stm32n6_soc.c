/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_soc.c
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
#include "hardware/stm32n6_pwr.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define PWR_CR2_VDDIO2EN    (1 << 0)
#define PWR_CR2_VDDIO3EN    (1 << 1)
#define PWR_CR2_VDDIO4EN    (1 << 2)
#define PWR_CR2_VDDIO5EN    (1 << 3)

/****************************************************************************
 * Public Functions
 ****************************************************************************/

void stm32n6_soc_early_init(void)
{
  uint32_t regval;

  regval = getreg32(STM32_RCC_AHB4ENR);
  regval |= (1 << 28);
  putreg32(regval, STM32_RCC_AHB4ENR);

  regval = getreg32(STM32_PWR_CR2);
  regval |= PWR_CR2_VDDIO2EN | PWR_CR2_VDDIO3EN |
            PWR_CR2_VDDIO4EN | PWR_CR2_VDDIO5EN;
  putreg32(regval, STM32_PWR_CR2);

#if defined(CONFIG_STM32N6_RIF) && defined(CONFIG_STM32N6_RIF_OPEN)
#endif
}
