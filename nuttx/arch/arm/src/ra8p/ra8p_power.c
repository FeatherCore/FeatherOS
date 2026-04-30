/****************************************************************************
 * arch/arm/src/ra8p/ra8p_power.c
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
#include <stdbool.h>

#include "arm_internal.h"
#include "hardware/ra8p_power.h"
#include "hardware/ra8p_memorymap.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* LPM (Low Power Mode) Register Offsets */
#define RA8P_LPM_LPMCR_OFFSET       0x00
#define RA8P_LPM_LPMSTCR_OFFSET     0x04
#define RA8P_LPM_LPMPSR_OFFSET      0x08

/* LPMCR bit definitions */
#define RA8P_LPMCR_LPM_SHIFT        0
#define RA8P_LPMCR_LPM_MASK         (0x07 << 0)
#define RA8P_LPMCR_LPM_SLEEP        0
#define RA8P_LPMCR_LPM_STANDBY      1
#define RA8P_LPMCR_LPM_DEEP_STANDBY 2

/* System Control Register Offsets */
#define RA8P_SYSTEM_SYSCR_OFFSET    0x000
#define RA8P_SYSTEM_SBYCR_OFFSET    0x004

/* SBYCR (Standby Control Register) bit definitions */
#define RA8P_SBYCR_SSBY             (1 << 0)
#define RA8P_SBYCR_OPE              (1 << 1)
#define RA8P_SBYCR_RAM_RETENTION    (1 << 2)

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_power_enter_sleep
 *
 * Description:
 *   Enter sleep mode. The CPU stops but peripherals continue to operate.
 *   Based on Zephyr RA8P1 power.c implementation.
 *
 ****************************************************************************/

void ra8p_power_enter_sleep(void)
{
  uint32_t regval;

  /* Configure sleep mode */
  regval = getreg32(RA8P_SYSTEM_CONTROL_BASE + RA8P_SYSTEM_SYSCR_OFFSET);
  regval &= ~RA8P_LPMCR_LPM_MASK;
  regval |= (RA8P_LPMCR_LPM_SLEEP << RA8P_LPMCR_LPM_SHIFT);
  putreg32(regval, RA8P_SYSTEM_CONTROL_BASE + RA8P_SYSTEM_SYSCR_OFFSET);

  /* Enter sleep mode using WFI instruction */
  __asm__ volatile ("wfi");
}

/****************************************************************************
 * Name: ra8p_power_enter_standby
 *
 * Description:
 *   Enter standby mode. Most clocks are stopped, RAM is retained.
 *   Based on Zephyr RA8P1 power.c implementation.
 *
 ****************************************************************************/

void ra8p_power_enter_standby(void)
{
  uint32_t regval;

  /* Configure standby mode with RAM retention */
  regval = getreg32(RA8P_SYSTEM_CONTROL_BASE + RA8P_SYSTEM_SYSCR_OFFSET);
  regval &= ~RA8P_LPMCR_LPM_MASK;
  regval |= (RA8P_LPMCR_LPM_STANDBY << RA8P_LPMCR_LPM_SHIFT);
  putreg32(regval, RA8P_SYSTEM_CONTROL_BASE + RA8P_SYSTEM_SYSCR_OFFSET);

  /* Enable RAM retention */
  regval = getreg32(RA8P_SYSTEM_CONTROL_BASE + RA8P_SYSTEM_SBYCR_OFFSET);
  regval |= RA8P_SBYCR_RAM_RETENTION;
  putreg32(regval, RA8P_SYSTEM_CONTROL_BASE + RA8P_SYSTEM_SBYCR_OFFSET);

  /* Enter standby mode using WFI instruction */
  __asm__ volatile ("wfi");
}

/****************************************************************************
 * Name: ra8p_power_enter_deep_standby
 *
 * Description:
 *   Enter deep standby mode. Lowest power mode with limited wake sources.
 *
 ****************************************************************************/

void ra8p_power_enter_deep_standby(void)
{
  uint32_t regval;

  /* Configure deep standby mode */
  regval = getreg32(RA8P_SYSTEM_CONTROL_BASE + RA8P_SYSTEM_SYSCR_OFFSET);
  regval &= ~RA8P_LPMCR_LPM_MASK;
  regval |= (RA8P_LPMCR_LPM_DEEP_STANDBY << RA8P_LPMCR_LPM_SHIFT);
  putreg32(regval, RA8P_SYSTEM_CONTROL_BASE + RA8P_SYSTEM_SYSCR_OFFSET);

  /* Enter deep standby mode using WFI instruction */
  __asm__ volatile ("wfi");
}

/****************************************************************************
 * Name: ra8p_power_exit_standby
 *
 * Description:
 *   Exit from standby mode. Called after wake-up.
 *
 ****************************************************************************/

void ra8p_power_exit_standby(void)
{
  /* Clear standby mode */
  uint32_t regval = getreg32(RA8P_SYSTEM_CONTROL_BASE + RA8P_SYSTEM_SYSCR_OFFSET);
  regval &= ~RA8P_LPMCR_LPM_MASK;
  putreg32(regval, RA8P_SYSTEM_CONTROL_BASE + RA8P_SYSTEM_SYSCR_OFFSET);
}

/****************************************************************************
 * Name: ra8p_power_configure_wake_source
 *
 * Description:
 *   Configure wake-up sources for low power modes.
 *
 * Input Parameters:
 *   source - Wake source bitmask
 *
 ****************************************************************************/

void ra8p_power_configure_wake_source(uint32_t source)
{
  /* Configure wake-up sources in LPMSTCR register */
  putreg32(source, RA8P_SYSTEM_CONTROL_BASE + RA8P_LPM_LPMSTCR_OFFSET);
}

/****************************************************************************
 * Name: ra8p_power_get_wake_source
 *
 * Description:
 *   Get the wake-up source that caused the last wake-up.
 *
 * Return Value:
 *   Wake source bitmask
 *
 ****************************************************************************/

uint32_t ra8p_power_get_wake_source(void)
{
  return getreg32(RA8P_SYSTEM_CONTROL_BASE + RA8P_LPM_LPMPSR_OFFSET);
}