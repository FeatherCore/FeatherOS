/****************************************************************************
 * arch/arm/src/stm32n6/hardware/stm32n6_gpio.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_GPIO_H
#define __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_GPIO_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "stm32n6_memorymap.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_GPIO_MODER_OFFSET        0x00
#define STM32_GPIO_OTYPER_OFFSET       0x04
#define STM32_GPIO_OSPEEDR_OFFSET      0x08
#define STM32_GPIO_PUPDR_OFFSET        0x0C
#define STM32_GPIO_IDR_OFFSET          0x10
#define STM32_GPIO_ODR_OFFSET          0x14
#define STM32_GPIO_BSRR_OFFSET         0x18
#define STM32_GPIO_LCKR_OFFSET         0x1C
#define STM32_GPIO_AFRL_OFFSET         0x20
#define STM32_GPIO_AFRH_OFFSET         0x24

#define GPIO_MODER_INPUT               0
#define GPIO_MODER_OUTPUT              1
#define GPIO_MODER_ALTERNATE           2
#define GPIO_MODER_ANALOG              3

#define GPIO_OTYPER_PP                 0
#define GPIO_OTYPER_OD                 1

#define GPIO_OSPEEDR_LOW               0
#define GPIO_OSPEEDR_MEDIUM            1
#define GPIO_OSPEEDR_HIGH              2
#define GPIO_OSPEEDR_VERYHIGH          3

#define GPIO_PUPDR_NONE                0
#define GPIO_PUPDR_UP                  1
#define GPIO_PUPDR_DOWN                2

#define GPIO_BSRR_SET_SHIFT            0
#define GPIO_BSRR_RESET_SHIFT          16

#endif /* __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_GPIO_H */