/****************************************************************************
 * arch/arm/src/ra8p/ra8p_cgc.c
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
#include "hardware/ra8p_cgc.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* CGC base address */
#define CGC_BASE        RA8P_CGC_BASE
#define MSTP_BASE       RA8P_CGC_MSTPA_BASE

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_cgc_wait_pll_stable
 ****************************************************************************/

static void ra8p_cgc_wait_pll_stable(void)
{
  volatile uint32_t timeout = 10000;
  while (timeout-- > 0)
    {
      __asm__ volatile ("nop");
    }
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_cgc_init
 *
 * Description:
 *   Initialize the clock generation circuit based on Zephyr RA8P1
 *   configuration. Reference: ek_ra8p1.dtsi, r7ka8p1xf.dtsi
 *
 ****************************************************************************/

void ra8p_cgc_init(void)
{
  uint32_t regval;

  /* Configure PLL for 1 GHz CPU clock
   * Reference: Zephyr r7ka8p1xf.dtsi PLL configuration
   * PLL: 24MHz * 250 / 3 = 2000MHz VCO
   * PLLP: 2000MHz / 2 = 1000MHz (CPU clock)
   * PLLQ: 2000MHz / 6 = 333MHz
   * PLLR: 2000MHz / 5 = 400MHz
   */

  /* Enable main oscillator (XTAL 24MHz) */
  putreg32(0, CGC_BASE + RA8P_CGC_MOSCCR_OFFSET);

  /* Wait for oscillator to stabilize */
  ra8p_cgc_wait_pll_stable();

  /* Configure PLL1 */
  regval = (RA8P_PLL_MUL << RA8P_PLLCR_PLLMUL_SHIFT) |
           (RA8P_PLL_DIV << RA8P_PLLCR_PLLDIV_SHIFT) |
           RA8P_PLLCR_PLLEN;
  putreg32(regval, CGC_BASE + RA8P_CGC_PLLCR_OFFSET);

  /* Wait for PLL to stabilize */
  ra8p_cgc_wait_pll_stable();

  /* Configure PLL2 for peripheral clocks
   * Reference: Zephyr ek_ra8p1.dtsi PLL2 configuration
   * PLL2: 24MHz * 300 / 3 = 2400MHz VCO
   * PLL2P: 2400MHz / 4 = 600MHz
   * PLL2Q: 2400MHz / 3 = 800MHz
   * PLL2R: 2400MHz / 5 = 480MHz
   */
  regval = (RA8P_PLL2_MUL << RA8P_PLL2CR_PLLMUL_SHIFT) |
           (RA8P_PLL2_DIV << RA8P_PLL2CR_PLLDIV_SHIFT) |
           RA8P_PLL2CR_PLLEN;
  putreg32(regval, CGC_BASE + RA8P_CGC_PLL2CR_OFFSET);

  /* Wait for PLL2 to stabilize */
  ra8p_cgc_wait_pll_stable();

  /* Configure clock dividers
   * Reference: Zephyr r7ka8p1xf.dtsi pclkblock configuration
   * CPUCLK0: 1000MHz (div=1)
   * CPUCLK1: 250MHz (div=4)
   * ICLK: 250MHz (div=4)
   * PCLKA: 125MHz (div=8)
   * PCLKB: 62.5MHz (div=16)
   * PCLKC: 125MHz (div=8)
   * PCLKD: 250MHz (div=4)
   * BCLK: 125MHz (div=8)
   */
  regval = (4 << RA8P_SCKDIVCR_ICK_SHIFT) |    /* ICLK = PLLP/4 = 250MHz */
           (8 << RA8P_SCKDIVCR_FCK_SHIFT) |    /* FCLK = PLLP/8 = 125MHz */
           (8 << RA8P_SCKDIVCR_PCKA_SHIFT) |   /* PCLKA = PLLP/8 = 125MHz */
           (16 << RA8P_SCKDIVCR_PCKB_SHIFT) |  /* PCLKB = PLLP/16 = 62.5MHz */
           (8 << RA8P_SCKDIVCR_PCKC_SHIFT) |   /* PCLKC = PLLP/8 = 125MHz */
           (4 << RA8P_SCKDIVCR_PCKD_SHIFT) |   /* PCLKD = PLLP/4 = 250MHz */
           (8 << RA8P_SCKDIVCR_BCK_SHIFT);     /* BCLK = PLLP/8 = 125MHz */
  putreg32(regval, CGC_BASE + RA8P_CGC_SCKDIVCR_OFFSET);

  /* Select PLL as system clock source */
  putreg32(RA8P_SCKSCR_CKSEL_PLL, CGC_BASE + RA8P_CGC_SCKSCR_OFFSET);
}

/****************************************************************************
 * Name: ra8p_cgc_enable_module
 ****************************************************************************/

void ra8p_cgc_enable_module(uint8_t mstp, uint8_t bit)
{
  uintptr_t mstp_base;
  uint32_t regval;

  if (mstp > 4)
    {
      return;
    }

  mstp_base = MSTP_BASE + (mstp * 4);
  regval = getreg32(mstp_base);
  regval &= ~(1 << bit);
  putreg32(regval, mstp_base);
}

/****************************************************************************
 * Name: ra8p_cgc_disable_module
 ****************************************************************************/

void ra8p_cgc_disable_module(uint8_t mstp, uint8_t bit)
{
  uintptr_t mstp_base;
  uint32_t regval;

  if (mstp > 4)
    {
      return;
    }

  mstp_base = MSTP_BASE + (mstp * 4);
  regval = getreg32(mstp_base);
  regval |= (1 << bit);
  putreg32(regval, mstp_base);
}

/****************************************************************************
 * Name: ra8p_cgc_get_iclk
 ****************************************************************************/

uint32_t ra8p_cgc_get_iclk(void)
{
  return 250000000; /* 250 MHz */
}

/****************************************************************************
 * Name: ra8p_cgc_get_pclka
 ****************************************************************************/

uint32_t ra8p_cgc_get_pclka(void)
{
  return 125000000; /* 125 MHz */
}

/****************************************************************************
 * Name: ra8p_cgc_get_pclkb
 ****************************************************************************/

uint32_t ra8p_cgc_get_pclkb(void)
{
  return 62500000; /* 62.5 MHz */
}

/****************************************************************************
 * Name: ra8p_cgc_get_pclkc
 ****************************************************************************/

uint32_t ra8p_cgc_get_pclkc(void)
{
  return 125000000; /* 125 MHz */
}

/****************************************************************************
 * Name: ra8p_cgc_get_pclkd
 ****************************************************************************/

uint32_t ra8p_cgc_get_pclkd(void)
{
  return 250000000; /* 250 MHz */
}