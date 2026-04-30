/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_i2s.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_I2S_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_I2S_H_

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "chip.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* I2S/SSIE Base Address */

#define RA8P_I2S0_BASE                          (0x4025d000)

/* I2S Register Offsets */

/* SSIE Control Register 0 (SSICR0) */

#define RA8P_I2S_SSICR0_OFFSET                 (0x000)
#define RA8P_I2S_SSICR0_SSIEN                (1 << 0)    /* Bit 0: I2S Enable */
#define RA8P_I2S_SSICR0_AFS                    (1 << 1)    /* Bit 1: Audio Format Select */
#define RA8P_I2S_SSICR0_BCKP                   (1 << 2)    /* Bit 2: BCK Polarity */
#define RA8P_I2S_SSICR0_LRP                    (1 << 3)    /* Bit 3: LRCK Polarity */
#define RA8P_I2S_SSICR0_MST                    (1 << 4)    /* Bit 4: Master Mode */
#define RA8P_I2S_SSICR0_SWL_MASK              (0x70)      /* Bits 5-7: System Word Length */
#define RA8P_I2S_SSICR0_SWL_SHIFT             (5)
#define RA8P_I2S_SSICR0_SWL_8BIT             (0x0 << 5)  /* 8-bit */
#define RA8P_I2S_SSICR0_SWL_16BIT            (0x1 << 5)  /* 16-bit */
#define RA8P_I2S_SSICR0_SWL_24BIT            (0x2 << 5)  /* 24-bit */
#define RA8P_I2S_SSICR0_SWL_32BIT            (0x3 << 5)  /* 32-bit */
#define RA8P_I2S_SSICR0_DWL_MASK              (0x380)     /* Bits 8-10: Data Word Length */
#define RA8P_I2S_SSICR0_DWL_SHIFT             (8)
#define RA8P_I2S_SSICR0_DWL_8BIT             (0x0 << 8)  /* 8-bit */
#define RA8P_I2S_SSICR0_DWL_16BIT            (0x1 << 8)  /* 16-bit */
#define RA8P_I2S_SSICR0_DWL_24BIT            (0x2 << 8)  /* 24-bit */
#define RA8P_I2S_SSICR0_DWL_32BIT            (0x3 << 8)  /* 32-bit */
#define RA8P_I2S_SSICR0_TEN                    (1 << 11)   /* Bit 11: Transmit Enable */
#define RA8P_I2S_SSICR0_REN                    (1 << 12)   /* Bit 12: Receive Enable */

/* SSIE Control Register 1 (SSICR1) */

#define RA8P_I2S_SSICR1_OFFSET                 (0x004)
#define RA8P_I2S_SSICR1_TCHNL_MASK           (0x7)       /* Bits 0-2: Transmit Channels */
#define RA8P_I2S_SSICR1_TCHNL_SHIFT          (0)
#define RA8P_I2S_SSICR1_RCHNL_MASK           (0x70)      /* Bits 4-6: Receive Channels */
#define RA8P_I2S_SSICR1_RCHNL_SHIFT          (4)
#define RA8P_I2S_SSICR1_TDTV_MASK            (0xF00)     /* Bits 8-11: Transmit Delay */
#define RA8P_I2S_SSICR1_TDTV_SHIFT           (8)
#define RA8P_I2S_SSICR1_RDTV_MASK            (0xF000)    /* Bits 12-15: Receive Delay */
#define RA8P_I2S_SSICR1_RDTV_SHIFT           (12)

/* SSIE Transmit FIFO Control Register (SSITFCR) */

#define RA8P_I2S_SSITFCR_OFFSET                (0x008)
#define RA8P_I2S_SSITFCR_TFRST               (1 << 0)    /* Bit 0: Transmit FIFO Reset */
#define RA8P_I2S_SSITFCR_TFIE                (1 << 1)    /* Bit 1: Transmit FIFO Interrupt Enable */

/* SSIE Receive FIFO Control Register (SSIRFCR) */

#define RA8P_I2S_SSIRFCR_OFFSET                (0x00C)
#define RA8P_I2S_SSIRFCR_RFRST               (1 << 0)    /* Bit 0: Receive FIFO Reset */
#define RA8P_I2S_SSIRFCR_RFIE                (1 << 1)    /* Bit 1: Receive FIFO Interrupt Enable */

/* SSIE Status Register (SSISR) */

#define RA8P_I2S_SSISR_OFFSET                  (0x010)
#define RA8P_I2S_SSISR_TFEMP                  (1 << 0)    /* Bit 0: Transmit FIFO Empty */
#define RA8P_I2S_SSISR_TFFUL                  (1 << 1)    /* Bit 1: Transmit FIFO Full */
#define RA8P_I2S_SSISR_RFEMP                  (1 << 2)    /* Bit 2: Receive FIFO Empty */
#define RA8P_I2S_SSISR_RFFUL                  (1 << 3)    /* Bit 3: Receive FIFO Full */
#define RA8P_I2S_SSISR_TFINT                  (1 << 4)    /* Bit 4: Transmit FIFO Interrupt */
#define RA8P_I2S_SSISR_RFINT                  (1 << 5)    /* Bit 5: Receive FIFO Interrupt */

/* SSIE Transmit FIFO Data Register (SSITDR) */

#define RA8P_I2S_SSITDR_OFFSET                 (0x014)
#define RA8P_I2S_SSITDR_DATA_MASK             (0xFFFFFFFF) /* Bits 0-31: Transmit Data */

/* SSIE Receive FIFO Data Register (SSIRDR) */

#define RA8P_I2S_SSIRDR_OFFSET                 (0x018)
#define RA8P_I2S_SSIRDR_DATA_MASK             (0xFFFFFFFF) /* Bits 0-31: Receive Data */

/* SSIE Clock Control Register (SSICCR) */

#define RA8P_I2S_SSICCR_OFFSET                 (0x01C)
#define RA8P_I2S_SSICCR_CKDIV_MASK            (0xFF)      /* Bits 0-7: Clock Divider */
#define RA8P_I2S_SSICCR_CKDIV_SHIFT           (0)

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* I2S audio format */

enum ra8p_i2s_format_e
{
  RA8P_I2S_FORMAT_I2S = 0,         /* Standard I2S format */
  RA8P_I2S_FORMAT_LEFT_JUSTIFIED,    /* Left justified */
  RA8P_I2S_FORMAT_RIGHT_JUSTIFIED,   /* Right justified */
  RA8P_I2S_FORMAT_DSP,              /* DSP format */
};

/* I2S word length */

enum ra8p_i2s_word_length_e
{
  RA8P_I2S_WLEN_8BIT = 0,           /* 8-bit */
  RA8P_I2S_WLEN_16BIT,             /* 16-bit */
  RA8P_I2S_WLEN_24BIT,             /* 24-bit */
  RA8P_I2S_WLEN_32BIT,             /* 32-bit */
};

/* I2S configuration */

struct ra8p_i2s_config_s
{
  uint32_t base;                    /* I2S base address */
  int irq;                          /* I2S interrupt number */
  enum ra8p_i2s_format_e format;   /* Audio format */
  enum ra8p_i2s_word_length_e word_len; /* Word length */
  uint32_t sample_rate;              /* Sample rate in Hz */
  uint8_t channels;                 /* Number of channels (1-8) */
  bool master_mode;                /* True for master, false for slave */
  bool tx_enable;                  /* Enable transmit */
  bool rx_enable;                  /* Enable receive */
};

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_I2S_H */