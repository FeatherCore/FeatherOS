/****************************************************************************
 * arch/arm/src/stm32n6/hardware/stm32n6_exti.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_EXTI_H
#define __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_EXTI_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "stm32n6_memorymap.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_EXTI_MAX_LINES        96

#define STM32_EXTI_IMR1_OFFSET       0x00
#define STM32_EXTI_EMR1_OFFSET       0x20
#define STM32_EXTI_RTSR1_OFFSET      0x40
#define STM32_EXTI_FTSR1_OFFSET      0x60
#define STM32_EXTI_SWIER1_OFFSET     0x80
#define STM32_EXTI_PR1_OFFSET        0xA0
#define STM32_EXTI_IMR2_OFFSET       0x04
#define STM32_EXTI_EMR2_OFFSET       0x24
#define STM32_EXTI_RTSR2_OFFSET      0x44
#define STM32_EXTI_FTSR2_OFFSET      0x64
#define STM32_EXTI_SWIER2_OFFSET     0x84
#define STM32_EXTI_PR2_OFFSET        0xA4
#define STM32_EXTI_IMR3_OFFSET       0x08
#define STM32_EXTI_EMR3_OFFSET       0x28
#define STM32_EXTI_RTSR3_OFFSET      0x48
#define STM32_EXTI_FTSR3_OFFSET      0x68
#define STM32_EXTI_SWIER3_OFFSET     0x88
#define STM32_EXTI_PR3_OFFSET        0xA8

#define EXTI_IMR_IM(n)               (1 << (n))
#define EXTI_EMR_EM(n)               (1 << (n))
#define EXTI_RTSR_RT(n)              (1 << (n))
#define EXTI_FTSR_FT(n)              (1 << (n))
#define EXTI_SWIER_SWI(n)            (1 << (n))
#define EXTI_PR_PIF(n)               (1 << (n))

#define EXTI_RISING_EDGE             0x01
#define EXTI_FALLING_EDGE            0x02
#define EXTI_BOTH_EDGES              0x03

#define EXTI_MODE_INTERRUPT          0x01
#define EXTI_MODE_EVENT              0x02

#endif /* __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_EXTI_H */