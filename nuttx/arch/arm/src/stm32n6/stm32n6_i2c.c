/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_i2c.c
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
#include <nuttx/i2c/i2c_master.h>

#include "arm_internal.h"
#include "chip.h"
#include "hardware/stm32n6_memorymap.h"
#include "hardware/stm32n6_rcc.h"
#include "stm32n6_i2c.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_I2C_CR1_OFFSET        0x00
#define STM32_I2C_CR2_OFFSET        0x04
#define STM32_I2C_OAR1_OFFSET       0x08
#define STM32_I2C_OAR2_OFFSET       0x0C
#define STM32_I2C_TIMINGR_OFFSET    0x10
#define STM32_I2C_TIMEOUTR_OFFSET   0x14
#define STM32_I2C_ISR_OFFSET        0x18
#define STM32_I2C_ICR_OFFSET        0x1C
#define STM32_I2C_PECR_OFFSET       0x20
#define STM32_I2C_RXDR_OFFSET       0x24
#define STM32_I2C_TXDR_OFFSET       0x28

#define TIMEOUT_VALUE               0xfffff

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static inline void stm32n6_i2c_putreg(uintptr_t i2cbase,
                                      uint32_t offset, uint32_t value)
{
  putreg32(value, i2cbase + offset);
}

static inline uint32_t stm32n6_i2c_getreg(uintptr_t i2cbase,
                                          uint32_t offset)
{
  return getreg32(i2cbase + offset);
}

static void stm32n6_i2c_enable_clock(uintptr_t i2cbase)
{
  uint32_t regval;

  regval = getreg32(STM32_RCC_APB1ENR);
  if (i2cbase == STM32_I2C1_BASE)
    {
      regval |= RCC_APB1ENR_I2C1EN;
    }
  else if (i2cbase == STM32_I2C2_BASE)
    {
      regval |= RCC_APB1ENR_I2C2EN;
    }
  else if (i2cbase == STM32_I2C3_BASE)
    {
      regval |= RCC_APB1ENR_I2C3EN;
    }

  putreg32(regval, STM32_RCC_APB1ENR);

  regval = getreg32(STM32_RCC_APB4ENR);
  if (i2cbase == STM32_I2C4_BASE)
    {
      regval |= RCC_APB4ENR_I2C4EN;
    }

  putreg32(regval, STM32_RCC_APB4ENR);
}

static int stm32n6_i2c_wait_isr(uintptr_t i2cbase, uint32_t mask)
{
  int timeout = TIMEOUT_VALUE;

  while ((stm32n6_i2c_getreg(i2cbase, STM32_I2C_ISR_OFFSET) & mask) == 0
         && timeout-- > 0);

  return timeout > 0 ? OK : -ETIMEDOUT;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

int stm32n6_i2c_initialize(uintptr_t i2cbase, uint32_t frequency)
{
  uint32_t timing;

  stm32n6_i2c_enable_clock(i2cbase);

  stm32n6_i2c_disable(i2cbase);

  timing = 0x00f02b86;
  stm32n6_i2c_putreg(i2cbase, STM32_I2C_TIMINGR_OFFSET, timing);

  stm32n6_i2c_enable(i2cbase);

  return OK;
}

/****************************************************************************
 * Name: stm32n6_i2cbus_initialize
 ****************************************************************************/

FAR struct i2c_master_s *stm32n6_i2cbus_initialize(int port)
{
  (void)port;

  return NULL;
}

/****************************************************************************
 * Name: stm32n6_i2cbus_uninitialize
 ****************************************************************************/

int stm32n6_i2cbus_uninitialize(FAR struct i2c_master_s *dev)
{
  (void)dev;

  return -ENOSYS;
}

void stm32n6_i2c_enable(uintptr_t i2cbase)
{
  uint32_t regval;

  regval = stm32n6_i2c_getreg(i2cbase, STM32_I2C_CR1_OFFSET);
  regval |= I2C_CR1_PE;
  stm32n6_i2c_putreg(i2cbase, STM32_I2C_CR1_OFFSET, regval);
}

void stm32n6_i2c_disable(uintptr_t i2cbase)
{
  uint32_t regval;

  regval = stm32n6_i2c_getreg(i2cbase, STM32_I2C_CR1_OFFSET);
  regval &= ~I2C_CR1_PE;
  stm32n6_i2c_putreg(i2cbase, STM32_I2C_CR1_OFFSET, regval);
}

int stm32n6_i2c_transfer(uintptr_t i2cbase, uint8_t addr,
                         const uint8_t *wbuffer, size_t wbuflen,
                         uint8_t *rbuffer, size_t rbuflen)
{
  uint32_t regval;
  int ret;
  size_t i;

  if (wbuflen > 0)
    {
      regval = (addr << 1) & I2C_CR2_SADD_MASK;
      regval |= (wbuflen << I2C_CR2_NBYTES_SHIFT) & I2C_CR2_NBYTES_MASK;
      regval |= I2C_CR2_START | I2C_CR2_AUTOEND;
      stm32n6_i2c_putreg(i2cbase, STM32_I2C_CR2_OFFSET, regval);

      for (i = 0; i < wbuflen; i++)
        {
          ret = stm32n6_i2c_wait_isr(i2cbase, I2C_ISR_TXIS);
          if (ret < 0)
            {
              return ret;
            }

          stm32n6_i2c_putreg(i2cbase, STM32_I2C_TXDR_OFFSET, wbuffer[i]);
        }

      ret = stm32n6_i2c_wait_isr(i2cbase, I2C_ISR_STOPF);
      if (ret < 0)
        {
          return ret;
        }

      stm32n6_i2c_putreg(i2cbase, STM32_I2C_ICR_OFFSET, I2C_ICR_STOPCF);
    }

  if (rbuflen > 0)
    {
      regval = (addr << 1) & I2C_CR2_SADD_MASK;
      regval |= I2C_CR2_RD_WRN;
      regval |= (rbuflen << I2C_CR2_NBYTES_SHIFT) & I2C_CR2_NBYTES_MASK;
      regval |= I2C_CR2_START | I2C_CR2_AUTOEND;
      stm32n6_i2c_putreg(i2cbase, STM32_I2C_CR2_OFFSET, regval);

      for (i = 0; i < rbuflen; i++)
        {
          ret = stm32n6_i2c_wait_isr(i2cbase, I2C_ISR_RXNE);
          if (ret < 0)
            {
              return ret;
            }

          rbuffer[i] =
            (uint8_t)stm32n6_i2c_getreg(i2cbase, STM32_I2C_RXDR_OFFSET);
        }

      ret = stm32n6_i2c_wait_isr(i2cbase, I2C_ISR_STOPF);
      if (ret < 0)
        {
          return ret;
        }

      stm32n6_i2c_putreg(i2cbase, STM32_I2C_ICR_OFFSET, I2C_ICR_STOPCF);
    }

  return OK;
}

bool stm32n6_i2c_busy(uintptr_t i2cbase)
{
  uint32_t regval;

  regval = stm32n6_i2c_getreg(i2cbase, STM32_I2C_ISR_OFFSET);
  return (regval & I2C_ISR_BUSY) != 0;
}
