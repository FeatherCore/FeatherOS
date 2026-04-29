/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_xspi.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_STM32N6_XSPI_H
#define __ARCH_ARM_SRC_STM32N6_STM32N6_XSPI_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <nuttx/spi/spi.h>
#include <nuttx/mtd/mtd.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* XSPI Register Offsets */
#define STM32_XSPI_CR_OFFSET        0x00
#define STM32_XSPI_DCR1_OFFSET      0x04
#define STM32_XSPI_DCR2_OFFSET      0x08
#define STM32_XSPI_DCR3_OFFSET      0x0C
#define STM32_XSPI_DCR4_OFFSET      0x10
#define STM32_XSPI_SR_OFFSET        0x14
#define STM32_XSPI_FCR_OFFSET       0x18
#define STM32_XSPI_DLR_OFFSET       0x20
#define STM32_XSPI_AR_OFFSET        0x24
#define STM32_XSPI_ABR_OFFSET       0x28
#define STM32_XSPI_DR_OFFSET        0x30
#define STM32_XSPI_PSMKR_OFFSET     0x34
#define STM32_XSPI_PSMAR_OFFSET     0x38
#define STM32_XSPI_CCR_OFFSET       0x3C
#define STM32_XSPI_TCR_OFFSET       0x40
#define STM32_XSPI_IR_OFFSET        0x44
#define STM32_XSPI_ABR2_OFFSET      0x48
#define STM32_XSPI_WRAPCR_OFFSET    0x50

/* XSPI Base Addresses */
#define STM32_XSPI1_BASE            (STM32N6_PERIPH_BASE + 0x08025000)
#define STM32_XSPI2_BASE            (STM32N6_PERIPH_BASE + 0x0802A000)
#define STM32_XSPI3_BASE            (STM32N6_PERIPH_BASE + 0x0802D000)

/* XSPI Control Register (CR) */
#define XSPI_CR_EN                  (1 << 0)   /* Enable */
#define XSPI_CR_ABORT               (1 << 1)   /* Abort command */
#define XSPI_CR_DMAEN               (1 << 2)   /* DMA enable */
#define XSPI_CR_TCEN                (1 << 3)   /* Timeout counter enable */
#define XSPI_CR_SSLBKE              (1 << 6)   /* SSL low-time block enable */
#define XSPI_CR_SSHIFTEN            (1 << 7)   /* Sample shift enable */
#define XSPI_CR_WRAPEN              (1 << 8)   /* Wrap mode enable */
#define XSPI_CR_FSEL                (1 << 16)  /* Flash selection */
#define XSPI_CR_MONO                (1 << 17)  /* Mono SPI mode */
#define XSPI_CR_APMS                (1 << 22)  /* Automatic poll mode stop */
#define XSPI_CR_PRESCALER_SHIFT     24
#define XSPI_CR_PRESCALER_MASK      (0x0F << XSPI_CR_PRESCALER_SHIFT)

/* XSPI Status Register (SR) */
#define XSPI_SR_TEF                 (1 << 0)   /* Transfer error flag */
#define XSPI_SR_TCF                 (1 << 1)   /* Transfer complete flag */
#define XSPI_SR_FTF                 (1 << 2)   /* FIFO threshold flag */
#define XSPI_SR_SMF                 (1 << 3)   /* Status match flag */
#define XSPI_SR_TOF                 (1 << 4)   /* Timeout flag */
#define XSPI_SR_BUSY                (1 << 5)   /* Busy flag */
#define XSPI_SR_FARSF               (1 << 6)   /* FIFO access request service flag */

/* XSPI Communication Configuration Register (CCR) */
#define XSPI_CCR_DDRM               (1 << 31)  /* Double data rate mode */
#define XSPI_CCR_DHHC               (1 << 30)  /* DHHC delay hold half cycle */
#define XSPI_CCR_SIOO               (1 << 29)  /* Send instruction only once */
#define XSPI_CCR_FMODE_SHIFT        26
#define XSPI_CCR_FMODE_MASK         (3 << XSPI_CCR_FMODE_SHIFT)
#define XSPI_CCR_FMODE_IND_WRITE    0        /* Indirect write */
#define XSPI_CCR_FMODE_IND_READ     1        /* Indirect read */
#define XSPI_CCR_FMODE_AUTO_POLL    2        /* Automatic polling */
#define XSPI_CCR_FMODE_MEM_MAP      3        /* Memory-mapped */
#define XSPI_CCR_DMODE_SHIFT        24
#define XSPI_CCR_DMODE_MASK         (3 << XSPI_CCR_DMODE_SHIFT)
#define XSPI_CCR_DMODE_NONE         0        /* No data */
#define XSPI_CCR_DMODE_1LINE        1        /* 1-line data */
#define XSPI_CCR_DMODE_2LINE        2        /* 2-line data */
#define XSPI_CCR_DMODE_4LINE        3        /* 4-line data */
#define XSPI_CCR_DMODE_8LINE        3        /* 8-line data */
#define XSPI_CCR_DUMMY_CYCLES_SHIFT 18
#define XSPI_CCR_DUMMY_CYCLES_MASK  (0x1F << XSPI_CCR_DUMMY_CYCLES_SHIFT)
#define XSPI_CCR_ABSIZE_SHIFT       16
#define XSPI_CCR_ABSIZE_MASK        (3 << XSPI_CCR_ABSIZE_SHIFT)
#define XSPI_CCR_ABMODE_SHIFT       14
#define XSPI_CCR_ABMODE_MASK        (3 << XSPI_CCR_ABMODE_SHIFT)
#define XSPI_CCR_ADSIZE_SHIFT       12
#define XSPI_CCR_ADSIZE_MASK        (3 << XSPI_CCR_ADSIZE_SHIFT)
#define XSPI_CCR_ADMODE_SHIFT       10
#define XSPI_CCR_ADMODE_MASK        (3 << XSPI_CCR_ADMODE_SHIFT)
#define XSPI_CCR_IMODE_SHIFT        8
#define XSPI_CCR_IMODE_MASK         (3 << XSPI_CCR_IMODE_SHIFT)
#define XSPI_CCR_IMODE_NONE         0        /* No instruction */
#define XSPI_CCR_IMODE_1LINE        1        /* 1-line instruction */
#define XSPI_CCR_IMODE_2LINE        2        /* 2-line instruction */
#define XSPI_CCR_IMODE_4LINE        3        /* 4-line instruction */
#define XSPI_CCR_IMODE_8LINE        3        /* 8-line instruction */
#define XSPI_CCR_INSTRUCTION_SHIFT  0
#define XSPI_CCR_INSTRUCTION_MASK   0xFF

/* Functional modes */
#define XSPI_CCR_FMODE_IND_WRITE    0
#define XSPI_CCR_FMODE_IND_READ     1
#define XSPI_CCR_FMODE_AUTO_POLL    2
#define XSPI_CCR_FMODE_MEM_MAP      3

/* Instruction, Address, Data Modes */
#define XSPI_CCR_IMODE_NONE         0
#define XSPI_CCR_IMODE_1LINE        1
#define XSPI_CCR_IMODE_2LINE        2
#define XSPI_CCR_IMODE_4LINE        3
#define XSPI_CCR_IMODE_8LINE        3

#define XSPI_CCR_ADMODE_NONE        0
#define XSPI_CCR_ADMODE_1LINE       1
#define XSPI_CCR_ADMODE_2LINE       2
#define XSPI_CCR_ADMODE_4LINE       3
#define XSPI_CCR_ADMODE_8LINE       3

#define XSPI_CCR_DMODE_NONE         0
#define XSPI_CCR_DMODE_1LINE        1
#define XSPI_CCR_DMODE_2LINE        2
#define XSPI_CCR_DMODE_4LINE        3
#define XSPI_CCR_DMODE_8LINE        3

/* Default configuration */
#define XSPI_FIFO_THRESHOLD         1
#define XSPI_TIMEOUT_MS             1000

/****************************************************************************
 * Public Types
 ****************************************************************************/

struct stm32n6_xspi_s
{
  struct spi_dev_s    dev;          /* SPI device interface */
  uint32_t            base;         /* XSPI register base address */
  uint32_t            frequency;    /* Requested bus frequency */
  uint32_t            actual;       /* Actual bus frequency */
  uint8_t             nbits;        /* Width of word in bits (8 or 16) */
  uint8_t             mode;         /* Mode 0,1,2,3 */
  bool                devid;        /* Device ID */
  struct wdog_s       wd;           /* Watchdog for timeout */
  uint32_t            prescaler;    /* Prescaler value */
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

int stm32n6_xspi_initialize(uint32_t base);

#endif /* __ARCH_ARM_SRC_STM32N6_STM32N6_XSPI_H */