/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_serial.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_STM32N6_SERIAL_H
#define __ARCH_ARM_SRC_STM32N6_STM32N6_SERIAL_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <stdint.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define USART_CR1_UE            (1 << 0)
#define USART_CR1_UESM          (1 << 1)
#define USART_CR1_RE            (1 << 2)
#define USART_CR1_TE            (1 << 3)
#define USART_CR1_RXNEIE        (1 << 5)
#define USART_CR1_TCIE          (1 << 6)
#define USART_CR1_TXEIE         (1 << 7)
#define USART_CR1_PEIE          (1 << 8)
#define USART_CR1_PS            (1 << 9)
#define USART_CR1_PCE           (1 << 10)
#define USART_CR1_M             (1 << 12)
#define USART_CR1_MME           (1 << 13)
#define USART_CR1_OVER8         (1 << 15)
#define USART_CR1_FIFOEN        (1 << 29)

#define USART_CR2_STOP_MASK     (3 << 12)
#define USART_CR2_STOP_1        (0 << 12)
#define USART_CR2_STOP_0_5      (1 << 12)
#define USART_CR2_STOP_2        (2 << 12)
#define USART_CR2_STOP_1_5      (3 << 12)

#define USART_CR3_EIE           (1 << 0)
#define USART_CR3_HDSEL         (1 << 3)
#define USART_CR3_DMAR          (1 << 6)
#define USART_CR3_DMAT          (1 << 7)
#define USART_CR3_ONEBIT        (1 << 11)
#define USART_CR3_OVRDIS        (1 << 12)

#define USART_ISR_PE            (1 << 0)
#define USART_ISR_FE            (1 << 1)
#define USART_ISR_NF            (1 << 2)
#define USART_ISR_ORE           (1 << 3)
#define USART_ISR_IDLE          (1 << 4)
#define USART_ISR_RXNE          (1 << 5)
#define USART_ISR_TC            (1 << 6)
#define USART_ISR_TXE           (1 << 7)
#define USART_ISR_CTSIF         (1 << 9)
#define USART_ISR_CTS           (1 << 10)
#define USART_ISR_BUSY          (1 << 16)

#define USART_ICR_PECF          (1 << 0)
#define USART_ICR_FECF          (1 << 1)
#define USART_ICR_NCF           (1 << 2)
#define USART_ICR_ORECF         (1 << 3)
#define USART_ICR_IDLECF        (1 << 4)
#define USART_ICR_TCCF          (1 << 6)
#define USART_ICR_CTSCF         (1 << 9)

/****************************************************************************
 * Public Types
 ****************************************************************************/

struct stm32n6_uart_s
{
  uintptr_t uartbase;
  uint32_t baudrate;
  uint32_t parity;
  uint32_t stopbits;
  uint32_t flowcontrol;
  uint32_t irq;
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

void stm32n6_uart_enable(uintptr_t uartbase);
void stm32n6_uart_disable(uintptr_t uartbase);
int stm32n6_uart_configure(uintptr_t uartbase, uint32_t baudrate,
                           uint32_t parity, uint32_t stopbits);
void stm32n6_uart_send(uintptr_t uartbase, uint8_t ch);
uint8_t stm32n6_uart_receive(uintptr_t uartbase);
bool stm32n6_uart_txready(uintptr_t uartbase);
bool stm32n6_uart_rxavailable(uintptr_t uartbase);

#endif /* __ARCH_ARM_SRC_STM32N6_STM32N6_SERIAL_H */
