/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_dmac.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_DMAC_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_DMAC_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* DMAC Base Address */

#define RA8P_DMAC_BASE            0x40005000

/* DMAC Register Offsets */

#define RA8P_DMAC_DMACST          0x0000    /* DMAC Status Register */
#define RA8P_DMAC_DMACEN          0x0004    /* DMAC Enable Register */
#define RA8P_DMAC_DMACT           0x0008    /* DMAC Trigger Register */
#define RA8P_DMAC_DMACTLS         0x000C    /* DMAC Transfer Status Register */
#define RA8P_DMAC_DMACTLC         0x0010    /* DMAC Transfer Clear Register */
#define RA8P_DMAC_DMCTRL          0x0014    /* DMAC Control Register */
#define RA8P_DMAC_DMACFG          0x0018    /* DMAC Configuration Register */
#define RA8P_DMAC_DMASTS          0x001C    /* DMAC Status Register */

/* Channel Registers (0-7) */

#define RA8P_DMAC_CH_OFFSET(n)    (0x0020 + ((n) * 0x40))

#define RA8P_DMAC_CHCTRL(n)       (RA8P_DMAC_CH_OFFSET(n) + 0x00)  /* Channel Control Register */
#define RA8P_DMAC_CHCFG(n)        (RA8P_DMAC_CH_OFFSET(n) + 0x04)  /* Channel Configuration Register */
#define RA8P_DMAC_CHNX(n)         (RA8P_DMAC_CH_OFFSET(n) + 0x08)  /* Channel Next Register */
#define RA8P_DMAC_CHXFERCFG(n)    (RA8P_DMAC_CH_OFFSET(n) + 0x0C)  /* Channel Transfer Configuration Register */
#define RA8P_DMAC_CHSRC(n)        (RA8P_DMAC_CH_OFFSET(n) + 0x10)  /* Channel Source Address Register */
#define RA8P_DMAC_CHDST(n)        (RA8P_DMAC_CH_OFFSET(n) + 0x14)  /* Channel Destination Address Register */
#define RA8P_DMAC_CHXFER(n)       (RA8P_DMAC_CH_OFFSET(n) + 0x18)  /* Channel Transfer Count Register */
#define RA8P_DMAC_CHXFERCNT(n)    (RA8P_DMAC_CH_OFFSET(n) + 0x1C)  /* Channel Transfer Counter Register */
#define RA8P_DMAC_CHXFERCNTL(n)   (RA8P_DMAC_CH_OFFSET(n) + 0x20)  /* Channel Transfer Counter Low Register */
#define RA8P_DMAC_CHXFERCNTH(n)   (RA8P_DMAC_CH_OFFSET(n) + 0x24)  /* Channel Transfer Counter High Register */
#define RA8P_DMAC_CHXFERCNTS(n)   (RA8P_DMAC_CH_OFFSET(n) + 0x28)  /* Channel Transfer Counter Status Register */
#define RA8P_DMAC_CHXFERCNTSL(n)  (RA8P_DMAC_CH_OFFSET(n) + 0x2C)  /* Channel Transfer Counter Status Low Register */
#define RA8P_DMAC_CHXFERCNTSH(n)  (RA8P_DMAC_CH_OFFSET(n) + 0x30)  /* Channel Transfer Counter Status High Register */

/* DMACST - DMAC Status Register */

#define DMAC_DMACST_DMST          (1 << 0)    /* DMAC Status */

/* DMACEN - DMAC Enable Register */

#define DMAC_DMACEN_DMEN          (1 << 0)    /* DMAC Enable */

/* DMACT - DMAC Trigger Register */

#define DMAC_DMACT_DMT_MASK       (0xFF << 0) /* DMAC Trigger Mask */
#define DMAC_DMACT_DMT_SHIFT      0

/* DMACTLS - DMAC Transfer Status Register */

#define DMAC_DMACTLS_DMTLS_MASK   (0xFF << 0) /* DMAC Transfer Status Mask */
#define DMAC_DMACTLS_DMTLS_SHIFT  0

/* DMACTLC - DMAC Transfer Clear Register */

#define DMAC_DMACTLC_DMTLC_MASK   (0xFF << 0) /* DMAC Transfer Clear Mask */
#define DMAC_DMACTLC_DMTLC_SHIFT  0

/* DMCTRL - DMAC Control Register */

#define DMAC_DMCTRL_DMCTRL_MASK   (0xFF << 0) /* DMAC Control Mask */
#define DMAC_DMCTRL_DMCTRL_SHIFT  0

/* DMACFG - DMAC Configuration Register */

#define DMAC_DMACFG_DMACFG_MASK   (0xFF << 0) /* DMAC Configuration Mask */
#define DMAC_DMACFG_DMACFG_SHIFT  0

/* CHCTRL - Channel Control Register */

#define DMAC_CHCTRL_SET           (1 << 0)    /* Channel Set */
#define DMAC_CHCTRL_CLR           (1 << 1)    /* Channel Clear */
#define DMAC_CHCTRL_STG           (1 << 2)    /* Channel Software Trigger */
#define DMAC_CHCTRL_END           (1 << 3)    /* Channel End */
#define DMAC_CHCTRL_TC            (1 << 4)    /* Channel Transfer Complete */
#define DMAC_CHCTRL_ES            (1 << 5)    /* Channel Error Status */
#define DMAC_CHCTRL_EN            (1 << 7)    /* Channel Enable */

/* CHCFG - Channel Configuration Register */

#define DMAC_CHCFG_SEL_MASK       (0x07 << 0)  /* Channel Select Mask */
#define DMAC_CHCFG_SEL_SHIFT      0
#define DMAC_CHCFG_REQD           (1 << 3)     /* Request Direction */
#define DMAC_CHCFG_LOEN           (1 << 4)     /* List End Enable */
#define DMAC_CHCFG_HIEN           (1 << 5)     /* Hardware Interrupt Enable */
#define DMAC_CHCFG_LVL_MASK       (0x03 << 6)  /* Channel Level Mask */
#define DMAC_CHCFG_LVL_SHIFT      6
#define DMAC_CHCFG_LVL_LOW        (0 << 6)     /* Low Level */
#define DMAC_CHCFG_LVL_MEDIUM     (1 << 6)     /* Medium Level */
#define DMAC_CHCFG_LVL_HIGH       (2 << 6)     /* High Level */
#define DMAC_CHCFG_LVL_HIGHEST    (3 << 6)     /* Highest Level */
#define DMAC_CHCFG_THC            (1 << 8)     /* Transfer Half Complete */
#define DMAC_CHCFG_RSW            (1 << 9)     /* Resource Switch */
#define DMAC_CHCFG_SBE            (1 << 10)    /* Source Burst Enable */
#define DMAC_CHCFG_DBE            (1 << 11)    /* Destination Burst Enable */
#define DMAC_CHCFG_DEM            (1 << 12)    /* Destination Enable Mode */
#define DMAC_CHCFG_TCM            (1 << 13)    /* Transfer Complete Mode */
#define DMAC_CHCFG_ALM            (1 << 14)    /* Auto List Mode */
#define DMAC_CHCFG_REN            (1 << 15)    /* Repeat Enable */
#define DMAC_CHCFG_RTM_MASK       (0x03 << 16) /* Repeat Mode Mask */
#define DMAC_CHCFG_RTM_SHIFT      16
#define DMAC_CHCFG_RTM_NORMAL     (0 << 16)    /* Normal Mode */
#define DMAC_CHCFG_RTM_REPEAT     (1 << 16)    /* Repeat Mode */
#define DMAC_CHCFG_RTM_BLOCK      (2 << 16)    /* Block Repeat Mode */
#define DMAC_CHCFG_AM_MASK        (0x07 << 20) /* Address Mode Mask */
#define DMAC_CHCFG_AM_SHIFT       20
#define DMAC_CHCFG_AM_INCREMENT   (0 << 20)    /* Increment */
#define DMAC_CHCFG_AM_DECREMENT   (1 << 20)    /* Decrement */
#define DMAC_CHCFG_AM_FIXED       (2 << 20)    /* Fixed */
#define DMAC_CHCFG_SAD_MASK       (0x03 << 24) /* Source Address Direction Mask */
#define DMAC_CHCFG_SAD_SHIFT      24
#define DMAC_CHCFG_DAD_MASK       (0x03 << 26) /* Destination Address Direction Mask */
#define DMAC_CHCFG_DAD_SHIFT      26
#define DMAC_CHCFG_TM_MASK        (0x03 << 28) /* Transfer Mode Mask */
#define DMAC_CHCFG_TM_SHIFT       28
#define DMAC_CHCFG_TM_NORMAL      (0 << 28)    /* Normal Transfer */
#define DMAC_CHCFG_TM_REPEAT      (1 << 28)    /* Repeat Transfer */
#define DMAC_CHCFG_TM_BLOCK       (2 << 28)    /* Block Transfer */
#define DMAC_CHCFG_TM_BLOCK_RPT   (3 << 28)    /* Block Repeat Transfer */
#define DMAC_CHCFG_DES_MASK       (0x03 << 30) /* Destination Enable Select Mask */
#define DMAC_CHCFG_DES_SHIFT      30
#define DMAC_CHCFG_DES_DISABLE    (0 << 30)    /* Disable */
#define DMAC_CHCFG_DES_ENABLE     (1 << 30)    /* Enable */
#define DMAC_CHCFG_DES_AUTO       (2 << 30)    /* Auto */

/* CHXFERCFG - Channel Transfer Configuration Register */

#define DMAC_CHXFERCFG_CTS_MASK   (0x03 << 0)  /* Clear Transfer Status Mask */
#define DMAC_CHXFERCFG_CTS_SHIFT  0
#define DMAC_CHXFERCFG_CTS_NONE   (0 << 0)     /* No Clear */
#define DMAC_CHXFERCFG_CTS_TC     (1 << 0)     /* Transfer Complete Clear */
#define DMAC_CHXFERCFG_CTS_ES     (2 << 0)     /* Error Status Clear */
#define DMAC_CHXFERCFG_CTS_ALL    (3 << 0)     /* All Clear */
#define DMAC_CHXFERCFG_SDS_MASK   (0x0F << 4)  /* Source Data Size Mask */
#define DMAC_CHXFERCFG_SDS_SHIFT  4
#define DMAC_CHXFERCFG_SDS_8BIT   (0 << 4)     /* 8-bit */
#define DMAC_CHXFERCFG_SDS_16BIT  (1 << 4)     /* 16-bit */
#define DMAC_CHXFERCFG_SDS_32BIT  (2 << 4)     /* 32-bit */
#define DMAC_CHXFERCFG_DDS_MASK   (0x0F << 8)  /* Destination Data Size Mask */
#define DMAC_CHXFERCFG_DDS_SHIFT  8
#define DMAC_CHXFERCFG_DDS_8BIT   (0 << 8)     /* 8-bit */
#define DMAC_CHXFERCFG_DDS_16BIT  (1 << 8)     /* 16-bit */
#define DMAC_CHXFERCFG_DDS_32BIT  (2 << 8)     /* 32-bit */
#define DMAC_CHXFERCFG_SAD_MASK   (0x03 << 12) /* Source Address Direction Mask */
#define DMAC_CHXFERCFG_SAD_SHIFT  12
#define DMAC_CHXFERCFG_DAD_MASK   (0x03 << 14) /* Destination Address Direction Mask */
#define DMAC_CHXFERCFG_DAD_SHIFT  14

/* Transfer sizes */

#define DMAC_TRANSFER_SIZE_1_BYTE   1
#define DMAC_TRANSFER_SIZE_2_BYTE   2
#define DMAC_TRANSFER_SIZE_4_BYTE   4

/* Transfer modes */

#define DMAC_TRANSFER_MODE_NORMAL   0
#define DMAC_TRANSFER_MODE_REPEAT   1
#define DMAC_TRANSFER_MODE_BLOCK    2

/* Address modes */

#define DMAC_ADDR_MODE_INCREMENTED  0
#define DMAC_ADDR_MODE_DECREMENTED  1
#define DMAC_ADDR_MODE_FIXED        2

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* DMAC Channel Configuration Structure */

struct ra8p_dmac_chcfg_s
{
  uint32_t src_addr;        /* Source address */
  uint32_t dst_addr;        /* Destination address */
  uint16_t length;          /* Transfer length */
  uint8_t  src_size;        /* Source data size (1, 2, or 4 bytes) */
  uint8_t  dst_size;        /* Destination data size (1, 2, or 4 bytes) */
  uint8_t  src_mode;        /* Source address mode */
  uint8_t  dst_mode;        /* Destination address mode */
  uint8_t  transfer_mode;   /* Transfer mode */
  uint8_t  channel;         /* Channel number (0-7) */
  uint8_t  irq;             /* IRQ number */
  uint8_t  priority;        /* Channel priority (0-3) */
};

/* DMAC Register Map */

struct ra8p_dmac_s
{
  volatile uint32_t dmacst;       /* DMAC Status Register */
  volatile uint32_t dmacen;       /* DMAC Enable Register */
  volatile uint32_t dmact;        /* DMAC Trigger Register */
  volatile uint32_t dmactls;      /* DMAC Transfer Status Register */
  volatile uint32_t dmactlc;      /* DMAC Transfer Clear Register */
  volatile uint32_t dmctrl;       /* DMAC Control Register */
  volatile uint32_t dmacfg;       /* DMAC Configuration Register */
  volatile uint32_t dmasts;       /* DMAC Status Register */
  volatile uint8_t  reserved0[8];
  struct
  {
    volatile uint32_t chctrl;     /* Channel Control Register */
    volatile uint32_t chcfg;      /* Channel Configuration Register */
    volatile uint32_t chnx;       /* Channel Next Register */
    volatile uint32_t chxfercfg;  /* Channel Transfer Configuration Register */
    volatile uint32_t chsrc;      /* Channel Source Address Register */
    volatile uint32_t chdst;      /* Channel Destination Address Register */
    volatile uint32_t chxfer;     /* Channel Transfer Count Register */
    volatile uint32_t chxfercnt;  /* Channel Transfer Counter Register */
    volatile uint32_t chxfercntl; /* Channel Transfer Counter Low Register */
    volatile uint32_t chxfercnth; /* Channel Transfer Counter High Register */
    volatile uint32_t chxfercnts; /* Channel Transfer Counter Status Register */
    volatile uint32_t chxfercntsl;/* Channel Transfer Counter Status Low Register */
    volatile uint32_t chxfercntsh;/* Channel Transfer Counter Status High Register */
    volatile uint8_t  reserved[16];
  } ch[8];
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_dmac_initialize
 *
 * Description:
 *   Initialize the DMA controller
 *
 ****************************************************************************/

int ra8p_dmac_initialize(void);

/****************************************************************************
 * Name: ra8p_dmac_configure
 *
 * Description:
 *   Configure a DMA channel for transfer
 *
 ****************************************************************************/

int ra8p_dmac_configure(uint8_t channel, struct ra8p_dmac_chcfg_s *cfg);

/****************************************************************************
 * Name: ra8p_dmac_start
 *
 * Description:
 *   Start a DMA transfer on the specified channel
 *
 ****************************************************************************/

int ra8p_dmac_start(uint8_t channel);

/****************************************************************************
 * Name: ra8p_dmac_stop
 *
 * Description:
 *   Stop a DMA transfer on the specified channel
 *
 ****************************************************************************/

int ra8p_dmac_stop(uint8_t channel);

/****************************************************************************
 * Name: ra8p_dmac_get_status
 *
 * Description:
 *   Get the status of a DMA channel
 *
 ****************************************************************************/

uint32_t ra8p_dmac_get_status(uint8_t channel);

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_DMAC_H */