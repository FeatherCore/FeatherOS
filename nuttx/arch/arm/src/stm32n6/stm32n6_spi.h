/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_spi.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_STM32N6_SPI_H
#define __ARCH_ARM_SRC_STM32N6_STM32N6_SPI_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <stdint.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define SPI_CR1_SPE             (1 << 0)
#define SPI_CR1_MSTR            (1 << 2)
#define SPI_CR1_BR_SHIFT        3
#define SPI_CR1_BR_MASK         (7 << SPI_CR1_BR_SHIFT)
#define SPI_CR1_CPOL            (1 << 1)
#define SPI_CR1_CPHA            (1 << 0)
#define SPI_CR1_LSBFIRST        (1 << 7)
#define SPI_CR1_SSI             (1 << 8)
#define SPI_CR1_SSM             (1 << 9)
#define SPI_CR1_RXONLY          (1 << 10)
#define SPI_CR1_CRCL            (1 << 11)
#define SPI_CR1_CRCNEXT         (1 << 12)
#define SPI_CR1_CRCEN           (1 << 13)
#define SPI_CR1_BIDIOE          (1 << 14)
#define SPI_CR1_BIDIMODE        (1 << 15)

#define SPI_CR2_RXDMAEN         (1 << 0)
#define SPI_CR2_TXDMAEN         (1 << 1)
#define SPI_CR2_SSOE            (1 << 2)
#define SPI_CR2_NSSP            (1 << 3)
#define SPI_CR2_FRF             (1 << 4)
#define SPI_CR2_ERRIE           (1 << 5)
#define SPI_CR2_RXNEIE          (1 << 6)
#define SPI_CR2_TXEIE           (1 << 7)
#define SPI_CR2_DS_SHIFT        8
#define SPI_CR2_DS_MASK         (0xf << SPI_CR2_DS_SHIFT)
#define SPI_CR2_FRXTH           (1 << 12)
#define SPI_CR2_LDMARX          (1 << 13)
#define SPI_CR2_LDMATX          (1 << 14)

#define SPI_SR_RXNE             (1 << 0)
#define SPI_SR_TXE              (1 << 1)
#define SPI_SR_CHSIDE           (1 << 2)
#define SPI_SR_UDR              (1 << 3)
#define SPI_SR_CRCERR           (1 << 4)
#define SPI_SR_MODF             (1 << 5)
#define SPI_SR_OVR              (1 << 6)
#define SPI_SR_BSY              (1 << 7)
#define SPI_SR_FRE              (1 << 8)
#define SPI_SR_FTLVL_SHIFT      11
#define SPI_SR_FTLVL_MASK       (3 << SPI_SR_FTLVL_SHIFT)
#define SPI_SR_FRLVL_SHIFT      13
#define SPI_SR_FRLVL_MASK       (3 << SPI_SR_FRLVL_SHIFT)

#define SPI_CFG1_MBR_SHIFT      28
#define SPI_CFG1_MBR_MASK       (7 << SPI_CFG1_MBR_SHIFT)
#define SPI_CFG1_FTHLV_SHIFT    24
#define SPI_CFG1_FTHLV_MASK     (0xf << SPI_CFG1_FTHLV_SHIFT)
#define SPI_CFG1_DSIZE_SHIFT    0
#define SPI_CFG1_DSIZE_MASK     (0x1f << SPI_CFG1_DSIZE_SHIFT)

#define SPI_CFG2_MIDI_SHIFT     4
#define SPI_CFG2_MIDI_MASK      (0xf << SPI_CFG2_MIDI_SHIFT)
#define SPI_CFG2_MSSI_SHIFT     0
#define SPI_CFG2_MSSI_MASK      (0xf << SPI_CFG2_MSSI_SHIFT)
#define SPI_CFG2_IOSWP          (1 << 15)
#define SPI_CFG2_COMM_SHIFT     17
#define SPI_CFG2_COMM_MASK      (3 << SPI_CFG2_COMM_SHIFT)
#define SPI_CFG2_SP             (1 << 26)
#define SPI_CFG2_MASTER         (1 << 22)
#define SPI_CFG2_LSBFRST        (1 << 23)
#define SPI_CFG2_CPHA           (1 << 24)
#define SPI_CFG2_CPOL           (1 << 25)
#define SPI_CFG2_SSM            (1 << 26)
#define SPI_CFG2_SSOE           (1 << 27)
#define SPI_CFG2_SSOM           (1 << 28)
#define SPI_CFG2_AFCNTR         (1 << 31)

#define SPI_IER_TXPIE           (1 << 0)
#define SPI_IER_RXPIE           (1 << 1)
#define SPI_IER_DXP             (1 << 2)
#define SPI_IER_TXEIE           (1 << 3)
#define SPI_IER_RXNEIE          (1 << 4)
#define SPI_IER_ERRIE           (1 << 5)
#define SPI_IER_MODFIE          (1 << 9)
#define SPI_IER_OVRIE           (1 << 10)
#define SPI_IER_TIFREIE         (1 << 11)
#define SPI_IER_CRCEIE          (1 << 12)

#define SPI_SR_TXP              (1 << 0)
#define SPI_SR_RXP              (1 << 1)
#define SPI_SR_DXP              (1 << 2)
#define SPI_SR_TXE              (1 << 3)
#define SPI_SR_RXNE             (1 << 4)
#define SPI_SR_CRCERR           (1 << 5)
#define SPI_SR_MODF             (1 << 6)
#define SPI_SR_OVR              (1 << 7)
#define SPI_SR_BUSY             (1 << 8)
#define SPI_SR_TIFRE            (1 << 9)
#define SPI_SR_SUSP             (1 << 11)
#define SPI_SR_TXC              (1 << 12)
#define SPI_SR_RXPLVL_SHIFT     13
#define SPI_SR_RXPLVL_MASK      (3 << SPI_SR_RXPLVL_SHIFT)
#define SPI_SR_RXWNE            (1 << 15)

/****************************************************************************
 * Public Types
 ****************************************************************************/

struct stm32n6_spidev_s
{
  uintptr_t spibase;
  uint32_t frequency;
  uint8_t mode;
  uint8_t nbits;
  int irq;
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

int stm32n6_spi_initialize(uintptr_t spibase, uint32_t frequency,
                           uint8_t mode, uint8_t nbits);
void stm32n6_spi_enable(uintptr_t spibase);
void stm32n6_spi_disable(uintptr_t spibase);
void stm32n6_spi_send(uintptr_t spibase, uint8_t ch);
uint8_t stm32n6_spi_receive(uintptr_t spibase);
void stm32n6_spi_exchange(uintptr_t spibase, uint8_t *txdata,
                          uint8_t *rxdata, size_t nwords);
bool stm32n6_spi_txready(uintptr_t spibase);
bool stm32n6_spi_rxavailable(uintptr_t spibase);

#endif /* __ARCH_ARM_SRC_STM32N6_STM32N6_SPI_H */
