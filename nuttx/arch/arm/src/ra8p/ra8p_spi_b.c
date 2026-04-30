/****************************************************************************
 * arch/arm/src/ra8p/ra8p_spi_b.c
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

#include "chip.h"
#include "arm_internal.h"
#include "hardware/ra8p_spi_b.h"

#if defined(CONFIG_RA8P_SPI_B0) || defined(CONFIG_RA8P_SPI_B1) || \
    defined(CONFIG_RA8P_SPI_B2) || defined(CONFIG_RA8P_SPI_B3)

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define RA8P_SPI0_BASE              RA8P_SPI0_BASE
#define RA8P_SPI1_BASE              RA8P_SPI1_BASE
#define RA8P_SPI2_BASE              RA8P_SPI2_BASE
#define RA8P_SPI3_BASE              RA8P_SPI3_BASE

/* SPI timeout in milliseconds */
#define RA8P_SPI_TIMEOUT_MS         1000

/* Default SPI clock frequency */
#define RA8P_SPI_DEFAULT_FREQ       1000000  /* 1 MHz */

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* RA8P1 SPI_B driver state structure */

struct ra8p_spi_b_priv_s
{
  uint32_t base;                  /* Base address of SPI registers */
  uint8_t channel;                /* SPI channel (0-3) */
  bool initialized;               /* Initialization flag */
  bool enabled;                   /* Enable flag */
  uint32_t frequency;             /* Current SPI frequency */
  uint8_t mode;                   /* SPI mode (0-3) */
  uint8_t bits;                   /* Bits per transfer (4-32) */
  uint32_t pclk;                  /* PCLK frequency for baud rate calculations */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int spi_b_set_frequency(struct ra8p_spi_b_priv_s *priv, uint32_t freq);
static void spi_b_set_mode(struct ra8p_spi_b_priv_s *priv, uint8_t mode);
static void spi_b_set_bits(struct ra8p_spi_b_priv_s *priv, uint8_t nbits);
static void spi_b_putreg32(struct ra8p_spi_b_priv_s *priv, uint32_t offset, uint32_t value);
static uint32_t spi_b_getreg32(struct ra8p_spi_b_priv_s *priv, uint32_t offset);
static void spi_b_putreg16(struct ra8p_spi_b_priv_s *priv, uint32_t offset, uint16_t value);
static uint16_t spi_b_getreg16(struct ra8p_spi_b_priv_s *priv, uint32_t offset);
static void spi_b_putreg8(struct ra8p_spi_b_priv_s *priv, uint32_t offset, uint8_t value);
static uint8_t spi_b_getreg8(struct ra8p_spi_b_priv_s *priv, uint32_t offset);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_spi_b_priv_s g_spi_b[4];  /* 4 SPI channels: SPI0-SPI3 */

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: spi_b_putreg32
 ****************************************************************************/

static inline void spi_b_putreg32(struct ra8p_spi_b_priv_s *priv, uint32_t offset, uint32_t value)
{
  putreg32(value, priv->base + offset);
}

/****************************************************************************
 * Name: spi_b_getreg32
 ****************************************************************************/

static inline uint32_t spi_b_getreg32(struct ra8p_spi_b_priv_s *priv, uint32_t offset)
{
  return getreg32(priv->base + offset);
}

/****************************************************************************
 * Name: spi_b_putreg16
 ****************************************************************************/

static inline void spi_b_putreg16(struct ra8p_spi_b_priv_s *priv, uint32_t offset, uint16_t value)
{
  putreg16(value, priv->base + offset);
}

/****************************************************************************
 * Name: spi_b_getreg16
 ****************************************************************************/

static inline uint16_t spi_b_getreg16(struct ra8p_spi_b_priv_s *priv, uint32_t offset)
{
  return getreg16(priv->base + offset);
}

/****************************************************************************
 * Name: spi_b_putreg8
 ****************************************************************************/

static inline void spi_b_putreg8(struct ra8p_spi_b_priv_s *priv, uint32_t offset, uint8_t value)
{
  putreg8(value, priv->base + offset);
}

/****************************************************************************
 * Name: spi_b_getreg8
 ****************************************************************************/

static inline uint8_t spi_b_getreg8(struct ra8p_spi_b_priv_s *priv, uint32_t offset)
{
  return getreg8(priv->base + offset);
}

/****************************************************************************
 * Name: spi_b_wait_tx_ready
 ****************************************************************************/

static int spi_b_wait_tx_ready(struct ra8p_spi_b_priv_s *priv)
{
  volatile int timeout = RA8P_SPI_TIMEOUT_MS * 1000;  /* microseconds */

  while (!(spi_b_getreg8(priv, RA8P_SPI_B_SPSR_OFFSET) & RA8P_SPI_B_SPSR_SPTEF) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  return OK;
}

/****************************************************************************
 * Name: spi_b_wait_rx_ready
 ****************************************************************************/

static int spi_b_wait_rx_ready(struct ra8p_spi_b_priv_s *priv)
{
  volatile int timeout = RA8P_SPI_TIMEOUT_MS * 1000;  /* microseconds */

  while (!(spi_b_getreg8(priv, RA8P_SPI_B_SPSR_OFFSET) & RA8P_SPI_B_SPSR_SPRF) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  return OK;
}

/****************************************************************************
 * Name: spi_b_calc_baudrate
 ****************************************************************************/

static int spi_b_calc_baudrate(struct ra8p_spi_b_priv_s *priv, uint32_t target_freq,
                               uint8_t *div_factor, uint8_t *clock_div)
{
  uint32_t pclk = priv->pclk;
  uint32_t div;
  uint8_t factor;

  if (target_freq == 0)
    {
      return -EINVAL;
    }

  /* Calculate divider to achieve closest frequency */
  div = pclk / target_freq;

  /* Find closest division factor from available options */
  for (factor = 0; factor <= 7; factor++)
    {
      uint32_t test_div = (pclk >> factor) / target_freq;
      if (test_div <= 255)  /* Max divider value */
        {
          *clock_div = (uint8_t)test_div;
          *div_factor = factor;
          return OK;
        }
    }

  /* If still couldn't find a valid combination, use maximum */
  *clock_div = 255;
  *div_factor = 7;
  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_spi_b_initialize
 *
 * Description:
 *   Initialize the SPI_B channel.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   frequency - SPI clock frequency in Hz
 *   mode - SPI mode (0-3)
 *   nbits - Number of bits per transfer (4-32)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_initialize(uint8_t channel, uint32_t frequency, uint8_t mode, uint8_t nbits)
{
  struct ra8p_spi_b_priv_s *priv;
  uint8_t spcr;
  uint8_t smr;
  uint8_t scr;
  uint8_t spbr;
  uint8_t div_factor;
  int ret;

  if (channel >= 4 || mode > 3 || nbits < 4 || nbits > 32)
    {
      return -EINVAL;
    }

  priv = &g_spi_b[channel];
  priv->channel = channel;

  /* Set base address based on channel */
  switch (channel)
    {
      case 0:  priv->base = RA8P_SPI0_BASE; break;
      case 1:  priv->base = RA8P_SPI1_BASE; break;
      case 2:  priv->base = RA8P_SPI2_BASE; break;
      case 3:  priv->base = RA8P_SPI3_BASE; break;
      default: return -EINVAL;
    }

  priv->frequency = frequency ? frequency : RA8P_SPI_DEFAULT_FREQ;
  priv->mode = mode;
  priv->bits = nbits;
  priv->pclk = 62500000;  /* Use appropriate PCLK frequency */
  priv->initialized = false;
  priv->enabled = false;

  /* Reset SPI module */
  uint8_t sphcr = spi_b_getreg8(priv, RA8P_SPI_B_SPHCR_OFFSET);
  sphcr |= RA8P_SPI_B_SPHCR_SPIRST;
  spi_b_putreg8(priv, RA8P_SPI_B_SPHCR_OFFSET, sphcr);

  /* Wait for reset to complete */
  volatile int timeout = 10000;
  while (spi_b_getreg8(priv, RA8P_SPI_B_SPHCR_OFFSET) & RA8P_SPI_B_SPHCR_SPIRST && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  /* Configure SPI mode register */
  smr = 0;
  smr |= (mode & 0x03);  /* Set CPOL and CPHA based on mode */

  /* SPI mode mapping:
   * Mode 0: CPOL=0, CPHA=0
   * Mode 1: CPOL=0, CPHA=1
   * Mode 2: CPOL=1, CPHA=0
   * Mode 3: CPOL=1, CPHA=1
   */
  if (mode & 0x02)  /* Set CPOL */
    {
      smr |= RA8P_SPI_B_SMR_CPOL;
    }

  if (mode & 0x01)  /* Set CPHA */
    {
      smr |= RA8P_SPI_B_SMR_CPHA;
    }

  spi_b_putreg8(priv, RA8P_SPI_B_SMR_OFFSET, smr);

  /* Configure SPI control register */
  spcr = RA8P_SPI_B_SPCR_SPMS |    /* Set to SPI mode */
         RA8P_SPI_B_SPCR_MSTR |    /* Master mode */
         RA8P_SPI_B_SPCR_MODFEN;   /* Mode fault detection enabled */

  /* Set data length */
  uint8_t bfc = (nbits - 1) & 0x1F;  /* BFCA bits: 0-31 correspond to 1-32 bits */
  uint8_t regval = spi_b_getreg8(priv, RA8P_SPI_B_BFCR_OFFSET);
  regval &= ~RA8P_SPI_B_BFCR_BFCA_MASK;
  regval |= (bfc << RA8P_SPI_B_BFCR_BFCA_SHIFT);
  spi_b_putreg8(priv, RA8P_SPI_B_BFCR_OFFSET, regval);

  /* Configure baud rate */
  uint8_t div, factor;
  ret = spi_b_calc_baudrate(priv, priv->frequency, &factor, &div);
  if (ret != OK)
    {
      return ret;
    }

  /* Set clock division factor */
  regval = spi_b_getreg8(priv, RA8P_SPI_B_SCR_OFFSET);
  regval &= ~RA8P_SPI_B_SCR_SPSSLF_MASK;
  regval |= (factor << RA8P_SPI_B_SCR_SPSSLF_SHIFT);
  spi_b_putreg8(priv, RA8P_SPI_B_SCR_OFFSET, regval);

  /* Set bit rate */
  spi_b_putreg8(priv, RA8P_SPI_B_SPBR_OFFSET, div);

  /* Configure pin functions */
  regval = spi_b_getreg8(priv, RA8P_SPI_B_SPPCR_OFFSET);
  regval |= RA8P_SPI_B_SPPCR_SPLP;   /* LoSSI mode disable */
  regval |= RA8P_SPI_B_SPPCR_SPLP2;  /* LoSSI2 mode disable */
  spi_b_putreg8(priv, RA8P_SPI_B_SPPCR_OFFSET, regval);

  /* Enable SPI */
  spcr |= RA8P_SPI_B_SPCR_SPE;
  spi_b_putreg8(priv, RA8P_SPI_B_SPCR_OFFSET, spcr);

  /* Wait for SPI to be enabled */
  timeout = 10000;
  while (!(spi_b_getreg8(priv, RA8P_SPI_B_SPSR_OFFSET) & RA8P_SPI_B_SPSR_SPIF) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  priv->initialized = true;
  priv->enabled = true;

  spiinfo("SPI_B%d initialized at 0x%08x, freq=%u Hz, mode=%u, bits=%u\n", 
          channel, priv->base, frequency, mode, nbits);
  return OK;
}

/****************************************************************************
 * Name: ra8p_spi_b_exchange
 *
 * Description:
 *   Exchange data on SPI bus.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   txdata - Pointer to TX data buffer (may be NULL for read-only)
 *   rxdata - Pointer to RX data buffer (may be NULL for write-only)
 *   nwords - Number of words to exchange
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_exchange(uint8_t channel, const void *txdata, void *rxdata, size_t nwords)
{
  struct ra8p_spi_b_priv_s *priv;
  const uint8_t *tx8 = (const uint8_t *)txdata;
  uint8_t *rx8 = (uint8_t *)rxdata;
  const uint16_t *tx16 = (const uint16_t *)txdata;
  uint16_t *rx16 = (uint16_t *)rxdata;
  size_t i;
  int ret;
  uint32_t data;
  uint32_t dummy;

  if (channel >= 4 || !g_spi_b[channel].enabled || nwords == 0)
    {
      return -EINVAL;
    }

  if ((txdata == NULL && rxdata == NULL))
    {
      return -EINVAL;
    }

  priv = &g_spi_b[channel];

  for (i = 0; i < nwords; i++)
    {
      /* Wait for transmit buffer to be ready */
      ret = spi_b_wait_tx_ready(priv);
      if (ret != OK)
        {
          return ret;
        }

      /* Prepare data to send */
      if (priv->bits <= 8)
        {
          /* 8-bit or less */
          if (txdata != NULL)
            {
              data = tx8 ? tx8[i] : 0xFF;
            }
          else
            {
              data = 0xFF;
            }
        }
      else if (priv->bits <= 16)
        {
          /* 9-16 bits */
          if (txdata != NULL)
            {
              data = tx16 ? tx16[i] : 0xFFFF;
            }
          else
            {
              data = 0xFFFF;
            }
        }
      else
        {
          /* 17-32 bits - treat as 32-bit */
          if (txdata != NULL)
            {
              data = ((const uint32_t *)txdata)[i];
            }
          else
            {
              data = 0xFFFFFFFF;
            }
        }

      /* Write data to SPI */
      spi_b_putreg32(priv, RA8P_SPI_B_SPDR_OFFSET, data);

      /* Wait for receive buffer to be ready */
      ret = spi_b_wait_rx_ready(priv);
      if (ret != OK)
        {
          return ret;
        }

      /* Read received data */
      dummy = spi_b_getreg32(priv, RA8P_SPI_B_SPDR_OFFSET);

      /* Store received data if buffer provided */
      if (rxdata != NULL)
        {
          if (priv->bits <= 8)
            {
              rx8[i] = (uint8_t)dummy;
            }
          else if (priv->bits <= 16)
            {
              rx16[i] = (uint16_t)dummy;
            }
          else
            {
              ((uint32_t *)rxdata)[i] = dummy;
            }
        }
    }

  spiinfo("SPI_B%d exchanged %zu words\n", channel, nwords);
  return OK;
}

/****************************************************************************
 * Name: ra8p_spi_b_set_frequency
 *
 * Description:
 *   Set SPI frequency.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   frequency - New SPI frequency in Hz
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_set_frequency(uint8_t channel, uint32_t frequency)
{
  struct ra8p_spi_b_priv_s *priv;
  uint8_t div, factor;
  int ret;
  uint8_t regval;

  if (channel >= 4 || frequency == 0 || !g_spi_b[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_spi_b[channel];

  /* Calculate new baud rate settings */
  ret = spi_b_calc_baudrate(priv, frequency, &factor, &div);
  if (ret != OK)
    {
      return ret;
    }

  /* Temporarily disable SPI for reconfiguration */
  regval = spi_b_getreg8(priv, RA8P_SPI_B_SPCR_OFFSET);
  regval &= ~RA8P_SPI_B_SPCR_SPE;  /* Clear SPE bit */
  spi_b_putreg8(priv, RA8P_SPI_B_SPCR_OFFSET, regval);

  /* Configure new clock division factor */
  regval = spi_b_getreg8(priv, RA8P_SPI_B_SCR_OFFSET);
  regval &= ~RA8P_SPI_B_SCR_SPSSLF_MASK;
  regval |= (factor << RA8P_SPI_B_SCR_SPSSLF_SHIFT);
  spi_b_putreg8(priv, RA8P_SPI_B_SCR_OFFSET, regval);

  /* Set new bit rate */
  spi_b_putreg8(priv, RA8P_SPI_B_SPBR_OFFSET, div);

  /* Re-enable SPI */
  regval = spi_b_getreg8(priv, RA8P_SPI_B_SPCR_OFFSET);
  regval |= RA8P_SPI_B_SPCR_SPE;  /* Set SPE bit */
  spi_b_putreg8(priv, RA8P_SPI_B_SPCR_OFFSET, regval);

  priv->frequency = frequency;

  spiinfo("SPI_B%d frequency set to %u Hz\n", channel, frequency);
  return OK;
}

/****************************************************************************
 * Name: ra8p_spi_b_set_mode
 *
 * Description:
 *   Set SPI mode (CPOL/CPHA).
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   mode - SPI mode (0-3)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_set_mode(uint8_t channel, uint8_t mode)
{
  struct ra8p_spi_b_priv_s *priv;
  uint8_t regval;

  if (channel >= 4 || mode > 3 || !g_spi_b[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_spi_b[channel];

  /* Temporarily disable SPI for reconfiguration */
  regval = spi_b_getreg8(priv, RA8P_SPI_B_SPCR_OFFSET);
  regval &= ~RA8P_SPI_B_SPCR_SPE;  /* Clear SPE bit */
  spi_b_putreg8(priv, RA8P_SPI_B_SPCR_OFFSET, regval);

  /* Update mode in SMR register */
  regval = spi_b_getreg8(priv, RA8P_SPI_B_SMR_OFFSET);
  regval &= ~(RA8P_SPI_B_SMR_CPOL | RA8P_SPI_B_SMR_CPHA);  /* Clear CPOL and CPHA */

  if (mode & 0x02)  /* Set CPOL */
    {
      regval |= RA8P_SPI_B_SMR_CPOL;
    }

  if (mode & 0x01)  /* Set CPHA */
    {
      regval |= RA8P_SPI_B_SMR_CPHA;
    }

  spi_b_putreg8(priv, RA8P_SPI_B_SMR_OFFSET, regval);

  /* Re-enable SPI */
  regval = spi_b_getreg8(priv, RA8P_SPI_B_SPCR_OFFSET);
  regval |= RA8P_SPI_B_SPCR_SPE;  /* Set SPE bit */
  spi_b_putreg8(priv, RA8P_SPI_B_SPCR_OFFSET, regval);

  priv->mode = mode;

  spiinfo("SPI_B%d mode set to %u\n", channel, mode);
  return OK;
}

/****************************************************************************
 * Name: ra8p_spi_b_set_bits
 *
 * Description:
 *   Set number of bits per transfer.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   nbits - Number of bits (4-32)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_set_bits(uint8_t channel, uint8_t nbits)
{
  struct ra8p_spi_b_priv_s *priv;
  uint8_t regval;

  if (channel >= 4 || (nbits < 4 && nbits > 32) || !g_spi_b[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_spi_b[channel];

  /* Temporarily disable SPI for reconfiguration */
  regval = spi_b_getreg8(priv, RA8P_SPI_B_SPCR_OFFSET);
  regval &= ~RA8P_SPI_B_SPCR_SPE;  /* Clear SPE bit */
  spi_b_putreg8(priv, RA8P_SPI_B_SPCR_OFFSET, regval);

  /* Set data length in BFCR register */
  uint8_t bfc = (nbits - 1) & 0x1F;  /* BFCA bits: 0-31 correspond to 1-32 bits */
  regval = spi_b_getreg8(priv, RA8P_SPI_B_BFCR_OFFSET);
  regval &= ~RA8P_SPI_B_BFCR_BFCA_MASK;
  regval |= (bfc << RA8P_SPI_B_BFCR_BFCA_SHIFT);
  spi_b_putreg8(priv, RA8P_SPI_B_BFCR_OFFSET, regval);

  /* Re-enable SPI */
  regval = spi_b_getreg8(priv, RA8P_SPI_B_SPCR_OFFSET);
  regval |= RA8P_SPI_B_SPCR_SPE;  /* Set SPE bit */
  spi_b_putreg8(priv, RA8P_SPI_B_SPCR_OFFSET, regval);

  priv->bits = nbits;

  spiinfo("SPI_B%d bits per transfer set to %u\n", channel, nbits);
  return OK;
}

/****************************************************************************
 * Name: ra8p_spi_b_enable
 *
 * Description:
 *   Enable the SPI channel.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_enable(uint8_t channel)
{
  struct ra8p_spi_b_priv_s *priv;
  uint8_t regval;

  if (channel >= 4 || !g_spi_b[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_spi_b[channel];

  /* Enable SPI */
  regval = spi_b_getreg8(priv, RA8P_SPI_B_SPCR_OFFSET);
  regval |= RA8P_SPI_B_SPCR_SPE;
  spi_b_putreg8(priv, RA8P_SPI_B_SPCR_OFFSET, regval);

  priv->enabled = true;

  spiinfo("SPI_B%d enabled\n", channel);
  return OK;
}

/****************************************************************************
 * Name: ra8p_spi_b_disable
 *
 * Description:
 *   Disable the SPI channel.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_disable(uint8_t channel)
{
  struct ra8p_spi_b_priv_s *priv;
  uint8_t regval;

  if (channel >= 4 || !g_spi_b[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_spi_b[channel];

  /* Disable SPI */
  regval = spi_b_getreg8(priv, RA8P_SPI_B_SPCR_OFFSET);
  regval &= ~RA8P_SPI_B_SPCR_SPE;
  spi_b_putreg8(priv, RA8P_SPI_B_SPCR_OFFSET, regval);

  priv->enabled = false;

  spiinfo("SPI_B%d disabled\n", channel);
  return OK;
}

#endif /* CONFIG_RA8P_SPI_B* */