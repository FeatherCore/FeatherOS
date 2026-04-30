/****************************************************************************
 * arch/arm/src/ra8p/ra8p_clockconfig_ext.c
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
#include <debug.h>

#include "chip.h"
#include "ra8p_config.h"
#include "ra8p_memorymap.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* System Control registers */

#define RA8P_SYSTEM_BASE      0x40010000
#define RA8P_SYSTEM_SCKDIVCR  (RA8P_SYSTEM_BASE + 0x0010)
#define RA8P_SYSTEM_SCKSCR    (RA8P_SYSTEM_BASE + 0x0008)
#define RA8P_SYSTEM_PLLCCR    (RA8P_SYSTEM_BASE + 0x001C)
#define RA8P_SYSTEM_PLL2CCR   (RA8P_SYSTEM_BASE + 0x0020)
#define RA8P_SYSTEM_SCKDIVCR2 (RA8P_SYSTEM_BASE + 0x0014)

/* SCKDIVCR bit definitions */

#define SCKDIVCR_ICK_SHIFT    0
#define SCKDIVCR_ICK_MASK     (7 << SCKDIVCR_ICK_SHIFT)
#define SCKDIVCR_ICK_DIV1     (0 << SCKDIVCR_ICK_SHIFT)
#define SCKDIVCR_ICK_DIV2     (1 << SCKDIVCR_ICK_SHIFT)
#define SCKDIVCR_ICK_DIV4     (2 << SCKDIVCR_ICK_SHIFT)
#define SCKDIVCR_ICK_DIV8     (3 << SCKDIVCR_ICK_SHIFT)
#define SCKDIVCR_ICK_DIV16    (4 << SCKDIVCR_ICK_SHIFT)
#define SCKDIVCR_ICK_DIV32    (5 << SCKDIVCR_ICK_SHIFT)
#define SCKDIVCR_ICK_DIV64    (6 << SCKDIVCR_ICK_SHIFT)

#define SCKDIVCR_FCK_SHIFT    8
#define SCKDIVCR_FCK_MASK     (7 << SCKDIVCR_FCK_SHIFT)
#define SCKDIVCR_FCK_DIV1     (0 << SCKDIVCR_FCK_SHIFT)
#define SCKDIVCR_FCK_DIV2     (1 << SCKDIVCR_FCK_SHIFT)
#define SCKDIVCR_FCK_DIV4     (2 << SCKDIVCR_FCK_SHIFT)
#define SCKDIVCR_FCK_DIV8     (3 << SCKDIVCR_FCK_SHIFT)
#define SCKDIVCR_FCK_DIV16    (4 << SCKDIVCR_FCK_SHIFT)
#define SCKDIVCR_FCK_DIV32    (5 << SCKDIVCR_FCK_SHIFT)
#define SCKDIVCR_FCK_DIV64    (6 << SCKDIVCR_FCK_SHIFT)

#define SCKDIVCR_PCKA_SHIFT   16
#define SCKDIVCR_PCKA_MASK    (7 << SCKDIVCR_PCKA_SHIFT)
#define SCKDIVCR_PCKA_DIV1    (0 << SCKDIVCR_PCKA_SHIFT)
#define SCKDIVCR_PCKA_DIV2    (1 << SCKDIVCR_PCKA_SHIFT)
#define SCKDIVCR_PCKA_DIV4    (2 << SCKDIVCR_PCKA_SHIFT)
#define SCKDIVCR_PCKA_DIV8    (3 << SCKDIVCR_PCKA_SHIFT)
#define SCKDIVCR_PCKA_DIV16   (4 << SCKDIVCR_PCKA_SHIFT)
#define SCKDIVCR_PCKA_DIV32   (5 << SCKDIVCR_PCKA_SHIFT)
#define SCKDIVCR_PCKA_DIV64   (6 << SCKDIVCR_PCKA_SHIFT)

#define SCKDIVCR_PCKB_SHIFT   24
#define SCKDIVCR_PCKB_MASK    (7 << SCKDIVCR_PCKB_SHIFT)
#define SCKDIVCR_PCKB_DIV1    (0 << SCKDIVCR_PCKB_SHIFT)
#define SCKDIVCR_PCKB_DIV2    (1 << SCKDIVCR_PCKB_SHIFT)
#define SCKDIVCR_PCKB_DIV4    (2 << SCKDIVCR_PCKB_SHIFT)
#define SCKDIVCR_PCKB_DIV8    (3 << SCKDIVCR_PCKB_SHIFT)
#define SCKDIVCR_PCKB_DIV16   (4 << SCKDIVCR_PCKB_SHIFT)
#define SCKDIVCR_PCKB_DIV32   (5 << SCKDIVCR_PCKB_SHIFT)
#define SCKDIVCR_PCKB_DIV64   (6 << SCKDIVCR_PCKB_SHIFT)

/* SCKDIVCR2 bit definitions */

#define SCKDIVCR2_PCKC_SHIFT  0
#define SCKDIVCR2_PCKC_MASK   (7 << SCKDIVCR2_PCKC_SHIFT)
#define SCKDIVCR2_PCKC_DIV1   (0 << SCKDIVCR2_PCKC_SHIFT)
#define SCKDIVCR2_PCKC_DIV2   (1 << SCKDIVCR2_PCKC_SHIFT)
#define SCKDIVCR2_PCKC_DIV4   (2 << SCKDIVCR2_PCKC_SHIFT)
#define SCKDIVCR2_PCKC_DIV8   (3 << SCKDIVCR2_PCKC_SHIFT)
#define SCKDIVCR2_PCKC_DIV16  (4 << SCKDIVCR2_PCKC_SHIFT)
#define SCKDIVCR2_PCKC_DIV32  (5 << SCKDIVCR2_PCKC_SHIFT)
#define SCKDIVCR2_PCKC_DIV64  (6 << SCKDIVCR2_PCKC_SHIFT)

#define SCKDIVCR2_PCKD_SHIFT  8
#define SCKDIVCR2_PCKD_MASK   (7 << SCKDIVCR2_PCKD_SHIFT)
#define SCKDIVCR2_PCKD_DIV1   (0 << SCKDIVCR2_PCKD_SHIFT)
#define SCKDIVCR2_PCKD_DIV2   (1 << SCKDIVCR2_PCKD_SHIFT)
#define SCKDIVCR2_PCKD_DIV4   (2 << SCKDIVCR2_PCKD_SHIFT)
#define SCKDIVCR2_PCKD_DIV8   (3 << SCKDIVCR2_PCKD_SHIFT)
#define SCKDIVCR2_PCKD_DIV16  (4 << SCKDIVCR2_PCKD_SHIFT)
#define SCKDIVCR2_PCKD_DIV32  (5 << SCKDIVCR2_PCKD_SHIFT)
#define SCKDIVCR2_PCKD_DIV64  (6 << SCKDIVCR2_PCKD_SHIFT)

#define SCKDIVCR2_BCK_SHIFT   16
#define SCKDIVCR2_BCK_MASK    (7 << SCKDIVCR2_BCK_SHIFT)
#define SCKDIVCR2_BCK_DIV1    (0 << SCKDIVCR2_BCK_SHIFT)
#define SCKDIVCR2_BCK_DIV2    (1 << SCKDIVCR2_BCK_SHIFT)
#define SCKDIVCR2_BCK_DIV4    (2 << SCKDIVCR2_BCK_SHIFT)
#define SCKDIVCR2_BCK_DIV8    (3 << SCKDIVCR2_BCK_SHIFT)
#define SCKDIVCR2_BCK_DIV16   (4 << SCKDIVCR2_BCK_SHIFT)
#define SCKDIVCR2_BCK_DIV32   (5 << SCKDIVCR2_BCK_SHIFT)
#define SCKDIVCR2_BCK_DIV64   (6 << SCKDIVCR2_BCK_SHIFT)

#define SCKDIVCR2_CPUCK_SHIFT 24
#define SCKDIVCR2_CPUCK_MASK  (7 << SCKDIVCR2_CPUCK_SHIFT)
#define SCKDIVCR2_CPUCK_DIV1  (0 << SCKDIVCR2_CPUCK_SHIFT)
#define SCKDIVCR2_CPUCK_DIV2  (1 << SCKDIVCR2_CPUCK_SHIFT)
#define SCKDIVCR2_CPUCK_DIV4  (2 << SCKDIVCR2_CPUCK_SHIFT)
#define SCKDIVCR2_CPUCK_DIV8  (3 << SCKDIVCR2_CPUCK_SHIFT)
#define SCKDIVCR2_CPUCK_DIV16 (4 << SCKDIVCR2_CPUCK_SHIFT)
#define SCKDIVCR2_CPUCK_DIV32 (5 << SCKDIVCR2_CPUCK_SHIFT)
#define SCKDIVCR2_CPUCK_DIV64 (6 << SCKDIVCR2_CPUCK_SHIFT)

/* SCKSCR bit definitions */

#define SCKSCR_CKSEL_SHIFT    0
#define SCKSCR_CKSEL_MASK     (7 << SCKSCR_CKSEL_SHIFT)
#define SCKSCR_CKSEL_LOCO     (0 << SCKSCR_CKSEL_SHIFT)  /* Low Speed On-Chip Oscillator */
#define SCKSCR_CKSEL_HOCO     (1 << SCKSCR_CKSEL_SHIFT)  /* High Speed On-Chip Oscillator */
#define SCKSCR_CKSEL_MAIN     (2 << SCKSCR_CKSEL_SHIFT)  /* Main Clock Oscillator */
#define SCKSCR_CKSEL_PLL      (3 << SCKSCR_CKSEL_SHIFT)  /* PLL */
#define SCKSCR_CKSEL_PLL2     (4 << SCKSCR_CKSEL_SHIFT)  /* PLL2 */

/* PLLCCR bit definitions */

#define PLLCCR_PLLSRC_SHIFT   0
#define PLLCCR_PLLSRC_MASK    (1 << PLLCCR_PLLSRC_SHIFT)
#define PLLCCR_PLLSRC_MAIN    (0 << PLLCCR_PLLSRC_SHIFT)  /* Main clock as PLL source */
#define PLLCCR_PLLSRC_HOCO    (1 << PLLCCR_PLLSRC_SHIFT)  /* HOCO as PLL source */

#define PLLCCR_PLLMUL_SHIFT   8
#define PLLCCR_PLLMUL_MASK    (0x3f << PLLCCR_PLLMUL_SHIFT)

#define PLLCCR_PLLDIV_SHIFT   16
#define PLLCCR_PLLDIV_MASK    (3 << PLLCCR_PLLDIV_SHIFT)
#define PLLCCR_PLLDIV_1       (0 << PLLCCR_PLLDIV_SHIFT)
#define PLLCCR_PLLDIV_2       (1 << PLLCCR_PLLDIV_SHIFT)
#define PLLCCR_PLLDIV_4       (2 << PLLCCR_PLLDIV_SHIFT)
#define PLLCCR_PLLDIV_8       (3 << PLLCCR_PLLDIV_SHIFT)

#define PLLCCR_PLLSTP         (1 << 24)  /* PLL Stop */

/* PLL2CCR bit definitions */

#define PLL2CCR_PLL2MUL_SHIFT 8
#define PLL2CCR_PLL2MUL_MASK  (0x3f << PLL2CCR_PLL2MUL_SHIFT)

#define PLL2CCR_PLL2DIV_SHIFT 16
#define PLL2CCR_PLL2DIV_MASK  (3 << PLL2CCR_PLL2DIV_SHIFT)
#define PLL2CCR_PLL2DIV_1     (0 << PLL2CCR_PLL2DIV_SHIFT)
#define PLL2CCR_PLL2DIV_2     (1 << PLL2CCR_PLL2DIV_SHIFT)
#define PLL2CCR_PLL2DIV_4     (2 << PLL2CCR_PLL2DIV_SHIFT)
#define PLL2CCR_PLL2DIV_8     (3 << PLL2CCR_PLL2DIV_SHIFT)

#define PLL2CCR_PLL2STP       (1 << 24)  /* PLL2 Stop */

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: wait_for_pll_stable
 *
 * Description:
 *   Wait for PLL to become stable.
 *
 ****************************************************************************/

static void wait_for_pll_stable(void)
{
  volatile int delay;

  /* Simple delay loop - should be replaced with proper wait for PLL ready */

  for (delay = 0; delay < 1000; delay++);
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_clockconfig_pll
 *
 * Description:
 *   Configure PLL for RA8P1 to achieve target CPU frequency.
 *
 ****************************************************************************/

void ra8p_clockconfig_pll(void)
{
  uint32_t regval;
  uint32_t pll_mul = 50;  /* 24MHz * 50 = 1200MHz VCO */
  uint32_t pll_div = 2;   /* 1200MHz / 2 = 600MHz CPU clock */
  uint32_t cpu_div = 3;   /* 600MHz / 3 = 200MHz ICLK */

  /* Step 1: Stop PLL if currently running */
  regval = getreg32(RA8P_SYSTEM_PLLCCR);
  regval |= PLLCCR_PLLSTP;
  putreg32(regval, RA8P_SYSTEM_PLLCCR);

  /* Step 2: Configure PLL settings */
  regval = PLLCCR_PLLSRC_MAIN;  /* Use main oscillator as PLL source */
  regval |= ((pll_mul & 0x3f) << PLLCCR_PLLMUL_SHIFT);
  regval |= ((pll_div & 0x3) << PLLCCR_PLLDIV_SHIFT);
  putreg32(regval, RA8P_SYSTEM_PLLCCR);

  /* Step 3: Enable PLL */
  regval = getreg32(RA8P_SYSTEM_PLLCCR);
  regval &= ~PLLCCR_PLLSTP;
  putreg32(regval, RA8P_SYSTEM_PLLCCR);

  /* Step 4: Wait for PLL to stabilize */
  wait_for_pll_stable();

  /* Step 5: Configure system clock dividers */
  regval = 0;
  regval |= SCKDIVCR_ICK_DIV1;    /* ICLK = CPU clock */
  regval |= SCKDIVCR_FCK_DIV2;    /* FCLK = ICLK/2 */
  regval |= SCKDIVCR_PCKA_DIV2;   /* PCLKA = ICLK/2 */
  regval |= SCKDIVCR_PCKB_DIV4;   /* PCLKB = ICLK/4 */
  putreg32(regval, RA8P_SYSTEM_SCKDIVCR);

  /* Additional dividers in SCKDIVCR2 */
  regval = 0;
  regval |= SCKDIVCR2_PCKC_DIV4;  /* PCLKC = ICLK/4 */
  regval |= SCKDIVCR2_PCKD_DIV4;  /* PCLKD = ICLK/4 */
  regval |= SCKDIVCR2_BCK_DIV4;   /* BCK = ICLK/4 */
  regval |= SCKDIVCR2_CPUCK_DIV1; /* CPUCK = ICLK */
  putreg32(regval, RA8P_SYSTEM_SCKDIVCR2);

  /* Step 6: Switch to PLL clock */
  regval = getreg32(RA8P_SYSTEM_SCKSCR);
  regval &= ~SCKSCR_CKSEL_MASK;
  regval |= SCKSCR_CKSEL_PLL;
  putreg32(regval, RA8P_SYSTEM_SCKSCR);

  /* Wait for clock switch to complete */
  while ((getreg32(RA8P_SYSTEM_SCKSCR) & SCKSCR_CKSEL_MASK) != SCKSCR_CKSEL_PLL);
}

/****************************************************************************
 * Name: ra8p_clockconfig
 *
 * Description:
 *   Configure system clocks for RA8P1.
 *
 ****************************************************************************/

void ra8p_clockconfig(void)
{
  /* Configure PLL for 200MHz ICLK */
  ra8p_clockconfig_pll();

  /* Configure FPU */
#ifdef CONFIG_ARCH_FPU
  /* Ensure FPU is enabled */
  volatile uint32_t cpacr = getreg32(0xe000ed88);
  cpacr |= (0xf << 20);  /* Set CP10 and CP11 full access */
  putreg32(cpacr, 0xe000ed88);
#endif
}