/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_gpio.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_STM32N6_GPIO_H
#define __ARCH_ARM_SRC_STM32N6_STM32N6_GPIO_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <stdint.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32N6_NGPIO          12

#define GPIO_MODE_INPUT        0
#define GPIO_MODE_OUTPUT       1
#define GPIO_MODE_ALTERNATE    2
#define GPIO_MODE_ANALOG       3

#define GPIO_OTYPE_PP          0
#define GPIO_OTYPE_OD          1

#define GPIO_OSPEED_LOW        0
#define GPIO_OSPEED_MEDIUM     1
#define GPIO_OSPEED_HIGH       2
#define GPIO_OSPEED_VERYHIGH   3

#define GPIO_PULL_NONE         0
#define GPIO_PULL_UP           1
#define GPIO_PULL_DOWN         2

#define GPIO_PIN_SET           1
#define GPIO_PIN_RESET         0

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

void stm32n6_gpioinit(void);
int stm32n6_configgpio(uint32_t cfgset);
void stm32n6_gpiowrite(uint32_t pinset, bool value);
bool stm32n6_gpioread(uint32_t pinset);

#endif /* __ARCH_ARM_SRC_STM32N6_STM32N6_GPIO_H */
