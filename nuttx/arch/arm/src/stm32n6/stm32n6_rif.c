/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_rif.c
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

#define RIFSC_BASE                 (STM32N6_PERIPH_BASE + 0x0030000)

#define RIFSC_RIMC_CR_OFFSET       0x00
#define RIFSC_RIMC_CR_CID1         (1 << 0)
#define RIFSC_RIMC_CR_SEC          (1 << 16)
#define RIFSC_RIMC_CR_PRIV         (1 << 17)

#define RIFSC_RISC_CR_OFFSET       0x00
#define RIFSC_RISC_CR_CID1         (1 << 0)
#define RIFSC_RISC_CR_SEC          (1 << 16)
#define RIFSC_RISC_CR_PRIV         (1 << 17)

/****************************************************************************
 * Public Functions
 ****************************************************************************/

#ifdef CONFIG_STM32N6_RIF

#ifdef CONFIG_STM32N6_RIF_OPEN

void stm32n6_rif_configure_master(uint32_t master_index, uint32_t config)
{
  uintptr_t base = RIFSC_BASE + 0x1000 + (master_index * 0x10);
  putreg32(config, base + RIFSC_RIMC_CR_OFFSET);
}

void stm32n6_rif_configure_slave(uint32_t slave_index, uint32_t config)
{
  uintptr_t base = RIFSC_BASE + 0x2000 + (slave_index * 0x10);
  putreg32(config, base + RIFSC_RISC_CR_OFFSET);
}

void stm32n6_rif_open(void)
{
  uint32_t regval;

  regval = getreg32(STM32_RCC_AHB4ENR);
  regval |= (1 << 24);
  putreg32(regval, STM32_RCC_AHB4ENR);

  stm32n6_rif_configure_master(0, RIFSC_RIMC_CR_CID1 | RIFSC_RIMC_CR_SEC | RIFSC_RIMC_CR_PRIV);
}

#endif /* CONFIG_STM32N6_RIF_OPEN */

#endif /* CONFIG_STM32N6_RIF */