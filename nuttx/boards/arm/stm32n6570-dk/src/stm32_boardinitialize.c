/****************************************************************************
 * boards/arm/stm32n6570-dk/src/stm32n6570-dk.h
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

#include <nuttx/config.h>

#include <stdint.h>
#include <stdbool.h>

#include "arm_internal.h"
#include "chip.h"
#include "stm32_gpio.h"

void stm32_boardinitialize(void)
{
#if defined(CONFIG_STM32N6_GPIO)
  stm32n6_gpioinit();
#endif

#if defined(CONFIG_STM32N6_UART1_SERIALDRIVER)
  stm32n6_uart_configure(STM32_USART1_BASE, STM32_USART1_BAUD);
#endif

#if defined(CONFIG_STM32N6_UART2_SERIALDRIVER)
  stm32n6_uart_configure(STM32_USART2_BASE, STM32_USART2_BAUD);
#endif
}