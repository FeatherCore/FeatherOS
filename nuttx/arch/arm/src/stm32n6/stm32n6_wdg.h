/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_wdg.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_STM32N6_WDG_H
#define __ARCH_ARM_SRC_STM32N6_STM32N6_WDG_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <stdint.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_IWDG_BASE            (STM32N6_PERIPH_BASE + 0x00048000)
#define STM32_WWDG_BASE           (STM32N6_PERIPH_BASE + 0x00002C00)

#define IWDG_KR_OFFSET           0x00
#define IWDG_PR_OFFSET           0x04
#define IWDG_RLR_OFFSET         0x08
#define IWDG_SR_OFFSET          0x0C
#define IWDG_WINR_OFFSET        0x10

#define IWDG_KR_UNLOCK          0xaaaa
#define IWDG_KR_START           0xcccc
#define IWDG_KR_RELOAD         0x5555

#define IWDG_PR_DIV_4           0
#define IWDG_PR_DIV_8           1
#define IWDG_PR_DIV_16          2
#define IWDG_PR_DIV_32          3
#define IWDG_PR_DIV_64          4
#define IWDG_PR_DIV_128         5
#define IWDG_PR_DIV_256         6

#define IWDG_SR_PVU            (1 << 0)
#define IWDG_SR_RVU            (1 << 1)
#define IWDG_SR_WVU            (1 << 2)

#define WWDG_CR_OFFSET          0x00
#define WWDG_CFR_OFFSET        0x04
#define WWDG_SR_OFFSET         0x08

#define WWDG_CR_T               0x7f
#define WWDG_CR_WDGA           (1 << 7)

#define WWDG_CFR_W             0x7f
#define WWDG_CFR_EWI           (1 << 9)

#define WWDG_SR_EWIF           (1 << 0)

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

int stm32n6_iwdg_init(uint32_t timeout_ms);
void stm32n6_iwdg_enable(void);
void stm32n6_iwdg_feed(void);
void stm32n6_iwdg_get_timeout(uint32_t *timeout_ms);
int stm32n6_wwdg_init(uint32_t timeout_ms, uint32_t window);
void stm32n6_wwdg_enable(void);
void stm32n6_wwdg_feed(void);

#endif /* __ARCH_ARM_SRC_STM32N6_STM32N6_WDG_H */