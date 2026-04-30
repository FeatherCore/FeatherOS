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

#include <stdbool.h>
#include <stdint.h>

/****************************************************************************
 * Public Types
 ****************************************************************************/

struct stm32n6_uart_s
{
  uintptr_t uartbase;
  uint32_t baud;
  uint8_t  bits;
  uint8_t  parity;
  uint8_t  stopbits2;
  uint16_t irq;
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

int stm32n6_uart_configure(uintptr_t uartbase, uint32_t baud,
                           uint8_t bits, uint8_t parity,
                           uint8_t stopbits2);
void stm32n6_uart_send(uintptr_t uartbase, int ch);
int stm32n6_uart_receive(uintptr_t uartbase);
bool stm32n6_uart_txready(uintptr_t uartbase);
bool stm32n6_uart_txempty(uintptr_t uartbase);
bool stm32n6_uart_rxavailable(uintptr_t uartbase);

#endif /* __ARCH_ARM_SRC_STM32N6_STM32N6_SERIAL_H */
