/****************************************************************************
 * arch/arm/src/ra8p/ra8p_iic.c
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
#include "hardware/ra8p_iic.h"

#ifdef CONFIG_RA8P_IIC

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define RA8P_IIC0_BASE                 RA8P_IIC0_BASE
#define RA8P_IIC1_BASE                 RA8P_IIC1_BASE
#define RA8P_IIC2_BASE                 RA8P_IIC2_BASE

/* IIC timeout in milliseconds */
#define RA8P_IIC_TIMEOUT_MS             1000
#define RA8P_IIC_CMD_TIMEOUT_MS         100

/* IIC bus frequency values */
#define RA8P_IIC_STANDARD_FREQ          100000   /* 100 kHz */
#define RA8P_IIC_FAST_FREQ              400000   /* 400 kHz */
#define RA8P_IIC_FAST_PLUS_FREQ         1000000  /* 1 MHz */
#define RA8P_IIC_ULTRA_FAST_FREQ        5000000  /* 5 MHz */

/* IIC bit rate register values */
#define RA8P_IIC_BRH_DEFAULT            0x14    /* Default BRH value */
#define RA8P_IIC_BRL_DEFAULT            0x14    /* Default BRL value */

/* Maximum number of IIC channels */
#define RA8P_IIC_MAX_CHANNELS           3

/* IIC register base addresses */
#define RA8P_IIC_REGS_BASE(n)           (RA8P_IIC0_BASE + (n * 0x100))

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* RA8P1 IIC driver state structure */

struct ra8p_iic_priv_s
{
  uint32_t base;                              /* Base address of IIC registers */
  uint8_t channel;                            /* IIC channel (0-2) */
  bool initialized;                           /* Initialization flag */
  bool enabled;                               /* Enable flag */
  uint32_t frequency;                         /* Current bus frequency */
  bool master_mode;                           /* Master mode flag */
  bool high_speed_mode;                       /* High speed mode flag */
  bool fast_mode_plus;                        /* Fast mode plus enabled */
  uint8_t addr_mode;                          /* Addressing mode (7/10-bit) */
  bool bus_busy;                             /* Bus busy flag */
  bool transaction_active;                    /* Transaction in progress */
  sem_t wait_sem;                             /* Wait semaphore for transactions */
  uint32_t error_flags;                       /* Current error flags */
  uint8_t noise_filter;                       /* Noise filter setting */
  bool pullup_enabled;                        /* Pull-up resistors enabled */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int iic_wait_ready(struct ra8p_iic_priv_s *priv);
static int iic_calc_timing(const struct ra8p_iic_priv_s *priv, uint32_t frequency,
                          uint8_t *brh, uint8_t *brl);
static void iic_putreg32(struct ra8p_iic_priv_s *priv, uint32_t offset, uint32_t value);
static uint32_t iic_getreg32(struct ra8p_iic_priv_s *priv, uint32_t offset);
static void iic_putreg16(struct ra8p_iic_priv_s *priv, uint32_t offset, uint16_t value);
static uint16_t iic_getreg16(struct ra8p_iic_priv_s *priv, uint32_t offset);
static void iic_putreg8(struct ra8p_iic_priv_s *priv, uint32_t offset, uint8_t value);
static uint8_t iic_getreg8(struct ra8p_iic_priv_s *priv, uint32_t offset);
static int iic_set_frequency(struct ra8p_iic_priv_s *priv, uint32_t freq);
static int iic_send_start(struct ra8p_iic_priv_s *priv);
static int iic_send_stop(struct ra8p_iic_priv_s *priv);
static int iic_send_addr(struct ra8p_iic_priv_s *priv, uint8_t addr, bool read);
static int iic_write_byte(struct ra8p_iic_priv_s *priv, uint8_t data);
static int iic_read_byte(struct ra8p_iic_priv_s *priv, uint8_t *data, bool nack_last);
static int iic_wait_transaction_complete(struct ra8p_iic_priv_s *priv);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_iic_priv_s g_iic[3];  /* Three IIC channels (0-2) */

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: iic_putreg32
 ****************************************************************************/

static inline void iic_putreg32(struct ra8p_iic_priv_s *priv, uint32_t offset, uint32_t value)
{
  putreg32(value, priv->base + offset);
}

/****************************************************************************
 * Name: iic_getreg32
 ****************************************************************************/

static inline uint32_t iic_getreg32(struct ra8p_iic_priv_s *priv, uint32_t offset)
{
  return getreg32(priv->base + offset);
}

/****************************************************************************
 * Name: iic_putreg16
 ****************************************************************************/

static inline void iic_putreg16(struct ra8p_iic_priv_s *priv, uint32_t offset, uint16_t value)
{
  putreg16(value, priv->base + offset);
}

/****************************************************************************
 * Name: iic_getreg16
 ****************************************************************************/

static inline uint16_t iic_getreg16(struct ra8p_iic_priv_s *priv, uint32_t offset)
{
  return getreg16(priv->base + offset);
}

/****************************************************************************
 * Name: iic_putreg8
 ****************************************************************************/

static inline void iic_putreg8(struct ra8p_iic_priv_s *priv, uint32_t offset, uint8_t value)
{
  putreg8(value, priv->base + offset);
}

/****************************************************************************
 * Name: iic_getreg8
 ****************************************************************************/

static inline uint8_t iic_getreg8(struct ra8p_iic_priv_s *priv, uint32_t offset)
{
  return getreg8(priv->base + offset);
}

/****************************************************************************
 * Name: iic_wait_ready
 ****************************************************************************/

static int iic_wait_ready(struct ra8p_iic_priv_s *priv)
{
  volatile int timeout = RA8P_IIC_TIMEOUT_MS * 1000;  /* microseconds */

  /* Wait for no command inhibit */
  while ((iic_getreg8(priv, RA8P_IIC_ICCR2_OFFSET) & RA8P_IIC_ICCR2_CMDINHIBIT) && timeout--)
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
 * Name: iic_wait_transaction_complete
 ****************************************************************************/

static int iic_wait_transaction_complete(struct ra8p_iic_priv_s *priv)
{
  volatile int timeout = RA8P_IIC_TIMEOUT_MS * 1000;  /* microseconds */
  uint8_t status;

  /* Wait for transaction to complete */
  while (timeout-- > 0)
    {
      status = iic_getreg8(priv, RA8P_IIC_ICSR2_OFFSET);
      
      /* Check for transaction completion flags */
      if (status & RA8P_IIC_ICSR2_TEND)  /* Transfer end */
        {
          /* Clear transaction end flag */
          iic_putreg8(priv, RA8P_IIC_ICCR_OFFSET, status);
          return OK;
        }
      else if (status & RA8P_IIC_ICSR2_NACKF)  /* NACK received */
        {
          /* Clear NACK flag */
          iic_putreg8(priv, RA8P_IIC_ICCR_OFFSET, RA8P_IIC_ICCR_WREL);
          return -ENXIO;
        }
      else if (status & RA8P_IIC_ICSR2_ALD)  /* Arbitration lost */
        {
          /* Clear arbitration lost flag */
          iic_putreg8(priv, RA8P_IIC_ICCR_OFFSET, RA8P_IIC_ICCR_WREL);
          return -EAGAIN;
        }
      
      up_udelay(1);
    }

  return -ETIMEDOUT;
}

/****************************************************************************
 * Name: iic_calc_timing
 ****************************************************************************/

static int iic_calc_timing(const struct ra8p_iic_priv_s *priv, uint32_t frequency,
                          uint8_t *brh, uint8_t *brl)
{
  uint32_t pclk = 62500000;  /* Use appropriate PCLK */
  uint32_t divisor;
  uint32_t br_total;
  uint8_t brh_val, brl_val;

  if (frequency == 0)
    {
      return -EINVAL;
    }

  /* Calculate divisor */
  divisor = pclk / frequency / 2;  /* Two phases (low + high) per cycle */

  /* BRH and BRL should add up to approximately divisor */
  brh_val = divisor / 2;  /* Half for high */
  brl_val = divisor - brh_val;  /* Remainder for low */

  /* Limit to valid range (1-255) */
  if (brh_val > 255 || brh_val < 1)
    {
      if (frequency > RA8P_IIC_STANDARD_FREQ)
        {
          /* High speed, try different ratio */
          brh_val = divisor / 3;
          brl_val = divisor - brh_val;
        }
      else
        {
          /* Low speed, try different ratio */
          brh_val = divisor * 2 / 3;
          brl_val = divisor - brh_val;
        }
    }

  if (brh_val > 255 || brh_val < 1 || brl_val > 255 || brl_val < 1)
    {
      return -EINVAL;  /* Frequency out of range */
    }

  *brh = brh_val;
  *brl = brl_val;

  return OK;
}

/****************************************************************************
 * Name: iic_set_frequency
 ****************************************************************************/

static int iic_set_frequency(struct ra8p_iic_priv_s *priv, uint32_t freq)
{
  uint8_t brh, brl;
  int ret;

  /* Calculate timing values */
  ret = iic_calc_timing(priv, freq, &brh, &brl);
  if (ret != OK)
    {
      return ret;
    }

  /* Disable IIC before changing frequency */
  uint8_t iccr1 = iic_getreg8(priv, RA8P_IIC_ICCR1_OFFSET);
  iccr1 &= ~RA8P_IIC_ICCR1_ICE;  /* Disable IIC */
  iic_putreg8(priv, RA8P_IIC_ICCR1_OFFSET, iccr1);

  /* Set BRH and BRL registers */
  iic_putreg8(priv, RA8P_IIC_ICBRH_OFFSET, brh);
  iic_putreg8(priv, RA8P_IIC_ICBRL_OFFSET, brl);

  /* Re-enable IIC */
  iccr1 |= RA8P_IIC_ICCR1_ICE;  /* Enable IIC */
  iic_putreg8(priv, RA8P_IIC_ICCR1_OFFSET, iccr1);

  priv->frequency = freq;

  iicinfo("IIC%d frequency set to %u Hz (BRH=%u, BRL=%u)\n", 
          priv->channel, freq, brh, brl);
  return OK;
}

/****************************************************************************
 * Name: iic_send_start
 ****************************************************************************/

static int iic_send_start(struct ra8p_iic_priv_s *priv)
{
  uint8_t iccr2;
  int ret;

  /* Wait for ready */
  ret = iic_wait_ready(priv);
  if (ret != OK)
    {
      return ret;
    }

  /* Send START condition */
  iccr2 = iic_getreg8(priv, RA8P_IIC_ICCR2_OFFSET);
  iccr2 |= RA8P_IIC_ICCR2_START;  /* Set START bit */
  iic_putreg8(priv, RA8P_IIC_ICCR2_OFFSET, iccr2);

  /* Wait for START to be sent */
  volatile int timeout = RA8P_IIC_CMD_TIMEOUT_MS * 1000;
  while ((iic_getreg8(priv, RA8P_IIC_ICCR2_OFFSET) & RA8P_IIC_ICCR2_START) && timeout--)
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
 * Name: iic_send_stop
 ****************************************************************************/

static int iic_send_stop(struct ra8p_iic_priv_s *priv)
{
  uint8_t iccr2;
  int ret;

  /* Wait for ready */
  ret = iic_wait_ready(priv);
  if (ret != OK)
    {
      return ret;
    }

  /* Send STOP condition */
  iccr2 = iic_getreg8(priv, RA8P_IIC_ICCR2_OFFSET);
  iccr2 |= RA8P_IIC_ICCR2_STOP;  /* Set STOP bit */
  iic_putreg8(priv, RA8P_IIC_ICCR2_OFFSET, iccr2);

  /* Wait for STOP to be sent */
  volatile int timeout = RA8P_IIC_CMD_TIMEOUT_MS * 1000;
  while ((iic_getreg8(priv, RA8P_IIC_ICCR2_OFFSET) & RA8P_IIC_ICCR2_STOP) && timeout--)
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
 * Name: iic_send_addr
 ****************************************************************************/

static int iic_send_addr(struct ra8p_iic_priv_s *priv, uint8_t addr, bool read)
{
  uint8_t addr_byte;
  int ret;

  /* Wait for ready */
  ret = iic_wait_ready(priv);
  if (ret != OK)
    {
      return ret;
    }

  /* Prepare address byte */
  if (priv->addr_mode == 10)  /* 10-bit addressing */
    {
      /* For 10-bit addressing, we need to send two address bytes */
      addr_byte = 0xF0 | ((addr >> 7) & 0x06) | (read ? 1 : 0);  /* First byte */
      
      /* Send first address byte */
      iic_putreg8(priv, RA8P_IIC_ICDRT_OFFSET, addr_byte);
      ret = iic_wait_transaction_complete(priv);
      if (ret != OK)
        {
          return ret;
        }

      /* Send second address byte */
      addr_byte = addr & 0xFF;
      iic_putreg8(priv, RA8P_IIC_ICDRT_OFFSET, addr_byte);
      ret = iic_wait_transaction_complete(priv);
      if (ret != OK)
        {
          return ret;
        }

      /* For read after write in 10-bit mode, send repeated start */
      if (read)
        {
          ret = iic_send_start(priv);
          if (ret != OK)
            {
              return ret;
            }

          /* Send address again with read bit */
          addr_byte = (addr << 1) | 1;
          iic_putreg8(priv, RA8P_IIC_ICDRT_OFFSET, addr_byte);
        }
    }
  else  /* 7-bit addressing */
    {
      /* 7-bit addressing: addr shifted left, LSB is R/W bit */
      addr_byte = (addr << 1) | (read ? 1 : 0);
      iic_putreg8(priv, RA8P_IIC_ICDRT_OFFSET, addr_byte);
    }

  /* Wait for address transfer to complete */
  ret = iic_wait_transaction_complete(priv);
  if (ret != OK)
    {
      return ret;
    }

  return OK;
}

/****************************************************************************
 * Name: iic_write_byte
 ****************************************************************************/

static int iic_write_byte(struct ra8p_iic_priv_s *priv, uint8_t data)
{
  int ret;

  /* Wait for ready */
  ret = iic_wait_ready(priv);
  if (ret != OK)
    {
      return ret;
    }

  /* Write data byte */
  iic_putreg8(priv, RA8P_IIC_ICDRT_OFFSET, data);

  /* Wait for transfer to complete */
  ret = iic_wait_transaction_complete(priv);
  if (ret != OK)
    {
      return ret;
    }

  return OK;
}

/****************************************************************************
 * Name: iic_read_byte
 ****************************************************************************/

static int iic_read_byte(struct ra8p_iic_priv_s *priv, uint8_t *data, bool nack_last)
{
  int ret;

  if (data == NULL)
    {
      return -EINVAL;
    }

  /* Wait for ready */
  ret = iic_wait_ready(priv);
  if (ret != OK)
    {
      return ret;
    }

  /* Set ACK/NACK control register */
  uint8_t icmr3 = iic_getreg8(priv, RA8P_IIC_ICMR3_OFFSET);
  if (nack_last)
    {
      icmr3 |= RA8P_IIC_ICMR3_ACKBT;  /* Send NACK for last byte */
    }
  else
    {
      icmr3 &= ~RA8P_IIC_ICMR3_ACKBT; /* Send ACK for more bytes */
    }
  iic_putreg8(priv, RA8P_IIC_ICMR3_OFFSET, icmr3);

  /* Read data byte */
  *data = iic_getreg8(priv, RA8P_IIC_ICDRR_OFFSET);

  /* Clear receive flag */
  uint8_t iccr = iic_getreg8(priv, RA8P_IIC_ICCR_OFFSET);
  iccr |= RA8P_IIC_ICCR_WREL;
  iic_putreg8(priv, RA8P_IIC_ICCR_OFFSET, iccr);

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_iic_initialize
 *
 * Description:
 *   Initialize the IIC controller based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *   frequency - I2C bus frequency in Hz
 *   addr_mode - Addressing mode (7 or 10 bit)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_initialize(uint8_t channel, uint32_t frequency, uint8_t addr_mode)
{
  struct ra8p_iic_priv_s *priv;
  uint8_t regval;
  int ret;

  if (channel >= 3)
    {
      return -EINVAL;
    }

  priv = &g_iic[channel];
  priv->channel = channel;
  priv->base = (channel == 0) ? RA8P_IIC0_BASE : 
               (channel == 1) ? RA8P_IIC1_BASE : RA8P_IIC2_BASE;
  priv->frequency = frequency ? frequency : RA8P_IIC_STANDARD_FREQ;
  priv->addr_mode = (addr_mode == 10) ? 10 : 7;  /* Only 7-bit and 10-bit supported */
  priv->master_mode = true;  /* Master mode by default */
  priv->initialized = false;
  priv->enabled = false;
  priv->high_speed_mode = false;
  priv->fast_mode_plus = false;
  priv->bus_busy = false;
  priv->transaction_active = false;
  priv->error_flags = 0;
  priv->noise_filter = 0;
  priv->pullup_enabled = true;

  /* Initialize semaphore */
  nxsem_init(&priv->wait_sem, 0, 1);

  /* Reset IIC module */
  regval = iic_getreg8(priv, RA8P_IIC_ICCR1_OFFSET);
  regval |= RA8P_IIC_ICCR1_IICRST;  /* Set reset */
  iic_putreg8(priv, RA8P_IIC_ICCR1_OFFSET, regval);

  /* Wait for reset complete */
  volatile int timeout = 10000;
  while ((iic_getreg8(priv, RA8P_IIC_ICCR1_OFFSET) & RA8P_IIC_ICCR1_IICRST) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  /* Clear reset */
  regval = iic_getreg8(priv, RA8P_IIC_ICCR1_OFFSET);
  regval &= ~RA8P_IIC_ICCR1_IICRST;
  iic_putreg8(priv, RA8P_IIC_ICCR1_OFFSET, regval);

  /* Configure ICMR1 register */
  regval = iic_getreg8(priv, RA8P_IIC_ICMR1_OFFSET);
  regval |= RA8P_IIC_ICMR1_CMC;  /* Enable I2C communication */
  regval &= ~RA8P_IIC_ICMR1_BFS; /* Set to normal I2C mode */
  if (priv->addr_mode == 10)
    {
      regval |= RA8P_IIC_ICMR1_SAEN;  /* Enable 10-bit slave addressing */
    }
  else
    {
      regval &= ~RA8P_IIC_ICMR1_SAEN; /* Disable 10-bit slave addressing */
    }
  iic_putreg8(priv, RA8P_IIC_ICMR1_OFFSET, regval);

  /* Configure ICMR2 register for timing */
  regval = iic_getreg8(priv, RA8P_IIC_ICMR2_OFFSET);
  regval |= (0x00 << RA8P_IIC_ICMR2_DLCS_SHIFT);  /* Set clock select */
  regval &= ~RA8P_IIC_ICMR2_TMOS;  /* Clear timeout monitor select */
  regval &= ~RA8P_IIC_ICMR2_NF_MASK;  /* Clear noise filter */
  regval |= (priv->noise_filter << RA8P_IIC_ICMR2_NF_SHIFT);  /* Set noise filter */
  iic_putreg8(priv, RA8P_IIC_ICMR2_OFFSET, regval);

  /* Configure ICFER register for functions */
  regval = iic_getreg8(priv, RA8P_IIC_ICFER_OFFSET);
  regval |= RA8P_IIC_ICFER_TMOE;   /* Enable timeout function */
  regval |= RA8P_IIC_ICFER_MALE;   /* Enable master arbitration lost */
  regval &= ~RA8P_IIC_ICFER_NACKE; /* Disable NACK enable for now */
  regval &= ~RA8P_IIC_ICFER_SALE;  /* Clear slave arbitration lost */
  regval |= RA8P_IIC_ICFER_NFE;    /* Enable noise filter */
  regval |= RA8P_IIC_ICFER_SCLE;   /* Enable SCL synchronous circuit */
  regval &= ~RA8P_IIC_ICFER_FMPE;  /* Clear FM+ enable for now */
  regval |= RA8P_IIC_ICFER_DTE;    /* Enable digital filter */
  iic_putreg8(priv, RA8P_IIC_ICFER_OFFSET, regval);

  /* Set bus timing based on frequency */
  ret = iic_set_frequency(priv, priv->frequency);
  if (ret != OK)
    {
      return ret;
    }

  /* Enable IIC */
  regval = iic_getreg8(priv, RA8P_IIC_ICCR1_OFFSET);
  regval |= RA8P_IIC_ICCR1_ICE;  /* Enable I2C */
  iic_putreg8(priv, RA8P_IIC_ICCR1_OFFSET, regval);

  /* Wait for I2C to be enabled */
  timeout = 10000;
  while (!(iic_getreg8(priv, RA8P_IIC_ICCR1_OFFSET) & RA8P_IIC_ICCR1_ICE) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  /* Configure master mode */
  if (priv->master_mode)
    {
      regval = iic_getreg8(priv, RA8P_IIC_ICCR2_OFFSET);
      regval |= RA8P_IIC_ICCR2_MST;  /* Set to master mode */
      iic_putreg8(priv, RA8P_IIC_ICCR2_OFFSET, regval);
    }

  priv->initialized = true;
  priv->enabled = true;

  iicinfo("IIC%d initialized at 0x%08x, freq=%u Hz, addr_mode=%u-bit\n", 
          channel, priv->base, frequency, addr_mode);
  return OK;
}

/****************************************************************************
 * Name: ra8p_iic_start
 *
 * Description:
 *   Start IIC controller operation based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_start(uint8_t channel)
{
  struct ra8p_iic_priv_s *priv;
  uint8_t regval;

  if (channel >= 3 || !g_iic[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_iic[channel];

  /* Enable IIC */
  regval = iic_getreg8(priv, RA8P_IIC_ICCR1_OFFSET);
  regval |= RA8P_IIC_ICCR1_ICE;  /* Enable I2C */
  iic_putreg8(priv, RA8P_IIC_ICCR1_OFFSET, regval);

  /* Wait for I2C to be enabled */
  volatile int timeout = 10000;
  while (!(iic_getreg8(priv, RA8P_IIC_ICCR1_OFFSET) & RA8P_IIC_ICCR1_ICE) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  /* Configure master mode if needed */
  regval = iic_getreg8(priv, RA8P_IIC_ICCR2_OFFSET);
  regval |= RA8P_IIC_ICCR2_MST;  /* Set to master mode */
  iic_putreg8(priv, RA8P_IIC_ICCR2_OFFSET, regval);

  priv->enabled = true;

  iicinfo("IIC%d started\n", channel);
  return OK;
}

/****************************************************************************
 * Name: ra8p_iic_stop
 *
 * Description:
 *   Stop IIC controller operation based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_stop(uint8_t channel)
{
  struct ra8p_iic_priv_s *priv;
  uint8_t regval;

  if (channel >= 3 || !g_iic[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_iic[channel];

  /* Send STOP condition if bus is active */
  if (priv->transaction_active)
    {
      ra8p_iic_send_stop(channel);
      priv->transaction_active = false;
    }

  /* Disable IIC */
  regval = iic_getreg8(priv, RA8P_IIC_ICCR1_OFFSET);
  regval &= ~RA8P_IIC_ICCR1_ICE;  /* Disable I2C */
  iic_putreg8(priv, RA8P_IIC_ICCR1_OFFSET, regval);

  priv->enabled = false;

  iicinfo("IIC%d stopped\n", channel);
  return OK;
}

/****************************************************************************
 * Name: ra8p_iic_transfer
 *
 * Description:
 *   Perform an I2C transfer based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *   msgs - Array of I2C messages
 *   count - Number of messages
 *
 * Returned Value:
 *   Number of messages processed on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_transfer(uint8_t channel, struct i2c_msg_s *msgs, int count)
{
  struct ra8p_iic_priv_s *priv;
  int ret;
  int i, j;
  struct i2c_msg_s *msg;

  if (channel >= 3 || !g_iic[channel].enabled || msgs == NULL || count <= 0)
    {
      return -EINVAL;
    }

  priv = &g_iic[channel];
  priv->transaction_active = true;

  for (i = 0; i < count; i++)
    {
      msg = &msgs[i];

      if (msg->flags & I2C_M_NOSTART)
        {
          /* Skip START condition */
        }
      else
        {
          /* Send START condition */
          ret = iic_send_start(priv);
          if (ret != OK)
            {
              priv->transaction_active = false;
              return ret;
            }
        }

      /* Send address */
      ret = iic_send_addr(priv, msg->addr, (msg->flags & I2C_M_READ) != 0);
      if (ret != OK)
        {
          iic_send_stop(priv);  /* Ensure STOP is sent */
          priv->transaction_active = false;
          return ret;
        }

      /* Process data */
      if (msg->flags & I2C_M_READ)
        {
          /* Read data */
          for (j = 0; j < msg->length; j++)
            {
              bool is_last = (j == msg->length - 1);
              ret = iic_read_byte(priv, &msg->buffer[j], is_last);
              if (ret != OK)
                {
                  iic_send_stop(priv);
                  priv->transaction_active = false;
                  return ret;
                }
            }
        }
      else
        {
          /* Write data */
          for (j = 0; j < msg->length; j++)
            {
              ret = iic_write_byte(priv, msg->buffer[j]);
              if (ret != OK)
                {
                  iic_send_stop(priv);
                  priv->transaction_active = false;
                  return ret;
                }
            }
        }

      /* Send STOP if required */
      if (!(msg->flags & I2C_M_NOSTOP))
        {
          if (i == count - 1)  /* Last message */
            {
              ret = iic_send_stop(priv);
              if (ret != OK)
                {
                  priv->transaction_active = false;
                  return ret;
                }
            }
        }
    }

  priv->transaction_active = false;

  iicinfo("IIC%d completed %d messages\n", channel, count);
  return count;
}

/****************************************************************************
 * Name: ra8p_iic_set_frequency
 *
 * Description:
 *   Set IIC bus frequency based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *   frequency - Clock frequency in Hz
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_set_frequency(uint8_t channel, uint32_t frequency)
{
  struct ra8p_iic_priv_s *priv;

  if (channel >= 3 || !g_iic[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_iic[channel];

  return iic_set_frequency(priv, frequency);
}

/****************************************************************************
 * Name: ra8p_iic_set_addr_mode
 *
 * Description:
 *   Set IIC addressing mode (7-bit or 10-bit) based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *   mode - Addressing mode (7 or 10)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_set_addr_mode(uint8_t channel, uint8_t mode)
{
  struct ra8p_iic_priv_s *priv;
  uint8_t regval;

  if (channel >= 3 || !g_iic[channel].initialized || (mode != 7 && mode != 10))
    {
      return -EINVAL;
    }

  priv = &g_iic[channel];

  /* Modify ICMR1 register for addressing mode */
  regval = iic_getreg8(priv, RA8P_IIC_ICMR1_OFFSET);
  if (mode == 10)
    {
      regval |= RA8P_IIC_ICMR1_SAEN;  /* Enable 10-bit addressing */
    }
  else
    {
      regval &= ~RA8P_IIC_ICMR1_SAEN; /* Disable 10-bit addressing */
    }
  iic_putreg8(priv, RA8P_IIC_ICMR1_OFFSET, regval);

  priv->addr_mode = mode;

  iicinfo("IIC%d address mode set to %u-bit\n", channel, mode);
  return OK;
}

/****************************************************************************
 * Name: ra8p_iic_is_initialized
 *
 * Description:
 *   Check if IIC controller is initialized based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *
 * Returned Value:
 *   true if initialized, false otherwise
 *
 ****************************************************************************/

bool ra8p_iic_is_initialized(uint8_t channel)
{
  if (channel >= 3)
    {
      return false;
    }

  return g_iic[channel].initialized;
}

/****************************************************************************
 * Name: ra8p_iic_is_enabled
 *
 * Description:
 *   Check if IIC controller is enabled based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *
 * Returned Value:
 *   true if enabled, false otherwise
 *
 ****************************************************************************/

bool ra8p_iic_is_enabled(uint8_t channel)
{
  if (channel >= 3)
    {
      return false;
    }

  return g_iic[channel].enabled;
}

/****************************************************************************
 * Name: ra8p_iic_get_status
 *
 * Description:
 *   Get IIC status flags based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *
 * Returned Value:
 *   Status flags
 *
 ****************************************************************************/

uint8_t ra8p_iic_get_status(uint8_t channel)
{
  struct ra8p_iic_priv_s *priv;

  if (channel >= 3 || !g_iic[channel].initialized)
    {
      return 0;
    }

  priv = &g_iic[channel];
  return iic_getreg8(priv, RA8P_IIC_ICSR2_OFFSET);
}

/****************************************************************************
 * Name: ra8p_iic_enable_interrupts
 *
 * Description:
 *   Enable IIC interrupts based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_enable_interrupts(uint8_t channel, bool enable)
{
  struct ra8p_iic_priv_s *priv;
  uint8_t regval;

  if (channel >= 3 || !g_iic[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_iic[channel];

  /* Read current interrupt enable register */
  regval = iic_getreg8(priv, RA8P_IIC_ICIER_OFFSET);

  if (enable)
    {
      regval |= RA8P_IIC_ICIER_TMOIE |    /* Timeout interrupt */
               RA8P_IIC_ICIER_ALIE |     /* Arbitration lost interrupt */
               RA8P_ICIER_NAKIE |        /* NACK interrupt */
               RA8P_IIC_ICIER_STIE |     /* Start interrupt */
               RA8P_IIC_ICIER_SPIE;      /* Stop interrupt */
    }
  else
    {
      regval &= ~(RA8P_IIC_ICIER_TMOIE | RA8P_IIC_ICIER_ALIE | 
                  RA8P_IIC_ICIER_NAKIE | RA8P_IIC_ICIER_STIE | 
                  RA8P_IIC_ICIER_SPIE);
    }

  iic_putreg8(priv, RA8P_IIC_ICIER_OFFSET, regval);

  iicinfo("IIC%d interrupts %s\n", channel, enable ? "enabled" : "disabled");
  return OK;
}

/****************************************************************************
 * Name: ra8p_iic_set_dma
 *
 * Description:
 *   Enable/disable DMA for IIC transfers based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *   enable - true to enable DMA, false to use PIO
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_set_dma(uint8_t channel, bool enable)
{
  struct ra8p_iic_priv_s *priv;
  uint8_t regval;

  if (channel >= 3 || !g_iic[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_iic[channel];

  /* Read current function enable register */
  regval = iic_getreg8(priv, RA8P_IIC_ICFER_OFFSET);

  if (enable)
    {
      regval |= RA8P_IIC_ICFER_DME;  /* Enable DMA mode */
    }
  else
    {
      regval &= ~RA8P_IIC_ICFER_DME; /* Disable DMA mode */
    }

  iic_putreg8(priv, RA8P_IIC_ICFER_OFFSET, regval);

  iicinfo("IIC%d DMA mode %s\n", channel, enable ? "enabled" : "disabled");
  return OK;
}

/****************************************************************************
 * Name: ra8p_iic_reset
 *
 * Description:
 *   Reset IIC controller based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_reset(uint8_t channel)
{
  struct ra8p_iic_priv_s *priv;
  uint8_t regval;

  if (channel >= 3 || !g_iic[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_iic[channel];

  /* Disable IIC first */
  regval = iic_getreg8(priv, RA8P_IIC_ICCR1_OFFSET);
  regval &= ~RA8P_IIC_ICCR1_ICE;
  iic_putreg8(priv, RA8P_IIC_ICCR1_OFFSET, regval);

  /* Reset IIC */
  regval |= RA8P_IIC_ICCR1_IICRST;
  iic_putreg8(priv, RA8P_IIC_ICCR1_OFFSET, regval);

  /* Wait for reset */
  volatile int timeout = 10000;
  while ((iic_getreg8(priv, RA8P_IIC_ICCR1_OFFSET) & RA8P_IIC_ICCR1_IICRST) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  /* Clear reset */
  regval = iic_getreg8(priv, RA8P_IIC_ICCR1_OFFSET);
  regval &= ~RA8P_IIC_ICCR1_IICRST;
  iic_putreg8(priv, RA8P_IIC_ICCR1_OFFSET, regval);

  /* Re-initialize */
  return ra8p_iic_initialize(channel, priv->frequency, priv->addr_mode);
}

/****************************************************************************
 * Name: ra8p_iic_clear_status
 *
 * Description:
 *   Clear IIC status flags based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *   flags - Flags to clear
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_clear_status(uint8_t channel, uint8_t flags)
{
  struct ra8p_iic_priv_s *priv;

  if (channel >= 3 || !g_iic[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_iic[channel];

  /* Clear specified flags by writing to status register */
  iic_putreg8(priv, RA8P_IIC_ICSR2_OFFSET, flags);

  return OK;
}

/****************************************************************************
 * Name: ra8p_iic_get_error_flags
 *
 * Description:
 *   Get IIC error flags based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *
 * Returned Value:
 *   Error flags
 *
 ****************************************************************************/

uint8_t ra8p_iic_get_error_flags(uint8_t channel)
{
  struct ra8p_iic_priv_s *priv;

  if (channel >= 3 || !g_iic[channel].initialized)
    {
      return 0;
    }

  priv = &g_iic[channel];
  return iic_getreg8(priv, RA8P_IIC_ICER_OFFSET);  /* Error register */
}

/****************************************************************************
 * Name: ra8p_iic_clear_errors
 *
 * Description:
 *   Clear IIC error flags based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_clear_errors(uint8_t channel)
{
  struct ra8p_iic_priv_s *priv;

  if (channel >= 3 || !g_iic[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_iic[channel];

  /* Clear error flags by writing to error register */
  iic_putreg8(priv, RA8P_IIC_ICER_OFFSET, 0xFF);

  /* Update internal error flags */
  priv->error_flags = 0;

  return OK;
}

/****************************************************************************
 * Name: ra8p_iic_is_error
 *
 * Description:
 *   Check if IIC has error flags set based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *
 * Returned Value:
 *   true if error occurred, false otherwise
 *
 ****************************************************************************/

bool ra8p_iic_is_error(uint8_t channel)
{
  struct ra8p_iic_priv_s *priv;

  if (channel >= 3 || !g_iic[channel].initialized)
    {
      return false;
    }

  priv = &g_iic[channel];
  uint8_t error_flags = iic_getreg8(priv, RA8P_IIC_ICER_OFFSET);
  return error_flags != 0;
}

/****************************************************************************
 * Name: ra8p_iic_enable_timeout
 *
 * Description:
 *   Enable/disable IIC timeout detection based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *   enable - true to enable timeout, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_enable_timeout(uint8_t channel, bool enable)
{
  struct ra8p_iic_priv_s *priv;
  uint8_t regval;

  if (channel >= 3 || !g_iic[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_iic[channel];

  /* Read current function enable register */
  regval = iic_getreg8(priv, RA8P_IIC_ICFER_OFFSET);

  if (enable)
    {
      regval |= RA8P_IIC_ICFER_TMOE;  /* Enable timeout */
    }
  else
    {
      regval &= ~RA8P_IIC_ICFER_TMOE; /* Disable timeout */
    }

  iic_putreg8(priv, RA8P_IIC_ICFER_OFFSET, regval);

  return OK;
}

/****************************************************************************
 * Name: ra8p_iic_is_busy
 *
 * Description:
 *   Check if IIC bus is busy based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *
 * Returned Value:
 *   true if busy, false otherwise
 *
 ****************************************************************************/

bool ra8p_iic_is_busy(uint8_t channel)
{
  struct ra8p_iic_priv_s *priv;

  if (channel >= 3 || !g_iic[channel].initialized)
    {
      return false;
    }

  priv = &g_iic[channel];

  uint8_t syssts = iic_getreg8(priv, RA8P_IIC_SYSSTS_OFFSET);
  return (syssts & RA8P_IIC_SYSSTS_BBSY) != 0;  /* Bus busy */
}

/****************************************************************************
 * Name: ra8p_iic_send_start
 *
 * Description:
 *   Send START condition on IIC bus based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_send_start(uint8_t channel)
{
  struct ra8p_iic_priv_s *priv;

  if (channel >= 3 || !g_iic[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_iic[channel];
  return iic_send_start(priv);
}

/****************************************************************************
 * Name: ra8p_iic_send_stop
 *
 * Description:
 *   Send STOP condition on IIC bus based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_send_stop(uint8_t channel)
{
  struct ra8p_iic_priv_s *priv;

  if (channel >= 3 || !g_iic[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_iic[channel];
  return iic_send_stop(priv);
}

/****************************************************************************
 * Name: ra8p_iic_write_byte
 *
 * Description:
 *   Write a byte to IIC bus based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *   data - Byte to write
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_write_byte(uint8_t channel, uint8_t data)
{
  struct ra8p_iic_priv_s *priv;

  if (channel >= 3 || !g_iic[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_iic[channel];
  return iic_write_byte(priv, data);
}

/****************************************************************************
 * Name: ra8p_iic_read_byte
 *
 * Description:
 *   Read a byte from IIC bus based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *   data - Pointer to store read byte
 *   nack_last - true to send NACK for last byte, false to send ACK
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_read_byte(uint8_t channel, uint8_t *data, bool nack_last)
{
  struct ra8p_iic_priv_s *priv;

  if (channel >= 3 || !g_iic[channel].initialized || data == NULL)
    {
      return -EINVAL;
    }

  priv = &g_iic[channel];
  return iic_read_byte(priv, data, nack_last);
}

/****************************************************************************
 * Name: ra8p_iic_set_pullup
 *
 * Description:
 *   Set IIC pull-up resistors based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *   enable - true to enable internal pull-ups, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_set_pullup(uint8_t channel, bool enable)
{
  struct ra8p_iic_priv_s *priv;
  uint8_t regval;

  if (channel >= 3 || !g_iic[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_iic[channel];

  /* Read current configuration */
  regval = iic_getreg8(priv, RA8P_IIC_ICCR1_OFFSET);

  if (enable)
    {
      regval |= RA8P_IIC_ICCR1_SDAI | RA8P_IIC_ICCR1_SCLIE;  /* Enable SDA/SCL pull-ups */
    }
  else
    {
      regval &= ~(RA8P_IIC_ICCR1_SDAI | RA8P_IIC_ICCR1_SCLIE); /* Disable SDA/SCL pull-ups */
    }

  iic_putreg8(priv, RA8P_IIC_ICCR1_OFFSET, regval);

  priv->pullup_enabled = enable;

  iicinfo("IIC%d pull-ups %s\n", channel, enable ? "enabled" : "disabled");
  return OK;
}

/****************************************************************************
 * Name: ra8p_iic_enable_fast_mode_plus
 *
 * Description:
 *   Enable/disable Fast Mode Plus (1 MHz) based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *   enable - true to enable FM+, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_enable_fast_mode_plus(uint8_t channel, bool enable)
{
  struct ra8p_iic_priv_s *priv;
  uint8_t regval;

  if (channel >= 3 || !g_iic[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_iic[channel];

  /* Read current function enable register */
  regval = iic_getreg8(priv, RA8P_IIC_ICFER_OFFSET);

  if (enable)
    {
      regval |= RA8P_IIC_ICFER_FMPE;  /* Enable Fast Mode Plus */
      priv->fast_mode_plus = true;
    }
  else
    {
      regval &= ~RA8P_IIC_ICFER_FMPE; /* Disable Fast Mode Plus */
      priv->fast_mode_plus = false;
    }

  iic_putreg8(priv, RA8P_IIC_ICFER_OFFSET, regval);

  iicinfo("IIC%d FM+ %s\n", channel, enable ? "enabled" : "disabled");
  return OK;
}

#endif /* CONFIG_RA8P_IIC */