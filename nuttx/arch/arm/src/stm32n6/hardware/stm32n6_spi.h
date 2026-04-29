/****************************************************************************
 * arch/arm/src/stm32n6/hardware/stm32n6_spi.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_SPI_H
#define __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_SPI_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "stm32n6_memorymap.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_SPI1_BASE            (STM32N6_PERIPH_BASE + 0x02003000)
#define STM32_SPI2_BASE            (STM32N6_PERIPH_BASE + 0x00003800)
#define STM32_SPI3_BASE            (STM32N6_PERIPH_BASE + 0x00003C00)
#define STM32_SPI4_BASE            (STM32N6_PERIPH_BASE + 0x02003400)
#define STM32_SPI5_BASE            (STM32N6_PERIPH_BASE + 0x02005000)
#define STM32_SPI6_BASE            (STM32N6_PERIPH_BASE + 0x06001400)

#define STM32_SPI_CR1_OFFSET        0x00
#define STM32_SPI_CR2_OFFSET        0x04
#define STM32_SPI_CFG1_OFFSET       0x00
#define STM32_SPI_CFG2_OFFSET       0x04
#define STM32_SPI_IER_OFFSET        0x08
#define STM32_SPI_SR_OFFSET         0x0C
#define STM32_SPI_IFCR_OFFSET       0x10
#define STM32_SPI_TXDR_OFFSET       0x14
#define STM32_SPI_RXDR_OFFSET       0x18
#define STM32_SPI_CRCPR_OFFSET      0x10
#define STM32_SPI_RXCRCR_OFFSET     0x14
#define STM32_SPI_TXCRCR_OFFSET     0x18
#define STM32_SPI_I2SCFGR_OFFSET    0x28

#define SPI_CR1_SPE                 (1 << 0)
#define SPI_CR1_MSTR                (1 << 2)
#define SPI_CR1_BR_SHIFT            3
#define SPI_CR1_BR_MASK             (7 << SPI_CR1_BR_SHIFT)
#define SPI_CR1_CPOL                (1 << 1)
#define SPI_CR1_CPHA                (1 << 0)
#define SPI_CR1_LSBFIRST            (1 << 7)
#define SPI_CR1_SSI                 (1 << 8)
#define SPI_CR1_SSM                 (1 << 9)
#define SPI_CR1_RXONLY              (1 << 10)
#define SPI_CR1_CRCL                (1 << 11)
#define SPI_CR1_CRCEN               (1 << 13)
#define SPI_CR1_BIDIOE              (1 << 14)
#define SPI_CR1_BIDIMODE            (1 << 15)

#define SPI_CR2_RXDMAEN             (1 << 0)
#define SPI_CR2_TXDMAEN             (1 << 1)
#define SPI_CR2_SSOE                (1 << 2)
#define SPI_CR2_NSSP                (1 << 3)
#define SPI_CR2_FRXTH               (1 << 12)
#define SPI_CR2_DS_SHIFT            8
#define SPI_CR2_DS_MASK             (0xf << SPI_CR2_DS_SHIFT)

#define SPI_SR_RXNE                 (1 << 0)
#define SPI_SR_TXE                  (1 << 1)
#define SPI_SR_CRCERR               (1 << 4)
#define SPI_SR_MODF                 (1 << 5)
#define SPI_SR_OVR                  (1 << 6)
#define SPI_SR_BSY                  (1 << 7)
#define SPI_SR_FRE                  (1 << 8)

#define SPI_CFG1_MBR_SHIFT          28
#define SPI_CFG1_MBR_MASK           (7 << SPI_CFG1_MBR_SHIFT)
#define SPI_CFG1_DSIZE_SHIFT        0
#define SPI_CFG1_DSIZE_MASK         (0x1f << SPI_CFG1_DSIZE_SHIFT)

#define SPI_CFG2_MASTER             (1 << 22)
#define SPI_CFG2_LSBFRST            (1 << 23)
#define SPI_CFG2_CPHA               (1 << 24)
#define SPI_CFG2_CPOL               (1 << 25)
#define SPI_CFG2_SSM                (1 << 26)
#define SPI_CFG2_SSOE               (1 << 27)
#define SPI_CFG2_AFCNTR             (1 << 31)

#endif /* __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_SPI_H */