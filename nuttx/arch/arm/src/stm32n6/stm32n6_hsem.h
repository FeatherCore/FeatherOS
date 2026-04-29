/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_hsem.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_STM32N6_HSEM_H
#define __ARCH_ARM_SRC_STM32N6_STM32N6_HSEM_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <stdint.h>
#include <stdbool.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define HSEM_MAX_SEMAPHORES        32

#define HSEM_LOCK_WAIT_FOREVER     0xFFFFFFFFU
#define HSEM_LOCK_DEFAULT_RETRY    0x100000U

#define CFG_HW_RNG_SEMID           0U
#define CFG_HW_PKA_SEMID           1U
#define CFG_HW_FLASH_SEMID         2U
#define CFG_HW_RCC_SEMID           3U
#define CFG_HW_ENTRY_STOP_MODE_SEMID  4U
#define CFG_HW_CLK48_CONFIG_SEMID  5U
#define CFG_HW_GPIO_SEMID          8U
#define CFG_HW_EXTI_SEMID          9U
#define CFG_HW_IPM_CPU1_SEMID      10U
#define CFG_HW_IPM_CPU2_SEMID      11U

#define HSEM_CR_COREID_SHIFT       8
#define HSEM_CR_COREID_MASK        (0xF << HSEM_CR_COREID_SHIFT)
#define HSEM_CR_COREID_CPU1        1
#define HSEM_CR_COREID_CPU2        2

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

int stm32n6_hsem_initialize(void);
void stm32n6_hsem_lock(uint32_t hsem, uint32_t retry);
int stm32n6_hsem_try_lock(uint32_t hsem);
void stm32n6_hsem_unlock(uint32_t hsem);
bool stm32n6_hsem_is_owned(uint32_t hsem);

#endif /* __ARCH_ARM_SRC_STM32N6_STM32N6_HSEM_H */