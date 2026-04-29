/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_jpeg.c
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
#include <errno.h>

#include "arm_internal.h"
#include "chip.h"
#include "hardware/stm32n6_memorymap.h"
#include "hardware/stm32n6_rcc.h"
#include "stm32n6_jpeg.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define JPEG_TIMEOUT_US              5000000  /* 5 seconds timeout */

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static inline void stm32n6_jpeg_putreg(uintptr_t jpegbase,
                                        uint32_t offset, uint32_t value)
{
  putreg32(value, jpegbase + offset);
}

static inline uint32_t stm32n6_jpeg_getreg(uintptr_t jpegbase,
                                            uint32_t offset)
{
  return getreg32(jpegbase + offset);
}

static void stm32n6_jpeg_enable_clock(void)
{
  uint32_t regval;

  regval = getreg32(STM32_RCC_AHB5ENR);
  regval |= (1 << 3);  /* JPEGEN */
  putreg32(regval, STM32_RCC_AHB5ENR);
}

static int stm32n6_jpeg_wait_for_notbusy(uintptr_t jpegbase)
{
  uint32_t timeout = JPEG_TIMEOUT_US;

  while ((stm32n6_jpeg_getreg(jpegbase, JPEG_SR_OFFSET) & JPEG_SR_COFHF) != 0)
    {
      if (timeout-- == 0)
        {
          return -ETIMEDOUT;
        }
      up_udelay(1);
    }

  return OK;
}

static int stm32n6_jpeg_wait_for_eoc(uintptr_t jpegbase)
{
  uint32_t timeout = JPEG_TIMEOUT_US;

  while ((stm32n6_jpeg_getreg(jpegbase, JPEG_SR_OFFSET) & JPEG_SR_EOCF) == 0)
    {
      if (timeout-- == 0)
        {
          return -ETIMEDOUT;
        }
      up_udelay(1);
    }

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

int stm32n6_jpeg_initialize(uintptr_t jpegbase, uint8_t format, uint16_t width, uint16_t height)
{
  uint32_t regval;

  stm32n6_jpeg_enable_clock();

  /* Disable JPEG before configuration */
  stm32n6_jpeg_disable(jpegbase);

  /* Configure format and dimensions */
  regval = stm32n6_jpeg_getreg(jpegbase, JPEG_CONFR0_OFFSET);
  regval &= ~(0x3 << 16);  /* Clear format bits */
  regval |= (format & 0x3) << 16;
  stm32n6_jpeg_putreg(jpegbase, JPEG_CONFR0_OFFSET, regval);

  /* Configure width and height */
  stm32n6_jpeg_putreg(jpegbase, JPEG_CONFR1_OFFSET, width);
  stm32n6_jpeg_putreg(jpegbase, JPEG_CONFR2_OFFSET, height);

  /* Configure other parameters as needed */
  /* Quality factors, Huffman tables, etc. */

  return OK;
}

void stm32n6_jpeg_enable(uintptr_t jpegbase)
{
  uint32_t regval;

  regval = stm32n6_jpeg_getreg(jpegbase, JPEG_CR_OFFSET);
  regval |= JPEG_CR_JCEN;
  stm32n6_jpeg_putreg(jpegbase, JPEG_CR_OFFSET, regval);
}

void stm32n6_jpeg_disable(uintptr_t jpegbase)
{
  uint32_t regval;

  regval = stm32n6_jpeg_getreg(jpegbase, JPEG_CR_OFFSET);
  regval &= ~JPEG_CR_JCEN;
  stm32n6_jpeg_putreg(jpegbase, JPEG_CR_OFFSET, regval);
}

int stm32n6_jpeg_encode(uintptr_t jpegbase, const uint8_t *input, size_t input_size,
                        uint8_t *output, size_t *output_size)
{
  uint32_t regval;
  size_t input_idx = 0;
  size_t output_idx = 0;

  /* Wait for not busy */
  if (stm32n6_jpeg_wait_for_notbusy(jpegbase) != OK)
    {
      return -ETIMEDOUT;
    }

  /* Set encode mode */
  regval = stm32n6_jpeg_getreg(jpegbase, JPEG_CR_OFFSET);
  regval &= ~JPEG_CR_CMD_MASK;
  regval |= JPEG_CR_CMD_ENCODE;
  regval |= JPEG_CR_DMAINEN | JPEG_CR_DMAOUTEN;  /* Enable DMA */
  stm32n6_jpeg_putreg(jpegbase, JPEG_CR_OFFSET, regval);

  /* Process input data */
  while (input_idx < input_size)
    {
      /* Wait for input FIFO threshold */
      while ((stm32n6_jpeg_getreg(jpegbase, JPEG_SR_OFFSET) & JPEG_SR_IFTF) == 0);

      /* Write data to input register */
      stm32n6_jpeg_putreg(jpegbase, JPEG_DIR_OFFSET, input[input_idx++] |
                          (input[input_idx] << 8) |
                          (input[input_idx+1] << 16) |
                          (input[input_idx+2] << 24));
      input_idx += 4;
    }

  /* Start encoding */
  regval = stm32n6_jpeg_getreg(jpegbase, JPEG_CR_OFFSET);
  regval |= JPEG_CR_START;
  stm32n6_jpeg_putreg(jpegbase, JPEG_CR_OFFSET, regval);

  /* Wait for encoding to complete */
  if (stm32n6_jpeg_wait_for_eoc(jpegbase) != OK)
    {
      return -ETIMEDOUT;
    }

  /* Read output data */
  while (output_idx < *output_size)
    {
      /* Wait for output FIFO threshold */
      while ((stm32n6_jpeg_getreg(jpegbase, JPEG_SR_OFFSET) & JPEG_SR_OFTR) == 0);

      /* Read data from output register */
      uint32_t data = stm32n6_jpeg_getreg(jpegbase, JPEG_DOR_OFFSET);
      if (output_idx < *output_size) output[output_idx++] = data & 0xFF;
      if (output_idx < *output_size) output[output_idx++] = (data >> 8) & 0xFF;
      if (output_idx < *output_size) output[output_idx++] = (data >> 16) & 0xFF;
      if (output_idx < *output_size) output[output_idx++] = (data >> 24) & 0xFF;
    }

  /* Update output size */
  *output_size = output_idx;

  /* Clear end of conversion flag */
  stm32n6_jpeg_putreg(jpegbase, JPEG_CFR_OFFSET, JPEG_CFR_CEOCF);

  return OK;
}

int stm32n6_jpeg_decode(uintptr_t jpegbase, const uint8_t *input, size_t input_size,
                        uint8_t *output, size_t *output_size)
{
  uint32_t regval;
  size_t input_idx = 0;
  size_t output_idx = 0;

  /* Wait for not busy */
  if (stm32n6_jpeg_wait_for_notbusy(jpegbase) != OK)
    {
      return -ETIMEDOUT;
    }

  /* Set decode mode */
  regval = stm32n6_jpeg_getreg(jpegbase, JPEG_CR_OFFSET);
  regval &= ~JPEG_CR_CMD_MASK;
  regval |= JPEG_CR_CMD_DECODE;
  regval |= JPEG_CR_DMAINEN | JPEG_CR_DMAOUTEN;  /* Enable DMA */
  stm32n6_jpeg_putreg(jpegbase, JPEG_CR_OFFSET, regval);

  /* Process input data (JPEG stream) */
  while (input_idx < input_size)
    {
      /* Wait for input FIFO threshold */
      while ((stm32n6_jpeg_getreg(jpegbase, JPEG_SR_OFFSET) & JPEG_SR_IFTF) == 0);

      /* Write data to input register */
      stm32n6_jpeg_putreg(jpegbase, JPEG_DIR_OFFSET, input[input_idx++] |
                          (input[input_idx] << 8) |
                          (input[input_idx+1] << 16) |
                          (input[input_idx+2] << 24));
      input_idx += 4;
    }

  /* Start decoding */
  regval = stm32n6_jpeg_getreg(jpegbase, JPEG_CR_OFFSET);
  regval |= JPEG_CR_START;
  stm32n6_jpeg_putreg(jpegbase, JPEG_CR_OFFSET, regval);

  /* Wait for decoding to complete */
  if (stm32n6_jpeg_wait_for_eoc(jpegbase) != OK)
    {
      return -ETIMEDOUT;
    }

  /* Read decoded output data */
  while (output_idx < *output_size)
    {
      /* Wait for output FIFO threshold */
      while ((stm32n6_jpeg_getreg(jpegbase, JPEG_SR_OFFSET) & JPEG_SR_OFTR) == 0);

      /* Read data from output register */
      uint32_t data = stm32n6_jpeg_getreg(jpegbase, JPEG_DOR_OFFSET);
      if (output_idx < *output_size) output[output_idx++] = data & 0xFF;
      if (output_idx < *output_size) output[output_idx++] = (data >> 8) & 0xFF;
      if (output_idx < *output_size) output[output_idx++] = (data >> 16) & 0xFF;
      if (output_idx < *output_size) output[output_idx++] = (data >> 24) & 0xFF;
    }

  /* Update output size */
  *output_size = output_idx;

  /* Clear end of conversion flag */
  stm32n6_jpeg_putreg(jpegbase, JPEG_CFR_OFFSET, JPEG_CFR_CEOCF);

  return OK;
}

void stm32n6_jpeg_pause(uintptr_t jpegbase)
{
  uint32_t regval;

  regval = stm32n6_jpeg_getreg(jpegbase, JPEG_CR_OFFSET);
  regval |= (JPEG_CMD_PAUSE << JPEG_CR_CMD_SHIFT);
  stm32n6_jpeg_putreg(jpegbase, JPEG_CR_OFFSET, regval);
}

void stm32n6_jpeg_resume(uintptr_t jpegbase)
{
  uint32_t regval;

  regval = stm32n6_jpeg_getreg(jpegbase, JPEG_CR_OFFSET);
  regval &= ~JPEG_CR_CMD_MASK;
  regval |= (JPEG_CMD_RESUME << JPEG_CR_CMD_SHIFT);
  stm32n6_jpeg_putreg(jpegbase, JPEG_CR_OFFSET, regval);
}

void stm32n6_jpeg_reset(uintptr_t jpegbase)
{
  uint32_t regval;

  /* Disable JPEG */
  regval = stm32n6_jpeg_getreg(jpegbase, JPEG_CR_OFFSET);
  regval &= ~JPEG_CR_JCEN;
  stm32n6_jpeg_putreg(jpegbase, JPEG_CR_OFFSET, regval);

  /* Re-enable JPEG */
  regval |= JPEG_CR_JCEN;
  stm32n6_jpeg_putreg(jpegbase, JPEG_CR_OFFSET, regval);
}

bool stm32n6_jpeg_busy(uintptr_t jpegbase)
{
  uint32_t sr = stm32n6_jpeg_getreg(jpegbase, JPEG_SR_OFFSET);
  return (sr & JPEG_SR_COFHF) != 0;
}