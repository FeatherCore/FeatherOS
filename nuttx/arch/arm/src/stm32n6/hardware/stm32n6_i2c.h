/****************************************************************************
 * arch/arm/src/stm32n6/hardware/stm32n6_i2c.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_I2C_H
#define __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_I2C_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "stm32n6_memorymap.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_I2C1_BASE            (STM32N6_PERIPH_BASE + 0x00005400)
#define STM32_I2C2_BASE            (STM32N6_PERIPH_BASE + 0x00005800)
#define STM32_I2C3_BASE            (STM32N6_PERIPH_BASE + 0x00005C00)
#define STM32_I2C4_BASE            (STM32N6_PERIPH_BASE + 0x06001C00)

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

#define I2C_CR1_PE                  (1 << 0)
#define I2C_CR1_TXIE                (1 << 1)
#define I2C_CR1_RXIE                (1 << 2)
#define I2C_CR1_ADDRIE              (1 << 3)
#define I2C_CR1_NACKIE              (1 << 4)
#define I2C_CR1_STOPIE              (1 << 5)
#define I2C_CR1_TCIE                (1 << 6)
#define I2C_CR1_ERRIE               (1 << 7)
#define I2C_CR1_DNF_SHIFT           8
#define I2C_CR1_DNF_MASK            (0xf << I2C_CR1_DNF_SHIFT)
#define I2C_CR1_ANFOFF              (1 << 12)
#define I2C_CR1_SWRST               (1 << 13)
#define I2C_CR1_TXDMAEN             (1 << 14)
#define I2C_CR1_RXDMAEN             (1 << 15)
#define I2C_CR1_SBC                 (1 << 16)
#define I2C_CR1_NOSTRETCH           (1 << 17)
#define I2C_CR1_WUPEN               (1 << 18)
#define I2C_CR1_GCEN                (1 << 19)

#define I2C_CR2_SADD_SHIFT          0
#define I2C_CR2_SADD_MASK           (0x3ff << I2C_CR2_SADD_SHIFT)
#define I2C_CR2_RD_WRN              (1 << 10)
#define I2C_CR2_ADD10               (1 << 11)
#define I2C_CR2_HEAD10R             (1 << 12)
#define I2C_CR2_START               (1 << 13)
#define I2C_CR2_STOP                (1 << 14)
#define I2C_CR2_NACK                (1 << 15)
#define I2C_CR2_NBYTES_SHIFT        16
#define I2C_CR2_NBYTES_MASK         (0xff << I2C_CR2_NBYTES_SHIFT)
#define I2C_CR2_RELOAD              (1 << 24)
#define I2C_CR2_AUTOEND             (1 << 25)
#define I2C_CR2_PECBYTE             (1 << 26)

#define I2C_TIMINGR_SCLL_SHIFT      0
#define I2C_TIMINGR_SCLL_MASK       (0xff << I2C_TIMINGR_SCLL_SHIFT)
#define I2C_TIMINGR_SCLH_SHIFT      8
#define I2C_TIMINGR_SCLH_MASK       (0xff << I2C_TIMINGR_SCLH_SHIFT)
#define I2C_TIMINGR_SDADEL_SHIFT    16
#define I2C_TIMINGR_SDADEL_MASK     (0xf << I2C_TIMINGR_SDADEL_SHIFT)
#define I2C_TIMINGR_SCLDEL_SHIFT    20
#define I2C_TIMINGR_SCLDEL_MASK     (0xf << I2C_TIMINGR_SCLDEL_SHIFT)
#define I2C_TIMINGR_PRESC_SHIFT     28
#define I2C_TIMINGR_PRESC_MASK      (0xf << I2C_TIMINGR_PRESC_SHIFT)

#define I2C_ISR_TXE                 (1 << 0)
#define I2C_ISR_TXIS                (1 << 1)
#define I2C_ISR_RXNE                (1 << 2)
#define I2C_ISR_ADDR                (1 << 3)
#define I2C_ISR_NACKF               (1 << 4)
#define I2C_ISR_STOPF               (1 << 5)
#define I2C_ISR_TC                  (1 << 6)
#define I2C_ISR_TCR                 (1 << 7)
#define I2C_ISR_BERR                (1 << 8)
#define I2C_ISR_ARLO                (1 << 9)
#define I2C_ISR_OVR                 (1 << 10)
#define I2C_ISR_PECERR              (1 << 11)
#define I2C_ISR_TIMEOUT             (1 << 12)
#define I2C_ISR_ALERT               (1 << 13)
#define I2C_ISR_BUSY                (1 << 15)
#define I2C_ISR_DIR                 (1 << 16)

#define I2C_ICR_ADDRCF              (1 << 3)
#define I2C_ICR_NACKCF              (1 << 4)
#define I2C_ICR_STOPCF              (1 << 5)
#define I2C_ICR_BERRCF              (1 << 8)
#define I2C_ICR_ARLOCF              (1 << 9)
#define I2C_ICR_OVRCF               (1 << 10)
#define I2C_ICR_PECCF               (1 << 11)
#define I2C_ICR_TIMOUTCF            (1 << 12)
#define I2C_ICR_ALERTCF             (1 << 13)

#endif /* __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_I2C_H */