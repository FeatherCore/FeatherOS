/****************************************************************************
 * arch/arm/include/stm32n6/chip.h
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

#ifndef __ARCH_ARM_INCLUDE_STM32N6_CHIP_H
#define __ARCH_ARM_INCLUDE_STM32N6_CHIP_H

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#if !defined(CONFIG_ARCH_CHIP_STM32N657XX)
#  error STM32N6 chip not identified
#endif

#define STM32N6_NGPIO              17
#define STM32N6_NUSART             6
#define STM32N6_NI2C               4
#define STM32N6_NSPI               6
#define STM32N6_NFDCAN             3
#define STM32N6_NXSPI              3

#define ARMV8M_PERIPHERAL_INTERRUPTS 180

/* 16 programmable interrupt priority levels. */

#define NVIC_SYSH_PRIORITY_MIN     0xf0
#define NVIC_SYSH_PRIORITY_DEFAULT 0x80
#define NVIC_SYSH_PRIORITY_MAX     0x00
#define NVIC_SYSH_PRIORITY_STEP    0x10

#endif /* __ARCH_ARM_INCLUDE_STM32N6_CHIP_H */
