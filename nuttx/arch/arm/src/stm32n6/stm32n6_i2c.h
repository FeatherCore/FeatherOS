/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_i2c.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_STM32N6_I2C_H
#define __ARCH_ARM_SRC_STM32N6_STM32N6_I2C_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <stdint.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define I2C_CR1_PE              (1 << 0)
#define I2C_CR1_TXIE            (1 << 1)
#define I2C_CR1_RXIE            (1 << 2)
#define I2C_CR1_ADDRIE          (1 << 3)
#define I2C_CR1_NACKIE          (1 << 4)
#define I2C_CR1_STOPIE          (1 << 5)
#define I2C_CR1_TCIE            (1 << 6)
#define I2C_CR1_ERRIE           (1 << 7)
#define I2C_CR1_DNF_SHIFT       8
#define I2C_CR1_DNF_MASK        (0xf << I2C_CR1_DNF_SHIFT)
#define I2C_CR1_ANFOFF          (1 << 12)
#define I2C_CR1_SWRST           (1 << 13)
#define I2C_CR1_TXDMAEN         (1 << 14)
#define I2C_CR1_RXDMAEN         (1 << 15)
#define I2C_CR1_SBC             (1 << 16)
#define I2C_CR1_NOSTRETCH       (1 << 17)
#define I2C_CR1_WUPEN           (1 << 18)
#define I2C_CR1_GCEN            (1 << 19)
#define I2C_CR1_SMBHEN          (1 << 20)
#define I2C_CR1_SMBDEN          (1 << 21)
#define I2C_CR1_ALERTEN         (1 << 22)
#define I2C_CR1_PECEN           (1 << 23)

#define I2C_CR2_SADD_SHIFT      0
#define I2C_CR2_SADD_MASK       (0x3ff << I2C_CR2_SADD_SHIFT)
#define I2C_CR2_RD_WRN          (1 << 10)
#define I2C_CR2_ADD10           (1 << 11)
#define I2C_CR2_HEAD10R         (1 << 12)
#define I2C_CR2_START           (1 << 13)
#define I2C_CR2_STOP            (1 << 14)
#define I2C_CR2_NACK            (1 << 15)
#define I2C_CR2_NBYTES_SHIFT    16
#define I2C_CR2_NBYTES_MASK     (0xff << I2C_CR2_NBYTES_SHIFT)
#define I2C_CR2_RELOAD          (1 << 24)
#define I2C_CR2_AUTOEND         (1 << 25)
#define I2C_CR2_PECBYTE         (1 << 26)

#define I2C_ISR_TXE             (1 << 0)
#define I2C_ISR_TXIS            (1 << 1)
#define I2C_ISR_RXNE            (1 << 2)
#define I2C_ISR_ADDR            (1 << 3)
#define I2C_ISR_NACKF           (1 << 4)
#define I2C_ISR_STOPF           (1 << 5)
#define I2C_ISR_TC              (1 << 6)
#define I2C_ISR_TCR             (1 << 7)
#define I2C_ISR_BERR            (1 << 8)
#define I2C_ISR_ARLO            (1 << 9)
#define I2C_ISR_OVR             (1 << 10)
#define I2C_ISR_PECERR          (1 << 11)
#define I2C_ISR_TIMEOUT         (1 << 12)
#define I2C_ISR_ALERT           (1 << 13)
#define I2C_ISR_BUSY            (1 << 15)
#define I2C_ISR_DIR             (1 << 16)
#define I2C_ISR_ADDCODE_SHIFT   17
#define I2C_ISR_ADDCODE_MASK    (0x7f << I2C_ISR_ADDCODE_SHIFT)

#define I2C_ICR_ADDRCF          (1 << 3)
#define I2C_ICR_NACKCF          (1 << 4)
#define I2C_ICR_STOPCF          (1 << 5)
#define I2C_ICR_BERRCF          (1 << 8)
#define I2C_ICR_ARLOCF          (1 << 9)
#define I2C_ICR_OVRCF           (1 << 10)
#define I2C_ICR_PECCF           (1 << 11)
#define I2C_ICR_TIMOUTCF        (1 << 12)
#define I2C_ICR_ALERTCF         (1 << 13)

/****************************************************************************
 * Public Types
 ****************************************************************************/

struct stm32n6_i2c_s
{
  uintptr_t i2cbase;
  uint32_t frequency;
  uint8_t addr;
  int irq;
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

int stm32n6_i2c_initialize(uintptr_t i2cbase, uint32_t frequency);
void stm32n6_i2c_enable(uintptr_t i2cbase);
void stm32n6_i2c_disable(uintptr_t i2cbase);
int stm32n6_i2c_transfer(uintptr_t i2cbase, uint8_t addr,
                         const uint8_t *wbuffer, size_t wbuflen,
                         uint8_t *rbuffer, size_t rbuflen);
bool stm32n6_i2c_busy(uintptr_t i2cbase);

#endif /* __ARCH_ARM_SRC_STM32N6_STM32N6_I2C_H */
