/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_eth.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_STM32N6_ETH_H
#define __ARCH_ARM_SRC_STM32N6_STM32N6_ETH_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <nuttx/net/netdev.h>
#include <nuttx/wdog.h>
#include <net/if.h>
#include <netinet/in.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_ETHMAC_BASE           (STM32N6_PERIPH_BASE + 0x08036000)

/* Register Offsets *********************************************************/

/* MAC registers */
#define STM32_ETH_MACCR_OFFSET      0x0000  /* MAC configuration register */
#define STM32_ETH_MACFFR_OFFSET     0x0004  /* MAC frame filter register */
#define STM32_ETH_MACHTHR_OFFSET    0x0008  /* MAC hash table high register */
#define STM32_ETH_MACHTLR_OFFSET    0x000C  /* MAC hash table low register */
#define STM32_ETH_MACMIIAR_OFFSET   0x0010  /* MAC MII address register */
#define STM32_ETH_MACMIIDR_OFFSET   0x0014  /* MAC MII data register */
#define STM32_ETH_MACFCR_OFFSET     0x0018  /* MAC flow control register */
#define STM32_ETH_MACVLANTR_OFFSET  0x001C  /* MAC VLAN tag register */
#define STM32_ETH_MACPWTR_OFFSET    0x0024  /* MAC PMT wake-up frame filter */
#define STM32_ETH_MACTSR_OFFSET     0x0028  /* MAC timestamp control status */
#define STM32_ETH_MACA0HR_OFFSET    0x0040  /* MAC address 0 high register */
#define STM32_ETH_MACA0LR_OFFSET    0x0044  /* MAC address 0 low register */
#define STM32_ETH_MACA1HR_OFFSET    0x0048  /* MAC address 1 high register */
#define STM32_ETH_MACA1LR_OFFSET    0x004C  /* MAC address 1 low register */
#define STM32_ETH_MACA2HR_OFFSET    0x0050  /* MAC address 2 high register */
#define STM32_ETH_MACA2LR_OFFSET    0x0054  /* MAC address 2 low register */
#define STM32_ETH_MACA3HR_OFFSET    0x0058  /* MAC address 3 high register */
#define STM32_ETH_MACA3LR_OFFSET    0x005C  /* MAC address 3 low register */

/* MTL registers */
#define STM32_ETH_MTL_OMR_OFFSET    0x0100  /* MTL operation mode register */
#define STM32_ETH_MTL_FCR_OFFSET    0x0104  /* MTL flush control register */
#define STM32_ETH_MTL_ISR_OFFSET    0x0108  /* MTL interrupt status register */
#define STM32_ETH_MTL_RQDCMTR_OFFSET 0x0110 /* MTL receive queue debug counter */

/* TX Queue registers */
#define STM32_ETH_MTL_TXQOMR_OFFSET 0x0110  /* MTL TX queue operation mode */
#define STM32_ETH_MTL_TXQUR_OFFSET  0x0114  /* MTL TX queue underflow */
#define STM32_ETH_MTL_TXQDR_OFFSET  0x0118  /* MTL TX queue debug */

/* RX Queue registers */
#define STM32_ETH_MTL_RXQOMR_OFFSET 0x0130  /* MTL RX queue operation mode */
#define STM32_ETH_MTL_RXQDR_OFFSET  0x0138  /* MTL RX queue debug */

/* DMA registers */
#define STM32_ETH_DMAMR_OFFSET      0x1000  /* DMA mode register */
#define STM32_ETH_DMASBMR_OFFSET    0x1004  /* DMA system bus mode register */
#define STM32_ETH_DMAISR_OFFSET     0x1008  /* DMA interrupt status register */
#define STM32_ETH_DMADSR_OFFSET     0x100C  /* DMA debug status register */
#define STM32_ETH_DMAIER_OFFSET     0x1018  /* DMA interrupt enable register */
#define STM32_ETH_DMAMFBOCR_OFFSET  0x1020  /* DMA missed frames overflow counter */
#define STM32_ETH_DMACHTDR_OFFSET   0x1048  /* DMA current host transmit descriptor */
#define STM32_ETH_DMACRHDR_OFFSET   0x104C  /* DMA current host receive descriptor */
#define STM32_ETH_DMACHTBAR_OFFSET  0x1050  /* DMA current host transmit buffer */
#define STM32_ETH_DMACRBAR_OFFSET   0x1054  /* DMA current host receive buffer */
#define STM32_ETH_DMATPDR_OFFSET    0x1058  /* DMA transmit poll demand register */
#define STM32_ETH_DMARPDR_OFFSET    0x105C  /* DMA receive poll demand register */

/* Register Bitfield Definitions ********************************************/

/* MAC Configuration Register (MACCR) */
#define ETH_MACCR_RE                (1 << 2)   /* Receiver Enable */
#define ETH_MACCR_TE                (1 << 3)   /* Transmitter Enable */
#define ETH_MACCR_DC                (1 << 4)   /* Deferral Check */
#define ETH_MACCR_BL_SHIFT          5          /* Back-off limit shift */
#define ETH_MACCR_BL_MASK           (3 << 5)   /* Back-off limit mask */
#define ETH_MACCR_APCS              (1 << 7)   /* Automatic Pad/CRC stripping */
#define ETH_MACCR_RD                (1 << 9)   /* Retry disable */
#define ETH_MACCR_IPCO              (1 << 10)  /* IPv4 checksum offload */
#define ETH_MACCR_DM                (1 << 11)  /* Duplex mode */
#define ETH_MACCR_LM                (1 << 12)  /* Loopback mode */
#define ETH_MACCR_ROD               (1 << 13)  /* Receive own disable */
#define ETH_MACCR_FES               (1 << 14)  /* Fast Ethernet speed */
#define ETH_MACCR_CSD               (1 << 16)  /* Carrier sense disable */
#define ETH_MACCR_IGPSC_SHIFT       17         /* Interframe gap shift */
#define ETH_MACCR_IGPSC_MASK        (7 << 17)  /* Interframe gap mask */
#define ETH_MACCR_JD                (1 << 22)  /* Jabber disable */
#define ETH_MACCR_WD                (1 << 23)  /* Watchdog disable */

/* MAC Frame Filter Register (MACFFR) */
#define ETH_MACFFR_PM               (1 << 0)   /* Promiscuous mode */
#define ETH_MACFFR_DAIF             (1 << 3)   /* DA Inverse filtering */
#define ETH_MACFFR_PAM              (1 << 4)   /* Pass all multicast */
#define ETH_MACFFR_DBF              (1 << 5)   /* Disable broadcast frames */
#define ETH_MACFFR_PCF_SHIFT        6          /* Pass control frames shift */
#define ETH_MACFFR_PCF_MASK         (3 << 6)   /* Pass control frames mask */
#define ETH_MACFFR_SAIF             (1 << 8)   /* SA Inverse filtering */
#define ETH_MACFFR_SAF              (1 << 9)   /* Source address filter */
#define ETH_MACFFR_HPF              (1 << 10)  /* Hash perfect filter */
#define ETH_MACFFR_RA               (1 << 31)  /* Receive all */

/* MII Address Register (MIIAR) */
#define ETH_MACMIIAR_MB             (1 << 0)   /* MII busy */
#define ETH_MACMIIAR_MW             (1 << 1)   /* MII write */
#define ETH_MACMIIAR_CR_SHIFT       6          /* Clock range shift */
#define ETH_MACMIIAR_CR_MASK        (0x1F << 6)
#define ETH_MACMIIAR_MR_SHIFT       6          /* MII register shift */
#define ETH_MACMIIAR_MR_MASK        (0x1F << 6)
#define ETH_MACMIIAR_PA_SHIFT       11         /* Physical layer address shift */
#define ETH_MACMIIAR_PA_MASK        (0x1F << 11)

/* DMA Mode Register (DMAMR) */
#define ETH_DMAMR_SWR               (1 << 0)   /* Software reset */
#define ETH_DMAMR_DA                (1 << 1)   /* DMA arbitration */
#define ETH_DMAMR_TXPR              (1 << 14)  /* Transmit priority */
#define ETH_DMAMR_USP               (1 << 13)  /* Use separate PBL */
#define ETH_DMAMR_PBL               (1 << 8)   /* Programmable burst length */

/* DMA Bus Mode Register (DMASBMR) */
#define ETH_DMASBMR_FB              (1 << 16)  /* Fixed burst */
#define ETH_DMASBMR_AAL             (1 << 25)  /* Address aligned beats */
#define ETH_DMASBMR_8PBL            (1 << 20)  /* 8 x Programmable burst length */
#define ETH_DMASBMR_PBL_MASK        0x700      /* PBL mask */

/* DMA Interrupt Enable Register (DMAIER) */
#define ETH_DMAIER_NISE             (1 << 15)  /* Normal interrupt summary */
#define ETH_DMAIER_AISE             (1 << 14)  /* Abnormal interrupt summary */
#define ETH_DMAIER_ESIE             (1 << 13)  /* Early transmit status */
#define ETH_DMAIER_FBEIE            (1 << 12)  /* Fatal bus error */
#define ETH_DMAIER_ETIE             (1 << 10)  /* Early transmit */
#define ETH_DMAIER_RWTIE            (1 << 9)   /* Receive watchdog timeout */
#define ETH_DMAIER_RPSIE            (1 << 8)   /* Receive process stopped */
#define ETH_DMAIER_RBUIE            (1 << 7)   /* Receive buffer unavailable */
#define ETH_DMAIER_RIE              (1 << 6)   /* Receive interrupt enable */
#define ETH_DMAIER_TUIE             (1 << 5)   /* Transmit underflow */
#define ETH_DMAIER_ROIE             (1 << 4)   /* Receive overflow */
#define ETH_DMAIER_TJTIE            (1 << 3)   /* Transmit jabber timeout */
#define ETH_DMAIER_TBUIE            (1 << 2)   /* Transmit buffer unavailable */
#define ETH_DMAIER_TPSIE            (1 << 1)   /* Transmit process stopped */
#define ETH_DMAIER_TIE              (1 << 0)   /* Transmit interrupt enable */

/* DMA Status Register (DMAISR) */
#define ETH_DMAISR_TS               (1 << 0)   /* Transmit status */
#define ETH_DMAISR_TPS              (1 << 1)   /* Transmit process stopped */
#define ETH_DMAISR_TBU              (1 << 2)   /* Transmit buffer unavailable */
#define ETH_DMAISR_TJT              (1 << 3)   /* Transmit jabber timeout */
#define ETH_DMAISR_RO               (1 << 4)   /* Receive overflow */
#define ETH_DMAISR_TU               (1 << 5)   /* Transmit underflow */
#define ETH_DMAISR_RPS              (1 << 6)   /* Receive process stopped */
#define ETH_DMAISR_RBU              (1 << 7)   /* Receive buffer unavailable */
#define ETH_DMAISR_R                (1 << 8)   /* Receive status */
#define ETH_DMAISR_RWT              (1 << 9)   /* Receive watchdog timeout */
#define ETH_DMAISR_ET               (1 << 10)  /* Early transmit */
#define ETH_DMAISR_FBE              (1 << 12)  /* Fatal bus error */
#define ETH_DMAISR_ER               (1 << 13)  /* Early transmit status */
#define ETH_DMAISR_AI               (1 << 14)  /* Abnormal interrupt summary */
#define ETH_DMAISR_NI               (1 << 15)  /* Normal interrupt summary */

/* Descriptor bit definitions */
#define ETH_DMATXDESC_OWN           (1 << 31)  /* Own bit */
#define ETH_DMATXDESC_IC            (1 << 30)  /* Interrupt on completion */
#define ETH_DMATXDESC_LS            (1 << 29)  /* Last segment */
#define ETH_DMATXDESC_FS            (1 << 28)  /* First segment */
#define ETH_DMATXDESC_DC            (1 << 27)  /* Disable CRC */
#define ETH_DMATXDESC_DP            (1 << 26)  /* Disable pad */
#define ETH_DMATXDESC_TTSE          (1 << 25)  /* Transmit time stamp enable */
#define ETH_DMATXDESC_CIC_MASK      (3 << 22)  /* Checksum insertion control */
#define ETH_DMATXDESC_CIC_BYPASS    (0 << 22)  /* Don't compute checksum */
#define ETH_DMATXDESC_CIC_IPHDR     (1 << 22)  /* Insert IP header checksum */
#define ETH_DMATXDESC_CIC_PAYLOAD   (2 << 22)  /* Insert IP payload checksum */
#define ETH_DMATXDESC_CIC_IPPL      (3 << 22)  /* Insert IP payload checksum */
#define ETH_DMATXDESC_TER           (1 << 21)  /* Transmit end of ring */
#define ETH_DMATXDESC_TCH           (1 << 20)  /* Second address chained */
#define ETH_DMATXDESC_TTSS          (1 << 17)  /* Tx time stamp status */
#define ETH_DMATXDESC_IHE           (1 << 16)  /* IP header error */
#define ETH_DMATXDESC_ES            (1 << 15)  /* Error summary */
#define ETH_DMATXDESC_JT            (1 << 14)  /* Jabber timeout */
#define ETH_DMATXDESC_FF            (1 << 13)  /* Frame flushed */
#define ETH_DMATXDESC_PCE           (1 << 12)  /* Payload checksum error */
#define ETH_DMATXDESC_LCO           (1 << 11)  /* Loss of carrier */
#define ETH_DMATXDESC_NC            (1 << 10)  /* No carrier */
#define ETH_DMATXDESC_LC            (1 << 9)   /* Late collision */
#define ETH_DMATXDESC_EC            (1 << 8)   /* Excessive collision */
#define ETH_DMATXDESC_VF            (1 << 7)   /* VLAN frame */
#define ETH_DMATXDESC_CC_MASK       (0x0f << 3)
#define ETH_DMATXDESC_ED            (1 << 2)   /* Excessive deferral */
#define ETH_DMATXDESC_UF            (1 << 1)   /* Underflow error */
#define ETH_DMATXDESC_DB            (1 << 0)   /* Deferred bit */

#define ETH_DMARXDESC_OWN           (1 << 31)  /* Own bit */
#define ETH_DMARXDESC_FS            (1 << 6)   /* First segment */
#define ETH_DMARXDESC_LS            (1 << 7)   /* Last segment */
#define ETH_DMARXDESC_FL_MASK       (0x3fff << 16)

/* Number of descriptors */
#define ETH_NTXDESC                 4
#define ETH_NRXDESC                 4

/* Buffer sizes */
#define ETH_TXBUF_SIZE              1520
#define ETH_RXBUF_SIZE              1520

/* Network constants */
#define ETH_MAX_FRAMELEN            1536  /* Max frame size */

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* Ethernet DMA descriptor */

struct stm32n6_eth_desc_s
{
  uint32_t des0;  /* Address field */
  uint32_t des1;  /* Control and buffer lengths */
  uint32_t des2;  /* Status */
  uint32_t des3;  /* Next descriptor address */
};

/* STM32N6 Ethernet MAC device structure */

struct stm32n6_eth_s
{
  /* NuttX callbacks */

  struct net_driver_s  dev;           /* Interface understood by the network stack */
  struct wdog_s        txtimeout;     /* TX timeout timer */

  /* Resources */

  uintptr_t            macbase;       /* MAC register base address */
  int                  irq;           /* Combined interrupt number */

  /* DMA descriptors */

  struct stm32n6_eth_desc_s  *txdesc;   /* TX descriptor array */
  struct stm32n6_eth_desc_s  *rxdesc;   /* RX descriptor array */
  uint8_t              *txbuf[ETH_NTXDESC];
  uint8_t              *rxbuf[ETH_NRXDESC];
  uint8_t              txhead;
  uint8_t              txtail;
  uint8_t              rxndx;

  /* Multicast hash table */

  uint32_t             hashtbl[2];

  /* Cached values */

  uint32_t             mac1;          /* Last value of MAC register 1 */
  uint32_t             mac2;          /* Last value of MAC register 2 */
  bool                 reset;         /* Reset in progress */
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

int stm32n6_eth_initialize(void);
int stm32n6_ethinitialize(int intf);

#endif /* __ARCH_ARM_SRC_STM32N6_STM32N6_ETH_H */
