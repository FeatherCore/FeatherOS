/****************************************************************************
 * arch/arm/src/stm32u5/hardware/stm32u5_wdg.h
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

#ifndef __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32U5_WDG_H
#define __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32U5_WDG_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include "chip.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Register Offsets *********************************************************/

/* Independent Watchdog (IWDG) */

#define STM32U5_IWDG_KR_OFFSET     0x0000  /* Key register */
#define STM32U5_IWDG_PR_OFFSET     0x0004  /* Prescaler register */
#define STM32U5_IWDG_RLR_OFFSET    0x0008  /* Reload register */
#define STM32U5_IWDG_SR_OFFSET     0x000c  /* Status register */
#define STM32U5_IWDG_WINR_OFFSET   0x0010  /* Window register */
#define STM32U5_IWDG_EWCR_OFFSET   0x0014  /* Early wakeup control register */

/* Window Watchdog (WWDG) */

#define STM32U5_WWDG_CR_OFFSET     0x0000  /* Control register */
#define STM32U5_WWDG_CFR_OFFSET    0x0004  /* Configuration register */
#define STM32U5_WWDG_SR_OFFSET     0x0008  /* Status register */

/* Register Addresses *******************************************************/

#define STM32U5_IWDG_KR            (STM32_IWDG_BASE+STM32U5_IWDG_KR_OFFSET)
#define STM32U5_IWDG_PR            (STM32_IWDG_BASE+STM32U5_IWDG_PR_OFFSET)
#define STM32U5_IWDG_RLR           (STM32_IWDG_BASE+STM32U5_IWDG_RLR_OFFSET)
#define STM32U5_IWDG_SR            (STM32_IWDG_BASE+STM32U5_IWDG_SR_OFFSET)
#define STM32U5_IWDG_WINR          (STM32_IWDG_BASE+STM32U5_IWDG_WINR_OFFSET)
#define STM32U5_IWDG_EWCR          (STM32_IWDG_BASE+STM32U5_IWDG_EWCR_OFFSET)

#define STM32U5_WWDG_CR            (STM32_WWDG_BASE+STM32U5_WWDG_CR_OFFSET)
#define STM32U5_WWDG_CFR           (STM32_WWDG_BASE+STM32U5_WWDG_CFR_OFFSET)
#define STM32U5_WWDG_SR            (STM32_WWDG_BASE+STM32U5_WWDG_SR_OFFSET)

/* Register Bitfield Definitions ********************************************/

/* Independent Watchdog Key Register (IWDG_KR) */

#define IWDG_KR_KEY_SHIFT          (0)       /* Bits 0-15: Key value */
#define IWDG_KR_KEY_MASK           (0xffff << IWDG_KR_KEY_SHIFT)

#define IWDG_KR_KEY_ENABLE         (0x5555)  /* Enable register access */
#define IWDG_KR_KEY_DISABLE        (0x0000)  /* Disable register access */
#define IWDG_KR_KEY_RELOAD         (0xaaaa)  /* Reload the counter */
#define IWDG_KR_KEY_START          (0xcccc)  /* Start the watchdog */

/* Independent Watchdog Prescaler Register (IWDG_PR) */

#define IWDG_PR_SHIFT              (0)       /* Bits 0-2: Prescaler divider */
#define IWDG_PR_MASK               (7 << IWDG_PR_SHIFT)
#define IWDG_PR_DIV4               (0 << IWDG_PR_SHIFT)  /* Divider /4 */
#define IWDG_PR_DIV8               (1 << IWDG_PR_SHIFT)  /* Divider /8 */
#define IWDG_PR_DIV16              (2 << IWDG_PR_SHIFT)  /* Divider /16 */
#define IWDG_PR_DIV32              (3 << IWDG_PR_SHIFT)  /* Divider /32 */
#define IWDG_PR_DIV64              (4 << IWDG_PR_SHIFT)  /* Divider /64 */
#define IWDG_PR_DIV128             (5 << IWDG_PR_SHIFT)  /* Divider /128 */
#define IWDG_PR_DIV256             (6 << IWDG_PR_SHIFT)  /* Divider /256 */

/* Independent Watchdog Reload Register (IWDG_RLR) */

#define IWDG_RLR_RL_SHIFT          (0)       /* Bits 0-11: Watchdog counter reload value */
#define IWDG_RLR_RL_MASK           (0x0fff << IWDG_RLR_RL_SHIFT)
#define IWDG_RLR_MAX               (0xfff)

/* Independent Watchdog Status Register (IWDG_SR) */

#define IWDG_SR_PVU                (1 << 0)  /* Bit 0: Prescaler value update */
#define IWDG_SR_RVU                (1 << 1)  /* Bit 1: Reload value update */
#define IWDG_SR_WVU                (1 << 2)  /* Bit 2: Window value update */
#define IWDG_SR_EWU                (1 << 3)  /* Bit 3: Early wakeup value update */

/* Independent Watchdog Window Register (IWDG_WINR) */

#define IWDG_WINR_WIN_SHIFT        (0)       /* Bits 0-11: Watchdog counter window value */
#define IWDG_WINR_WIN_MASK         (0x0fff << IWDG_WINR_WIN_SHIFT)

/* Independent Watchdog Early Wakeup Control Register (IWDG_EWCR) */

#define IWDG_EWCR_EWIT_SHIFT       (0)       /* Bits 0-11: Early wakeup interrupt threshold */
#define IWDG_EWCR_EWIT_MASK        (0x0fff << IWDG_EWCR_EWIT_SHIFT)
#define IWDG_EWCR_EWIE             (1 << 16)  /* Bit 16: Early wakeup interrupt enable */

/* Window Watchdog Control Register (WWDG_CR) */

#define WWDG_CR_T_SHIFT            (0)       /* Bits 0-6: 7-bit counter */
#define WWDG_CR_T_MASK             (0x7f << WWDG_CR_T_SHIFT)
#define WWDG_CR_T_MAX              (0x3f << WWDG_CR_T_SHIFT)
#define WWDG_CR_T_RESET            (0x40 << WWDG_CR_T_SHIFT)
#define WWDG_CR_WDGA               (1 << 7)  /* Bit 7: Activation bit */

/* Window Watchdog Configuration Register (WWDG_CFR) */

#define WWDG_CFR_W_SHIFT           (0)       /* Bits 0-6: 7-bit window value */
#define WWDG_CFR_W_MASK            (0x7f << WWDG_CFR_W_SHIFT)

#define WWDG_CFR_WDGTB_SHIFT       (7)       /* Bits 7-8: Timer base */
#define WWDG_CFR_WDGTB_MASK        (3 << WWDG_CFR_WDGTB_SHIFT)
#define WWDG_CFR_PCLK1             (0 << WWDG_CFR_WDGTB_SHIFT)  /* PCLK1/4096/1 */
#define WWDG_CFR_PCLK1d2           (1 << WWDG_CFR_WDGTB_SHIFT)  /* PCLK1/4096/2 */
#define WWDG_CFR_PCLK1d4           (2 << WWDG_CFR_WDGTB_SHIFT)  /* PCLK1/4096/4 */
#define WWDG_CFR_PCLK1d8           (3 << WWDG_CFR_WDGTB_SHIFT)  /* PCLK1/4096/8 */

#define WWDG_CFR_EWI               (1 << 9)  /* Bit 9: Early wakeup interrupt */

/* Window Watchdog Status Register (WWDG_SR) */

#define WWDG_SR_EWIF               (1 << 0)  /* Bit 0: Early wakeup interrupt flag */

#endif /* __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32U5_WDG_H */