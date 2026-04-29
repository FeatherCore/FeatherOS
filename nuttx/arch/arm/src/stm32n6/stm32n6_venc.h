/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_venc.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_STM32N6_VENC_H
#define __ARCH_ARM_SRC_STM32N6_STM32N6_VENC_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <stdint.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_VENC_BASE              (STM32N6_PERIPH_BASE + 0x08005000)

#define VENC_CR_OFFSET               0x00
#define VENC_SR_OFFSET               0x04
#define VENC_CFR_OFFSET              0x08
#define VENC_IER_OFFSET              0x0C
#define VENC_ISR_OFFSET              0x10
#define VENC_ICR_OFFSET              0x14
#define VENC_CTR1_OFFSET             0x18
#define VENC_CTR2_OFFSET             0x1C
#define VENC_CTR3_OFFSET             0x20
#define VENC_CTR4_OFFSET             0x24
#define VENC_CTR5_OFFSET             0x28
#define VENC_CTR6_OFFSET             0x2C
#define VENC_CTR7_OFFSET             0x30
#define VENC_CTR8_OFFSET             0x34
#define VENC_CTR9_OFFSET             0x38
#define VENC_CTR10_OFFSET            0x3C
#define VENC_CTR11_OFFSET            0x40
#define VENC_CTR12_OFFSET            0x44
#define VENC_CTR13_OFFSET            0x48
#define VENC_CTR14_OFFSET            0x4C
#define VENC_CTR15_OFFSET            0x50
#define VENC_CTR16_OFFSET            0x54
#define VENC_CTR17_OFFSET            0x58
#define VENC_CTR18_OFFSET            0x5C
#define VENC_CTR19_OFFSET            0x60
#define VENC_CTR20_OFFSET            0x64
#define VENC_CTR21_OFFSET            0x68
#define VENC_CTR22_OFFSET            0x6C
#define VENC_CTR23_OFFSET            0x70
#define VENC_CTR24_OFFSET            0x74
#define VENC_CTR25_OFFSET            0x78
#define VENC_CTR26_OFFSET            0x7C
#define VENC_CTR27_OFFSET            0x80
#define VENC_CTR28_OFFSET            0x84
#define VENC_CTR29_OFFSET            0x88
#define VENC_CTR30_OFFSET            0x8C
#define VENC_CTR31_OFFSET            0x90
#define VENC_CTR32_OFFSET            0x94
#define VENC_CTR33_OFFSET            0x98
#define VENC_CTR34_OFFSET            0x9C
#define VENC_CTR35_OFFSET            0xA0
#define VENC_CTR36_OFFSET            0xA4
#define VENC_CTR37_OFFSET            0xA8
#define VENC_CTR38_OFFSET            0xAC
#define VENC_CTR39_OFFSET            0xB0
#define VENC_CTR40_OFFSET            0xB4
#define VENC_CTR41_OFFSET            0xB8
#define VENC_CTR42_OFFSET            0xBC
#define VENC_CTR43_OFFSET            0xC0
#define VENC_CTR44_OFFSET            0xC4
#define VENC_CTR45_OFFSET            0xC8
#define VENC_CTR46_OFFSET            0xCC
#define VENC_CTR47_OFFSET            0xD0
#define VENC_CTR48_OFFSET            0xD4
#define VENC_CTR49_OFFSET            0xD8
#define VENC_CTR50_OFFSET            0xDC
#define VENC_CTR51_OFFSET            0xE0
#define VENC_CTR52_OFFSET            0xE4
#define VENC_CTR53_OFFSET            0xE8
#define VENC_CTR54_OFFSET            0xEC
#define VENC_CTR55_OFFSET            0xF0
#define VENC_CTR56_OFFSET            0xF4
#define VENC_CTR57_OFFSET            0xF8
#define VENC_CTR58_OFFSET            0xFC
#define VENC_CTR59_OFFSET            0x100
#define VENC_CTR60_OFFSET            0x104
#define VENC_CTR61_OFFSET            0x108
#define VENC_CTR62_OFFSET            0x10C
#define VENC_CTR63_OFFSET            0x110
#define VENC_CTR64_OFFSET            0x114
#define VENC_CTR65_OFFSET            0x118
#define VENC_CTR66_OFFSET            0x11C
#define VENC_CTR67_OFFSET            0x120
#define VENC_CTR68_OFFSET            0x124
#define VENC_CTR69_OFFSET            0x128
#define VENC_CTR70_OFFSET            0x12C
#define VENC_CTR71_OFFSET            0x130
#define VENC_CTR72_OFFSET            0x134
#define VENC_CTR73_OFFSET            0x138
#define VENC_CTR74_OFFSET            0x13C
#define VENC_CTR75_OFFSET            0x140
#define VENC_CTR76_OFFSET            0x144
#define VENC_CTR77_OFFSET            0x148
#define VENC_CTR78_OFFSET            0x14C
#define VENC_CTR79_OFFSET            0x150
#define VENC_CTR80_OFFSET            0x154
#define VENC_CTR81_OFFSET            0x158
#define VENC_CTR82_OFFSET            0x15C
#define VENC_CTR83_OFFSET            0x160
#define VENC_CTR84_OFFSET            0x164
#define VENC_CTR85_OFFSET            0x168
#define VENC_CTR86_OFFSET            0x16C
#define VENC_CTR87_OFFSET            0x170
#define VENC_CTR88_OFFSET            0x174
#define VENC_CTR89_OFFSET            0x178
#define VENC_CTR90_OFFSET            0x17C
#define VENC_CTR91_OFFSET            0x180
#define VENC_CTR92_OFFSET            0x184
#define VENC_CTR93_OFFSET            0x188
#define VENC_CTR94_OFFSET            0x18C
#define VENC_CTR95_OFFSET            0x190
#define VENC_CTR96_OFFSET            0x194
#define VENC_CTR97_OFFSET            0x198
#define VENC_CTR98_OFFSET            0x19C
#define VENC_CTR99_OFFSET            0x1A0
#define VENC_CTR100_OFFSET           0x1A4
#define VENC_CTR101_OFFSET           0x1A8
#define VENC_CTR102_OFFSET           0x1AC
#define VENC_CTR103_OFFSET           0x1B0
#define VENC_CTR104_OFFSET           0x1B4
#define VENC_CTR105_OFFSET           0x1B8
#define VENC_CTR106_OFFSET           0x1BC
#define VENC_CTR107_OFFSET           0x1C0
#define VENC_CTR108_OFFSET           0x1C4
#define VENC_CTR109_OFFSET           0x1C8
#define VENC_CTR110_OFFSET           0x1CC
#define VENC_CTR111_OFFSET           0x1D0
#define VENC_CTR112_OFFSET           0x1D4
#define VENC_CTR113_OFFSET           0x1D8
#define VENC_CTR114_OFFSET           0x1DC
#define VENC_CTR115_OFFSET           0x1E0
#define VENC_CTR116_OFFSET           0x1E4
#define VENC_CTR117_OFFSET           0x1E8
#define VENC_CTR118_OFFSET           0x1EC
#define VENC_CTR119_OFFSET           0x1F0
#define VENC_CTR120_OFFSET           0x1F4
#define VENC_CTR121_OFFSET           0x1F8
#define VENC_CTR122_OFFSET           0x1FC
#define VENC_CTR123_OFFSET           0x200
#define VENC_CTR124_OFFSET           0x204
#define VENC_CTR125_OFFSET           0x208
#define VENC_CTR126_OFFSET           0x20C
#define VENC_CTR127_OFFSET           0x210
#define VENC_CTR128_OFFSET           0x214

/* VENC Control Register (CR) */
#define VENC_CR_EN                   (1 << 0)
#define VENC_CR_START                (1 << 1)
#define VENC_CR_SUSP                 (1 << 2)
#define VENC_CR_ABORT                (1 << 3)
#define VENC_CR_DMAINEN              (1 << 4)
#define VENC_CR_DMAOUTEN             (1 << 5)
#define VENC_CR_IRQEN                (1 << 6)

/* VENC Status Register (SR) */
#define VENC_SR_BUSY                 (1 << 0)
#define VENC_SR_EOCF                 (1 << 1)
#define VENC_SR_PAUSF                (1 << 2)
#define VENC_SR_ERRF                 (1 << 3)

/* VENC Interrupt Enable Register (IER) */
#define VENC_IER_EOCIE               (1 << 1)
#define VENC_IER_PAUSIE              (1 << 2)
#define VENC_IER_ERRIE               (1 << 3)

/* VENC Interrupt Status Register (ISR) */
#define VENC_ISR_EOCF                (1 << 1)
#define VENC_ISR_PAUSF               (1 << 2)
#define VENC_ISR_ERRF                (1 << 3)

/* Video formats */
#define VENC_FORMAT_YUV420           0
#define VENC_FORMAT_YUV422           1
#define VENC_FORMAT_NV12             2
#define VENC_FORMAT_RGB565           3
#define VENC_FORMAT_ARGB8888         4

/* Video codecs */
#define VENC_CODEC_H264              0
#define VENC_CODEC_HEVC              1

/****************************************************************************
 * Public Types
 ****************************************************************************/

struct stm32n6_venc_s
{
  uintptr_t vencbase;
  uint8_t format;
  uint8_t codec;
  uint16_t width;
  uint16_t height;
  uint32_t bitrate;
  uint8_t quality;
  int irq;
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

int stm32n6_venc_initialize(uintptr_t vencbase, uint8_t format, uint8_t codec, 
                            uint16_t width, uint16_t height);
void stm32n6_venc_enable(uintptr_t vencbase);
void stm32n6_venc_disable(uintptr_t vencbase);
int stm32n6_venc_encode_frame(uintptr_t vencbase, const uint8_t *input, 
                              size_t input_size, uint8_t *output, 
                              size_t *output_size);
void stm32n6_venc_start(uintptr_t vencbase);
void stm32n6_venc_stop(uintptr_t vencbase);
void stm32n6_venc_suspend(uintptr_t vencbase);
void stm32n6_venc_resume(uintptr_t vencbase);
bool stm32n6_venc_busy(uintptr_t vencbase);

#endif /* __ARCH_ARM_SRC_STM32N6_STM32N6_VENC_H */