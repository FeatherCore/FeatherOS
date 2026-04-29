/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_xspi.c
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
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <debug.h>

#include <nuttx/arch.h>
#include <nuttx/irq.h>
#include <nuttx/spi/spi.h>
#include <arch/board/board.h>

#include "arm_internal.h"
#include "chip.h"
#include "hardware/stm32n6_memorymap.h"
#include "hardware/stm32n6_rcc.h"
#include "stm32n6_gpio.h"
#include "stm32n6_xspi.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define XSPI_TIMEOUT_US             1000000  /* 1 second timeout */

/* Helper macros */

#define XSPI_PUTREG(base, offset, value) \
  putreg32(value, (base) + (offset))
#define XSPI_GETREG(base, offset) \
  getreg32((base) + (offset))

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static inline void xspi_sem_wait(struct stm32n6_xspi_s *priv)
{
  while (nxsem_wait(&priv->dev.lock) < 0)
    {
      DEBUGASSERT(errno == EINTR);
    }
}

static inline void xspi_sem_post(struct stm32n6_xspi_s *priv)
{
  nxsem_post(&priv->dev.lock);
}

static void xspi_enable_clk(uint32_t base)
{
  uint32_t regval;

  /* Enable clock for the appropriate XSPI instance */
  regval = getreg32(STM32_RCC_AHB5ENR);
  if (base == STM32_XSPI1_BASE)
    {
      regval |= RCC_AHB5ENR_XSPI1EN;
    }
  else if (base == STM32_XSPI2_BASE)
    {
      regval |= RCC_AHB5ENR_XSPI2EN;
    }
  else if (base == STM32_XSPI3_BASE)
    {
      regval |= RCC_AHB5ENR_XSPI3EN;
    }
  putreg32(regval, STM32_RCC_AHB5ENR);
}

static int xspi_wait_ready(uint32_t base)
{
  uint32_t timeout = XSPI_TIMEOUT_US;

  while ((XSPI_GETREG(base, STM32_XSPI_SR_OFFSET) & XSPI_SR_BUSY) != 0)
    {
      if (timeout-- == 0)
        {
          return -ETIMEDOUT;
        }
      up_udelay(1);
    }

  return OK;
}

static int xspi_configure(struct stm32n6_xspi_s *priv)
{
  uint32_t regval;
  uint32_t prescaler;

  /* Calculate prescaler based on requested frequency */
  uint32_t ahb_freq = STM32N6_HCLK_FREQUENCY;
  prescaler = (ahb_freq + priv->frequency - 1) / priv->frequency;
  if (prescaler > 256) prescaler = 256;
  if (prescaler < 1) prescaler = 1;
  prescaler--;  /* Register value is prescaler - 1 */

  /* Store actual frequency */
  priv->actual = ahb_freq / (prescaler + 1);

  /* Disable XSPI before configuration */
  XSPI_PUTREG(priv->base, STM32_XSPI_CR_OFFSET, 0);

  /* Wait for ready state */
  xspi_wait_ready(priv->base);

  /* Configure DCR registers for memory characteristics */
  regval = XSPI_GETREG(priv->base, STM32_XSPI_DCR1_OFFSET);
  regval &= ~(0xFFFFF << 0);  /* Clear DEVSIZE */
  regval |= ((23) << 0);      /* Assuming 8MB device size (2^23) */
  XSPI_PUTREG(priv->base, STM32_XSPI_DCR1_OFFSET, regval);

  /* Configure CS high time */
  regval = XSPI_GETREG(priv->base, STM32_XSPI_DCR2_OFFSET);
  regval &= ~(0xFF << 8);     /* Clear CSHT */
  regval |= (7 << 8);         /* 8 cycles CS high time */
  XSPI_PUTREG(priv->base, STM32_XSPI_DCR2_OFFSET, regval);

  /* Configure FIFO threshold */
  regval = XSPI_GETREG(priv->base, STM32_XSPI_CR_OFFSET);
  regval &= ~(0x1F << 8);     /* Clear FTHOLD */
  regval |= ((XSPI_FIFO_THRESHOLD - 1) << 8);  /* FIFO threshold */
  XSPI_PUTREG(priv->base, STM32_XSPI_CR_OFFSET, regval);

  /* Apply prescaler */
  regval = XSPI_GETREG(priv->base, STM32_XSPI_CR_OFFSET);
  regval &= ~XSPI_CR_PRESCALER_MASK;
  regval |= ((prescaler & 0x0F) << XSPI_CR_PRESCALER_SHIFT);
  XSPI_PUTREG(priv->base, STM32_XSPI_CR_OFFSET, regval);

  /* Clear flags */
  XSPI_PUTREG(priv->base, STM32_XSPI_FCR_OFFSET, 0x1F);

  /* Enable XSPI */
  regval = XSPI_GETREG(priv->base, STM32_XSPI_CR_OFFSET);
  regval |= XSPI_CR_EN;
  XSPI_PUTREG(priv->base, STM32_XSPI_CR_OFFSET, regval);

  return OK;
}

/* SPI V-table */

static int xspi_lock(struct spi_dev_s *dev, bool lock)
{
  struct stm32n6_xspi_s *priv = (struct stm32n6_xspi_s *)dev;

  if (lock)
    {
      xspi_sem_wait(priv);
    }
  else
    {
      xspi_sem_post(priv);
    }

  return OK;
}

static uint32_t xspi_setfrequency(struct spi_dev_s *dev, uint32_t frequency)
{
  struct stm32n6_xspi_s *priv = (struct stm32n6_xspi_s *)dev;
  irqstate_t flags;

  flags = enter_critical_section();

  /* Check if frequency is the same */
  if (priv->frequency == frequency)
    {
      leave_critical_section(flags);
      return priv->actual;
    }

  /* Save the new frequency */
  priv->frequency = frequency;

  /* Reconfigure with new frequency */
  xspi_configure(priv);

  leave_critical_section(flags);
  return priv->actual;
}

static void xspi_setmode(struct spi_dev_s *dev, enum spi_mode_e mode)
{
  /* XSPI handles modes differently, typically set in CCR */
  /* This is handled in command configuration */
}

static void xspi_setbits(struct spi_dev_s *dev, int nbits)
{
  /* XSPI handles data width in CCR register */
  /* This is handled in command configuration */
}

static uint32_t xspi_send(struct spi_dev_s *dev, const void *txbuffer,
                          void *rxbuffer, size_t nwords)
{
  struct stm32n6_xspi_s *priv = (struct stm32n6_xspi_s *)dev;
  uint32_t regval;
  size_t i;
  const uint8_t *txptr = (const uint8_t *)txbuffer;
  uint8_t *rxptr = (uint8_t *)rxbuffer;

  /* Wait for not busy */
  if (xspi_wait_ready(priv->base) != OK)
    {
      return 0;
    }

  /* Configure for indirect mode */
  regval = XSPI_CCR_FMODE_IND_WRITE;
  XSPI_PUTREG(priv->base, STM32_XSPI_CCR_OFFSET, regval);

  /* Set data length */
  XSPI_PUTREG(priv->base, STM32_XSPI_DLR_OFFSET, nwords - 1);

  /* For write operations, send data */
  if (txbuffer != NULL)
    {
      for (i = 0; i < nwords; i++)
        {
          /* Wait until FIFO is ready */
          while ((XSPI_GETREG(priv->base, STM32_XSPI_SR_OFFSET) & XSPI_SR_FTF) == 0);

          /* Write data to DR */
          XSPI_PUTREG(priv->base, STM32_XSPI_DR_OFFSET, txptr[i]);
        }
    }

  /* Wait for transfer complete */
  while ((XSPI_GETREG(priv->base, STM32_XSPI_SR_OFFSET) & XSPI_SR_TCF) == 0);

  /* For read operations, read data */
  if (rxbuffer != NULL)
    {
      /* Reconfigure for read */
      regval = XSPI_CCR_FMODE_IND_READ;
      XSPI_PUTREG(priv->base, STM32_XSPI_CCR_OFFSET, regval);
      XSPI_PUTREG(priv->base, STM32_XSPI_DLR_OFFSET, nwords - 1);

      for (i = 0; i < nwords; i++)
        {
          /* Wait until data is available */
          while ((XSPI_GETREG(priv->base, STM32_XSPI_SR_OFFSET) & XSPI_SR_FTF) == 0);

          /* Read data from DR */
          rxptr[i] = (uint8_t)XSPI_GETREG(priv->base, STM32_XSPI_DR_OFFSET);
        }
    }

  /* Clear transfer complete flag */
  XSPI_PUTREG(priv->base, STM32_XSPI_FCR_OFFSET, XSPI_SR_TCF);

  return nwords;
}

static void xspi_exchange(struct spi_dev_s *dev, const void *txbuffer,
                          void *rxbuffer, size_t nwords)
{
  xspi_send(dev, txbuffer, rxbuffer, nwords);
}

static void xspi_sndblock(struct spi_dev_s *dev, const void *buffer, size_t nwords)
{
  xspi_send(dev, buffer, NULL, nwords);
}

static void xspi_recvblock(struct spi_dev_s *dev, void *buffer, size_t nwords)
{
  xspi_send(dev, NULL, buffer, nwords);
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

int stm32n6_xspi_initialize(uint32_t base)
{
  struct stm32n6_xspi_s *priv;
  int ret;

  /* Allocate memory for the driver instance */
  priv = (struct stm32n6_xspi_s *)kmm_zalloc(sizeof(struct stm32n6_xspi_s));
  if (priv == NULL)
    {
      return -ENOMEM;
    }

  /* Initialize the driver instance */
  priv->base = base;
  priv->frequency = 1000000;  /* 1MHz default */
  priv->nbits = 8;            /* 8-bit words */
  priv->mode = SPIDEV_MODE0;  /* Mode 0 */

  /* Initialize the SPI device structure */
  priv->dev.ops = &g_xspiops;
  nxsem_init(&priv->dev.lock, 0, 1);

  /* Enable XSPI clock */
  xspi_enable_clk(base);

  /* Configure XSPI */
  ret = xspi_configure(priv);
  if (ret < 0)
    {
      kmm_free(priv);
      return ret;
    }

  /* Register the device */
  spi_register(&priv->dev, 0);

  return OK;
}

/* SPI vtable */
static const struct spi_ops_s g_xspiops =
{
  .lock              = xspi_lock,
  .select            = NULL,  /* External chip select control */
  .setfrequency      = xspi_setfrequency,
  .setmode           = xspi_setmode,
  .setbits           = xspi_setbits,
  .status            = NULL,
  .cmddata           = NULL,
  .send              = xspi_send,
  .sndblock          = xspi_sndblock,
  .recvblock         = xspi_recvblock,
  .exchange          = xspi_exchange,
#ifdef CONFIG_SPI_CALLBACK
  .registercallback  = NULL,
#endif
#ifdef CONFIG_SPI_TRANSFER_MUX
  .transfer          = NULL,
#endif
};