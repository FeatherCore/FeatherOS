/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_spi.c
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

#include <sys/types.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>
#include <assert.h>
#include <debug.h>
#include <errno.h>

#include <nuttx/irq.h>
#include <nuttx/arch.h>
#include <nuttx/spi/spi.h>

#include "arm_internal.h"
#include "chip.h"
#include "hardware/stm32n6_memorymap.h"
#include "hardware/stm32n6_rcc.h"
#include "stm32n6_spi.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_SPI_CR1_OFFSET        0x00
#define STM32_SPI_CR2_OFFSET        0x04
#define STM32_SPI_SR_OFFSET         0x08
#define STM32_SPI_DR_OFFSET         0x0C
#define STM32_SPI_CRCPR_OFFSET      0x10
#define STM32_SPI_RXCRCR_OFFSET     0x14
#define STM32_SPI_TXCRCR_OFFSET     0x18

#define STM32_SPI_CFG1_OFFSET       0x00
#define STM32_SPI_CFG2_OFFSET       0x04
#define STM32_SPI_IER_OFFSET        0x08
#define STM32_SPI_SR_OFFSET         0x0C
#define STM32_SPI_IFCR_OFFSET       0x10
#define STM32_SPI_TXDR_OFFSET       0x14
#define STM32_SPI_RXDR_OFFSET       0x18
#define STM32_SPI_CR1_OFFSET        0x20
#define STM32_SPI_CR2_OFFSET        0x24
#define STM32_SPI_I2SCFGR_OFFSET    0x28

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static inline void stm32n6_spi_putreg(uintptr_t spibase,
                                      uint32_t offset, uint32_t value)
{
  putreg32(value, spibase + offset);
}

static inline uint32_t stm32n6_spi_getreg(uintptr_t spibase,
                                          uint32_t offset)
{
  return getreg32(spibase + offset);
}

static void stm32n6_spi_enable_clock(uintptr_t spibase)
{
  uint32_t regval;

  regval = getreg32(STM32_RCC_APB2ENR);
  if (spibase == STM32_SPI1_BASE)
    {
      regval |= RCC_APB2ENR_SPI1EN;
    }
  else if (spibase == STM32_SPI4_BASE)
    {
      regval |= RCC_APB2ENR_SPI4EN;
    }
  putreg32(regval, STM32_RCC_APB2ENR);

  regval = getreg32(STM32_RCC_APB1ENR);
  if (spibase == STM32_SPI2_BASE)
    {
      regval |= (1 << 14);
    }
  else if (spibase == STM32_SPI3_BASE)
    {
      regval |= (1 << 15);
    }
  putreg32(regval, STM32_RCC_APB1ENR);
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

int stm32n6_spi_initialize(uintptr_t spibase, uint32_t frequency,
                           uint8_t mode, uint8_t nbits)
{
  uint32_t regval;
  uint32_t prescaler;
  uint32_t clock;

  stm32n6_spi_enable_clock(spibase);

  stm32n6_spi_disable(spibase);

  clock = STM32N6_PCLK1_FREQUENCY;
  prescaler = clock / frequency;
  if (prescaler > 256)
    {
      prescaler = 256;
    }
  else if (prescaler < 2)
    {
      prescaler = 2;
    }

  regval = 0;
  regval |= SPI_CR1_MSTR;

  if (mode & SPI_MODE_CPOL)
    {
      regval |= SPI_CR1_CPOL;
    }
  if (mode & SPI_MODE_CPHA)
    {
      regval |= SPI_CR1_CPHA;
    }

  stm32n6_spi_putreg(spibase, STM32_SPI_CR1_OFFSET, regval);

  regval = 0;
  regval |= SPI_CR2_SSOE | SPI_CR2_FRXTH;
  stm32n6_spi_putreg(spibase, STM32_SPI_CR2_OFFSET, regval);

  return OK;
}

void stm32n6_spi_enable(uintptr_t spibase)
{
  uint32_t regval;

  regval = stm32n6_spi_getreg(spibase, STM32_SPI_CR1_OFFSET);
  regval |= SPI_CR1_SPE;
  stm32n6_spi_putreg(spibase, STM32_SPI_CR1_OFFSET, regval);
}

void stm32n6_spi_disable(uintptr_t spibase)
{
  uint32_t regval;

  regval = stm32n6_spi_getreg(spibase, STM32_SPI_CR1_OFFSET);
  regval &= ~SPI_CR1_SPE;
  stm32n6_spi_putreg(spibase, STM32_SPI_CR1_OFFSET, regval);
}

void stm32n6_spi_send(uintptr_t spibase, uint8_t ch)
{
  while (!stm32n6_spi_txready(spibase));

  stm32n6_spi_putreg(spibase, STM32_SPI_DR_OFFSET, (uint32_t)ch);
}

uint8_t stm32n6_spi_receive(uintptr_t spibase)
{
  while (!stm32n6_spi_rxavailable(spibase));

  return (uint8_t)stm32n6_spi_getreg(spibase, STM32_SPI_DR_OFFSET);
}

void stm32n6_spi_exchange(uintptr_t spibase, uint8_t *txdata,
                          uint8_t *rxdata, size_t nwords)
{
  size_t i;

  for (i = 0; i < nwords; i++)
    {
      if (txdata)
        {
          stm32n6_spi_send(spibase, txdata[i]);
        }
      else
        {
          stm32n6_spi_send(spibase, 0xff);
        }

      if (rxdata)
        {
          rxdata[i] = stm32n6_spi_receive(spibase);
        }
      else
        {
          stm32n6_spi_receive(spibase);
        }
    }
}

bool stm32n6_spi_txready(uintptr_t spibase)
{
  uint32_t regval;

  regval = stm32n6_spi_getreg(spibase, STM32_SPI_SR_OFFSET);
  return (regval & SPI_SR_TXE) != 0;
}

bool stm32n6_spi_rxavailable(uintptr_t spibase)
{
  uint32_t regval;

  regval = stm32n6_spi_getreg(spibase, STM32_SPI_SR_OFFSET);
  return (regval & SPI_SR_RXNE) != 0;
}
