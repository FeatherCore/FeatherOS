/****************************************************************************
 * arch/arm/src/stm32u5/hardware/stm32_ltdc.h
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

#ifndef __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32_LTDC_H
#define __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32_LTDC_H

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Register Offsets *********************************************************/

#define STM32_LTDC_SSCR_OFFSET       0x0000  /* LTDC Synchronization Size Configuration Register */
#define STM32_LTDC_BPCR_OFFSET       0x0004  /* LTDC Back Porch Configuration Register */
#define STM32_LTDC_AWCR_OFFSET       0x0008  /* LTDC Active Width Configuration Register */
#define STM32_LTDC_TWCR_OFFSET       0x000c  /* LTDC Total Width Configuration Register */
#define STM32_LTDC_GCR_OFFSET        0x0010  /* LTDC Global Control Register */
#define STM32_LTDC_SRCR_OFFSET       0x0014  /* LTDC Shadow Reload Control Register */
#define STM32_LTDC_BCCR_OFFSET       0x0018  /* LTDC Background Color Configuration Register */
#define STM32_LTDC_IER_OFFSET        0x001c  /* LTDC Interrupt Enable Register */
#define STM32_LTDC_ISR_OFFSET        0x0020  /* LTDC Interrupt Status Register */
#define STM32_LTDC_ICR_OFFSET        0x0024  /* LTDC Interrupt Clear Register */
#define STM32_LTDC_LIPCR_OFFSET      0x0028  /* LTDC Line Interrupt Position Configuration Register */
#define STM32_LTDC_CPSR_OFFSET       0x002c  /* LTDC Current Position Status Register */
#define STM32_LTDC_CDSR_OFFSET       0x0030  /* LTDC Current Display Status Register */
#define STM32_LTDC_L1CR_OFFSET       0x0084  /* LTDC Layer 1 Control Register */
#define STM32_LTDC_L1WHPCR_OFFSET    0x0088  /* LTDC Layer 1 Window Horizontal Position Configuration Register */
#define STM32_LTDC_L1WVPCR_OFFSET    0x008c  /* LTDC Layer 1 Window Vertical Position Configuration Register */
#define STM32_LTDC_L1CKCR_OFFSET     0x0090  /* LTDC Layer 1 Color Keying Configuration Register */
#define STM32_LTDC_L1PFCR_OFFSET     0x0094  /* LTDC Layer 1 Pixel Format Configuration Register */
#define STM32_LTDC_L1CACR_OFFSET     0x0098  /* LTDC Layer 1 Constant Alpha Configuration Register */
#define STM32_LTDC_L1DCCR_OFFSET     0x009c  /* LTDC Layer 1 Default Color Configuration Register */
#define STM32_LTDC_L1BFCR_OFFSET     0x00a0  /* LTDC Layer 1 Blending Factors Configuration Register */
#define STM32_LTDC_L1CFBAR_OFFSET    0x00ac  /* LTDC Layer 1 Color Frame Buffer Address Register */
#define STM32_LTDC_L1CFBLR_OFFSET    0x00b0  /* LTDC Layer 1 Color Frame Buffer Length Register */
#define STM32_LTDC_L1CFBLNR_OFFSET   0x00b4  /* LTDC Layer 1 ColorFrame Buffer Line Number Register */
#define STM32_LTDC_L1CLUTWR_OFFSET   0x00c4  /* LTDC Layer 1 CLUT Write Register */
#define STM32_LTDC_L2CR_OFFSET       0x0104  /* LTDC Layer 2 Control Register */
#define STM32_LTDC_L2WHPCR_OFFSET    0x0108  /* LTDC Layer 2 Window Horizontal Position Configuration Register */
#define STM32_LTDC_L2WVPCR_OFFSET    0x010c  /* LTDC Layer 2 Window Vertical Position Configuration Register */
#define STM32_LTDC_L2CKCR_OFFSET     0x0110  /* LTDC Layer 2 Color Keying Configuration Register */
#define STM32_LTDC_L2PFCR_OFFSET     0x0114  /* LTDC Layer 2 Pixel Format Configuration Register */
#define STM32_LTDC_L2CACR_OFFSET     0x0118  /* LTDC Layer 2 Constant Alpha Configuration Register */
#define STM32_LTDC_L2DCCR_OFFSET     0x011c  /* LTDC Layer 2 Default Color Configuration Register */
#define STM32_LTDC_L2BFCR_OFFSET     0x0120  /* LTDC Layer 2 Blending Factors Configuration Register */
#define STM32_LTDC_L2CFBAR_OFFSET    0x012c  /* LTDC Layer 2 Color Frame Buffer Address Register */
#define STM32_LTDC_L2CFBLR_OFFSET    0x0130  /* LTDC Layer 2 Color Frame Buffer Length Register */
#define STM32_LTDC_L2CFBLNR_OFFSET   0x0134  /* LTDC Layer 2 ColorFrame Buffer Line Number Register */
#define STM32_LTDC_L2CLUTWR_OFFSET   0x0144  /* LTDC Layer 2 CLUT Write Register */

/* Register Addresses *******************************************************/

#define STM32_LTDC_SSCR              (STM32_LTDC_BASE + STM32_LTDC_SSCR_OFFSET)
#define STM32_LTDC_BPCR              (STM32_LTDC_BASE + STM32_LTDC_BPCR_OFFSET)
#define STM32_LTDC_AWCR              (STM32_LTDC_BASE + STM32_LTDC_AWCR_OFFSET)
#define STM32_LTDC_TWCR              (STM32_LTDC_BASE + STM32_LTDC_TWCR_OFFSET)
#define STM32_LTDC_GCR               (STM32_LTDC_BASE + STM32_LTDC_GCR_OFFSET)
#define STM32_LTDC_SRCR              (STM32_LTDC_BASE + STM32_LTDC_SRCR_OFFSET)
#define STM32_LTDC_BCCR              (STM32_LTDC_BASE + STM32_LTDC_BCCR_OFFSET)
#define STM32_LTDC_IER               (STM32_LTDC_BASE + STM32_LTDC_IER_OFFSET)
#define STM32_LTDC_ISR               (STM32_LTDC_BASE + STM32_LTDC_ISR_OFFSET)
#define STM32_LTDC_ICR               (STM32_LTDC_BASE + STM32_LTDC_ICR_OFFSET)
#define STM32_LTDC_LIPCR             (STM32_LTDC_BASE + STM32_LTDC_LIPCR_OFFSET)
#define STM32_LTDC_CPSR              (STM32_LTDC_BASE + STM32_LTDC_CPSR_OFFSET)
#define STM32_LTDC_CDSR              (STM32_LTDC_BASE + STM32_LTDC_CDSR_OFFSET)
#define STM32_LTDC_L1CR              (STM32_LTDC_BASE + STM32_LTDC_L1CR_OFFSET)
#define STM32_LTDC_L1WHPCR           (STM32_LTDC_BASE + STM32_LTDC_L1WHPCR_OFFSET)
#define STM32_LTDC_L1WVPCR           (STM32_LTDC_BASE + STM32_LTDC_L1WVPCR_OFFSET)
#define STM32_LTDC_L1CKCR            (STM32_LTDC_BASE + STM32_LTDC_L1CKCR_OFFSET)
#define STM32_LTDC_L1PFCR            (STM32_LTDC_BASE + STM32_LTDC_L1PFCR_OFFSET)
#define STM32_LTDC_L1CACR            (STM32_LTDC_BASE + STM32_LTDC_L1CACR_OFFSET)
#define STM32_LTDC_L1DCCR            (STM32_LTDC_BASE + STM32_LTDC_L1DCCR_OFFSET)
#define STM32_LTDC_L1BFCR            (STM32_LTDC_BASE + STM32_LTDC_L1BFCR_OFFSET)
#define STM32_LTDC_L1CFBAR           (STM32_LTDC_BASE + STM32_LTDC_L1CFBAR_OFFSET)
#define STM32_LTDC_L1CFBLR           (STM32_LTDC_BASE + STM32_LTDC_L1CFBLR_OFFSET)
#define STM32_LTDC_L1CFBLNR          (STM32_LTDC_BASE + STM32_LTDC_L1CFBLNR_OFFSET)
#define STM32_LTDC_L1CLUTWR          (STM32_LTDC_BASE + STM32_LTDC_L1CLUTWR_OFFSET)
#define STM32_LTDC_L2CR              (STM32_LTDC_BASE + STM32_LTDC_L2CR_OFFSET)
#define STM32_LTDC_L2WHPCR           (STM32_LTDC_BASE + STM32_LTDC_L2WHPCR_OFFSET)
#define STM32_LTDC_L2WVPCR           (STM32_LTDC_BASE + STM32_LTDC_L2WVPCR_OFFSET)
#define STM32_LTDC_L2CKCR            (STM32_LTDC_BASE + STM32_LTDC_L2CKCR_OFFSET)
#define STM32_LTDC_L2PFCR            (STM32_LTDC_BASE + STM32_LTDC_L2PFCR_OFFSET)
#define STM32_LTDC_L2CACR            (STM32_LTDC_BASE + STM32_LTDC_L2CACR_OFFSET)
#define STM32_LTDC_L2DCCR            (STM32_LTDC_BASE + STM32_LTDC_L2DCCR_OFFSET)
#define STM32_LTDC_L2BFCR            (STM32_LTDC_BASE + STM32_LTDC_L2BFCR_OFFSET)
#define STM32_LTDC_L2CFBAR           (STM32_LTDC_BASE + STM32_LTDC_L2CFBAR_OFFSET)
#define STM32_LTDC_L2CFBLR           (STM32_LTDC_BASE + STM32_LTDC_L2CFBLR_OFFSET)
#define STM32_LTDC_L2CFBLNR          (STM32_LTDC_BASE + STM32_LTDC_L2CFBLNR_OFFSET)
#define STM32_LTDC_L2CLUTWR          (STM32_LTDC_BASE + STM32_LTDC_L2CLUTWR_OFFSET)

/* Register Bitfield Definitions ********************************************/

/* LTDC Synchronization Size Configuration Register */

#define LTDC_SSCR_VSH_SHIFT          (0)       /* Bits 0-10: Vertical Synchronization Height */
#define LTDC_SSCR_VSH_MASK           (0x7ff << LTDC_SSCR_VSH_SHIFT)
#define LTDC_SSCR_HSW_SHIFT          (16)      /* Bits 16-26: Horizontal Synchronization Width */
#define LTDC_SSCR_HSW_MASK           (0x7ff << LTDC_SSCR_HSW_SHIFT)

/* LTDC Back Porch Configuration Register */

#define LTDC_BPCR_AVBP_SHIFT         (0)       /* Bits 0-10: Accumulated Vertical Back Porch */
#define LTDC_BPCR_AVBP_MASK          (0x7ff << LTDC_BPCR_AVBP_SHIFT)
#define LTDC_BPCR_AHBP_SHIFT         (16)      /* Bits 16-26: Accumulated Horizontal Back Porch */
#define LTDC_BPCR_AHBP_MASK          (0x7ff << LTDC_BPCR_AHBP_SHIFT)

/* LTDC Active Width Configuration Register */

#define LTDC_AWCR_AAH_SHIFT          (0)       /* Bits 0-10: Accumulated Active Height */
#define LTDC_AWCR_AAH_MASK           (0x7ff << LTDC_AWCR_AAH_SHIFT)
#define LTDC_AWCR_AAW_SHIFT          (16)      /* Bits 16-26: Accumulated Active Width */
#define LTDC_AWCR_AAW_MASK           (0x7ff << LTDC_AWCR_AAW_SHIFT)

/* LTDC Total Width Configuration Register */

#define LTDC_TWCR_TOTALH_SHIFT       (0)       /* Bits 0-10: Total Height */
#define LTDC_TWCR_TOTALH_MASK        (0x7ff << LTDC_TWCR_TOTALH_SHIFT)
#define LTDC_TWCR_TOTALW_SHIFT       (16)      /* Bits 16-26: Total Width */
#define LTDC_TWCR_TOTALW_MASK        (0x7ff << LTDC_TWCR_TOTALW_SHIFT)

/* LTDC Global Control Register */

#define LTDC_GCR_LTDCEN              (1 << 0)  /* Bit 0: LCD-TFT controller enable bit */
#define LTDC_GCR_DBW_SHIFT           (4)       /* Bits 4-7: Dither Blue Width */
#define LTDC_GCR_DBW_MASK            (0xf << LTDC_GCR_DBW_SHIFT)
#define LTDC_GCR_DGW_SHIFT           (8)       /* Bits 8-11: Dither Green Width */
#define LTDC_GCR_DGW_MASK            (0xf << LTDC_GCR_DGW_SHIFT)
#define LTDC_GCR_DRW_SHIFT           (12)      /* Bits 12-15: Dither Red Width */
#define LTDC_GCR_DRW_MASK            (0xf << LTDC_GCR_DRW_SHIFT)
#define LTDC_GCR_DEN                 (1 << 16) /* Bit 16: Dither Enable */
#define LTDC_GCR_PCPOL               (1 << 28) /* Bit 28: Pixel Clock Polarity */
#define LTDC_GCR_DEPOL               (1 << 29) /* Bit 29: Data Enable Polarity */
#define LTDC_GCR_VSPOL               (1 << 30) /* Bit 30: Vertical Synchronization Polarity */
#define LTDC_GCR_HSPOL               (1 << 31) /* Bit 31: Horizontal Synchronization Polarity */

/* LTDC Shadow Reload Control Register */

#define LTDC_SRCR_IMR                (1 << 0)  /* Bit 0: Immediate Reload */
#define LTDC_SRCR_VBR                (1 << 1)  /* Bit 1: Vertical Blanking Reload */

/* LTDC Background Color Configuration Register */

#define LTDC_BCCR_BCBLUE_SHIFT       (0)       /* Bits 0-7: Background Blue value */
#define LTDC_BCCR_BCBLUE_MASK        (0xff << LTDC_BCCR_BCBLUE_SHIFT)
#define LTDC_BCCR_BCGREEN_SHIFT      (8)       /* Bits 8-15: Background Green value */
#define LTDC_BCCR_BCGREEN_MASK       (0xff << LTDC_BCCR_BCGREEN_SHIFT)
#define LTDC_BCCR_BCRED_SHIFT        (16)      /* Bits 16-23: Background Red value */
#define LTDC_BCCR_BCRED_MASK         (0xff << LTDC_BCCR_BCRED_SHIFT)

/* LTDC Interrupt Enable Register */

#define LTDC_IER_LIE                 (1 << 0)  /* Bit 0: Line Interrupt Enable */
#define LTDC_IER_FUIE                (1 << 1)  /* Bit 1: FIFO Underrun Interrupt Enable */
#define LTDC_IER_TERRIE              (1 << 2)  /* Bit 2: Transfer Error Interrupt Enable */
#define LTDC_IER_RRIE                (1 << 3)  /* Bit 3: Register Reload interrupt enable */

/* LTDC Interrupt Status Register */

#define LTDC_ISR_LIF                 (1 << 0)  /* Bit 0: Line Interrupt Flag */
#define LTDC_ISR_FUIF                (1 << 1)  /* Bit 1: FIFO Underrun Interrupt Flag */
#define LTDC_ISR_TERRIF              (1 << 2)  /* Bit 2: Transfer Error Interrupt Flag */
#define LTDC_ISR_RRIF                (1 << 3)  /* Bit 3: Register Reload interrupt Flag */

/* LTDC Interrupt Clear Register */

#define LTDC_ICR_CLIF                (1 << 0)  /* Bit 0: Clears the Line Interrupt Flag */
#define LTDC_ICR_CFUIF               (1 << 1)  /* Bit 1: Clears the FIFO Underrun Interrupt Flag */
#define LTDC_ICR_CTERRIF             (1 << 2)  /* Bit 2: Clears the Transfer Error Interrupt Flag */
#define LTDC_ICR_CRRIF               (1 << 3)  /* Bit 3: Clears Register Reload interrupt Flag */

/* LTDC Layer Control Register */

#define LTDC_LCR_LEN                 (1 << 0)  /* Bit 0: Layer Enable */
#define LTDC_LCR_COLKEN              (1 << 1)  /* Bit 1: Color Keying Enable */
#define LTDC_LCR_CLUTEN              (1 << 4)  /* Bit 4: Color Lockup Table Enable */

/* LTDC Pixel Format Register */

#define LTDC_PFCR_PF_SHIFT           (0)       /* Bits 0-2: Pixel Format */
#define LTDC_PFCR_PF_MASK            (0x7 << LTDC_PFCR_PF_SHIFT)
#  define LTDC_PFCR_PF_ARGB8888      (0 << LTDC_PFCR_PF_SHIFT)  /* ARGB8888 */
#  define LTDC_PFCR_PF_RGB888        (1 << LTDC_PFCR_PF_SHIFT)  /* RGB888 */
#  define LTDC_PFCR_PF_RGB565        (2 << LTDC_PFCR_PF_SHIFT)  /* RGB565 */
#  define LTDC_PFCR_PF_ARGB1555      (3 << LTDC_PFCR_PF_SHIFT)  /* ARGB1555 */
#  define LTDC_PFCR_PF_ARGB4444      (4 << LTDC_PFCR_PF_SHIFT)  /* ARGB4444 */
#  define LTDC_PFCR_PF_L8            (5 << LTDC_PFCR_PF_SHIFT)  /* L8 */
#  define LTDC_PFCR_PF_AL44          (6 << LTDC_PFCR_PF_SHIFT)  /* AL44 */
#  define LTDC_PFCR_PF_AL88          (7 << LTDC_PFCR_PF_SHIFT)  /* AL88 */

/* LTDC Constant Alpha Register */

#define LTDC_CACR_CONSTA_SHIFT       (0)       /* Bits 0-7: Constant Alpha */
#define LTDC_CACR_CONSTA_MASK        (0xff << LTDC_CACR_CONSTA_SHIFT)

/* LTDC Blending Factor Register */

#define LTDC_BFCR_BF2_SHIFT          (0)       /* Bits 0-2: Blending Factor 2 */
#define LTDC_BFCR_BF2_MASK           (0x7 << LTDC_BFCR_BF2_SHIFT)
#  define LTDC_BFCR_BF2_CA           (0 << LTDC_BFCR_BF2_SHIFT)  /* Const Alpha */
#  define LTDC_BFCR_BF2_PAxCA        (1 << LTDC_BFCR_BF2_SHIFT)  /* Pixel Alpha * Const Alpha */
#  define LTDC_BFCR_BF2_NOCONTRA     (2 << LTDC_BFCR_BF2_SHIFT)  /* 1 - (Pixel Alpha * Const Alpha) */
#  define LTDC_BFCR_BF2_NOPIXEL      (3 << LTDC_BFCR_BF2_SHIFT)  /* 1 - (Pixel Alpha * Const Alpha) */
#define LTDC_BFCR_BF1_SHIFT          (8)       /* Bits 8-10: Blending Factor 1 */
#define LTDC_BFCR_BF1_MASK           (0x7 << LTDC_BFCR_BF1_SHIFT)
#  define LTDC_BFCR_BF1_CA           (0 << LTDC_BFCR_BF1_SHIFT)  /* Const Alpha */
#  define LTDC_BFCR_BF1_PAxCA        (1 << LTDC_BFCR_BF1_SHIFT)  /* Pixel Alpha * Const Alpha */

/* LTDC Color Frame Buffer Length Register */

#define LTDC_CFBLR_CFBLL_SHIFT       (0)       /* Bits 0-13: Color Frame Buffer Line Length */
#define LTDC_CFBLR_CFBLL_MASK        (0x3fff << LTDC_CFBLR_CFBLL_SHIFT)
#define LTDC_CFBLR_CFBP_SHIFT        (16)      /* Bits 16-29: Color Frame Buffer Pitch in bytes */
#define LTDC_CFBLR_CFBP_MASK         (0x3fff << LTDC_CFBLR_CFBP_SHIFT)

/* LTDC Color Frame Buffer Line Number Register */

#define LTDC_CFBLNR_CFBLNBR_SHIFT    (0)       /* Bits 0-10: Color Frame Buffer Line Number */
#define LTDC_CFBLNR_CFBLNBR_MASK     (0x7ff << LTDC_CFBLNR_CFBLNBR_SHIFT)

#endif /* __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32_LTDC_H */