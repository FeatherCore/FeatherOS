/****************************************************************************
 * arch/arm/src/stm32n6/chip.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_CHIP_H
#define __ARCH_ARM_SRC_STM32N6_CHIP_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

/* Include the memory map and IRQ definitions */

#include "hardware/stm32n6_memorymap.h"
#include "hardware/stm32n6_irq.h"

/* Include RCC and PWR definitions */

#include "hardware/stm32n6_rcc.h"
#include "hardware/stm32n6_pwr.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* STM32N6 Identification */

#define STM32N6_CHIPID_DEV_ID_STM32N657XX  0x485

/* CPU frequency */

#ifdef CONFIG_STM32N6_SYSCLK_FREQUENCY
#  define STM32N6_SYSCLK_FREQUENCY  CONFIG_STM32N6_SYSCLK_FREQUENCY
#else
#  define STM32N6_SYSCLK_FREQUENCY  64000000  /* Default HSI 64 MHz */
#endif

/* HCLK (AHB) frequency */

#ifdef CONFIG_STM32N6_HCLK_FREQUENCY
#  define STM32N6_HCLK_FREQUENCY  CONFIG_STM32N6_HCLK_FREQUENCY
#else
#  define STM32N6_HCLK_FREQUENCY  (STM32N6_SYSCLK_FREQUENCY / CONFIG_STM32N6_AHB_PRESCALER)
#endif

/* APB frequencies */

#define STM32N6_PCLK1_FREQUENCY  (STM32N6_HCLK_FREQUENCY / CONFIG_STM32N6_APB1_PRESCALER)
#define STM32N6_PCLK2_FREQUENCY  (STM32N6_HCLK_FREQUENCY / CONFIG_STM32N6_APB2_PRESCALER)
#define STM32N6_PCLK4_FREQUENCY  (STM32N6_HCLK_FREQUENCY / CONFIG_STM32N6_APB4_PRESCALER)
#define STM32N6_PCLK5_FREQUENCY  (STM32N6_HCLK_FREQUENCY / CONFIG_STM32N6_APB5_PRESCALER)

/* Clock source frequencies */

#define STM32N6_HSI_FREQUENCY    CONFIG_STM32N6_HSI_FREQUENCY
#define STM32N6_HSE_FREQUENCY    CONFIG_STM32N6_HSE_FREQUENCY
#define STM32N6_LSI_FREQUENCY    CONFIG_STM32N6_LSI_FREQUENCY
#define STM32N6_LSE_FREQUENCY    CONFIG_STM32N6_LSE_FREQUENCY

/* Timer clock frequencies
 * If APB prescaler is 1, timer clock = PCLKx
 * Otherwise, timer clock = PCLKx * 2
 */

#define STM32N6_APB1_TIM_FREQUENCY  (STM32N6_PCLK1_FREQUENCY * (CONFIG_STM32N6_APB1_PRESCALER == 1 ? 1 : 2))
#define STM32N6_APB2_TIM_FREQUENCY  (STM32N6_PCLK2_FREQUENCY * (CONFIG_STM32N6_APB2_PRESCALER == 1 ? 1 : 2))

/* Cache line sizes */

#define ARMV8M_DCACHE_LINE_SIZE  CONFIG_ARMV8M_DCACHE_LINE_SIZE
#define ARMV8M_ICACHE_LINE_SIZE  CONFIG_ARMV8M_ICACHE_LINE_SIZE

/* Voltage scaling */

#define STM32N6_VOS_SCALE0   0  /* Highest performance */
#define STM32N6_VOS_SCALE1   1
#define STM32N6_VOS_SCALE2   2  /* Lowest power */

/****************************************************************************
 * Public Types
 ****************************************************************************/

/****************************************************************************
 * Inline Functions
 ****************************************************************************/

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

#endif /* __ARCH_ARM_SRC_STM32N6_CHIP_H */
