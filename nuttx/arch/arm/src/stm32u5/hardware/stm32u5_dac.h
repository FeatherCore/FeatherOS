/****************************************************************************
 * arch/arm/src/stm32u5/hardware/stm32u5_dac.h
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

#ifndef __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32U5_DAC_H
#define __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32U5_DAC_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include "chip.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Register Offsets *********************************************************/

#define STM32U5_DAC_CR_OFFSET       0x0000  /* DAC control register */
#define STM32U5_DAC_SWTRIGR_OFFSET  0x0004  /* DAC software trigger register */
#define STM32U5_DAC_DHR12R1_OFFSET  0x0008  /* DAC channel 1 12-bit right-aligned data holding register */
#define STM32U5_DAC_DHR12L1_OFFSET  0x000c  /* DAC channel 1 12-bit left aligned data holding register */
#define STM32U5_DAC_DHR8R1_OFFSET  0x0010  /* DAC channel 1 8-bit right aligned data holding register */
#define STM32U5_DAC_DHR12R2_OFFSET  0x0014  /* DAC channel 2 12-bit right aligned data holding register */
#define STM32U5_DAC_DHR12L2_OFFSET  0x0018  /* DAC channel 2 12-bit left aligned data holding register */
#define STM32U5_DAC_DHR8R2_OFFSET  0x001c  /* DAC channel 2 8-bit right-aligned data holding register */
#define STM32U5_DAC_DHR12RD_OFFSET  0x0020  /* Dual DAC 12-bit right-aligned data holding register */
#define STM32U5_DAC_DHR12LD_OFFSET  0x0024  /* DUAL DAC 12-bit left aligned data holding register */
#define STM32U5_DAC_DHR8RD_OFFSET  0x0028  /* DUAL DAC 8-bit right aligned data holding register */
#define STM32U5_DAC_DOR1_OFFSET     0x002c  /* DAC channel 1 data output register */
#define STM32U5_DAC_DOR2_OFFSET     0x0030  /* DAC channel 2 data output register */
#define STM32U5_DAC_SR_OFFSET       0x0034  /* DAC status register */
#define STM32U5_DAC_CCR_OFFSET      0x0038  /* DAC calibration control register */
#define STM32U5_DAC_MCR_OFFSET      0x003c  /* DAC mode control register */
#define STM32U5_DAC_SHSR1_OFFSET    0x0040  /* DAC Sample and Hold sample time register 1 */
#define STM32U5_DAC_SHSR2_OFFSET    0x0044  /* DAC Sample and Hold sample time register 2 */
#define STM32U5_DAC_SHHR_OFFSET     0x0048  /* DAC Sample and Hold hold time register */
#define STM32U5_DAC_SHRR_OFFSET     0x004c  /* DAC Sample and Hold refresh time register */

/* Register Addresses *******************************************************/

#define STM32U5_DAC_CR              (STM32_DAC1_BASE+STM32U5_DAC_CR_OFFSET)
#define STM32U5_DAC_SWTRIGR         (STM32_DAC1_BASE+STM32U5_DAC_SWTRIGR_OFFSET)
#define STM32U5_DAC_DHR12R1        (STM32_DAC1_BASE+STM32U5_DAC_DHR12R1_OFFSET)
#define STM32U5_DAC_DHR12L1        (STM32_DAC1_BASE+STM32U5_DAC_DHR12L1_OFFSET)
#define STM32U5_DAC_DHR8R1         (STM32_DAC1_BASE+STM32U5_DAC_DHR8R1_OFFSET)
#define STM32U5_DAC_DHR12R2        (STM32_DAC1_BASE+STM32U5_DAC_DHR12R2_OFFSET)
#define STM32U5_DAC_DHR12L2        (STM32_DAC1_BASE+STM32U5_DAC_DHR12L2_OFFSET)
#define STM32U5_DAC_DHR8R2         (STM32_DAC1_BASE+STM32U5_DAC_DHR8R2_OFFSET)
#define STM32U5_DAC_DHR12RD        (STM32_DAC1_BASE+STM32U5_DAC_DHR12RD_OFFSET)
#define STM32U5_DAC_DHR12LD        (STM32_DAC1_BASE+STM32U5_DAC_DHR12LD_OFFSET)
#define STM32U5_DAC_DHR8RD         (STM32_DAC1_BASE+STM32U5_DAC_DHR8RD_OFFSET)
#define STM32U5_DAC_DOR1            (STM32_DAC1_BASE+STM32U5_DAC_DOR1_OFFSET)
#define STM32U5_DAC_DOR2            (STM32_DAC1_BASE+STM32U5_DAC_DOR2_OFFSET)
#define STM32U5_DAC_SR              (STM32_DAC1_BASE+STM32U5_DAC_SR_OFFSET)
#define STM32U5_DAC_CCR             (STM32_DAC1_BASE+STM32U5_DAC_CCR_OFFSET)
#define STM32U5_DAC_MCR             (STM32_DAC1_BASE+STM32U5_DAC_MCR_OFFSET)
#define STM32U5_DAC_SHSR1           (STM32_DAC1_BASE+STM32U5_DAC_SHSR1_OFFSET)
#define STM32U5_DAC_SHSR2           (STM32_DAC1_BASE+STM32U5_DAC_SHSR2_OFFSET)
#define STM32U5_DAC_SHHR            (STM32_DAC1_BASE+STM32U5_DAC_SHHR_OFFSET)
#define STM32U5_DAC_SHRR            (STM32_DAC1_BASE+STM32U5_DAC_SHRR_OFFSET)

/* Register Bitfield Definitions ********************************************/

/* DAC control register */

#define DAC_CR_EN1                (1 << 0)   /* Bit 0: DAC channel 1 enable */
#define DAC_CR_BOFF1             (1 << 1)   /* Bit 1: DAC channel 1 output buffer disable */
#define DAC_CR_TEN1              (1 << 2)   /* Bit 2: DAC channel 1 trigger enable */
#define DAC_CR_TSEL1_SHIFT       (3)        /* Bits 3-5: DAC channel 1 trigger selection */
#define DAC_CR_TSEL1_MASK        (7 << DAC_CR_TSEL1_SHIFT)
#define DAC_CR_TSEL1_TIM6        (0 << DAC_CR_TSEL1_SHIFT)  /* Timer 6 TRGO event */
#define DAC_CR_TSEL1_TIM8        (1 << DAC_CR_TSEL1_SHIFT)  /* Timer 8 TRGO event */
#define DAC_CR_TSEL1_TIM7        (2 << DAC_CR_TSEL1_SHIFT)  /* Timer 7 TRGO event */
#define DAC_CR_TSEL1_TIM5        (3 << DAC_CR_TSEL1_SHIFT)  /* Timer 5 TRGO event */
#define DAC_CR_TSEL1_TIM2        (4 << DAC_CR_TSEL1_SHIFT)  /* Timer 2 TRGO event */
#define DAC_CR_TSEL1_TIM4        (5 << DAC_CR_TSEL1_SHIFT)  /* Timer 4 TRGO event */
#define DAC_CR_TSEL1_EXT9        (6 << DAC_CR_TSEL1_SHIFT)  /* External line9 */
#define DAC_CR_TSEL1_SW          (7 << DAC_CR_TSEL1_SHIFT)  /* Software trigger */

#define DAC_CR_WAVE1_SHIFT       (6)        /* Bits 6-7: DAC channel 1 noise/triangle wave generation */
#define DAC_CR_WAVE1_MASK        (3 << DAC_CR_WAVE1_SHIFT)
#define DAC_CR_WAVE1_DISABLED    (0 << DAC_CR_WAVE1_SHIFT)  /* Wave generation disabled */
#define DAC_CR_WAVE1_NOISE       (1 << DAC_CR_WAVE1_SHIFT)  /* Noise wave generation enabled */
#define DAC_CR_WAVE1_TRIANGLE    (2 << DAC_CR_WAVE1_SHIFT)  /* Triangle wave generation enabled */

#define DAC_CR_MAMP1_SHIFT       (8)       /* Bits 8-11: DAC channel 1 mask/amplitude selector */
#define DAC_CR_MAMP1_MASK        (15 << DAC_CR_MAMP1_SHIFT)
#define DAC_CR_MAMP1_AMP1        (0 << DAC_CR_MAMP1_SHIFT)   /* Amplitude = 1 */
#define DAC_CR_MAMP1_AMP3        (1 << DAC_CR_MAMP1_SHIFT)   /* Amplitude = 3 */
#define DAC_CR_MAMP1_AMP7        (2 << DAC_CR_MAMP1_SHIFT)   /* Amplitude = 7 */
#define DAC_CR_MAMP1_AMP15       (3 << DAC_CR_MAMP1_SHIFT)   /* Amplitude = 15 */
#define DAC_CR_MAMP1_AMP31       (4 << DAC_CR_MAMP1_SHIFT)   /* Amplitude = 31 */
#define DAC_CR_MAMP1_AMP63       (5 << DAC_CR_MAMP1_SHIFT)   /* Amplitude = 63 */
#define DAC_CR_MAMP1_AMP127      (6 << DAC_CR_MAMP1_SHIFT)   /* Amplitude = 127 */
#define DAC_CR_MAMP1_AMP255      (7 << DAC_CR_MAMP1_SHIFT)   /* Amplitude = 255 */
#define DAC_CR_MAMP1_AMP511      (8 << DAC_CR_MAMP1_SHIFT)   /* Amplitude = 511 */
#define DAC_CR_MAMP1_AMP1023     (9 << DAC_CR_MAMP1_SHIFT)   /* Amplitude = 1023 */
#define DAC_CR_MAMP1_AMP2047     (10 << DAC_CR_MAMP1_SHIFT)  /* Amplitude = 2047 */
#define DAC_CR_MAMP1_AMP4095     (11 << DAC_CR_MAMP1_SHIFT)  /* Amplitude = 4095 */

#define DAC_CR_DMAEN1             (1 << 12)  /* Bit 12: DAC channel 1 DMA enable */
#define DAC_CR_DMAUDRIE1         (1 << 13)  /* Bit 13: DAC channel 1 DMA underrun interrupt enable */
#define DAC_CR_CEN1              (1 << 14)  /* Bit 14: DAC channel 1 calibration enable */

#define DAC_CR_EN2                (1 << 16)  /* Bit 16: DAC channel 2 enable */
#define DAC_CR_BOFF2             (1 << 17)  /* Bit 17: DAC channel 2 output buffer disable */
#define DAC_CR_TEN2              (1 << 18)  /* Bit 18: DAC channel 2 trigger enable */
#define DAC_CR_TSEL2_SHIFT       (19)       /* Bits 19-21: DAC channel 2 trigger selection */
#define DAC_CR_TSEL2_MASK        (7 << DAC_CR_TSEL2_SHIFT)
#define DAC_CR_TSEL2_TIM6        (0 << DAC_CR_TSEL2_SHIFT)  /* Timer 6 TRGO event */
#define DAC_CR_TSEL2_TIM8        (1 << DAC_CR_TSEL2_SHIFT)  /* Timer 8 TRGO event */
#define DAC_CR_TSEL2_TIM7        (2 << DAC_CR_TSEL2_SHIFT)  /* Timer 7 TRGO event */
#define DAC_CR_TSEL2_TIM5        (3 << DAC_CR_TSEL2_SHIFT)  /* Timer 5 TRGO event */
#define DAC_CR_TSEL2_TIM2        (4 << DAC_CR_TSEL2_SHIFT)  /* Timer 2 TRGO event */
#define DAC_CR_TSEL2_TIM4        (5 << DAC_CR_TSEL2_SHIFT)  /* Timer 4 TRGO event */
#define DAC_CR_TSEL2_EXT9        (6 << DAC_CR_TSEL2_SHIFT)  /* External line9 */
#define DAC_CR_TSEL2_SW          (7 << DAC_CR_TSEL2_SHIFT)  /* Software trigger */

#define DAC_CR_WAVE2_SHIFT       (22)       /* Bits 22-23: DAC channel 2 noise/triangle wave generation */
#define DAC_CR_WAVE2_MASK        (3 << DAC_CR_WAVE2_SHIFT)
#define DAC_CR_WAVE2_DISABLED    (0 << DAC_CR_WAVE2_SHIFT)  /* Wave generation disabled */
#define DAC_CR_WAVE2_NOISE       (1 << DAC_CR_WAVE2_SHIFT)  /* Noise wave generation enabled */
#define DAC_CR_WAVE2_TRIANGLE    (2 << DAC_CR_WAVE2_SHIFT)  /* Triangle wave generation enabled */

#define DAC_CR_MAMP2_SHIFT       (24)       /* Bits 24-27: DAC channel 2 mask/amplitude selector */
#define DAC_CR_MAMP2_MASK        (15 << DAC_CR_MAMP2_SHIFT)
#define DAC_CR_MAMP2_AMP1        (0 << DAC_CR_MAMP2_SHIFT)   /* Amplitude = 1 */
#define DAC_CR_MAMP2_AMP3        (1 << DAC_CR_MAMP2_SHIFT)   /* Amplitude = 3 */
#define DAC_CR_MAMP2_AMP7        (2 << DAC_CR_MAMP2_SHIFT)   /* Amplitude = 7 */
#define DAC_CR_MAMP2_AMP15       (3 << DAC_CR_MAMP2_SHIFT)   /* Amplitude = 15 */
#define DAC_CR_MAMP2_AMP31       (4 << DAC_CR_MAMP2_SHIFT)   /* Amplitude = 31 */
#define DAC_CR_MAMP2_AMP63       (5 << DAC_CR_MAMP2_SHIFT)   /* Amplitude = 63 */
#define DAC_CR_MAMP2_AMP127      (6 << DAC_CR_MAMP2_SHIFT)   /* Amplitude = 127 */
#define DAC_CR_MAMP2_AMP255      (7 << DAC_CR_MAMP2_SHIFT)   /* Amplitude = 255 */
#define DAC_CR_MAMP2_AMP511      (8 << DAC_CR_MAMP2_SHIFT)   /* Amplitude = 511 */
#define DAC_CR_MAMP2_AMP1023     (9 << DAC_CR_MAMP2_SHIFT)   /* Amplitude = 1023 */
#define DAC_CR_MAMP2_AMP2047     (10 << DAC_CR_MAMP2_SHIFT)  /* Amplitude = 2047 */
#define DAC_CR_MAMP2_AMP4095     (11 << DAC_CR_MAMP2_SHIFT)  /* Amplitude = 4095 */

#define DAC_CR_DMAEN2             (1 << 28)  /* Bit 28: DAC channel 2 DMA enable */
#define DAC_CR_DMAUDRIE2          (1 << 29)  /* Bit 29: DAC channel 2 DMA underrun interrupt enable */
#define DAC_CR_CEN2              (1 << 30)  /* Bit 30: DAC channel 2 calibration enable */

/* DAC software trigger register */

#define DAC_SWTRIGR_SWTRIG1      (1 << 0)   /* Bit 0: DAC channel 1 software trigger */
#define DAC_SWTRIGR_SWTRIG2      (1 << 1)   /* Bit 1: DAC channel 2 software trigger */

/* DAC data holding registers */

#define DAC_DHR12R_MASK          (0x0fff)   /* 12-bit right aligned data mask */
#define DAC_DHR12L_MASK          (0xfff0)   /* 12-bit left aligned data mask */
#define DAC_DHR8R_MASK           (0x00ff)    /* 8-bit right aligned data mask */

/* Dual DAC 12-bit right-aligned data holding register */

#define DAC_DHR12RD_DACC1_SHIFT  (0)        /* Bits 0-11: DAC channel 1 12-bit right-aligned data */
#define DAC_DHR12RD_DACC1_MASK   (0xfff << DAC_DHR12RD_DACC1_SHIFT)
#define DAC_DHR12RD_DACC2_SHIFT  (16)       /* Bits 16-27: DAC channel 2 12-bit right-aligned data */
#define DAC_DHR12RD_DACC2_MASK   (0xfff << DAC_DHR12RD_DACC2_SHIFT)

/* Dual DAC 12-bit left-aligned data holding register */

#define DAC_DHR12LD_DACC1_SHIFT  (4)        /* Bits 4-15: DAC channel 1 12-bit left-aligned data */
#define DAC_DHR12LD_DACC1_MASK   (0xfff << DAC_DHR12LD_DACC1_SHIFT)
#define DAC_DHR12LD_DACC2_SHIFT  (20)       /* Bits 20-31: DAC channel 2 12-bit left-aligned data */
#define DAC_DHR12LD_DACC2_MASK   (0xfff << DAC_DHR12LD_DACC2_SHIFT)

/* Dual DAC 8-bit right aligned data holding register */

#define DAC_DHR8RD_DACC1_SHIFT   (0)        /* Bits 0-7: DAC channel 1 8-bit right-aligned data */
#define DAC_DHR8RD_DACC1_MASK    (0xff << DAC_DHR8RD_DACC1_SHIFT)
#define DAC_DHR8RD_DACC2_SHIFT   (8)        /* Bits 8-15: DAC channel 2 8-bit right-aligned data */
#define DAC_DHR8RD_DACC2_MASK    (0xff << DAC_DHR8RD_DACC2_SHIFT)

/* DAC data output register */

#define DAC_DOR_MASK             (0x0fff)   /* 12-bit data output mask */

/* DAC status register */

#define DAC_SR_DMAUDR1          (1 << 13)  /* Bit 13: DAC channel 1 DMA underrun flag */
#define DAC_SR_CALFLAG1         (1 << 14)  /* Bit 14: DAC channel 1 calibration offset status */
#define DAC_SR_BWST1            (1 << 15)  /* Bit 15: DAC channel 1 busy writing sample time flag */
#define DAC_SR_DMAUDR2          (1 << 29)  /* Bit 29: DAC channel 2 DMA underrun flag */
#define DAC_SR_CALFLAG2         (1 << 30)  /* Bit 30: DAC channel 2 calibration offset status */
#define DAC_SR_BWST2            (1 << 31)  /* Bit 31: DAC channel 2 busy writing sample time flag */

/* DAC calibration control register */

#define DAC_CCR_OTRIM1_SHIFT    (0)        /* Bits 0-4: DAC channel 1 offset trimming value */
#define DAC_CCR_OTRIM1_MASK     (0x1f << DAC_CCR_OTRIM1_SHIFT)
#define DAC_CCR_OTRIM2_SHIFT    (16)       /* Bits 16-20: DAC channel 2 offset trimming value */
#define DAC_CCR_OTRIM2_MASK     (0x1f << DAC_CCR_OTRIM2_SHIFT)

/* DAC mode control register */

#define DAC_MCR_MODE1_SHIFT      (0)        /* Bits 0-2: DAC channel 1 mode */
#define DAC_MCR_MODE1_MASK       (7 << DAC_MCR_MODE1_SHIFT)
#define DAC_MCR_MODE1_NORMAL     (0)         /* Normal mode */
#define DAC_MCR_MODE1_SAMPLE    (1)         /* Sample and hold mode */

#define DAC_MCR_MODE2_SHIFT      (16)       /* Bits 16-18: DAC channel 2 mode */
#define DAC_MCR_MODE2_MASK       (7 << DAC_MCR_MODE2_SHIFT)
#define DAC_MCR_MODE2_NORMAL     (0)         /* Normal mode */
#define DAC_MCR_MODE2_SAMPLE     (1)         /* Sample and hold mode */

/* DAC Sample and Hold sample time register */

#define DAC_SHSR1_TSAMPLE1_MASK  (0x3ff)   /* 10-bit sample time mask */
#define DAC_SHSR2_TSAMPLE2_MASK (0x3ff)   /* 10-bit sample time mask */

/* DAC Sample and Hold hold time register */

#define DAC_SHHR_THOLD1_SHIFT    (0)        /* Bits 0-9: DAC channel 1 hold time */
#define DAC_SHHR_THOLD1_MASK     (0x3ff << DAC_SHHR_THOLD1_SHIFT)
#define DAC_SHHR_THOLD2_SHIFT    (16)       /* Bits 16-25: DAC channel 2 hold time */
#define DAC_SHHR_THOLD2_MASK     (0x3ff << DAC_SHHR_THOLD2_SHIFT)

/* DAC Sample and Hold refresh time register */

#define DAC_SHRR_TREFRESH1_SHIFT (0)        /* Bits 0-7: DAC channel 1 refresh time */
#define DAC_SHRR_TREFRESH1_MASK  (0xff << DAC_SHRR_TREFRESH1_SHIFT)
#define DAC_SHRR_TREFRESH2_SHIFT (16)       /* Bits 16-23: DAC channel 2 refresh time */
#define DAC_SHRR_TREFRESH2_MASK  (0xff << DAC_SHRR_TREFRESH2_SHIFT)

#endif /* __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32U5_DAC_H */