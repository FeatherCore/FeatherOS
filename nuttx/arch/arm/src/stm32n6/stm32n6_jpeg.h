/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_jpeg.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_STM32N6_JPEG_H
#define __ARCH_ARM_SRC_STM32N6_STM32N6_JPEG_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <stdint.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_JPEG_BASE              (STM32N6_PERIPH_BASE + 0x08023000)

#define JPEG_CR_OFFSET               0x00
#define JPEG_SR_OFFSET               0x04
#define JPEG_CFR_OFFSET              0x08
#define JPEG_DOR_OFFSET              0x0C
#define JPEG_DIR_OFFSET              0x10
#define JPEG_QMEM0_OFFSET            0x14
#define JPEG_QMEM1_OFFSET            0x18
#define JPEG_QMEM2_OFFSET            0x1C
#define JPEG_QMEM3_OFFSET            0x20
#define JPEG_HUFFMIN_OFFSET          0x24
#define JPEG_HUFFBASE_OFFSET         0x28
#define JPEG_HUFFSYMB0_OFFSET        0x2C
#define JPEG_HUFFSYMB1_OFFSET        0x30
#define JPEG_HUFFSYMB2_OFFSET        0x34
#define JPEG_HUFFSYMB3_OFFSET        0x38
#define JPEG_HUFFSYMB4_OFFSET        0x3C
#define JPEG_CONFR0_OFFSET           0x40
#define JPEG_CONFR1_OFFSET           0x44
#define JPEG_CONFR2_OFFSET           0x48
#define JPEG_CONFR3_OFFSET           0x4C
#define JPEG_CONFR4_OFFSET           0x50
#define JPEG_CONFR5_OFFSET           0x54
#define JPEG_CONFR6_OFFSET           0x58
#define JPEG_CONFR7_OFFSET           0x5C
#define JPEG_ADDR_OFFSET             0x80
#define JPEG_IDR_OFFSET              0x00  /* When accessing internal memory */

/* JPEG Control Register (CR) */
#define JPEG_CR_JCEN                 (1 << 0)
#define JPEG_CR_IFTIE                (1 << 1)
#define JPEG_CR_OFTEIE               (1 << 2)
#define JPEG_CR_EOCIE                (1 << 3)
#define JPEG_CR_HPDIE                (1 << 4)
#define JPEG_CR_DMAINEN              (1 << 5)
#define JPEG_CR_DMAOUTEN             (1 << 6)
#define JPEG_CR_CMD_SHIFT            8
#define JPEG_CR_CMD_MASK             (3 << JPEG_CR_CMD_SHIFT)
#define JPEG_CR_CMD_ENCODE           (0 << JPEG_CR_CMD_SHIFT)
#define JPEG_CR_CMD_DECODE           (1 << JPEG_CR_CMD_SHIFT)
#define JPEG_CR_START               (1 << 12)

/* JPEG Status Register (SR) */
#define JPEG_SR_IFTF                 (1 << 0)
#define JPEG_SR_OFTR                 (1 << 1)
#define JPEG_SR_EOCF                 (1 << 2)
#define JPEG_SR_HPDF                 (1 << 3)
#define JPEG_SR_COFHF                (1 << 4)

/* JPEG Clear Flag Register (CFR) */
#define JPEG_CFR_CIFTF               (1 << 0)
#define JPEG_CFR_COFTR               (1 << 1)
#define JPEG_CFR_CEOCF               (1 << 2)
#define JPEG_CFR_CHPDF               (1 << 3)
#define JPEG_CFR_CCOFHF              (1 << 4)

/* JPEG Input Data Register (DIR) */
/* Data is written to this register for encoding */

/* JPEG Output Data Register (DOR) */
/* Data is read from this register for decoding */

/* Command values */
#define JPEG_CMD_ENCODE              0
#define JPEG_CMD_DECODE              1
#define JPEG_CMD_PAUSE               2
#define JPEG_CMD_RESUME              3

/* Format values */
#define JPEG_FORMAT_YCBCR422         0
#define JPEG_FORMAT_YCBCR420         1
#define JPEG_FORMAT_GRAYSCALE        2
#define JPEG_FORMAT_CMYK             3

/****************************************************************************
 * Public Types
 ****************************************************************************/

struct stm32n6_jpeg_s
{
  uintptr_t jpegbase;
  uint8_t format;
  uint16_t width;
  uint16_t height;
  int irq;
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

int stm32n6_jpeg_initialize(uintptr_t jpegbase, uint8_t format, uint16_t width, uint16_t height);
void stm32n6_jpeg_enable(uintptr_t jpegbase);
void stm32n6_jpeg_disable(uintptr_t jpegbase);
int stm32n6_jpeg_encode(uintptr_t jpegbase, const uint8_t *input, size_t input_size,
                        uint8_t *output, size_t *output_size);
int stm32n6_jpeg_decode(uintptr_t jpegbase, const uint8_t *input, size_t input_size,
                        uint8_t *output, size_t *output_size);
void stm32n6_jpeg_pause(uintptr_t jpegbase);
void stm32n6_jpeg_resume(uintptr_t jpegbase);
void stm32n6_jpeg_reset(uintptr_t jpegbase);
bool stm32n6_jpeg_busy(uintptr_t jpegbase);

#endif /* __ARCH_ARM_SRC_STM32N6_STM32N6_JPEG_H */