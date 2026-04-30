/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_mbox.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_MBOX_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_MBOX_H_

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "chip.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* MBOX (Mailbox) Base Address */

#define RA8P_MBOX_BASE                         (0x40007000)

/* MBOX Register Offsets */

/* Mailbox Control Register (MBXCR) */

#define RA8P_MBOX_MBXCR_OFFSET                (0x000)
#define RA8P_MBOX_MBXCR_MBEN                 (1 << 0)    /* Bit 0: Mailbox Enable */
#define RA8P_MBOX_MBXCR_MBRESET               (1 << 1)    /* Bit 1: Mailbox Reset */

/* Mailbox Interrupt Enable Register (MBXIER) */

#define RA8P_MBOX_MBXIER_OFFSET                (0x004)
#define RA8P_MBOX_MBXIER_MBIE                 (1 << 0)    /* Bit 0: Mailbox Interrupt Enable */

/* Mailbox Status Register (MBXSR) */

#define RA8P_MBOX_MBXSR_OFFSET                 (0x008)
#define RA8P_MBOX_MBXSR_MBR                 (1 << 0)    /* Bit 0: Mailbox Ready */
#define RA8P_MBOX_MBXSR_MBF                 (1 << 1)    /* Bit 1: Mailbox Full */
#define RA8P_MBOX_MBXSR_MBE                 (1 << 2)    /* Bit 2: Mailbox Empty */

/* Mailbox Data Register (MBXDR) */

#define RA8P_MBOX_MBXDR_OFFSET                 (0x00C)
#define RA8P_MBOX_MBXDR_DATA_MASK             (0xFFFFFFFF) /* Bits 0-31: Mailbox Data */

/* Mailbox Channel 0-7 Registers */

#define RA8P_MBOX_CH_OFFSET(n)                 (0x010 + ((n) * 0x10))

/* Channel Control Register (CCRn) */

#define RA8P_MBOX_CCR_OFFSET(n)                 (RA8P_MBOX_CH_OFFSET(n) + 0x000)
#define RA8P_MBOX_CCR_CE                       (1 << 0)    /* Bit 0: Channel Enable */
#define RA8P_MBOX_CCR_CRF                      (1 << 1)    /* Bit 1: Channel Received Flag */
#define RA8P_MBOX_CCR_CTF                      (1 << 2)    /* Bit 2: Channel Transmit Flag */

/* Channel Interrupt Enable Register (CIERn) */

#define RA8P_MBOX_CIER_OFFSET(n)                (RA8P_MBOX_CH_OFFSET(n) + 0x004)
#define RA8P_MBOX_CIER_CIE                      (1 << 0)    /* Bit 0: Channel Interrupt Enable */

/* Channel Data Register (CDRn) */

#define RA8P_MBOX_CDR_OFFSET(n)                 (RA8P_MBOX_CH_OFFSET(n) + 0x008)
#define RA8P_MBOX_CDR_DATA_MASK              (0xFFFFFFFF) /* Bits 0-31: Channel Data */

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* Mailbox channel number */

enum ra8p_mbox_channel_e
{
  RA8P_MBOX_CHANNEL_0 = 0,      /* Channel 0 */
  RA8P_MBOX_CHANNEL_1,            /* Channel 1 */
  RA8P_MBOX_CHANNEL_2,            /* Channel 2 */
  RA8P_MBOX_CHANNEL_3,            /* Channel 3 */
  RA8P_MBOX_CHANNEL_4,            /* Channel 4 */
  RA8P_MBOX_CHANNEL_5,            /* Channel 5 */
  RA8P_MBOX_CHANNEL_6,            /* Channel 6 */
  RA8P_MBOX_CHANNEL_7,            /* Channel 7 */
};

/* Mailbox callback structure */

struct ra8p_mbox_callback_s
{
  void (*received)(uint32_t data); /* Data received callback */
  void (*transmitted)(void);    /* Data transmitted callback */
  void *priv;                  /* Private data */
};

/* Mailbox configuration */

struct ra8p_mbox_config_s
{
  uint32_t base;                    /* MBOX base address */
  int irq;                          /* MBOX interrupt number */
  uint8_t num_channels;             /* Number of channels (1-8) */
  bool initialized;                 /* True if initialized */
};

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_MBOX_H */