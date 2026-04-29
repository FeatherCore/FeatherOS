/****************************************************************************
 * arch/arm/src/stm32n6/hardware/stm32n6_tim.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_TIM_H
#define __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_TIM_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "stm32n6_memorymap.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_TIM1_BASE            (STM32N6_PERIPH_BASE + 0x02000000)
#define STM32_TIM2_BASE            (STM32N6_PERIPH_BASE + 0x00000000)
#define STM32_TIM3_BASE            (STM32N6_PERIPH_BASE + 0x00000400)
#define STM32_TIM4_BASE            (STM32N6_PERIPH_BASE + 0x00000800)
#define STM32_TIM5_BASE            (STM32N6_PERIPH_BASE + 0x00000C00)
#define STM32_TIM6_BASE            (STM32N6_PERIPH_BASE + 0x00001000)
#define STM32_TIM7_BASE            (STM32N6_PERIPH_BASE + 0x00001400)
#define STM32_TIM8_BASE            (STM32N6_PERIPH_BASE + 0x02000400)
#define STM32_TIM9_BASE            (STM32N6_PERIPH_BASE + 0x02004C00)
#define STM32_TIM10_BASE           (STM32N6_PERIPH_BASE + 0x00003000)
#define STM32_TIM11_BASE           (STM32N6_PERIPH_BASE + 0x00003400)
#define STM32_TIM12_BASE           (STM32N6_PERIPH_BASE + 0x00001800)
#define STM32_TIM13_BASE           (STM32N6_PERIPH_BASE + 0x00001C00)
#define STM32_TIM14_BASE           (STM32N6_PERIPH_BASE + 0x00002000)
#define STM32_TIM15_BASE           (STM32N6_PERIPH_BASE + 0x02004000)
#define STM32_TIM16_BASE           (STM32N6_PERIPH_BASE + 0x02004400)
#define STM32_TIM17_BASE           (STM32N6_PERIPH_BASE + 0x02004800)
#define STM32_TIM18_BASE           (STM32N6_PERIPH_BASE + 0x02003C00)

#define STM32_TIM_CR1_OFFSET        0x00
#define STM32_TIM_CR2_OFFSET        0x04
#define STM32_TIM_SMCR_OFFSET       0x08
#define STM32_TIM_DIER_OFFSET       0x0C
#define STM32_TIM_SR_OFFSET         0x10
#define STM32_TIM_EGR_OFFSET        0x14
#define STM32_TIM_CCMR1_OFFSET      0x18
#define STM32_TIM_CCMR2_OFFSET      0x1C
#define STM32_TIM_CCER_OFFSET       0x20
#define STM32_TIM_CNT_OFFSET        0x24
#define STM32_TIM_PSC_OFFSET        0x28
#define STM32_TIM_ARR_OFFSET        0x2C
#define STM32_TIM_RCR_OFFSET        0x30
#define STM32_TIM_CCR1_OFFSET       0x34
#define STM32_TIM_CCR2_OFFSET       0x38
#define STM32_TIM_CCR3_OFFSET       0x3C
#define STM32_TIM_CCR4_OFFSET       0x40
#define STM32_TIM_BDTR_OFFSET       0x44
#define STM32_TIM_DCR_OFFSET        0x48
#define STM32_TIM_DMAR_OFFSET       0x4C
#define STM32_TIM_OR1_OFFSET        0x50
#define STM32_TIM_CCMR3_OFFSET      0x54
#define STM32_TIM_CCR5_OFFSET       0x58
#define STM32_TIM_CCR6_OFFSET       0x5C
#define STM32_TIM_OR2_OFFSET        0x60
#define STM32_TIM_OR3_OFFSET        0x70
#define STM32_TIM_OR4_OFFSET        0x80

#define TIM_CR1_CEN                 (1 << 0)
#define TIM_CR1_UDIS                (1 << 1)
#define TIM_CR1_URS                 (1 << 2)
#define TIM_CR1_OPM                 (1 << 3)
#define TIM_CR1_DIR                 (1 << 4)
#define TIM_CR1_ARPE                (1 << 7)

#define TIM_DIER_UIE                (1 << 0)
#define TIM_DIER_CC1IE              (1 << 1)
#define TIM_DIER_CC2IE              (1 << 2)
#define TIM_DIER_CC3IE              (1 << 3)
#define TIM_DIER_CC4IE              (1 << 4)
#define TIM_DIER_TIE                (1 << 6)
#define TIM_DIER_UDE                (1 << 8)

#define TIM_SR_UIF                  (1 << 0)
#define TIM_SR_CC1IF                (1 << 1)
#define TIM_SR_CC2IF                (1 << 2)
#define TIM_SR_CC3IF                (1 << 3)
#define TIM_SR_CC4IF                (1 << 4)
#define TIM_SR_TIF                  (1 << 6)

#define TIM_EGR_UG                  (1 << 0)
#define TIM_EGR_CC1G                (1 << 1)
#define TIM_EGR_CC2G                (1 << 2)
#define TIM_EGR_CC3G                (1 << 3)
#define TIM_EGR_CC4G                (1 << 4)
#define TIM_EGR_TG                  (1 << 6)

#endif /* __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_TIM_H */