/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_lowsetup.c
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

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

#include <stdint.h>

#include "arm_internal.h"
#include "chip.h"
#include "hardware/stm32n6_memorymap.h"
#include "hardware/stm32n6_uart.h"
#include "stm32n6_start.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define USART_CR1_UE        (1 << 0)
#define USART_CR1_TE        (1 << 3)
#define USART_CR1_RE        (1 << 2)
#define USART_CR1_M0        (1 << 12)
#define USART_CR1_OVER8     (1 << 15)
#define USART_CR2_STOP_1    0
#define USART_BRR_DIV_MASK  0xffff

/****************************************************************************
 * Private Functions
 ****************************************************************************/

#if defined(CONFIG_STM32N6_UART1) || defined(CONFIG_STM32N6_UART2) || \
    defined(CONFIG_STM32N6_UART3) || defined(CONFIG_STM32N6_UART4) || \
    defined(CONFIG_STM32N6_UART5) || defined(CONFIG_STM32N6_UART6)

static inline void stm32n6_uart_configure(uintptr_t uart_base,
                                          uint32_t baudrate)
{
  uint32_t regval;
  uint32_t brr;

  regval = getreg32(uart_base + STM32_USART_CR1_OFFSET);
  regval &= ~USART_CR1_UE;
  putreg32(regval, uart_base + STM32_USART_CR1_OFFSET);

  regval = 0;
  regval |= USART_CR1_TE | USART_CR1_RE;
  putreg32(regval, uart_base + STM32_USART_CR1_OFFSET);

  regval = getreg32(uart_base + STM32_USART_CR2_OFFSET);
  regval &= ~(3 << 12);
  regval |= USART_CR2_STOP_1;
  putreg32(regval, uart_base + STM32_USART_CR2_OFFSET);

  brr = (STM32N6_PCLK1_FREQUENCY + (baudrate / 2)) / baudrate;
  putreg32(brr & USART_BRR_DIV_MASK, uart_base + STM32_USART_BRR_OFFSET);

  regval = getreg32(uart_base + STM32_USART_CR1_OFFSET);
  regval |= USART_CR1_UE;
  putreg32(regval, uart_base + STM32_USART_CR1_OFFSET);
}
#endif

/****************************************************************************
 * Public Functions
 ****************************************************************************/

void stm32n6_lowsetup(void)
{
#if defined(CONFIG_STM32N6_UART1) && defined(CONFIG_STM32N6_UART1_SERIALDRIVER)
  stm32n6_uart_configure(STM32_USART1_BASE, CONFIG_STM32N6_UART1_BAUD);
#endif

#if defined(CONFIG_STM32N6_UART2) && defined(CONFIG_STM32N6_UART2_SERIALDRIVER)
  stm32n6_uart_configure(STM32_USART2_BASE, CONFIG_STM32N6_UART2_BAUD);
#endif

#if defined(CONFIG_STM32N6_UART3) && defined(CONFIG_STM32N6_UART3_SERIALDRIVER)
  stm32n6_uart_configure(STM32_USART3_BASE, CONFIG_STM32N6_UART3_BAUD);
#endif

#if defined(CONFIG_STM32N6_UART4) && defined(CONFIG_STM32N6_UART4_SERIALDRIVER)
  stm32n6_uart_configure(STM32_UART4_BASE, CONFIG_STM32N6_UART4_BAUD);
#endif

#if defined(CONFIG_STM32N6_UART5) && defined(CONFIG_STM32N6_UART5_SERIALDRIVER)
  stm32n6_uart_configure(STM32_UART5_BASE, CONFIG_STM32N6_UART5_BAUD);
#endif

#if defined(CONFIG_STM32N6_UART6) && defined(CONFIG_STM32N6_UART6_SERIALDRIVER)
  stm32n6_uart_configure(STM32_USART6_BASE, CONFIG_STM32N6_UART6_BAUD);
#endif
}

void arm_lowputc(char ch)
{
#if defined(CONFIG_STM32N6_UART1)
  while ((getreg32(STM32_USART1_BASE + STM32_USART_ISR_OFFSET) &
          (1 << 7)) == 0);
  putreg32((uint32_t)ch, STM32_USART1_BASE + STM32_USART_TDR_OFFSET);
#elif defined(CONFIG_STM32N6_UART2)
  while ((getreg32(STM32_USART2_BASE + STM32_USART_ISR_OFFSET) &
          (1 << 7)) == 0);
  putreg32((uint32_t)ch, STM32_USART2_BASE + STM32_USART_TDR_OFFSET);
#endif
}
