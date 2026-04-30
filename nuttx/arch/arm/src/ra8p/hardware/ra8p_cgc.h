/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_cgc.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_CGC_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_CGC_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "ra8p_memorymap.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* CGC Register Offsets - Based on RA8P1 hardware manual */
#define RA8P_CGC_SCKDIVCR_OFFSET    0x020
#define RA8P_CGC_SCKDIVCR2_OFFSET   0x024
#define RA8P_CGC_SCKDIVCR3_OFFSET   0x028
#define RA8P_CGC_SCKSCR_OFFSET      0x040
#define RA8P_CGC_PLLCR_OFFSET       0x060
#define RA8P_CGC_PLLCR2_OFFSET      0x064
#define RA8P_CGC_PLL2CR_OFFSET      0x068
#define RA8P_CGC_PLL2CR2_OFFSET     0x06C
#define RA8P_CGC_MOSCCR_OFFSET      0x100
#define RA8P_CGC_HOCOCR_OFFSET      0x140
#define RA8P_CGC_LOCOCR_OFFSET      0x180
#define RA8P_CGC_MOCOCR_OFFSET      0x1A0

/* MSTP Register Offsets */
#define RA8P_MSTPA_OFFSET           0x000
#define RA8P_MSTPB_OFFSET           0x004
#define RA8P_MSTPC_OFFSET           0x008
#define RA8P_MSTPD_OFFSET           0x00C
#define RA8P_MSTPE_OFFSET           0x010

/* SCKDIVCR bit definitions */
#define RA8P_SCKDIVCR_ICK_SHIFT     0
#define RA8P_SCKDIVCR_ICK_MASK      (0x0F << 0)
#define RA8P_SCKDIVCR_FCK_SHIFT     4
#define RA8P_SCKDIVCR_FCK_MASK      (0x0F << 4)
#define RA8P_SCKDIVCR_PCKA_SHIFT    8
#define RA8P_SCKDIVCR_PCKA_MASK     (0x0F << 8)
#define RA8P_SCKDIVCR_PCKB_SHIFT    12
#define RA8P_SCKDIVCR_PCKB_MASK     (0x0F << 12)
#define RA8P_SCKDIVCR_PCKC_SHIFT    16
#define RA8P_SCKDIVCR_PCKC_MASK     (0x0F << 16)
#define RA8P_SCKDIVCR_PCKD_SHIFT    20
#define RA8P_SCKDIVCR_PCKD_MASK     (0x0F << 20)
#define RA8P_SCKDIVCR_BCK_SHIFT     24
#define RA8P_SCKDIVCR_BCK_MASK      (0x0F << 24)

/* SCKSCR Clock Selection Values */
#define RA8P_SCKSCR_CKSEL_HOCO      0
#define RA8P_SCKSCR_CKSEL_MOCO      1
#define RA8P_SCKSCR_CKSEL_LOCO      2
#define RA8P_SCKSCR_CKSEL_MAIN_OSC  3
#define RA8P_SCKSCR_CKSEL_SUB_OSC   4
#define RA8P_SCKSCR_CKSEL_PLL       5
#define RA8P_SCKSCR_CKSEL_PLL2      6

/* PLLCR bit definitions */
#define RA8P_PLLCR_PLLMUL_SHIFT     0
#define RA8P_PLLCR_PLLMUL_MASK      (0x3F << 0)
#define RA8P_PLLCR_PLLDIV_SHIFT     8
#define RA8P_PLLCR_PLLDIV_MASK      (0x03 << 8)
#define RA8P_PLLCR_PLLEN            (1 << 31)

/* PLL2CR bit definitions */
#define RA8P_PLL2CR_PLLMUL_SHIFT    0
#define RA8P_PLL2CR_PLLMUL_MASK     (0x3F << 0)
#define RA8P_PLL2CR_PLLDIV_SHIFT    8
#define RA8P_PLL2CR_PLLDIV_MASK     (0x03 << 8)
#define RA8P_PLL2CR_PLLEN           (1 << 31)

/* Clock frequencies - Based on Zephyr RA8P1 configuration */
#define RA8P_XTAL_FREQUENCY         24000000   /* 24 MHz external crystal */
#define RA8P_HOCO_FREQUENCY         48000000   /* 48 MHz HOCO */
#define RA8P_MOCO_FREQUENCY         8000000    /* 8 MHz MOCO */
#define RA8P_LOCO_FREQUENCY         32768      /* 32.768 kHz LOCO */

/* PLL configuration for 1 GHz CPU clock */
/* PLL: 24MHz * 250 / 3 = 2000MHz VCO, then dividers for outputs */
#define RA8P_PLL_MUL                250
#define RA8P_PLL_DIV                3
#define RA8P_PLL_VCO_FREQUENCY      (RA8P_XTAL_FREQUENCY * RA8P_PLL_MUL / RA8P_PLL_DIV)

/* PLL output frequencies */
#define RA8P_PLLP_FREQUENCY         1000000000 /* 1000 MHz - CPU clock */
#define RA8P_PLLQ_FREQUENCY         333333333  /* 333 MHz */
#define RA8P_PLLR_FREQUENCY         400000000  /* 400 MHz */

/* PLL2 configuration - Based on Zephyr ek_ra8p1.dtsi */
#define RA8P_PLL2_MUL               300
#define RA8P_PLL2_DIV               3
#define RA8P_PLL2_VCO_FREQUENCY     (RA8P_XTAL_FREQUENCY * RA8P_PLL2_MUL / RA8P_PLL2_DIV)

#define RA8P_PLL2P_FREQUENCY        600000000  /* 600 MHz */
#define RA8P_PLL2Q_FREQUENCY        800000000  /* 800 MHz */
#define RA8P_PLL2R_FREQUENCY        480000000  /* 480 MHz */

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_cgc_init
 *
 * Description:
 *   Initialize the clock generation circuit.
 *   Based on Zephyr RA8P1 clock configuration.
 *
 ****************************************************************************/

void ra8p_cgc_init(void);

/****************************************************************************
 * Name: ra8p_cgc_enable_module
 *
 * Description:
 *   Enable a peripheral module clock.
 *
 * Input Parameters:
 *   mstp - MSTP register (MSTPA-MSTPE)
 *   bit  - Bit position in the MSTP register
 *
 ****************************************************************************/

void ra8p_cgc_enable_module(uint8_t mstp, uint8_t bit);

/****************************************************************************
 * Name: ra8p_cgc_disable_module
 *
 * Description:
 *   Disable a peripheral module clock.
 *
 * Input Parameters:
 *   mstp - MSTP register (MSTPA-MSTPE)
 *   bit  - Bit position in the MSTP register
 *
 ****************************************************************************/

void ra8p_cgc_disable_module(uint8_t mstp, uint8_t bit);

/****************************************************************************
 * Name: ra8p_cgc_get_iclk
 *
 * Description:
 *   Get the ICLK frequency.
 *
 ****************************************************************************/

uint32_t ra8p_cgc_get_iclk(void);

/****************************************************************************
 * Name: ra8p_cgc_get_pclka
 *
 * Description:
 *   Get the PCLKA frequency.
 *
 ****************************************************************************/

uint32_t ra8p_cgc_get_pclka(void);

/****************************************************************************
 * Name: ra8p_cgc_get_pclkb
 *
 * Description:
 *   Get the PCLKB frequency.
 *
 ****************************************************************************/

uint32_t ra8p_cgc_get_pclkb(void);

/****************************************************************************
 * Name: ra8p_cgc_get_pclkc
 *
 * Description:
 *   Get the PCLKC frequency.
 *
 ****************************************************************************/

uint32_t ra8p_cgc_get_pclkc(void);

/****************************************************************************
 * Name: ra8p_cgc_get_pclkd
 *
 * Description:
 *   Get the PCLKD frequency.
 *
 ****************************************************************************/

uint32_t ra8p_cgc_get_pclkd(void);

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_CGC_H */