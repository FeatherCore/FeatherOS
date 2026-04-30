/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_crc.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_CRC_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_CRC_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "chip.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* CRC Base Address */

#define RA8P_CRC_BASE                           (0x40226000)

/* CRC Register Offsets */

/* CRC Control Register (CRCCR) */

#define RA8P_CRC_CRCCR_OFFSET                   (0x000)
#define RA8P_CRC_CRCCR_CRCEN                    (1 << 0)    /* Bit 0: CRC Enable */
#define RA8P_CRC_CRCCR_CRCS                     (1 << 1)    /* Bit 1: CRC Start */
#define RA8P_CRC_CRCCR_CRCRST                   (1 << 2)    /* Bit 2: CRC Reset */
#define RA8P_CRC_CRCCR_CRCDIR                   (1 << 3)    /* Bit 3: CRC Direction */
#define RA8P_CRC_CRCCR_CRCDIR_LSB               (0 << 3)    /* LSB first */
#define RA8P_CRC_CRCCR_CRCDIR_MSB               (1 << 3)    /* MSB first */
#define RA8P_CRC_CRCCR_CRCPOL_MASK              (0x70)      /* Bits 4-6: CRC Polynomial Select */
#define RA8P_CRC_CRCCR_CRCPOL_SHIFT             (4)
#define RA8P_CRC_CRCCR_CRCPOL_CRC8              (0x0 << 4)  /* CRC-8 */
#define RA8P_CRC_CRCCR_CRCPOL_CRC16             (0x1 << 4)  /* CRC-16 */
#define RA8P_CRC_CRCCR_CRCPOL_CRC_CCITT         (0x2 << 4)  /* CRC-CCITT */
#define RA8P_CRC_CRCCR_CRCPOL_CRC32             (0x3 << 4)  /* CRC-32 */
#define RA8P_CRC_CRCCR_CRCPOL_CRC32C            (0x4 << 4)  /* CRC-32C */
#define RA8P_CRC_CRCCR_LMS                      (1 << 7)    /* Bit 7: LMS Mode */
#define RA8P_CRC_CRCCR_DORSEL_MASK              (0x300)     /* Bits 8-9: Data Order Select */
#define RA8P_CRC_CRCCR_DORSEL_SHIFT             (8)
#define RA8P_CRC_CRCCR_DORSEL_8BIT              (0x0 << 8)  /* 8-bit */
#define RA8P_CRC_CRCCR_DORSEL_16BIT             (0x1 << 8)  /* 16-bit */
#define RA8P_CRC_CRCCR_DORSEL_32BIT             (0x2 << 8)  /* 32-bit */

/* CRC Data Input Register (CRCDIR) */

#define RA8P_CRC_CRCDIR_OFFSET                  (0x004)
#define RA8P_CRC_CRCDIR_DATA_MASK               (0xFFFFFFFF) /* Bits 0-31: CRC Data Input */

/* CRC Data Output Register (CRCDOR) */

#define RA8P_CRC_CRCDOR_OFFSET                  (0x008)
#define RA8P_CRC_CRCDOR_DATA_MASK               (0xFFFFFFFF) /* Bits 0-31: CRC Data Output */

/* CRC Status Register (CRCSTR) */

#define RA8P_CRC_CRCSTR_OFFSET                  (0x00C)
#define RA8P_CRC_CRCSTR_CRCST                   (1 << 0)    /* Bit 0: CRC Status */
#define RA8P_CRC_CRCSTR_CRCINT                  (1 << 1)    /* Bit 1: CRC Interrupt */

/* CRC Initial Value Register (CRCINIT) */

#define RA8P_CRC_CRCINIT_OFFSET                 (0x010)
#define RA8P_CRC_CRCINIT_INIT_MASK              (0xFFFFFFFF) /* Bits 0-31: CRC Initial Value */

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* CRC polynomial types */

enum ra8p_crc_polynomial_e
{
  RA8P_CRC_POLY_CRC8 = 0,           /* CRC-8: x^8 + x^2 + x^1 + 1 */
  RA8P_CRC_POLY_CRC16,              /* CRC-16: x^16 + x^15 + x^2 + 1 */
  RA8P_CRC_POLY_CRC_CCITT,          /* CRC-CCITT: x^16 + x^12 + x^5 + 1 */
  RA8P_CRC_POLY_CRC32,              /* CRC-32: IEEE 802.3 */
  RA8P_CRC_POLY_CRC32C,             /* CRC-32C: Castagnoli */
};

/* CRC bit order */

enum ra8p_crc_bit_order_e
{
  RA8P_CRC_BIT_ORDER_MSB = 0,       /* MSB first */
  RA8P_CRC_BIT_ORDER_LSB,          /* LSB first */
};

/* CRC data width */

enum ra8p_crc_data_width_e
{
  RA8P_CRC_DATA_WIDTH_8BIT = 0,     /* 8-bit data */
  RA8P_CRC_DATA_WIDTH_16BIT,        /* 16-bit data */
  RA8P_CRC_DATA_WIDTH_32BIT,        /* 32-bit data */
};

/* CRC configuration */

struct ra8p_crc_config_s
{
  enum ra8p_crc_polynomial_e polynomial; /* CRC polynomial */
  enum ra8p_crc_bit_order_e bit_order;   /* Bit order */
  enum ra8p_crc_data_width_e data_width; /* Data width */
  uint32_t seed;                         /* Initial CRC value */
};

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_CRC_H */