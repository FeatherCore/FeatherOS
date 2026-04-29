/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_pwr.c
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
#include <debug.h>

#include "arm_internal.h"
#include "chip.h"
#include "hardware/stm32n6_memorymap.h"
#include "stm32n6_pwr.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_PWR_CR1_OFFSET        0x00
#define STM32_PWR_CSR1_OFFSET       0x04
#define STM32_PWR_CR2_OFFSET        0x08
#define STM32_PWR_CR3_OFFSET        0x0C
#define STM32_PWR_CR4_OFFSET        0x10
#define STM32_PWR_CPUCR_OFFSET      0x14
#define STM32_PWR_D3CR_OFFSET       0x18
#define STM32_PWR_WUCR_OFFSET       0x20
#define STM32_PWR_WUSCR_OFFSET      0x24
#define STM32_PWR_WUSR_OFFSET       0x28
#define STM32_PWR_SECCFGR_OFFSET    0x30
#define STM32_PWR_PRIVCFGR_OFFSET   0x34

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static inline uint32_t stm32n6_pwr_getreg(uint32_t offset)
{
  return getreg32(STM32_PWR_BASE + offset);
}

static inline void stm32n6_pwr_putreg(uint32_t offset, uint32_t value)
{
  putreg32(value, STM32_PWR_BASE + offset);
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

void stm32n6_pwr_init(void)
{
  uint32_t regval;

  regval = stm32n6_pwr_getreg(STM32_PWR_CR1_OFFSET);
  regval |= PWR_CR1_DBP;
  stm32n6_pwr_putreg(STM32_PWR_CR1_OFFSET, regval);
}

void stm32n6_pwr_set_voltage_scaling(uint8_t scale)
{
  uint32_t regval;

  regval = stm32n6_pwr_getreg(STM32_PWR_CR1_OFFSET);
  regval &= ~PWR_CR1_VOS_MASK;
  switch (scale)
    {
      case 0:
        regval |= PWR_CR1_VOS_SCALE0;
        break;
      case 1:
        regval |= PWR_CR1_VOS_SCALE1;
        break;
      case 2:
        regval |= PWR_CR1_VOS_SCALE2;
        break;
      case 3:
        regval |= PWR_CR1_VOS_SCALE3;
        break;
      default:
        break;
    }
  stm32n6_pwr_putreg(STM32_PWR_CR1_OFFSET, regval);
}

void stm32n6_pwr_enter_stop_mode(uint8_t mode)
{
  uint32_t regval;

  regval = stm32n6_pwr_getreg(STM32_PWR_CR1_OFFSET);
  regval &= ~PWR_CR1_LPMS_MASK;
  regval |= mode;
  stm32n6_pwr_putreg(STM32_PWR_CR1_OFFSET, regval);

  /* Set SLEEPDEEP bit of Cortex System Control Register */
  regval = getreg32(NVIC_SYSCON_BASE + NVIC_SYSCON_ACTLR_OFFSET);
  regval |= NVIC_SYSCON_ACTLR_SLEEPDEEP;
  putreg32(regval, NVIC_SYSCON_BASE + NVIC_SYSCON_ACTLR_OFFSET);

  /* WFI instruction */
  asm ("wfi");

  /* Clear SLEEPDEEP bit */
  regval = getreg32(NVIC_SYSCON_BASE + NVIC_SYSCON_ACTLR_OFFSET);
  regval &= ~NVIC_SYSCON_ACTLR_SLEEPDEEP;
  putreg32(regval, NVIC_SYSCON_BASE + NVIC_SYSCON_ACTLR_OFFSET);
}

void stm32n6_pwr_enter_standby_mode(void)
{
  uint32_t regval;

  regval = stm32n6_pwr_getreg(STM32_PWR_CR1_OFFSET);
  regval &= ~PWR_CR1_LPMS_MASK;
  regval |= PWR_CR1_LPMS_STANDBY;
  stm32n6_pwr_putreg(STM32_PWR_CR1_OFFSET, regval);

  /* Set SLEEPDEEP bit of Cortex System Control Register */
  regval = getreg32(NVIC_SYSCON_BASE + NVIC_SYSCON_ACTLR_OFFSET);
  regval |= NVIC_SYSCON_ACTLR_SLEEPDEEP;
  putreg32(regval, NVIC_SYSCON_BASE + NVIC_SYSCON_ACTLR_OFFSET);

  /* WFI instruction */
  asm ("wfi");
}

void stm32n6_pwr_clear_standby_flag(void)
{
  stm32n6_pwr_putreg(STM32_PWR_SCR_CSBF_OFFSET, PWR_SCR_CSBF);
}

void stm32n6_pwr_clear_wakeup_flags(void)
{
  stm32n6_pwr_putreg(STM32_PWR_SCR_CWUF1_OFFSET, PWR_SCR_CWUF1 | PWR_SCR_CWUF2 | PWR_SCR_CWUF3);
}

void stm32n6_pwr_enable_backup_domain_access(bool enable)
{
  uint32_t regval;

  regval = stm32n6_pwr_getreg(STM32_PWR_CR1_OFFSET);
  if (enable)
    {
      regval |= PWR_CR1_DBP;
    }
  else
    {
      regval &= ~PWR_CR1_DBP;
    }
  stm32n6_pwr_putreg(STM32_PWR_CR1_OFFSET, regval);
}

bool stm32n6_pwr_is_backup_domain_access_enabled(void)
{
  uint32_t regval;

  regval = stm32n6_pwr_getreg(STM32_PWR_CR1_OFFSET);
  return (regval & PWR_CR1_DBP) != 0;
}
