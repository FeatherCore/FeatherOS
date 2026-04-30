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

#include <errno.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#include <debug.h>
#include <nuttx/kmalloc.h>
#include <nuttx/mutex.h>
#include <nuttx/spi/qspi.h>

#include <arch/board/board.h>

#include "arm_internal.h"
#include "hardware/stm32n6_memorymap.h"
#include "hardware/stm32n6_rcc.h"
#include "stm32n6_gpio.h"
#include "stm32n6_xspi.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32N6_XSPI_TIMEOUT        1000000
#define STM32N6_XSPI1_DEVSIZE       25
#define STM32N6_XSPI2_DEVSIZE       26
#define STM32N6_XSPI_DEFAULT_NBITS  8

#ifndef CONFIG_MX66UW_QSPI_FREQUENCY
#  define CONFIG_MX66UW_QSPI_FREQUENCY 24000000
#endif

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct stm32n6_xspidev_s
{
  struct qspi_dev_s qspi;
  uintptr_t base;
  uint32_t frequency;
  uint32_t actual;
  uint32_t devsize;
  enum qspi_mode_e mode;
  int nbits;
  mutex_t lock;
  uint8_t intf;
  bool initialized;
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int stm32n6_xspi_lock(FAR struct qspi_dev_s *dev, bool lock);
static uint32_t stm32n6_xspi_setfrequency(FAR struct qspi_dev_s *dev,
                                          uint32_t frequency);
static void stm32n6_xspi_setmode(FAR struct qspi_dev_s *dev,
                                 enum qspi_mode_e mode);
static void stm32n6_xspi_setbits(FAR struct qspi_dev_s *dev, int nbits);
static int stm32n6_xspi_command(FAR struct qspi_dev_s *dev,
                                FAR struct qspi_cmdinfo_s *cmdinfo);
static int stm32n6_xspi_memory(FAR struct qspi_dev_s *dev,
                               FAR struct qspi_meminfo_s *meminfo);
static FAR void *stm32n6_xspi_alloc(FAR struct qspi_dev_s *dev,
                                    size_t buflen);
static void stm32n6_xspi_free(FAR struct qspi_dev_s *dev,
                              FAR void *buffer);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static const struct qspi_ops_s g_xspiops =
{
  .lock         = stm32n6_xspi_lock,
  .setfrequency = stm32n6_xspi_setfrequency,
  .setmode      = stm32n6_xspi_setmode,
  .setbits      = stm32n6_xspi_setbits,
  .command      = stm32n6_xspi_command,
  .memory       = stm32n6_xspi_memory,
  .alloc        = stm32n6_xspi_alloc,
  .free         = stm32n6_xspi_free,
};

#ifdef CONFIG_STM32N6_XSPI1
static struct stm32n6_xspidev_s g_xspi1dev =
{
  .qspi.ops = &g_xspiops,
  .base     = STM32_XSPI1_BASE,
  .intf     = 1,
  .devsize  = STM32N6_XSPI1_DEVSIZE,
  .nbits    = STM32N6_XSPI_DEFAULT_NBITS,
  .lock     = NXMUTEX_INITIALIZER,
};
#endif

#ifdef CONFIG_STM32N6_XSPI2
static struct stm32n6_xspidev_s g_xspi2dev =
{
  .qspi.ops = &g_xspiops,
  .base     = STM32_XSPI2_BASE,
  .intf     = 2,
  .devsize  = STM32N6_XSPI2_DEVSIZE,
  .nbits    = STM32N6_XSPI_DEFAULT_NBITS,
  .lock     = NXMUTEX_INITIALIZER,
};
#endif

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static inline FAR struct stm32n6_xspidev_s *
stm32n6_xspi_priv(FAR struct qspi_dev_s *dev)
{
  return (FAR struct stm32n6_xspidev_s *)dev;
}

static inline uint32_t
stm32n6_xspi_getreg(FAR struct stm32n6_xspidev_s *priv,
                    unsigned int offset)
{
  return getreg32(priv->base + offset);
}

static inline void stm32n6_xspi_putreg(FAR struct stm32n6_xspidev_s *priv,
                                       unsigned int offset, uint32_t value)
{
  putreg32(value, priv->base + offset);
}

static void stm32n6_xspi_clearflags(FAR struct stm32n6_xspidev_s *priv)
{
  stm32n6_xspi_putreg(priv, STM32_XSPI_FCR_OFFSET, XSPI_FCR_ALL);
}

static int stm32n6_xspi_wait(FAR struct stm32n6_xspidev_s *priv,
                             uint32_t mask, bool set)
{
  uint32_t regval;
  int timeout;

  for (timeout = STM32N6_XSPI_TIMEOUT; timeout > 0; timeout--)
    {
      regval = stm32n6_xspi_getreg(priv, STM32_XSPI_SR_OFFSET);
      if ((regval & (XSPI_SR_TEF | XSPI_SR_TOF)) != 0)
        {
          stm32n6_xspi_clearflags(priv);
          return -EIO;
        }

      if (((regval & mask) != 0) == set)
        {
          return OK;
        }
    }

  return -ETIMEDOUT;
}

static int stm32n6_xspi_wait_ready(FAR struct stm32n6_xspidev_s *priv)
{
  return stm32n6_xspi_wait(priv, XSPI_SR_BUSY, false);
}

static uint32_t stm32n6_xspi_addrsize(uint8_t addrlen)
{
  if (addrlen == 0)
    {
      return 0;
    }

  if (addrlen > 4)
    {
      addrlen = 4;
    }

  return addrlen - 1;
}

static int stm32n6_xspi_io(FAR struct stm32n6_xspidev_s *priv,
                           uint32_t ccr, uint32_t addr, bool hasaddr,
                           FAR void *buffer, uint32_t buflen, bool write)
{
  FAR uint8_t *rxbuffer;
  FAR const uint8_t *txbuffer;
  uint32_t word;
  uint32_t chunk;
  uint32_t ndx;
  uint32_t regval;
  int ret;

  if (buflen > 0 && buffer == NULL)
    {
      return -EINVAL;
    }

  ret = stm32n6_xspi_wait_ready(priv);
  if (ret < 0)
    {
      return ret;
    }

  stm32n6_xspi_clearflags(priv);

  if (buflen > 0)
    {
      stm32n6_xspi_putreg(priv, STM32_XSPI_DLR_OFFSET, buflen - 1);
    }

  stm32n6_xspi_putreg(priv, STM32_XSPI_CCR_OFFSET, ccr);

  if (hasaddr)
    {
      stm32n6_xspi_putreg(priv, STM32_XSPI_AR_OFFSET, addr);
    }

  if (buflen == 0)
    {
      ret = stm32n6_xspi_wait(priv, XSPI_SR_TCF, true);
      stm32n6_xspi_clearflags(priv);
      return ret;
    }

  if (write)
    {
      txbuffer = buffer;
      ndx = 0;

      while (ndx < buflen)
        {
          ret = stm32n6_xspi_wait(priv, XSPI_SR_FTF, true);
          if (ret < 0)
            {
              return ret;
            }

          word = 0;
          chunk = buflen - ndx;
          if (chunk > sizeof(uint32_t))
            {
              chunk = sizeof(uint32_t);
            }

          word = txbuffer[ndx];
          if (chunk > 1)
            {
              word |= (uint32_t)txbuffer[ndx + 1] << 8;
            }

          if (chunk > 2)
            {
              word |= (uint32_t)txbuffer[ndx + 2] << 16;
            }

          if (chunk > 3)
            {
              word |= (uint32_t)txbuffer[ndx + 3] << 24;
            }

          stm32n6_xspi_putreg(priv, STM32_XSPI_DR_OFFSET, word);
          ndx += chunk;
        }
    }
  else
    {
      rxbuffer = buffer;
      ndx = 0;

      while (ndx < buflen)
        {
          ret = stm32n6_xspi_wait(priv, XSPI_SR_FTF | XSPI_SR_TCF, true);
          if (ret < 0)
            {
              return ret;
            }

          regval = stm32n6_xspi_getreg(priv, STM32_XSPI_SR_OFFSET);
          if ((regval & XSPI_SR_FTF) == 0 && (regval & XSPI_SR_TCF) != 0)
            {
              break;
            }

          word = stm32n6_xspi_getreg(priv, STM32_XSPI_DR_OFFSET);
          chunk = buflen - ndx;
          if (chunk > sizeof(uint32_t))
            {
              chunk = sizeof(uint32_t);
            }

          rxbuffer[ndx] = (uint8_t)word;
          if (chunk > 1)
            {
              rxbuffer[ndx + 1] = (uint8_t)(word >> 8);
            }

          if (chunk > 2)
            {
              rxbuffer[ndx + 2] = (uint8_t)(word >> 16);
            }

          if (chunk > 3)
            {
              rxbuffer[ndx + 3] = (uint8_t)(word >> 24);
            }

          ndx += chunk;
        }
    }

  ret = stm32n6_xspi_wait(priv, XSPI_SR_TCF, true);
  stm32n6_xspi_clearflags(priv);
  return ret;
}

static uint32_t stm32n6_xspi_cmdccr(FAR const struct qspi_cmdinfo_s *cmdinfo)
{
  uint32_t ccr;
  uint32_t mode;

  ccr = cmdinfo->cmd & XSPI_CCR_INSTRUCTION_MASK;

  if (QSPICMD_ISIQUAD(cmdinfo->flags))
    {
      mode = XSPI_CCR_IMODE_4LINE;
    }
  else if (QSPICMD_ISIDUAL(cmdinfo->flags))
    {
      mode = XSPI_CCR_IMODE_2LINE;
    }
  else
    {
      mode = XSPI_CCR_IMODE_1LINE;
    }

  ccr |= mode << XSPI_CCR_IMODE_SHIFT;

  if (QSPICMD_ISADDRESS(cmdinfo->flags))
    {
      ccr |= XSPI_CCR_ADMODE_1LINE << XSPI_CCR_ADMODE_SHIFT;
      ccr |= stm32n6_xspi_addrsize(cmdinfo->addrlen) <<
             XSPI_CCR_ADSIZE_SHIFT;
    }

  if (QSPICMD_ISDATA(cmdinfo->flags))
    {
      ccr |= XSPI_CCR_DMODE_1LINE << XSPI_CCR_DMODE_SHIFT;
      ccr |= (QSPICMD_ISWRITE(cmdinfo->flags) ?
              XSPI_CCR_FMODE_IND_WRITE : XSPI_CCR_FMODE_IND_READ) <<
             XSPI_CCR_FMODE_SHIFT;
    }

  return ccr;
}

static uint32_t stm32n6_xspi_memccr(FAR const struct qspi_meminfo_s *meminfo)
{
  uint32_t ccr;
  uint32_t datamode;
  uint32_t instmode;

  ccr = meminfo->cmd & XSPI_CCR_INSTRUCTION_MASK;
  datamode = XSPI_CCR_DMODE_1LINE;
  instmode = XSPI_CCR_IMODE_1LINE;

  if (QSPIMEM_ISQUADIO(meminfo->flags))
    {
      datamode = XSPI_CCR_DMODE_4LINE;
    }
  else if (QSPIMEM_ISDUALIO(meminfo->flags))
    {
      datamode = XSPI_CCR_DMODE_2LINE;
    }

  if (QSPIMEM_ISIQUAD(meminfo->flags))
    {
      instmode = XSPI_CCR_IMODE_4LINE;
    }
  else if (QSPIMEM_ISIDUAL(meminfo->flags))
    {
      instmode = XSPI_CCR_IMODE_2LINE;
    }

  ccr |= instmode << XSPI_CCR_IMODE_SHIFT;
  ccr |= stm32n6_xspi_addrsize(meminfo->addrlen) << XSPI_CCR_ADSIZE_SHIFT;
  ccr |= datamode << XSPI_CCR_ADMODE_SHIFT;
  ccr |= datamode << XSPI_CCR_DMODE_SHIFT;
  ccr |= ((uint32_t)meminfo->dummies << XSPI_CCR_DUMMY_CYCLES_SHIFT) &
         XSPI_CCR_DUMMY_CYCLES_MASK;
  ccr |= (QSPIMEM_ISWRITE(meminfo->flags) ?
          XSPI_CCR_FMODE_IND_WRITE : XSPI_CCR_FMODE_IND_READ) <<
         XSPI_CCR_FMODE_SHIFT;

  return ccr;
}

static void stm32n6_xspi_configgpio(FAR struct stm32n6_xspidev_s *priv)
{
#ifdef CONFIG_STM32N6_GPIO
  if (priv->intf == 1)
    {
#ifdef CONFIG_STM32N6_XSPI1
      stm32n6_configgpio(GPIO_XSPI1_NCS);
      stm32n6_configgpio(GPIO_XSPI1_DQS0);
      stm32n6_configgpio(GPIO_XSPI1_DQS1);
      stm32n6_configgpio(GPIO_XSPI1_CLK);
      stm32n6_configgpio(GPIO_XSPI1_IO0);
      stm32n6_configgpio(GPIO_XSPI1_IO1);
      stm32n6_configgpio(GPIO_XSPI1_IO2);
      stm32n6_configgpio(GPIO_XSPI1_IO3);
      stm32n6_configgpio(GPIO_XSPI1_IO4);
      stm32n6_configgpio(GPIO_XSPI1_IO5);
      stm32n6_configgpio(GPIO_XSPI1_IO6);
      stm32n6_configgpio(GPIO_XSPI1_IO7);
      stm32n6_configgpio(GPIO_XSPI1_IO8);
      stm32n6_configgpio(GPIO_XSPI1_IO9);
      stm32n6_configgpio(GPIO_XSPI1_IO10);
      stm32n6_configgpio(GPIO_XSPI1_IO11);
      stm32n6_configgpio(GPIO_XSPI1_IO12);
      stm32n6_configgpio(GPIO_XSPI1_IO13);
      stm32n6_configgpio(GPIO_XSPI1_IO14);
      stm32n6_configgpio(GPIO_XSPI1_IO15);
#endif
    }
  else if (priv->intf == 2)
    {
#ifdef CONFIG_STM32N6_XSPI2
      stm32n6_configgpio(GPIO_XSPI2_NCS);
      stm32n6_configgpio(GPIO_XSPI2_DQS);
      stm32n6_configgpio(GPIO_XSPI2_CLK);
      stm32n6_configgpio(GPIO_XSPI2_IO0);
      stm32n6_configgpio(GPIO_XSPI2_IO1);
      stm32n6_configgpio(GPIO_XSPI2_IO2);
      stm32n6_configgpio(GPIO_XSPI2_IO3);
      stm32n6_configgpio(GPIO_XSPI2_IO4);
      stm32n6_configgpio(GPIO_XSPI2_IO5);
      stm32n6_configgpio(GPIO_XSPI2_IO6);
      stm32n6_configgpio(GPIO_XSPI2_IO7);
#endif
    }
#endif
}

static void stm32n6_xspi_enableclock(FAR struct stm32n6_xspidev_s *priv)
{
  uint32_t regval;
  uint32_t mask;

  mask = RCC_AHB5ENR_XSPIMEN;

  if (priv->intf == 1)
    {
      mask |= RCC_AHB5ENR_XSPI1EN;
    }
  else if (priv->intf == 2)
    {
      mask |= RCC_AHB5ENR_XSPI2EN;
    }

  regval = getreg32(STM32_RCC_AHB5ENR);
  regval |= mask;
  putreg32(regval, STM32_RCC_AHB5ENR);
}

static void stm32n6_xspi_reset(FAR struct stm32n6_xspidev_s *priv)
{
  uint32_t regval;
  uint32_t mask;

  mask = RCC_AHB5ENR_XSPIMEN;

  if (priv->intf == 1)
    {
      mask |= RCC_AHB5ENR_XSPI1EN;
    }
  else if (priv->intf == 2)
    {
      mask |= RCC_AHB5ENR_XSPI2EN;
    }

  regval = getreg32(STM32_RCC_AHB5RSTR);
  regval |= mask;
  putreg32(regval, STM32_RCC_AHB5RSTR);

  regval &= ~mask;
  putreg32(regval, STM32_RCC_AHB5RSTR);
}

static void stm32n6_xspi_hwinit(FAR struct stm32n6_xspidev_s *priv)
{
  uint32_t regval;

  stm32n6_xspi_configgpio(priv);
  stm32n6_xspi_enableclock(priv);
  stm32n6_xspi_reset(priv);

  regval = stm32n6_xspi_getreg(priv, STM32_XSPI_CR_OFFSET);
  regval &= ~XSPI_CR_EN;
  stm32n6_xspi_putreg(priv, STM32_XSPI_CR_OFFSET, regval);

  stm32n6_xspi_putreg(priv, STM32_XSPI_DCR1_OFFSET,
                      XSPI_DCR1_DEVSIZE(priv->devsize) |
                      XSPI_DCR1_CSHT(2));
  stm32n6_xspi_clearflags(priv);

  stm32n6_xspi_setfrequency(&priv->qspi, CONFIG_MX66UW_QSPI_FREQUENCY);
  stm32n6_xspi_setmode(&priv->qspi, QSPIDEV_MODE0);
  stm32n6_xspi_setbits(&priv->qspi, STM32N6_XSPI_DEFAULT_NBITS);
}

/****************************************************************************
 * QSPI Methods
 ****************************************************************************/

static int stm32n6_xspi_lock(FAR struct qspi_dev_s *dev, bool lock)
{
  FAR struct stm32n6_xspidev_s *priv = stm32n6_xspi_priv(dev);

  if (lock)
    {
      return nxmutex_lock(&priv->lock);
    }

  nxmutex_unlock(&priv->lock);
  return OK;
}

static uint32_t stm32n6_xspi_setfrequency(FAR struct qspi_dev_s *dev,
                                          uint32_t frequency)
{
  FAR struct stm32n6_xspidev_s *priv = stm32n6_xspi_priv(dev);
  uint32_t regval;
  uint32_t clock;
  uint32_t prescaler;

  clock = CONFIG_STM32N6_HCLK_FREQUENCY;

  if (frequency == 0 || frequency > clock)
    {
      frequency = clock;
    }

  prescaler = (clock + frequency - 1) / frequency;
  if (prescaler > 0)
    {
      prescaler--;
    }

  if (prescaler > 15)
    {
      prescaler = 15;
    }

  stm32n6_xspi_wait_ready(priv);

  regval = stm32n6_xspi_getreg(priv, STM32_XSPI_CR_OFFSET);
  regval &= ~(XSPI_CR_EN | XSPI_CR_PRESCALER_MASK);
  regval |= prescaler << XSPI_CR_PRESCALER_SHIFT;
  stm32n6_xspi_putreg(priv, STM32_XSPI_CR_OFFSET, regval);

  regval |= XSPI_CR_EN;
  stm32n6_xspi_putreg(priv, STM32_XSPI_CR_OFFSET, regval);

  priv->frequency = frequency;
  priv->actual = clock / (prescaler + 1);

  return priv->actual;
}

static void stm32n6_xspi_setmode(FAR struct qspi_dev_s *dev,
                                 enum qspi_mode_e mode)
{
  FAR struct stm32n6_xspidev_s *priv = stm32n6_xspi_priv(dev);

  priv->mode = mode;
}

static void stm32n6_xspi_setbits(FAR struct qspi_dev_s *dev, int nbits)
{
  FAR struct stm32n6_xspidev_s *priv = stm32n6_xspi_priv(dev);

  if (nbits > 0)
    {
      priv->nbits = nbits;
    }
}

static int stm32n6_xspi_command(FAR struct qspi_dev_s *dev,
                                FAR struct qspi_cmdinfo_s *cmdinfo)
{
  FAR struct stm32n6_xspidev_s *priv = stm32n6_xspi_priv(dev);
  bool hasaddr;
  bool write;
  uint32_t buflen;
  uint32_t ccr;

  if (cmdinfo == NULL || cmdinfo->cmd > XSPI_CCR_INSTRUCTION_MASK)
    {
      return -EINVAL;
    }

  hasaddr = QSPICMD_ISADDRESS(cmdinfo->flags);
  write = QSPICMD_ISWRITE(cmdinfo->flags);
  buflen = QSPICMD_ISDATA(cmdinfo->flags) ? cmdinfo->buflen : 0;
  ccr = stm32n6_xspi_cmdccr(cmdinfo);

  return stm32n6_xspi_io(priv, ccr, cmdinfo->addr, hasaddr,
                         cmdinfo->buffer, buflen, write);
}

static int stm32n6_xspi_memory(FAR struct qspi_dev_s *dev,
                               FAR struct qspi_meminfo_s *meminfo)
{
  FAR struct stm32n6_xspidev_s *priv = stm32n6_xspi_priv(dev);
  uint32_t ccr;

  if (meminfo == NULL || meminfo->cmd > XSPI_CCR_INSTRUCTION_MASK)
    {
      return -EINVAL;
    }

  if (QSPIMEM_ISSCRAMBLE(meminfo->flags))
    {
      return -ENOSYS;
    }

  ccr = stm32n6_xspi_memccr(meminfo);
  return stm32n6_xspi_io(priv, ccr, meminfo->addr, true, meminfo->buffer,
                         meminfo->buflen, QSPIMEM_ISWRITE(meminfo->flags));
}

static FAR void *stm32n6_xspi_alloc(FAR struct qspi_dev_s *dev,
                                    size_t buflen)
{
  (void)dev;

  return kmm_malloc(buflen);
}

static void stm32n6_xspi_free(FAR struct qspi_dev_s *dev, FAR void *buffer)
{
  (void)dev;

  kmm_free(buffer);
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32n6_xspi_initialize
 ****************************************************************************/

FAR struct qspi_dev_s *stm32n6_xspi_initialize(int intf)
{
  FAR struct stm32n6_xspidev_s *priv;

  priv = NULL;

#ifdef CONFIG_STM32N6_XSPI1
  if (intf == 1)
    {
      priv = &g_xspi1dev;
    }
#endif

#ifdef CONFIG_STM32N6_XSPI2
  if (intf == 2)
    {
      priv = &g_xspi2dev;
    }
#endif

  if (priv == NULL)
    {
      return NULL;
    }

  if (!priv->initialized)
    {
      stm32n6_xspi_hwinit(priv);
      priv->initialized = true;
    }

  return &priv->qspi;
}
