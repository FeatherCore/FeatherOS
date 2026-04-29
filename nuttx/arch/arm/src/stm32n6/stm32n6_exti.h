/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_exti.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_STM32N6_EXTI_H
#define __ARCH_ARM_SRC_STM32N6_STM32N6_EXTI_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <stdint.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define EXTI_MAX_LINES           96

#define EXTI_RISING_EDGE         0x01
#define EXTI_FALLING_EDGE        0x02
#define EXTI_BOTH_EDGES          0x03

#define EXTI_MODE_INTERRUPT      0x01
#define EXTI_MODE_EVENT          0x02

/****************************************************************************
 * Public Types
 ****************************************************************************/

typedef void (*exti_handler_t)(uint32_t line, void *arg);

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

int stm32n6_exti_initialize(void);
int stm32n6_exti_configure(uint32_t line, uint32_t trigger, uint32_t mode);
int stm32n6_exti_enable(uint32_t line);
int stm32n6_exti_disable(uint32_t line);
int stm32n6_exti_register(uint32_t line, exti_handler_t handler, void *arg);
void stm32n6_exti_clear(uint32_t line);
bool stm32n6_exti_pending(uint32_t line);

#endif /* __ARCH_ARM_SRC_STM32N6_STM32N6_EXTI_H */