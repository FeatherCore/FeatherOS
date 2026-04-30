/****************************************************************************
 * arch/arm/src/ra8p/ra8p_sdhc.c
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
#include "hardware/ra8p_sdhc.h"
#include "hardware/ra8p_memorymap.h"

#ifdef CONFIG_RA8P_SDHC

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define RA8P_SDHC0_BASE               RA8P_SDHC0_BASE
#define RA8P_SDHC1_BASE               RA8P_SDHC1_BASE

/* SDHC timeout in milliseconds */
#define RA8P_SDHC_TIMEOUT_MS          5000
#define RA8P_SDHC_CMD_TIMEOUT_MS      1000

/* SDHC block size */
#define RA8P_SDHC_BLOCK_SIZE          512

/* Clock divider for identification mode (400 kHz) */
#define RA8P_SDHC_IDENTIFICATION_DIVIDER  96  /* Assuming 62.5MHz / 96 ≈ 400kHz */

/* Clock divider for data transfer mode */
#define RA8P_SDHC_TRANSFER_DIVIDER    2   /* Faster clock for data transfer */

/* CMD16 argument - Set Block Length to 512 bytes */
#define RA8P_SDHC_CMD16_ARG           512

/* Maximum retry attempts */
#define RA8P_SDHC_MAX_RETRIES         3

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* RA8P1 SDHC driver state structure */

struct ra8p_sdhc_priv_s
{
  uint32_t base;                  /* Base address of SDHC registers */
  uint8_t channel;                /* SDHC channel (0 or 1) */
  bool initialized;               /* Initialization flag */
  bool enabled;                   /* Enable flag */
  bool card_inserted;             /* Card detection status */
  bool high_capacity;             /* SDHC card flag */
  bool sdio;                      /* SDIO card */
  bool mmc;                       /* MMC card */
  uint32_t card_capacity;         /* Card capacity in bytes */
  uint32_t max_frequency;         /* Maximum supported frequency */
  uint32_t current_frequency;     /* Current clock frequency */
  uint8_t bus_width;              /* Current bus width (1, 4, or 8) */
  uint32_t rca;                  /* Relative card address */
  uint32_t ocr;                  /* OCR value from card */
  uint8_t csd[16];               /* CSD register value */
  uint8_t cid[16];               /* CID register value */
  bool write_protected;           /* Write protect status */
  sem_t wait_sem;                 /* Wait semaphore for transfers */
  struct sdio_dev_s *driver;      /* SDIO device driver */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int sdhc_wait_ready(struct ra8p_sdhc_priv_s *priv);
static int sdhc_set_frequency(struct ra8p_sdhc_priv_s *priv, uint32_t freq);
static int sdhc_send_cmd(struct ra8p_sdhc_priv_s *priv, uint32_t cmd, uint32_t arg, uint8_t resp_type);
static void sdhc_putreg32(struct ra8p_sdhc_priv_s *priv, uint32_t offset, uint32_t value);
static uint32_t sdhc_getreg32(struct ra8p_sdhc_priv_s *priv, uint32_t offset);
static void sdhc_putreg16(struct ra8p_sdhc_priv_s *priv, uint32_t offset, uint16_t value);
static uint16_t sdhc_getreg16(struct ra8p_sdhc_priv_s *priv, uint32_t offset);
static void sdhc_putreg8(struct ra8p_sdhc_priv_s *priv, uint32_t offset, uint8_t value);
static uint8_t sdhc_getreg8(struct ra8p_sdhc_priv_s *priv, uint32_t offset);
static int sdhc_identify_card(struct ra8p_sdhc_priv_s *priv);
static int sdhc_wait_cmd_complete(struct ra8p_sdhc_priv_s *priv);
static int sdhc_wait_transfer_complete(struct ra8p_sdhc_priv_s *priv);
static int sdhc_wait_data_ready(struct ra8p_sdhc_priv_s *priv);
static int sdhc_reset_controller(struct ra8p_sdhc_priv_s *priv);
static int sdhc_power_up(struct ra8p_sdhc_priv_s *priv);
static int sdhc_power_down(struct ra8p_sdhc_priv_s *priv);
static int sdhc_set_bus_width(struct ra8p_sdhc_priv_s *priv, uint8_t width);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_sdhc_priv_s g_sdhc[2];  /* Two SDHC channels */

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: sdhc_putreg32
 ****************************************************************************/

static inline void sdhc_putreg32(struct ra8p_sdhc_priv_s *priv, uint32_t offset, uint32_t value)
{
  putreg32(value, priv->base + offset);
}

/****************************************************************************
 * Name: sdhc_getreg32
 ****************************************************************************/

static inline uint32_t sdhc_getreg32(struct ra8p_sdhc_priv_s *priv, uint32_t offset)
{
  return getreg32(priv->base + offset);
}

/****************************************************************************
 * Name: sdhc_putreg16
 ****************************************************************************/

static inline void sdhc_putreg16(struct ra8p_sdhc_priv_s *priv, uint32_t offset, uint16_t value)
{
  putreg16(value, priv->base + offset);
}

/****************************************************************************
 * Name: sdhc_getreg16
 ****************************************************************************/

static inline uint16_t sdhc_getreg16(struct ra8p_sdhc_priv_s *priv, uint32_t offset)
{
  return getreg16(priv->base + offset);
}

/****************************************************************************
 * Name: sdhc_putreg8
 ****************************************************************************/

static inline void sdhc_putreg8(struct ra8p_sdhc_priv_s *priv, uint32_t offset, uint8_t value)
{
  putreg8(value, priv->base + offset);
}

/****************************************************************************
 * Name: sdhc_getreg8
 ****************************************************************************/

static inline uint8_t sdhc_getreg8(struct ra8p_sdhc_priv_s *priv, uint32_t offset)
{
  return getreg8(priv->base + offset);
}

/****************************************************************************
 * Name: sdhc_wait_ready
 ****************************************************************************/

static int sdhc_wait_ready(struct ra8p_sdhc_priv_s *priv)
{
  volatile int timeout = RA8P_SDHC_TIMEOUT_MS * 1000;  /* microseconds */

  /* Wait for no command inhibit */
  while ((sdhc_getreg32(priv, RA8P_SDHC_SD_PRESENT_STATE_OFFSET) & 
          RA8P_SDHC_SD_PRESENT_STATE_CMD_INHIBIT_CMD) && timeout--)
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
 * Name: sdhc_wait_cmd_complete
 ****************************************************************************/

static int sdhc_wait_cmd_complete(struct ra8p_sdhc_priv_s *priv)
{
  volatile int timeout = RA8P_SDHC_CMD_TIMEOUT_MS * 1000;  /* microseconds */
  uint32_t status;

  /* Wait for command complete interrupt */
  while (timeout-- > 0)
    {
      status = sdhc_getreg32(priv, RA8P_SDHC_SD_INT_STATUS_OFFSET);
      if (status & RA8P_SDHC_SD_INT_STATUS_CMD_COMPLETE)
        {
          /* Clear command complete interrupt */
          sdhc_putreg32(priv, RA8P_SDHC_SD_INT_STATUS_OFFSET, RA8P_SDHC_SD_INT_STATUS_CMD_COMPLETE);
          return OK;
        }
      up_udelay(1);
    }

  return -ETIMEDOUT;
}

/****************************************************************************
 * Name: sdhc_wait_transfer_complete
 ****************************************************************************/

static int sdhc_wait_transfer_complete(struct ra8p_sdhc_priv_s *priv)
{
  volatile int timeout = RA8P_SDHC_TIMEOUT_MS * 1000;  /* microseconds */
  uint32_t status;

  /* Wait for transfer complete interrupt */
  while (timeout-- > 0)
    {
      status = sdhc_getreg32(priv, RA8P_SDHC_SD_INT_STATUS_OFFSET);
      if (status & RA8P_SDHC_SD_INT_STATUS_TRANSFER_COMPLETE)
        {
          /* Clear transfer complete interrupt */
          sdhc_putreg32(priv, RA8P_SDHC_SD_INT_STATUS_OFFSET, RA8P_SDHC_SD_INT_STATUS_TRANSFER_COMPLETE);
          return OK;
        }
      up_udelay(1);
    }

  return -ETIMEDOUT;
}

/****************************************************************************
 * Name: sdhc_wait_data_ready
 ****************************************************************************/

static int sdhc_wait_data_ready(struct ra8p_sdhc_priv_s *priv)
{
  volatile int timeout = RA8P_SDHC_TIMEOUT_MS * 1000;  /* microseconds */
  uint32_t status;

  /* Wait for buffer read/write ready interrupt */
  while (timeout-- > 0)
    {
      status = sdhc_getreg32(priv, RA8P_SDHC_SD_INT_STATUS_OFFSET);
      if (status & (RA8P_SDHC_SD_INT_STATUS_BUF_READ_READY | RA8P_SDHC_SD_INT_STATUS_BUF_WRITE_READY))
        {
          /* Clear buffer ready interrupt */
          sdhc_putreg32(priv, RA8P_SDHC_SD_INT_STATUS_OFFSET, 
                       status & (RA8P_SDHC_SD_INT_STATUS_BUF_READ_READY | 
                                RA8P_SDHC_SD_INT_STATUS_BUF_WRITE_READY));
          return OK;
        }
      up_udelay(1);
    }

  return -ETIMEDOUT;
}

/****************************************************************************
 * Name: sdhc_set_frequency
 ****************************************************************************/

static int sdhc_set_frequency(struct ra8p_sdhc_priv_s *priv, uint32_t freq)
{
  uint32_t clk_ctrl;
  uint32_t div;
  uint32_t pclk = 62500000;  /* Use appropriate PCLK */

  if (freq == 0)
    {
      return -EINVAL;
    }

  /* Calculate clock divider */
  div = (pclk + freq - 1) / freq;  /* Round up */

  /* Check if divider is valid */
  if (div == 0)
    {
      div = 1;
    }

  /* Ensure divider is even */
  if (div & 1)
    {
      div++;
    }

  if (div > 256)
    {
      return -EINVAL;  /* Frequency too low */
    }

  /* Disable SD clock first */
  clk_ctrl = sdhc_getreg32(priv, RA8P_SDHC_SD_CLOCK_CONTROL_OFFSET);
  clk_ctrl &= ~RA8P_SDHC_SD_CLOCK_CONTROL_SDCLKEN;
  sdhc_putreg32(priv, RA8P_SDHC_SD_CLOCK_CONTROL_OFFSET, clk_ctrl);

  /* Configure clock divider */
  clk_ctrl &= ~RA8P_SDHC_SD_CLOCK_CONTROL_SDCLKFSEL_MASK;
  clk_ctrl |= ((div / 2) << RA8P_SDHC_SD_CLOCK_CONTROL_SDCLKFSEL_SHIFT);

  /* Enable internal clock */
  clk_ctrl |= RA8P_SDHC_SD_CLOCK_CONTROL_INTCLKEN;

  sdhc_putreg32(priv, RA8P_SDHC_SD_CLOCK_CONTROL_OFFSET, clk_ctrl);

  /* Wait for internal clock stable */
  volatile int timeout = 10000;
  while (!(sdhc_getreg32(priv, RA8P_SDHC_SD_CLOCK_CONTROL_OFFSET) & 
           RA8P_SDHC_SD_CLOCK_CONTROL_INTCLKSTABLE) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  /* Enable SD clock */
  clk_ctrl |= RA8P_SDHC_SD_CLOCK_CONTROL_SDCLKEN;
  sdhc_putreg32(priv, RA8P_SDHC_SD_CLOCK_CONTROL_OFFSET, clk_ctrl);

  priv->current_frequency = freq;

  sdhcinfo("SDHC%d clock set to %u Hz (divider=%u)\n", priv->channel, freq, div);
  return OK;
}

/****************************************************************************
 * Name: sdhc_send_cmd
 ****************************************************************************/

static int sdhc_send_cmd(struct ra8p_sdhc_priv_s *priv, uint32_t cmd, uint32_t arg, uint8_t resp_type)
{
  uint32_t cmd_reg;
  uint32_t mode;
  int ret;

  /* Wait for no command inhibit */
  ret = sdhc_wait_ready(priv);
  if (ret != OK)
    {
      return ret;
    }

  /* Clear interrupt status */
  sdhc_putreg32(priv, RA8P_SDHC_SD_INT_STATUS_OFFSET, 0xFFFFFFFF);

  /* Set command argument */
  sdhc_putreg32(priv, RA8P_SDHC_SD_ARGUMENT_OFFSET, arg);

  /* Configure command register */
  cmd_reg = (cmd & 0x3F) << 8;  /* Command index */

  /* Set response type */
  switch (resp_type)
    {
      case 0:  /* No response */
        cmd_reg |= RA8P_SDHC_SD_COMMAND_RSP_TYPE_NONE;
        break;
      case 1:  /* R1, R5, R6, R7 - 48-bit with CRC and busy check */
        cmd_reg |= RA8P_SDHC_SD_COMMAND_RSP_TYPE_48_CHK;
        cmd_reg |= RA8P_SDHC_SD_COMMAND_CRC_CHECK;
        cmd_reg |= RA8P_SDHC_SD_COMMAND_INDEX_CHECK;
        break;
      case 2:  /* R2 - 136-bit with CRC */
        cmd_reg |= RA8P_SDHC_SD_COMMAND_RSP_TYPE_136;
        cmd_reg |= RA8P_SDHC_SD_COMMAND_CRC_CHECK;
        break;
      case 3:  /* R3 - 48-bit without CRC */
        cmd_reg |= RA8P_SDHC_SD_COMMAND_RSP_TYPE_48;
        break;
      default:
        return -EINVAL;
    }

  /* Check for data present */
  if (cmd == 17 || cmd == 18 || cmd == 24 || cmd == 25)  /* Read/Write commands */
    {
      cmd_reg |= RA8P_SDHC_SD_COMMAND_DATA_PRESENT;
    }

  sdhc_putreg16(priv, RA8P_SDHC_SD_COMMAND_OFFSET, cmd_reg);

  /* Wait for command complete */
  ret = sdhc_wait_cmd_complete(priv);
  if (ret != OK)
    {
      return ret;
    }

  /* Read response if expected */
  if (resp_type != 0)
    {
      priv->response[0] = sdhc_getreg32(priv, RA8P_SDHC_SD_RESPONSE0_OFFSET);
      if (resp_type == 2)  /* R2 - 136-bit response */
        {
          priv->response[1] = sdhc_getreg32(priv, RA8P_SDHC_SD_RESPONSE1_OFFSET);
          priv->response[2] = sdhc_getreg32(priv, RA8P_SDHC_SD_RESPONSE2_OFFSET);
          priv->response[3] = sdhc_getreg32(priv, RA8P_SDHC_SD_RESPONSE3_OFFSET);
        }
    }

  return OK;
}

/****************************************************************************
 * Name: sdhc_identify_card
 ****************************************************************************/

static int sdhc_identify_card(struct ra8p_sdhc_priv_s *priv)
{
  uint32_t arg;
  int ret;
  int tries;

  /* Send CMD0 - GO_IDLE_STATE */
  ret = sdhc_send_cmd(priv, 0, 0, 0);  /* CMD0: GO_IDLE_STATE */
  if (ret != OK)
    {
      return ret;
    }

  /* Send CMD8 - SEND_IF_COND */
  arg = (0x01 << 8) | 0xAA;  /* Supply voltage: 2.7-3.6V, Check pattern: 0xAA */
  ret = sdhc_send_cmd(priv, 8, arg, 1);  /* CMD8: SEND_IF_COND */
  if (ret != OK)
    {
      /* CMD8 may fail for older SD cards - continue anyway */
      sdhcwarn("CMD8 failed, may be older SD card\n");
    }

  /* Send CMD55 + ACMD41 to identify card */
  tries = 0;
  do
    {
      /* Send CMD55 - APP_CMD */
      ret = sdhc_send_cmd(priv, 55, 0, 1);  /* CMD55: APP_CMD */
      if (ret != OK)
        {
          /* CMD55 might fail if card is not SDIO */
          ret = sdhc_send_cmd(priv, 1, 0, 1);  /* Send CMD1 instead */
          if (ret != OK)
            {
              return ret;
            }
        }
      else
        {
          /* Send ACMD41 - SD_SEND_OP_COND */
          arg = 0x40ff8000;  /* HCS bit set, voltage window 2.7-3.6V */
          ret = sdhc_send_cmd(priv, 41, arg, 1);  /* ACMD41 */
          if (ret != OK)
            {
              return ret;
            }
        }

      tries++;
      if (tries > 100)
        {
          return -ETIMEDOUT;
        }

      up_udelay(1000);  /* 1ms delay */
    }
  while (!(priv->response[0] & 0x80000000) && tries < 100);  /* Wait for OCR busy bit */

  if (!(priv->response[0] & 0x80000000))
    {
      return -ETIMEDOUT;
    }

  /* Check if card is high capacity */
  priv->ocr = priv->response[0];
  priv->high_capacity = (priv->response[0] & 0x40000000) != 0;  /* CCS bit */

  /* Get CID */
  ret = sdhc_send_cmd(priv, 2, 0, 2);  /* CMD2: ALL_SEND_CID */
  if (ret != OK)
    {
      return ret;
    }

  /* Store CID in local array */
  memcpy(priv->cid, priv->response, 16);

  /* Send CMD3 - SET_RELATIVE_ADDR */
  ret = sdhc_send_cmd(priv, 3, 0, 1);  /* CMD3: SEND_RELATIVE_ADDR */
  if (ret != OK)
    {
      return ret;
    }

  /* Store RCA from response */
  priv->rca = (priv->response[0] >> 16) & 0xFFFF;

  /* Get CSD */
  ret = sdhc_send_cmd(priv, 9, (priv->rca << 16), 2);  /* CMD9: SEND_CSD */
  if (ret != OK)
    {
      return ret;
    }

  /* Store CSD in local array */
  memcpy(priv->csd, priv->response, 16);

  /* Select card */
  ret = sdhc_send_cmd(priv, 7, (priv->rca << 16), 1);  /* CMD7: SELECT_CARD */
  if (ret != OK)
    {
      return ret;
    }

  /* Check for SDIO card */
  ret = sdhc_send_cmd(priv, 52, 0x00000000, 1);  /* CMD52 might reveal SDIO */
  if (ret == OK)
    {
      priv->sdio = true;
    }
  else
    {
      priv->sdio = false;
    }

  sdhcinfo("SD card identified: HC=%s, OCR=0x%08x, RCA=0x%04x, CSD=0x%08x...\n",
           priv->high_capacity ? "yes" : "no", priv->ocr, priv->rca, 
           *(uint32_t*)priv->csd);
  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_sdhc_initialize
 *
 * Description:
 *   Initialize the SDHC controller based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   max_freq - Maximum SD clock frequency in Hz
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_initialize(uint8_t channel, uint32_t max_freq)
{
  struct ra8p_sdhc_priv_s *priv;
  uint32_t regval;
  int ret;

  if (channel >= 2)
    {
      return -EINVAL;
    }

  priv = &g_sdhc[channel];
  priv->channel = channel;
  priv->base = (channel == 0) ? RA8P_SDHC0_BASE : RA8P_SDHC1_BASE;
  priv->max_frequency = max_freq ? max_freq : 25000000;  /* Default to 25MHz */
  priv->current_frequency = 400000;  /* Start with identification frequency */
  priv->initialized = false;
  priv->enabled = false;
  priv->card_inserted = false;
  priv->high_capacity = false;
  priv->sdio = false;
  priv->mmc = false;
  priv->card_capacity = 0;
  priv->bus_width = 1;  /* Default to 1-bit mode */
  priv->rca = 0;
  priv->ocr = 0;
  priv->write_protected = false;

  /* Initialize semaphore */
  nxsem_init(&priv->wait_sem, 0, 1);

  /* Reset SDHC */
  sdhc_putreg8(priv, RA8P_SDHC_SD_SOFTWARE_RESET_OFFSET, 
               RA8P_SDHC_SD_SOFTWARE_RESET_ALL);

  /* Wait for reset complete */
  volatile int timeout = 10000;
  while (sdhc_getreg8(priv, RA8P_SDHC_SD_SOFTWARE_RESET_OFFSET) & 
         RA8P_SDHC_SD_SOFTWARE_RESET_ALL)
    {
      if (timeout-- <= 0)
        {
          return -ETIMEDOUT;
        }
      up_udelay(1);
    }

  /* Configure clock for identification phase (400 kHz max) */
  ret = sdhc_set_frequency(priv, 400000);  /* Identification phase clock */
  if (ret != OK)
    {
      return ret;
    }

  /* Set timeout control */
  sdhc_putreg8(priv, RA8P_SDHC_SD_TIMEOUT_CONTROL_OFFSET, 0x0E);  /* Appropriate timeout */

  /* Configure power control */
  regval = sdhc_getreg8(priv, RA8P_SDHC_SD_POWER_CONTROL_OFFSET);
  regval |= RA8P_SDHC_SD_POWER_CONTROL_PWON;  /* Power on */
  regval |= RA8P_SDHC_SD_POWER_CONTROL_VOLT_330;  /* 3.3V */
  sdhc_putreg8(priv, RA8P_SDHC_SD_POWER_CONTROL_OFFSET, regval);

  /* Wait for power stable */
  timeout = 10000;
  while (!(sdhc_getreg8(priv, RA8P_SDHC_SD_POWER_CONTROL_OFFSET) & 
           RA8P_SDHC_SD_POWER_CONTROL_PWON) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  /* Configure host control */
  regval = sdhc_getreg8(priv, RA8P_SDHC_SD_HOST_CONTROL_OFFSET);
  regval &= ~RA8P_SDHC_SD_HOST_CONTROL_DTW_4BIT;  /* Clear 4-bit mode initially */
  regval &= ~RA8P_SDHC_SD_HOST_CONTROL_DTW_8BIT;  /* Clear 8-bit mode initially */
  regval &= ~RA8P_SDHC_SD_HOST_CONTROL_HISPD;    /* Clear high speed mode */
  sdhc_putreg8(priv, RA8P_SDHC_SD_HOST_CONTROL_OFFSET, regval);

  /* Set block size */
  sdhc_putreg16(priv, RA8P_SDHC_SD_BLOCK_SIZE_OFFSET, RA8P_SDHC_BLOCK_SIZE);

  /* Enable interrupts */
  regval = RA8P_SDHC_SD_INT_ENABLE_CMD_COMPLETE |
           RA8P_SDHC_SD_INT_ENABLE_TRANSFER_COMPLETE |
           RA8P_SDHC_SD_INT_ENABLE_CARD_INSERTION |
           RA8P_SDHC_SD_INT_ENABLE_CARD_REMOVAL;
  sdhc_putreg32(priv, RA8P_SDHC_SD_INT_ENABLE_OFFSET, regval);

  /* Clear all interrupt status */
  sdhc_putreg32(priv, RA8P_SDHC_SD_INT_STATUS_OFFSET, 0xFFFFFFFF);

  /* Identify and configure card */
  ret = sdhc_identify_card(priv);
  if (ret != OK)
    {
      return ret;
    }

  /* Set block length to 512 bytes */
  ret = sdhc_send_cmd(priv, 16, RA8P_SDHC_BLOCK_SIZE, 1);  /* CMD16: SET_BLOCKLEN */
  if (ret != OK)
    {
      return ret;
    }

  /* Set higher clock frequency for data transfer */
  ret = sdhc_set_frequency(priv, priv->max_frequency);
  if (ret != OK)
    {
      return ret;
    }

  /* Configure transfer timing parameters */
  regval = sdhc_getreg8(priv, RA8P_SDHC_SD_TIMEOUT_CONTROL_OFFSET);
  regval &= ~RA8P_SDHC_SD_TIMEOUT_CONTROL_DTC_VAL_MASK;
  regval |= (0x0E << RA8P_SDHC_SD_TIMEOUT_CONTROL_DTC_VAL_SHIFT); /* Set timeout */
  sdhc_putreg8(priv, RA8P_SDHC_SD_TIMEOUT_CONTROL_OFFSET, regval);

  /* Calculate card capacity from CSD */
  if (priv->csd[0] & 0xC0)  /* CSD version 2.0 (SDHC/SDXC) */
    {
      /* Calculate capacity from CSD v2.0 */
      uint32_t c_size = ((uint32_t)(priv->csd[7] & 0x3F) << 16) |
                        ((uint32_t)priv->csd[8] << 8) |
                         (uint32_t)priv->csd[9];
      priv->card_capacity = (c_size + 1) * 512 * 1024;  /* Capacity in bytes */
    }
  else  /* CSD version 1.0 (Standard SD) */
    {
      /* Calculate capacity from CSD v1.0 */
      uint32_t c_size = ((uint32_t)(priv->csd[6] & 0x03) << 10) |
                        ((uint32_t)priv->csd[7] << 2) |
                        ((uint32_t)priv->csd[8] >> 6);
      uint32_t c_size_mult = ((priv->csd[9] & 0x03) << 1) |
                             ((priv->csd[10] >> 7) & 0x01);
      uint32_t read_bl_len = (priv->csd[5] >> 6) & 0x0F;
      uint32_t mult = 1 << (c_size_mult + 2);
      uint32_t blocknr = (c_size + 1) * mult;
      uint32_t block_len = 1 << read_bl_len;
      priv->card_capacity = blocknr * block_len;
    }

  priv->initialized = true;
  priv->enabled = true;

  sdhcinfo("SDHC%d initialized at 0x%08x, max_freq=%u Hz, capacity=%u MB\n", 
           channel, priv->base, max_freq, (unsigned int)(priv->card_capacity / (1024 * 1024)));
  return OK;
}

/****************************************************************************
 * Name: ra8p_sdhc_card_inserted
 *
 * Description:
 *   Check if SD card is inserted based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   true if card is inserted, false otherwise
 *
 ****************************************************************************/

bool ra8p_sdhc_card_inserted(uint8_t channel)
{
  struct ra8p_sdhc_priv_s *priv;

  if (channel >= 2 || !g_sdhc[channel].initialized)
    {
      return false;
    }

  priv = &g_sdhc[channel];

  /* Check card detection status in present state register */
  uint32_t present_state = sdhc_getreg32(priv, RA8P_SDHC_SD_PRESENT_STATE_OFFSET);
  priv->card_inserted = (present_state & RA8P_SDHC_SD_PRESENT_STATE_CARD_INSERTED) != 0;

  return priv->card_inserted;
}

/****************************************************************************
 * Name: ra8p_sdhc_read_block
 *
 * Description:
 *   Read a block from SD card based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   block_num - Block number to read
 *   buffer - Buffer to read data into
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_read_block(uint8_t channel, uint32_t block_num, uint8_t *buffer)
{
  struct ra8p_sdhc_priv_s *priv;
  uint32_t cmd_arg;
  uint32_t regval;
  int ret;
  int i;
  uint32_t *buf32 = (uint32_t *)buffer;

  if (channel >= 2 || !g_sdhc[channel].enabled || buffer == NULL)
    {
      return -EINVAL;
    }

  priv = &g_sdhc[channel];

  /* Wait for no data inhibit */
  volatile int timeout = 100000;
  while ((sdhc_getreg32(priv, RA8P_SDHC_SD_PRESENT_STATE_OFFSET) & 
          RA8P_SDHC_SD_PRESENT_STATE_CMD_INHIBIT_DAT) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  /* Prepare command argument */
  if (priv->high_capacity)
    {
      /* SDHC uses native addressing (block address) */
      cmd_arg = block_num;
    }
  else
    {
      /* Standard SD uses byte address */
      cmd_arg = block_num * RA8P_SDHC_BLOCK_SIZE;
    }

  /* Configure transfer mode for single block read */
  regval = RA8P_SDHC_SD_TRANSFER_MODE_DMAEN |
           RA8P_SDHC_SD_TRANSFER_MODE_BCEN |
           RA8P_SDHC_SD_TRANSFER_MODE_RDEN;  /* Read enable */
  sdhc_putreg16(priv, RA8P_SDHC_SD_TRANSFER_MODE_OFFSET, regval);

  /* Set block size and count */
  sdhc_putreg16(priv, RA8P_SDHC_SD_BLOCK_SIZE_OFFSET, RA8P_SDHC_BLOCK_SIZE);
  sdhc_putreg16(priv, RA8P_SDHC_SD_BLOCK_COUNT_OFFSET, 1);

  /* Send READ_SINGLE_BLOCK command (CMD17) */
  ret = sdhc_send_cmd(priv, 17, cmd_arg, 1);  /* CMD17: READ_SINGLE_BLOCK */
  if (ret != OK)
    {
      return ret;
    }

  /* Wait for buffer read ready interrupt */
  timeout = RA8P_SDHC_TIMEOUT_MS * 1000;
  while (!(sdhc_getreg32(priv, RA8P_SDHC_SD_INT_STATUS_OFFSET) & 
           RA8P_SDHC_SD_INT_STATUS_BUF_READ_READY) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  /* Clear buffer read ready interrupt */
  sdhc_putreg32(priv, RA8P_SDHC_SD_INT_STATUS_OFFSET, RA8P_SDHC_SD_INT_STATUS_BUF_READ_READY);

  /* Read data from FIFO in 32-bit words */
  for (i = 0; i < (RA8P_SDHC_BLOCK_SIZE / 4); i++)
    {
      buf32[i] = sdhc_getreg32(priv, RA8P_SDHC_SD_BUFFER_OFFSET);
    }

  /* Wait for transfer complete */
  ret = sdhc_wait_transfer_complete(priv);
  if (ret != OK)
    {
      return ret;
    }

  sdhcinfo("SDHC%d read block %u successfully\n", channel, block_num);
  return OK;
}

/****************************************************************************
 * Name: ra8p_sdhc_write_block
 *
 * Description:
 *   Write a block to SD card based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   block_num - Block number to write
 *   buffer - Buffer containing data to write
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_write_block(uint8_t channel, uint32_t block_num, const uint8_t *buffer)
{
  struct ra8p_sdhc_priv_s *priv;
  uint32_t cmd_arg;
  uint32_t regval;
  int ret;
  int i;
  const uint32_t *buf32 = (const uint32_t *)buffer;

  if (channel >= 2 || !g_sdhc[channel].enabled || buffer == NULL)
    {
      return -EINVAL;
    }

  priv = &g_sdhc[channel];

  /* Wait for no data inhibit */
  volatile int timeout = 100000;
  while ((sdhc_getreg32(priv, RA8P_SDHC_SD_PRESENT_STATE_OFFSET) & 
          RA8P_SDHC_SD_PRESENT_STATE_CMD_INHIBIT_DAT) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  /* Prepare command argument */
  if (priv->high_capacity)
    {
      /* SDHC uses native addressing (block address) */
      cmd_arg = block_num;
    }
  else
    {
      /* Standard SD uses byte address */
      cmd_arg = block_num * RA8P_SDHC_BLOCK_SIZE;
    }

  /* Configure transfer mode for single block write */
  regval = RA8P_SDHC_SD_TRANSFER_MODE_DMAEN |
           RA8P_SDHC_SD_TRANSFER_MODE_BCEN |
           RA8P_SDHC_SD_TRANSFER_MODE_WRITEN;  /* Write enable */
  sdhc_putreg16(priv, RA8P_SDHC_SD_TRANSFER_MODE_OFFSET, regval);

  /* Set block size and count */
  sdhc_putreg16(priv, RA8P_SDHC_SD_BLOCK_SIZE_OFFSET, RA8P_SDHC_BLOCK_SIZE);
  sdhc_putreg16(priv, RA8P_SDHC_SD_BLOCK_COUNT_OFFSET, 1);

  /* Wait for buffer write ready */
  timeout = RA8P_SDHC_TIMEOUT_MS * 1000;
  while (!(sdhc_getreg32(priv, RA8P_SDHC_SD_INT_STATUS_OFFSET) & 
           RA8P_SDHC_SD_INT_STATUS_BUF_WRITE_READY) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  /* Write data to FIFO in 32-bit words */
  for (i = 0; i < (RA8P_SDHC_BLOCK_SIZE / 4); i++)
    {
      sdhc_putreg32(priv, RA8P_SDHC_SD_BUFFER_OFFSET, buf32[i]);
    }

  /* Clear buffer write ready interrupt */
  sdhc_putreg32(priv, RA8P_SDHC_SD_INT_STATUS_OFFSET, RA8P_SDHC_SD_INT_STATUS_BUF_WRITE_READY);

  /* Send WRITE_SINGLE_BLOCK command (CMD24) */
  ret = sdhc_send_cmd(priv, 24, cmd_arg, 1);  /* CMD24: WRITE_SINGLE_BLOCK */
  if (ret != OK)
    {
      return ret;
    }

  /* Wait for transfer complete */
  ret = sdhc_wait_transfer_complete(priv);
  if (ret != OK)
    {
      return ret;
    }

  sdhcinfo("SDHC%d wrote block %u successfully\n", channel, block_num);
  return OK;
}

/****************************************************************************
 * Name: ra8p_sdhc_set_bus_width
 *
 * Description:
 *   Set SDHC bus width (1, 4, or 8 bit) based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   width   - Bus width (1, 4, or 8)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_set_bus_width(uint8_t channel, uint8_t width)
{
  struct ra8p_sdhc_priv_s *priv;
  uint8_t host_ctrl;

  if (channel >= 2 || (width != 1 && width != 4 && width != 8))
    {
      return -EINVAL;
    }

  priv = &g_sdhc[channel];

  host_ctrl = sdhc_getreg8(priv, RA8P_SDHC_SD_HOST_CONTROL_OFFSET);

  switch (width)
    {
      case 1:
        host_ctrl &= ~(RA8P_SDHC_SD_HOST_CONTROL_DTW_4BIT | 
                       RA8P_SDHC_SD_HOST_CONTROL_DTW_8BIT);
        break;

      case 4:
        host_ctrl |= RA8P_SDHC_SD_HOST_CONTROL_DTW_4BIT;
        host_ctrl &= ~RA8P_SDHC_SD_HOST_CONTROL_DTW_8BIT;
        break;

      case 8:
        host_ctrl &= ~RA8P_SDHC_SD_HOST_CONTROL_DTW_4BIT;
        host_ctrl |= RA8P_SDHC_SD_HOST_CONTROL_DTW_8BIT;
        break;

      default:
        return -EINVAL;
    }

  sdhc_putreg8(priv, RA8P_SDHC_SD_HOST_CONTROL_OFFSET, host_ctrl);

  priv->bus_width = width;

  sdhcinfo("SDHC%d bus width set to %d bits\n", channel, width);
  return OK;
}

/****************************************************************************
 * Name: ra8p_sdhc_set_frequency
 *
 * Description:
 *   Set SDHC clock frequency based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   frequency - Clock frequency in Hz
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_set_frequency(uint8_t channel, uint32_t frequency)
{
  struct ra8p_sdhc_priv_s *priv;

  if (channel >= 2 || !g_sdhc[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_sdhc[channel];

  return sdhc_set_frequency(priv, frequency);
}

/****************************************************************************
 * Name: ra8p_sdhc_get_card_info
 *
 * Description:
 *   Get SD card information based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   info - Pointer to card info structure to fill
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_get_card_info(uint8_t channel, struct ra8p_sdhc_cardinfo_s *info)
{
  struct ra8p_sdhc_priv_s *priv;

  if (channel >= 2 || info == NULL || !g_sdhc[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_sdhc[channel];

  if (!priv->card_inserted)
    {
      return -ENODEV;
    }

  /* Fill card info */
  info->capacity = priv->card_capacity;
  memcpy(info->ocr, &priv->ocr, sizeof(uint32_t));
  info->rca = priv->rca;
  memcpy(info->csd, priv->csd, 16);
  memcpy(info->cid, priv->cid, 16);
  info->high_capacity = priv->high_capacity;
  info->sdio = priv->sdio;
  info->mmc = priv->mmc;
  info->write_protected = priv->write_protected;
  info->bus_width = priv->bus_width;
  info->max_frequency = priv->max_frequency;

  return OK;
}

/****************************************************************************
 * Name: ra8p_sdhc_power_up
 *
 * Description:
 *   Power up SDHC controller based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_power_up(uint8_t channel)
{
  struct ra8p_sdhc_priv_s *priv;
  uint8_t regval;

  if (channel >= 2 || !g_sdhc[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_sdhc[channel];

  /* Enable power */
  regval = sdhc_getreg8(priv, RA8P_SDHC_SD_POWER_CONTROL_OFFSET);
  regval |= RA8P_SDHC_SD_POWER_CONTROL_PWON;
  regval |= RA8P_SDHC_SD_POWER_CONTROL_VOLT_330;  /* 3.3V */
  sdhc_putreg8(priv, RA8P_SDHC_SD_POWER_CONTROL_OFFSET, regval);

  /* Wait for power stable */
  volatile int timeout = 10000;
  while (!(sdhc_getreg8(priv, RA8P_SDHC_SD_POWER_CONTROL_OFFSET) & 
           RA8P_SDHC_SD_POWER_CONTROL_PWON) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  /* Enable clock */
  regval = sdhc_getreg8(priv, RA8P_SDHC_SD_CLOCK_CONTROL_OFFSET);
  regval |= RA8P_SDHC_SD_CLOCK_CONTROL_INTCLKEN | RA8P_SDHC_SD_CLOCK_CONTROL_SDCLKEN;
  sdhc_putreg8(priv, RA8P_SDHC_SD_CLOCK_CONTROL_OFFSET, regval);

  /* Wait for clock stable */
  timeout = 10000;
  while (!(sdhc_getreg8(priv, RA8P_SDHC_SD_CLOCK_CONTROL_OFFSET) & 
           RA8P_SDHC_SD_CLOCK_CONTROL_INTCLKSTABLE) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  priv->enabled = true;

  sdhcinfo("SDHC%d powered up\n", channel);
  return OK;
}

/****************************************************************************
 * Name: ra8p_sdhc_power_down
 *
 * Description:
 *   Power down SDHC controller based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_power_down(uint8_t channel)
{
  struct ra8p_sdhc_priv_s *priv;
  uint8_t regval;

  if (channel >= 2 || !g_sdhc[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_sdhc[channel];

  /* Disable clock */
  regval = sdhc_getreg8(priv, RA8P_SDHC_SD_CLOCK_CONTROL_OFFSET);
  regval &= ~(RA8P_SDHC_SD_CLOCK_CONTROL_SDCLKEN | RA8P_SDHC_SD_CLOCK_CONTROL_INTCLKEN);
  sdhc_putreg8(priv, RA8P_SDHC_SD_CLOCK_CONTROL_OFFSET, regval);

  /* Disable power */
  regval = sdhc_getreg8(priv, RA8P_SDHC_SD_POWER_CONTROL_OFFSET);
  regval &= ~RA8P_SDHC_SD_POWER_CONTROL_PWON;
  sdhc_putreg8(priv, RA8P_SDHC_SD_POWER_CONTROL_OFFSET, regval);

  priv->enabled = false;

  sdhcinfo("SDHC%d powered down\n", channel);
  return OK;
}

/****************************************************************************
 * Name: ra8p_sdhc_read_blocks
 *
 * Description:
 *   Read multiple blocks from SD card based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   start_block - Starting block number
 *   buffer - Buffer to read data into
 *   nblocks - Number of blocks to read
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_read_blocks(uint8_t channel, uint32_t start_block, 
                         uint8_t *buffer, uint32_t nblocks)
{
  struct ra8p_sdhc_priv_s *priv;
  uint32_t cmd_arg;
  uint32_t regval;
  int ret;
  int i, j;
  uint32_t *buf32 = (uint32_t *)buffer;

  if (channel >= 2 || !g_sdhc[channel].enabled || buffer == NULL || nblocks == 0)
    {
      return -EINVAL;
    }

  priv = &g_sdhc[channel];

  /* Wait for no data inhibit */
  volatile int timeout = 100000;
  while ((sdhc_getreg32(priv, RA8P_SDHC_SD_PRESENT_STATE_OFFSET) & 
          RA8P_SDHC_SD_PRESENT_STATE_CMD_INHIBIT_DAT) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  /* Prepare command argument */
  if (priv->high_capacity)
    {
      /* SDHC uses native addressing (block address) */
      cmd_arg = start_block;
    }
  else
    {
      /* Standard SD uses byte address */
      cmd_arg = start_block * RA8P_SDHC_BLOCK_SIZE;
    }

  /* Configure transfer mode for multiple block read */
  regval = RA8P_SDHC_SD_TRANSFER_MODE_DMAEN |
           RA8P_SDHC_SD_TRANSFER_MODE_BCEN |
           RA8P_SDHC_SD_TRANSFER_MODE_MULBLK |
           RA8P_SDHC_SD_TRANSFER_MODE_RDEN;  /* Read enable */
  sdhc_putreg16(priv, RA8P_SDHC_SD_TRANSFER_MODE_OFFSET, regval);

  /* Set block size and count */
  sdhc_putreg16(priv, RA8P_SDHC_SD_BLOCK_SIZE_OFFSET, RA8P_SDHC_BLOCK_SIZE);
  sdhc_putreg16(priv, RA8P_SDHC_SD_BLOCK_COUNT_OFFSET, nblocks);

  /* Send READ_MULTIPLE_BLOCK command (CMD18) */
  ret = sdhc_send_cmd(priv, 18, cmd_arg, 1);  /* CMD18: READ_MULTIPLE_BLOCK */
  if (ret != OK)
    {
      return ret;
    }

  /* Read all blocks */
  for (i = 0; i < nblocks; i++)
    {
      /* Wait for buffer read ready interrupt */
      timeout = RA8P_SDHC_TIMEOUT_MS * 1000;
      while (!(sdhc_getreg32(priv, RA8P_SDHC_SD_INT_STATUS_OFFSET) & 
               RA8P_SDHC_SD_INT_STATUS_BUF_READ_READY) && timeout--)
        {
          up_udelay(1);
        }

      if (timeout <= 0)
        {
          return -ETIMEDOUT;
        }

      /* Clear buffer read ready interrupt */
      sdhc_putreg32(priv, RA8P_SDHC_SD_INT_STATUS_OFFSET, RA8P_SDHC_SD_INT_STATUS_BUF_READ_READY);

      /* Read data from FIFO in 32-bit words */
      for (j = 0; j < (RA8P_SDHC_BLOCK_SIZE / 4); j++)
        {
          buf32[(i * (RA8P_SDHC_BLOCK_SIZE / 4)) + j] = sdhc_getreg32(priv, RA8P_SDHC_SD_BUFFER_OFFSET);
        }
    }

  /* Wait for transfer complete */
  ret = sdhc_wait_transfer_complete(priv);
  if (ret != OK)
    {
      return ret;
    }

  sdhcinfo("SDHC%d read %u blocks starting from %u successfully\n", 
           channel, nblocks, start_block);
  return OK;
}

/****************************************************************************
 * Name: ra8p_sdhc_write_blocks
 *
 * Description:
 *   Write multiple blocks to SD card based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   start_block - Starting block number
 *   buffer - Buffer containing data to write
 *   nblocks - Number of blocks to write
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_write_blocks(uint8_t channel, uint32_t start_block, 
                          const uint8_t *buffer, uint32_t nblocks)
{
  struct ra8p_sdhc_priv_s *priv;
  uint32_t cmd_arg;
  uint32_t regval;
  int ret;
  int i, j;
  const uint32_t *buf32 = (const uint32_t *)buffer;

  if (channel >= 2 || !g_sdhc[channel].enabled || buffer == NULL || nblocks == 0)
    {
      return -EINVAL;
    }

  priv = &g_sdhc[channel];

  /* Wait for no data inhibit */
  volatile int timeout = 100000;
  while ((sdhc_getreg32(priv, RA8P_SDHC_SD_PRESENT_STATE_OFFSET) & 
          RA8P_SDHC_SD_PRESENT_STATE_CMD_INHIBIT_DAT) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  /* Prepare command argument */
  if (priv->high_capacity)
    {
      /* SDHC uses native addressing (block address) */
      cmd_arg = start_block;
    }
  else
    {
      /* Standard SD uses byte address */
      cmd_arg = start_block * RA8P_SDHC_BLOCK_SIZE;
    }

  /* Configure transfer mode for multiple block write */
  regval = RA8P_SDHC_SD_TRANSFER_MODE_DMAEN |
           RA8P_SDHC_SD_TRANSFER_MODE_BCEN |
           RA8P_SDHC_SD_TRANSFER_MODE_MULBLK |
           RA8P_SDHC_SD_TRANSFER_MODE_WRITEN;  /* Write enable */
  sdhc_putreg16(priv, RA8P_SDHC_SD_TRANSFER_MODE_OFFSET, regval);

  /* Set block size and count */
  sdhc_putreg16(priv, RA8P_SDHC_SD_BLOCK_SIZE_OFFSET, RA8P_SDHC_BLOCK_SIZE);
  sdhc_putreg16(priv, RA8P_SDHC_SD_BLOCK_COUNT_OFFSET, nblocks);

  /* Send WRITE_MULTIPLE_BLOCK command (CMD25) */
  ret = sdhc_send_cmd(priv, 25, cmd_arg, 1);  /* CMD25: WRITE_MULTIPLE_BLOCK */
  if (ret != OK)
    {
      return ret;
    }

  /* Write all blocks */
  for (i = 0; i < nblocks; i++)
    {
      /* Wait for buffer write ready */
      timeout = RA8P_SDHC_TIMEOUT_MS * 1000;
      while (!(sdhc_getreg32(priv, RA8P_SDHC_SD_INT_STATUS_OFFSET) & 
               RA8P_SDHC_SD_INT_STATUS_BUF_WRITE_READY) && timeout--)
        {
          up_udelay(1);
        }

      if (timeout <= 0)
        {
          return -ETIMEDOUT;
        }

      /* Write data to FIFO in 32-bit words */
      for (j = 0; j < (RA8P_SDHC_BLOCK_SIZE / 4); j++)
        {
          sdhc_putreg32(priv, RA8P_SDHC_SD_BUFFER_OFFSET, 
                       buf32[(i * (RA8P_SDHC_BLOCK_SIZE / 4)) + j]);
        }

      /* Clear buffer write ready interrupt */
      sdhc_putreg32(priv, RA8P_SDHC_SD_INT_STATUS_OFFSET, RA8P_SDHC_SD_INT_STATUS_BUF_WRITE_READY);
    }

  /* Wait for transfer complete */
  ret = sdhc_wait_transfer_complete(priv);
  if (ret != OK)
    {
      return ret;
    }

  sdhcinfo("SDHC%d wrote %u blocks starting from %u successfully\n", 
           channel, nblocks, start_block);
  return OK;
}

/****************************************************************************
 * Name: ra8p_sdhc_is_initialized
 *
 * Description:
 *   Check if SDHC is initialized based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   true if initialized, false otherwise
 *
 ****************************************************************************/

bool ra8p_sdhc_is_initialized(uint8_t channel)
{
  if (channel >= 2)
    {
      return false;
    }

  return g_sdhc[channel].initialized;
}

/****************************************************************************
 * Name: ra8p_sdhc_is_enabled
 *
 * Description:
 *   Check if SDHC channel is enabled based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   true if enabled, false otherwise
 *
 ****************************************************************************/

bool ra8p_sdhc_is_enabled(uint8_t channel)
{
  if (channel >= 2)
    {
      return false;
    }

  return g_sdhc[channel].enabled;
}

/****************************************************************************
 * Name: ra8p_sdhc_get_status
 *
 * Description:
 *   Get SDHC status flags based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   Status flags
 *
 ****************************************************************************/

uint32_t ra8p_sdhc_get_status(uint8_t channel)
{
  struct ra8p_sdhc_priv_s *priv;

  if (channel >= 2 || !g_sdhc[channel].initialized)
    {
      return 0;
    }

  priv = &g_sdhc[channel];
  return sdhc_getreg32(priv, RA8P_SDHC_SD_PRESENT_STATE_OFFSET);
}

/****************************************************************************
 * Name: ra8p_sdhc_reset
 *
 * Description:
 *   Reset SDHC controller based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_reset(uint8_t channel)
{
  struct ra8p_sdhc_priv_s *priv;

  if (channel >= 2 || !g_sdhc[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_sdhc[channel];

  /* Perform software reset */
  sdhc_putreg8(priv, RA8P_SDHC_SD_SOFTWARE_RESET_OFFSET, 
               RA8P_SDHC_SD_SOFTWARE_RESET_ALL);

  /* Wait for reset to complete */
  volatile int timeout = 10000;
  while (sdhc_getreg8(priv, RA8P_SDHC_SD_SOFTWARE_RESET_OFFSET) & 
         RA8P_SDHC_SD_SOFTWARE_RESET_ALL)
    {
      if (timeout-- <= 0)
        {
          return -ETIMEDOUT;
        }
      up_udelay(1);
    }

  /* Clear interrupt status */
  sdhc_putreg32(priv, RA8P_SDHC_SD_INT_STATUS_OFFSET, 0xFFFFFFFF);

  /* Re-enable controller with current settings */
  return ra8p_sdhc_initialize(channel, priv->max_frequency);
}

/****************************************************************************
 * Name: ra8p_sdhc_set_dma
 *
 * Description:
 *   Enable/disable DMA for SDHC transfers based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   enable - true to enable DMA, false to use PIO
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_set_dma(uint8_t channel, bool enable)
{
  struct ra8p_sdhc_priv_s *priv;
  uint16_t regval;

  if (channel >= 2 || !g_sdhc[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_sdhc[channel];

  /* Read current transfer mode register */
  regval = sdhc_getreg16(priv, RA8P_SDHC_SD_TRANSFER_MODE_OFFSET);

  if (enable)
    {
      regval |= RA8P_SDHC_SD_TRANSFER_MODE_DMAEN;  /* Enable DMA */
    }
  else
    {
      regval &= ~RA8P_SDHC_SD_TRANSFER_MODE_DMAEN; /* Disable DMA */
    }

  sdhc_putreg16(priv, RA8P_SDHC_SD_TRANSFER_MODE_OFFSET, regval);

  return OK;
}

/****************************************************************************
 * Name: ra8p_sdhc_enable_high_speed
 *
 * Description:
 *   Enable/disable high-speed mode for SDHC based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   enable - true to enable high-speed mode, false for normal speed
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_enable_high_speed(uint8_t channel, bool enable)
{
  struct ra8p_sdhc_priv_s *priv;
  uint8_t host_ctrl;

  if (channel >= 2 || !g_sdhc[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_sdhc[channel];

  host_ctrl = sdhc_getreg8(priv, RA8P_SDHC_SD_HOST_CONTROL_OFFSET);

  if (enable)
    {
      host_ctrl |= RA8P_SDHC_SD_HOST_CONTROL_HISPD;  /* Enable high-speed */
    }
  else
    {
      host_ctrl &= ~RA8P_SDHC_SD_HOST_CONTROL_HISPD; /* Disable high-speed */
    }

  sdhc_putreg8(priv, RA8P_SDHC_SD_HOST_CONTROL_OFFSET, host_ctrl);

  return OK;
}

/****************************************************************************
 * Name: ra8p_sdhc_is_card_write_protected
 *
 * Description:
 *   Check if SD card is write protected based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   true if write protected, false otherwise
 *
 ****************************************************************************/

bool ra8p_sdhc_is_card_write_protected(uint8_t channel)
{
  struct ra8p_sdhc_priv_s *priv;

  if (channel >= 2 || !g_sdhc[channel].initialized)
    {
      return false;
    }

  priv = &g_sdhc[channel];

  /* Check write protect switch in present state register */
  uint32_t present_state = sdhc_getreg32(priv, RA8P_SDHC_SD_PRESENT_STATE_OFFSET);
  return (present_state & RA8P_SDHC_SD_PRESENT_STATE_WR_PROTECT_SW) != 0;
}

/****************************************************************************
 * Name: ra8p_sdhc_enable_interrupts
 *
 * Description:
 *   Enable/disable SDHC interrupts based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   enable - true to enable interrupts, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_enable_interrupts(uint8_t channel, bool enable)
{
  struct ra8p_sdhc_priv_s *priv;
  uint32_t regval;

  if (channel >= 2 || !g_sdhc[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_sdhc[channel];

  /* Read current interrupt enable register */
  regval = sdhc_getreg32(priv, RA8P_SDHC_SD_INT_ENABLE_OFFSET);

  if (enable)
    {
      /* Enable important interrupts */
      regval |= RA8P_SDHC_SD_INT_ENABLE_CMD_COMPLETE |
               RA8P_SDHC_SD_INT_ENABLE_TRANSFER_COMPLETE |
               RA8P_SDHC_SD_INT_ENABLE_CARD_INSERTION |
               RA8P_SDHC_SD_INT_ENABLE_CARD_REMOVAL;
    }
  else
    {
      /* Disable all interrupts */
      regval = 0;
    }

  sdhc_putreg32(priv, RA8P_SDHC_SD_INT_ENABLE_OFFSET, regval);

  return OK;
}

/****************************************************************************
 * Name: ra8p_sdhc_get_error_flags
 *
 * Description:
 *   Get SDHC error flags based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   Error flags
 *
 ****************************************************************************/

uint32_t ra8p_sdhc_get_error_flags(uint8_t channel)
{
  struct ra8p_sdhc_priv_s *priv;

  if (channel >= 2 || !g_sdhc[channel].initialized)
    {
      return 0;
    }

  priv = &g_sdhc[channel];

  /* Get error flags from interrupt status */
  uint32_t status = sdhc_getreg32(priv, RA8P_SDHC_SD_INT_STATUS_OFFSET);
  uint32_t errors = 0;

  /* Map status flags to error flags */
  if (status & RA8P_SDHC_SD_INT_STATUS_CMD_ERROR)
    {
      errors |= 1;  /* Command error */
    }
  if (status & RA8P_SDHC_SD_INT_STATUS_DATA_TIMEOUT_ERROR)
    {
      errors |= 2;  /* Data timeout error */
    }
  if (status & RA8P_SDHC_SD_INT_STATUS_DATA_CRC_ERROR)
    {
      errors |= 4;  /* Data CRC error */
    }
  if (status & RA8P_SDHC_SD_INT_STATUS_DATA_END_ERROR)
    {
      errors |= 8;  /* Data end error */
    }
  if (status & RA8P_SDHC_SD_INT_STATUS_CURRENT_LIMIT_ERROR)
    {
      errors |= 16; /* Current limit error */
    }

  return errors;
}

/****************************************************************************
 * Name: ra8p_sdhc_clear_errors
 *
 * Description:
 *   Clear SDHC error flags based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_clear_errors(uint8_t channel)
{
  struct ra8p_sdhc_priv_s *priv;

  if (channel >= 2 || !g_sdhc[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_sdhc[channel];

  /* Clear all error flags in interrupt status */
  uint32_t status = sdhc_getreg32(priv, RA8P_SDHC_SD_INT_STATUS_OFFSET);
  status &= (RA8P_SDHC_SD_INT_STATUS_CMD_ERROR |
             RA8P_SDHC_SD_INT_STATUS_DATA_TIMEOUT_ERROR |
             RA8P_SDHC_SD_INT_STATUS_DATA_CRC_ERROR |
             RA8P_SDHC_SD_INT_STATUS_DATA_END_ERROR |
             RA8P_SDHC_SD_INT_STATUS_CURRENT_LIMIT_ERROR);
  sdhc_putreg32(priv, RA8P_SDHC_SD_INT_STATUS_OFFSET, status);

  return OK;
}

#endif /* CONFIG_RA8P_SDHC */