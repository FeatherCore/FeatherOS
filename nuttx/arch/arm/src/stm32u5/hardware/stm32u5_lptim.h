/****************************************************************************
 * arch/arm/src/stm32u5/hardware/stm32u5_lptim.h
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

#ifndef __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32U5_LPTIM_H
#define __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32U5_LPTIM_H

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Register Offsets *********************************************************/

#define STM32U5_LPTIM_ISR_OFFSET   0x0000  /* Interrupt and Status Register */
#define STM32U5_LPTIM_ICR_OFFSET   0x0004  /* Interrupt Clear Register */
#define STM32U5_LPTIM_IER_OFFSET   0x0008  /* Interrupt Enable Register */
#define STM32U5_LPTIM_CFGR_OFFSET  0x000c  /* Configuration Register */
#define STM32U5_LPTIM_CR_OFFSET    0x0010  /* Control Register */
#define STM32U5_LPTIM_CMP_OFFSET   0x0014  /* Compare Register */
#define STM32U5_LPTIM_ARR_OFFSET   0x0018  /* Autoreload Register */
#define STM32U5_LPTIM_CNT_OFFSET   0x001c  /* Counter Register */
#define STM32U5_LPTIM_CFGR2_OFFSET 0x0024  /* Configuration Register 2 */
#define STM32U5_LPTIM_OR_OFFSET    0x002c  /* Option Register */

/* Register Addresses *******************************************************/

#define STM32U5_LPTIM1_ISR         (STM32_LPTIM1_BASE+STM32U5_LPTIM_ISR_OFFSET)
#define STM32U5_LPTIM1_ICR         (STM32_LPTIM1_BASE+STM32U5_LPTIM_ICR_OFFSET)
#define STM32U5_LPTIM1_IER         (STM32_LPTIM1_BASE+STM32U5_LPTIM_IER_OFFSET)
#define STM32U5_LPTIM1_CFGR        (STM32_LPTIM1_BASE+STM32U5_LPTIM_CFGR_OFFSET)
#define STM32U5_LPTIM1_CR          (STM32_LPTIM1_BASE+STM32U5_LPTIM_CR_OFFSET)
#define STM32U5_LPTIM1_CMP         (STM32_LPTIM1_BASE+STM32U5_LPTIM_CMP_OFFSET)
#define STM32U5_LPTIM1_ARR         (STM32_LPTIM1_BASE+STM32U5_LPTIM_ARR_OFFSET)
#define STM32U5_LPTIM1_CNT         (STM32_LPTIM1_BASE+STM32U5_LPTIM_CNT_OFFSET)
#define STM32U5_LPTIM1_CFGR2       (STM32_LPTIM1_BASE+STM32U5_LPTIM_CFGR2_OFFSET)
#define STM32U5_LPTIM1_OR          (STM32_LPTIM1_BASE+STM32U5_LPTIM_OR_OFFSET)

#define STM32U5_LPTIM2_ISR         (STM32_LPTIM2_BASE+STM32U5_LPTIM_ISR_OFFSET)
#define STM32U5_LPTIM2_ICR         (STM32_LPTIM2_BASE+STM32U5_LPTIM_ICR_OFFSET)
#define STM32U5_LPTIM2_IER         (STM32_LPTIM2_BASE+STM32U5_LPTIM_IER_OFFSET)
#define STM32U5_LPTIM2_CFGR        (STM32_LPTIM2_BASE+STM32U5_LPTIM_CFGR_OFFSET)
#define STM32U5_LPTIM2_CR          (STM32_LPTIM2_BASE+STM32U5_LPTIM_CR_OFFSET)
#define STM32U5_LPTIM2_CMP         (STM32_LPTIM2_BASE+STM32U5_LPTIM_CMP_OFFSET)
#define STM32U5_LPTIM2_ARR         (STM32_LPTIM2_BASE+STM32U5_LPTIM_ARR_OFFSET)
#define STM32U5_LPTIM2_CNT         (STM32_LPTIM2_BASE+STM32U5_LPTIM_CNT_OFFSET)
#define STM32U5_LPTIM2_CFGR2       (STM32_LPTIM2_BASE+STM32U5_LPTIM_CFGR2_OFFSET)
#define STM32U5_LPTIM2_OR          (STM32_LPTIM2_BASE+STM32U5_LPTIM_OR_OFFSET)

#define STM32U5_LPTIM3_ISR         (STM32_LPTIM3_BASE+STM32U5_LPTIM_ISR_OFFSET)
#define STM32U5_LPTIM3_ICR         (STM32_LPTIM3_BASE+STM32U5_LPTIM_ICR_OFFSET)
#define STM32U5_LPTIM3_IER         (STM32_LPTIM3_BASE+STM32U5_LPTIM_IER_OFFSET)
#define STM32U5_LPTIM3_CFGR        (STM32_LPTIM3_BASE+STM32U5_LPTIM_CFGR_OFFSET)
#define STM32U5_LPTIM3_CR          (STM32_LPTIM3_BASE+STM32U5_LPTIM_CR_OFFSET)
#define STM32U5_LPTIM3_CMP         (STM32_LPTIM3_BASE+STM32U5_LPTIM_CMP_OFFSET)
#define STM32U5_LPTIM3_ARR         (STM32_LPTIM3_BASE+STM32U5_LPTIM_ARR_OFFSET)
#define STM32U5_LPTIM3_CNT         (STM32_LPTIM3_BASE+STM32U5_LPTIM_CNT_OFFSET)
#define STM32U5_LPTIM3_CFGR2       (STM32_LPTIM3_BASE+STM32U5_LPTIM_CFGR2_OFFSET)
#define STM32U5_LPTIM3_OR          (STM32_LPTIM3_BASE+STM32U5_LPTIM_OR_OFFSET)

#define STM32U5_LPTIM4_ISR         (STM32_LPTIM4_BASE+STM32U5_LPTIM_ISR_OFFSET)
#define STM32U5_LPTIM4_ICR         (STM32_LPTIM4_BASE+STM32U5_LPTIM_ICR_OFFSET)
#define STM32U5_LPTIM4_IER         (STM32_LPTIM4_BASE+STM32U5_LPTIM_IER_OFFSET)
#define STM32U5_LPTIM4_CFGR        (STM32_LPTIM4_BASE+STM32U5_LPTIM_CFGR_OFFSET)
#define STM32U5_LPTIM4_CR          (STM32_LPTIM4_BASE+STM32U5_LPTIM_CR_OFFSET)
#define STM32U5_LPTIM4_CMP         (STM32_LPTIM4_BASE+STM32U5_LPTIM_CMP_OFFSET)
#define STM32U5_LPTIM4_ARR         (STM32_LPTIM4_BASE+STM32U5_LPTIM_ARR_OFFSET)
#define STM32U5_LPTIM4_CNT         (STM32_LPTIM4_BASE+STM32U5_LPTIM_CNT_OFFSET)
#define STM32U5_LPTIM4_CFGR2       (STM32_LPTIM4_BASE+STM32U5_LPTIM_CFGR2_OFFSET)
#define STM32U5_LPTIM4_OR          (STM32_LPTIM4_BASE+STM32U5_LPTIM_OR_OFFSET)

/* Register Bitfield Definitions ********************************************/

/* Interrupt and Status Register (ISR) */

#define LPTIM_ISR_CMPM             (1 << 0)   /* Bit 0: Compare match */
#define LPTIM_ISR_ARRM             (1 << 1)   /* Bit 1: Autoreload match */
#define LPTIM_ISR_EXTTRIG          (1 << 2)   /* Bit 2: External trigger edge event */
#define LPTIM_ISR_CMPOK            (1 << 3)   /* Bit 3: Compare register update OK */
#define LPTIM_ISR_ARROK            (1 << 4)   /* Bit 4: Autoreload register update OK */
#define LPTIM_ISR_UP               (1 << 5)   /* Bit 5: Counter direction change down to up */
#define LPTIM_ISR_DOWN             (1 << 6)   /* Bit 6: Counter direction change up to down */
#define LPTIM_ISR_DIREOK           (1 << 10)  /* Bit 10: Direction change update OK */
#define LPTIM_ISRUEOK             (1 << 11)  /* Bit 11: UE flag update OK */

/* Interrupt Clear Register (ICR) */

#define LPTIM_ICR_CMPMCF           (1 << 0)   /* Bit 0: Compare match flag clear */
#define LPTIM_ICR_ARRMCF           (1 << 1)   /* Bit 1: Autoreload match flag clear */
#define LPTIM_ICR_EXTTRIGCF        (1 << 2)   /* Bit 2: External trigger edge event flag clear */
#define LPTIM_ICR_CMPOKCF          (1 << 3)   /* Bit 3: Compare register update OK flag clear */
#define LPTIM_ICR_ARROKCF          (1 << 4)   /* Bit 4: Autoreload register update OK flag clear */
#define LPTIM_ICR_UPCF             (1 << 5)   /* Bit 5: Counter direction change down to up flag clear */
#define LPTIM_ICR_DOWNCF           (1 << 6)   /* Bit 6: Counter direction change up to down flag clear */
#define LPTIM_ICR_DIREOKCF         (1 << 10)  /* Bit 10: Direction change update OK flag clear */
#define LPTIM_ICRUEOKCF           (1 << 11)  /* Bit 11: UE flag update OK flag clear */

/* Interrupt Enable Register (IER) */

#define LPTIM_IER_CMPMIE           (1 << 0)   /* Bit 0: Compare match interrupt enable */
#define LPTIM_IER_ARRMIE           (1 << 1)   /* Bit 1: Autoreload match interrupt enable */
#define LPTIM_IER_EXTTRIGIE        (1 << 2)   /* Bit 2: External trigger edge event interrupt enable */
#define LPTIM_IER_CMPOKIE          (1 << 3)   /* Bit 3: Compare register update OK interrupt enable */
#define LPTIM_IER_ARROKIE          (1 << 4)   /* Bit 4: Autoreload register update OK interrupt enable */
#define LPTIM_IERUPIE             (1 << 5)   /* Bit 5: Counter direction change down to up interrupt enable */
#define LPTIM_IER_DOWNIE           (1 << 6)   /* Bit 6: Counter direction change up to down interrupt enable */
#define LPTIM_IER_DIREOKIE         (1 << 10)  /* Bit 10: Direction change update OK interrupt enable */
#define LPTIM_IERUEOKIE           (1 << 11)  /* Bit 11: UE flag update OK interrupt enable */

/* Configuration Register (CFGR) */

#define LPTIM_CFGR_CKSEL           (1 << 0)   /* Bit 0: Clock selector */
#define LPTIM_CFGR_CKPOL_SHIFT     (1)        /* Bits 1-2: Clock polarity */
#define LPTIM_CFGR_CKPOL_MASK      (3 << LPTIM_CFGR_CKPOL_SHIFT)
#  define LPTIM_CFGR_CKPOL_RISING  (0 << LPTIM_CFGR_CKPOL_SHIFT)  /* Rising edge */
#  define LPTIM_CFGR_CKPOL_FALLING (1 << LPTIM_CFGR_CKPOL_SHIFT)  /* Falling edge */
#  define LPTIM_CFGR_CKPOL_BOTH    (2 << LPTIM_CFGR_CKPOL_SHIFT)  /* Both edges */

#define LPTIM_CFGR_CKFLT_SHIFT     (3)        /* Bits 3-4: Digital filter for external clock */
#define LPTIM_CFGR_CKFLT_MASK      (3 << LPTIM_CFGR_CKFLT_SHIFT)
#  define LPTIM_CFGR_CKFLT_NONE    (0 << LPTIM_CFGR_CKFLT_SHIFT)  /* No filter */
#  define LPTIM_CFGR_CKFLT_2       (1 << LPTIM_CFGR_CKFLT_SHIFT)  /* 2 clock cycles */
#  define LPTIM_CFGR_CKFLT_4       (2 << LPTIM_CFGR_CKFLT_SHIFT)  /* 4 clock cycles */
#  define LPTIM_CFGR_CKFLT_8       (3 << LPTIM_CFGR_CKFLT_SHIFT)  /* 8 clock cycles */

#define LPTIM_CFGR_TRGFLT_SHIFT    (6)        /* Bits 6-7: Digital filter for trigger */
#define LPTIM_CFGR_TRGFLT_MASK    (3 << LPTIM_CFGR_TRGFLT_SHIFT)
#  define LPTIM_CFGR_TRGFLT_NONE   (0 << LPTIM_CFGR_TRGFLT_SHIFT)  /* No filter */
#  define LPTIM_CFGR_TRGFLT_2      (1 << LPTIM_CFGR_TRGFLT_SHIFT)  /* 2 clock cycles */
#  define LPTIM_CFGR_TRGFLT_4      (2 << LPTIM_CFGR_TRGFLT_SHIFT)  /* 4 clock cycles */
#  define LPTIM_CFGR_TRGFLT_8      (3 << LPTIM_CFGR_TRGFLT_SHIFT)  /* 8 clock cycles */

#define LPTIM_CFGR_PRESC_SHIFT     (9)        /* Bits 9-11: Clock prescaler */
#define LPTIM_CFGR_PRESC_MASK      (7 << LPTIM_CFGR_PRESC_SHIFT)
#  define LPTIM_CFGR_PRESC_DIV1    (0 << LPTIM_CFGR_PRESC_SHIFT)  /* Divide by 1 */
#  define LPTIM_CFGR_PRESC_DIV2    (1 << LPTIM_CFGR_PRESC_SHIFT)  /* Divide by 2 */
#  define LPTIM_CFGR_PRESC_DIV4    (2 << LPTIM_CFGR_PRESC_SHIFT)  /* Divide by 4 */
#  define LPTIM_CFGR_PRESC_DIV8    (3 << LPTIM_CFGR_PRESC_SHIFT)  /* Divide by 8 */
#  define LPTIM_CFGR_PRESC_DIV16   (4 << LPTIM_CFGR_PRESC_SHIFT)  /* Divide by 16 */
#  define LPTIM_CFGR_PRESC_DIV32   (5 << LPTIM_CFGR_PRESC_SHIFT)  /* Divide by 32 */
#  define LPTIM_CFGR_PRESC_DIV64   (6 << LPTIM_CFGR_PRESC_SHIFT)  /* Divide by 64 */
#  define LPTIM_CFGR_PRESC_DIV128  (7 << LPTIM_CFGR_PRESC_SHIFT)  /* Divide by 128 */

#define LPTIM_CFGR_TRIGSEL_SHIFT   (13)       /* Bits 13-15: Trigger selector */
#define LPTIM_CFGR_TRIGSEL_MASK    (7 << LPTIM_CFGR_TRIGSEL_SHIFT)

#define LPTIM_CFGR_TRIGEN_SHIFT    (17)       /* Bits 17-18: Trigger enable and polarity */
#define LPTIM_CFGR_TRIGEN_MASK    (3 << LPTIM_CFGR_TRIGEN_SHIFT)
#  define LPTIM_CFGR_TRIGEN_NONE   (0 << LPTIM_CFGR_TRIGEN_SHIFT)  /* Trigger disabled */
#  define LPTIM_CFGR_TRIGEN_RISING (1 << LPTIM_CFGR_TRIGEN_SHIFT)  /* Rising edge */
#  define LPTIM_CFGR_TRIGEN_FALLING (2 << LPTIM_CFGR_TRIGEN_SHIFT) /* Falling edge */
#  define LPTIM_CFGR_TRIGEN_BOTH   (3 << LPTIM_CFGR_TRIGEN_SHIFT)  /* Both edges */

#define LPTIM_CFGR_TIMOUT          (1 << 19)  /* Bit 19: Timeout enable */
#define LPTIM_CFGR_WAVE            (1 << 20)  /* Bit 20: Waveform shape */
#define LPTIM_CFGR_WAVPOL          (1 << 21)  /* Bit 21: Waveform polarity */
#define LPTIM_CFGR_PRELOAD        (1 << 22)  /* Bit 22: Register update mode */
#define LPTIM_CFGR_COUNTMODE       (1 << 23)  /* Bit 23: Counter mode enable */
#define LPTIM_CFGR_ENC             (1 << 24)  /* Bit 24: Encoder mode enable */

/* Control Register (CR) */

#define LPTIM_CR_ENABLE            (1 << 0)   /* Bit 0: LPTIM enable */
#define LPTIM_CR_SNGSTRT           (1 << 1)   /* Bit 1: Timer start in single mode */
#define LPTIM_CR_CNTSTRT           (1 << 2)   /* Bit 2: Timer start in continuous mode */
#define LPTIM_CR_COUNTRST         (1 << 3)   /* Bit 3: Counter reset */
#define LPTIM_CR_RSTARE           (1 << 4)   /* Bit 4: Reset after read enable */

/* Compare Register (CMP) */

#define LPTIM_CMP_CMP_SHIFT        (0)        /* Bits 0-15: Compare value */
#define LPTIM_CMP_CMP_MASK        (0xffff << LPTIM_CMP_CMP_SHIFT)

/* Autoreload Register (ARR) */

#define LPTIM_ARR_ARR_SHIFT        (0)        /* Bits 0-15: Autoreload value */
#define LPTIM_ARR_ARR_MASK        (0xffff << LPTIM_ARR_ARR_SHIFT)

/* Counter Register (CNT) */

#define LPTIM_CNT_CNT_SHIFT        (0)        /* Bits 0-15: Counter value */
#define LPTIM_CNT_CNT_MASK        (0xffff << LPTIM_CNT_CNT_SHIFT)

/* Configuration Register 2 (CFGR2) */

#define LPTIM_CFGR2UESEL_SHIFT     (0)        /* Bits 0-3: Update event source selection */
#define LPTIM_CFGR2UESEL_MASK     (0xf << LPTIM_CFGR2UESEL_SHIFT)
#  define LPTIM_CFGR2UESEL_CMP    (0 << LPTIM_CFGR2UESEL_SHIFT)   /* UE from CMP match */
#  define LPTIM_CFGR2UESEL_ARR    (1 << LPTIM_CFGR2UESEL_SHIFT)    /* UE from ARR match */
#  define LPTIM_CFGR2UESEL_EXT    (2 << LPTIM_CFGR2UESEL_SHIFT)    /* UE from external trigger */
#  define LPTIM_CFGR2UESEL_CNT    (3 << LPTIM_CFGR2UESEL_OFFSET)   /* UE from counter overflow */

/* Option Register (OR) */

#define LPTIM_OR_OR_SHIFT          (0)        /* Bits 0-3: Option register */
#define LPTIM_OR_OR_MASK          (0xf << LPTIM_OR_OR_SHIFT)

#endif /* __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32U5_LPTIM_H */