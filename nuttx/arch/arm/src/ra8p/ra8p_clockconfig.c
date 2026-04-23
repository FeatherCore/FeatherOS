/****************************************************************************
 * arch/arm/src/ra8p/ra8p_clockconfig.c
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

#include "ra8p_clockconfig.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* RA8P1 Clock frequencies - typical values */
#ifndef CONFIG_RA8P_ICLK_FREQUENCY
#  define CONFIG_RA8P_ICLK_FREQUENCY    200000000  /* 200 MHz */
#endif

#ifndef CONFIG_RA8P_PCLKA_FREQUENCY
#  define CONFIG_RA8P_PCLKA_FREQUENCY   120000000  /* 120 MHz */
#endif

#ifndef CONFIG_RA8P_PCLKB_FREQUENCY
#  define CONFIG_RA8P_PCLKB_FREQUENCY    60000000  /* 60 MHz */
#endif

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_clockconfig
 ****************************************************************************/

void ra8p_clockconfig(void)
{
  /* RA8P starts with internal RC oscillator (LOCO ~ 4 MHz).
   * For now, keep default configuration set by boot ROM.
   * Board-specific code should configure clocking if needed.
   */

#ifdef CONFIG_ARCH_FPU
  /* Ensure FPU is enabled */
  volatile uint32_t cpacr = getreg32(0xe000ed88);
  cpacr |= (0xf << 20);  /* Set CP10 and CP11 full access */
  putreg32(cpacr, 0xe000ed88);
#endif
}

/****************************************************************************
 * Name: ra8p_get_sysclk
 ****************************************************************************/

uint32_t ra8p_get_sysclk(void)
{
  return CONFIG_RA8P_ICLK_FREQUENCY;
}

/****************************************************************************
 * Name: ra8p_get_pclka
 ****************************************************************************/

uint32_t ra8p_get_pclka(void)
{
  return CONFIG_RA8P_PCLKA_FREQUENCY;
}

/****************************************************************************
 * Name: ra8p_get_pclkb
 ****************************************************************************/

uint32_t ra8p_get_pclkb(void)
{
  return CONFIG_RA8P_PCLKB_FREQUENCY;
}
