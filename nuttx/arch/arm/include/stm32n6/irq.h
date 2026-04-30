/****************************************************************************
 * arch/arm/include/stm32n6/irq.h
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

/* This file should never be included directly but, rather, only indirectly
 * through nuttx/irq.h.
 */

#ifndef __ARCH_ARM_INCLUDE_STM32N6_IRQ_H
#define __ARCH_ARM_INCLUDE_STM32N6_IRQ_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_IRQ_RESERVED       (0)
#define STM32_IRQ_NMI            (2)
#define STM32_IRQ_HARDFAULT      (3)
#define STM32_IRQ_MEMFAULT       (4)
#define STM32_IRQ_BUSFAULT       (5)
#define STM32_IRQ_USAGEFAULT     (6)
#define STM32_IRQ_SVCALL         (11)
#define STM32_IRQ_DBGMONITOR     (12)
#define STM32_IRQ_PENDSV         (14)
#define STM32_IRQ_SYSTICK        (15)

#define STM32_IRQ_FIRST          (16)

#define STM32_IRQ_WWDG           (STM32_IRQ_FIRST + 0)
#define STM32_IRQ_PVD            (STM32_IRQ_FIRST + 1)
#define STM32_IRQ_TAMP           (STM32_IRQ_FIRST + 2)
#define STM32_IRQ_RTC            (STM32_IRQ_FIRST + 3)
#define STM32_IRQ_EXTI0          (STM32_IRQ_FIRST + 4)
#define STM32_IRQ_EXTI1          (STM32_IRQ_FIRST + 5)
#define STM32_IRQ_EXTI2          (STM32_IRQ_FIRST + 6)
#define STM32_IRQ_EXTI3          (STM32_IRQ_FIRST + 7)
#define STM32_IRQ_EXTI4          (STM32_IRQ_FIRST + 8)
#define STM32_IRQ_ADC1_2         (STM32_IRQ_FIRST + 30)
#define STM32_IRQ_JPEG           (STM32_IRQ_FIRST + 45)
#define STM32_IRQ_VENC           (STM32_IRQ_FIRST + 46)
#define STM32_IRQ_I2C1_EV        (STM32_IRQ_FIRST + 84)
#define STM32_IRQ_I2C1_ER        (STM32_IRQ_FIRST + 85)
#define STM32_IRQ_I2C2_EV        (STM32_IRQ_FIRST + 86)
#define STM32_IRQ_I2C2_ER        (STM32_IRQ_FIRST + 87)
#define STM32_IRQ_I2C4_EV        (STM32_IRQ_FIRST + 90)
#define STM32_IRQ_I2C4_ER        (STM32_IRQ_FIRST + 91)
#define STM32_IRQ_TIM1_BRK       (STM32_IRQ_FIRST + 96)
#define STM32_IRQ_TIM1_UP        (STM32_IRQ_FIRST + 97)
#define STM32_IRQ_TIM1_TRG_COM   (STM32_IRQ_FIRST + 98)
#define STM32_IRQ_TIM1_CC        (STM32_IRQ_FIRST + 99)
#define STM32_IRQ_SPI5           (STM32_IRQ_FIRST + 141)
#define STM32_IRQ_USART1         (STM32_IRQ_FIRST + 143)
#define STM32_IRQ_USART2         (STM32_IRQ_FIRST + 144)
#define STM32_IRQ_XSPI1          (STM32_IRQ_FIRST + 154)
#define STM32_IRQ_XSPI2          (STM32_IRQ_FIRST + 155)
#define STM32_IRQ_SDMMC2         (STM32_IRQ_FIRST + 159)
#define STM32_IRQ_USB_OTG_HS1    (STM32_IRQ_FIRST + 161)
#define STM32_IRQ_ETH            (STM32_IRQ_FIRST + 163)
#define STM32_IRQ_FDCAN1_IT0     (STM32_IRQ_FIRST + 164)
#define STM32_IRQ_FDCAN1_IT1     (STM32_IRQ_FIRST + 165)
#define STM32_IRQ_LTDC           (STM32_IRQ_FIRST + 177)
#define STM32_IRQ_LTDC_ER        (STM32_IRQ_FIRST + 178)
#define STM32_IRQ_NPU            (STM32_IRQ_FIRST + 179)

#define STM32_IRQ_NEXTINTS       180
#define NR_IRQS                  (STM32_IRQ_FIRST + STM32_IRQ_NEXTINTS)

#endif /* __ARCH_ARM_INCLUDE_STM32N6_IRQ_H */
