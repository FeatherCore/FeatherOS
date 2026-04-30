/****************************************************************************
 * arch/arm/src/stm32u5/hardware/stm32u5_rng.h
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

#ifndef __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32U5_RNG_H
#define __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32U5_RNG_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include "chip.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Register Offsets *********************************************************/

#define STM32U5_RNG_CR_OFFSET     0x0000  /* RNG Control Register */
#define STM32U5_RNG_SR_OFFSET     0x0004  /* RNG Status Register */
#define STM32U5_RNG_DR_OFFSET     0x0008  /* RNG Data Register */
#define STM32U5_RNG_MSCR_OFFSET   0x000c  /* RNG Mode Control Register */
#define STM32U5_RNG_HTCR_OFFSET   0x0010  /* RNG Health Test Control Register */
#define STM32U5_RNG_HTCV_OFFSET   0x0014  /* RNG Health Test Configuration Value Register */

/* Register Addresses *******************************************************/

#define STM32U5_RNG_CR            (STM32U5_RNG_BASE+STM32U5_RNG_CR_OFFSET)
#define STM32U5_RNG_SR            (STM32U5_RNG_BASE+STM32U5_RNG_SR_OFFSET)
#define STM32U5_RNG_DR            (STM32U5_RNG_BASE+STM32U5_RNG_DR_OFFSET)
#define STM32U5_RNG_MSCR          (STM32U5_RNG_BASE+STM32U5_RNG_MSCR_OFFSET)
#define STM32U5_RNG_HTCR          (STM32U5_RNG_BASE+STM32U5_RNG_HTCR_OFFSET)
#define STM32U5_RNG_HTCV          (STM32U5_RNG_BASE+STM32U5_RNG_HTCV_OFFSET)

/* Register Bitfield Definitions ********************************************/

/* RNG Control Register (RNG_CR) */

#define RNG_CR_RNGEN              (1 << 2)   /* Bit 2: RNG enable */
#define RNG_CR_IE                 (1 << 3)   /* Bit 3: Interrupt enable */
#define RNG_CR_CED                (1 << 5)   /* Bit 5: Clock error detection disable */
#define RNG_CR_ENTEST             (1 << 6)   /* Bit 6: NIST test enable */
#define RNG_CR_CONDFST           (1 << 8)   /* Bit 8: Conditioning soft reset */
#define RNG_CR_NSEN               (1 << 12)  /* Bit 12: NIST small entropy mode */
#define RNG_CR_CLKDIV_SHIFT       (8)        /* Bits 8-11: Clock divider ratio */
#define RNG_CR_CLKDIV_MASK       (0xf << RNG_CR_CLKDIV_SHIFT)
#define RNG_CR_RNG_CONFIG3_SHIFT  (16)       /* Bits 16-23: RNG configuration 3 */
#define RNG_CR_RNG_CONFIG3_MASK   (0xff << RNG_CR_RNG_CONFIG3_SHIFT)
#define RNG_CR_NIST_CATEG_SHIFT   (24)       /* Bits 24-26: NIST compliance category */
#define RNG_CR_NIST_CATEG_MASK   (7 << RNG_CR_NIST_CATEG_SHIFT)
#define RNG_CR_RNG_CONFIG2_SHIFT  (28)       /* Bits 28-31: RNG configuration 2 */
#define RNG_CR_RNG_CONFIG2_MASK  (0xf << RNG_CR_RNG_CONFIG2_SHIFT)

/* RNG Status Register (RNG_SR) */

#define RNG_SR_DRDY               (1 << 0)   /* Bit 0: Data ready */
#define RNG_SR_CECS               (1 << 1)   /* Bit 1: Clock error current status */
#define RNG_SR_SECS               (1 << 2)   /* Bit 2: Seed error current status */
#define RNG_SR_CEIS               (1 << 5)   /* Bit 5: Clock error interrupt status */
#define RNG_SR_SEIS               (1 << 6)   /* Bit 6: Seed error interrupt status */
#define RNG_SR_TRNGER             (1 << 10)  /* Bit 10: Trng error status */
#define RNG_SR_TISTART            (1 << 11)  /* Bit 11: Test start interrupt status */
#define RNG_SR_FAULT_DETECTED     (1 << 12)  /* Bit 12: Fault detected */
#define RNG_SR_TFSLTF             (1 << 13)  /* Bit 13: Test failed since last seed */
#define RNG_SR_TSTSF              (1 << 14)  /* Bit 14: Test started flag */
#define RNG_SR_TCF                (1 << 15)  /* Bit 15: Test configuration flag */

/* RNG Mode Control Register (RNG_MSCR) */

#define RNG_MSCR_MRNGEN           (1 << 0)   /* Bit 0: Master RNG enable */

/* RNG Health Test Control Register (RNG_HTCR) */

#define RNG_HTCR_HTCFG_SHIFT     (0)        /* Bits 0-15: Health test configuration */
#define RNG_HTCR_HTCFG_MASK     (0xffff << RNG_HTCR_HTCFG_SHIFT)
#define RNG_HTCR_HTEIS           (1 << 16)   /* Bit 16: Health test error interrupt status */
#define RNG_HTCR_HTSTE           (1 << 17)   /* Bit 17: Health test start */

#endif /* __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32U5_RNG_H */