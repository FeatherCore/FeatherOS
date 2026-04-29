/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_eth.c
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

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

#include <sys/types.h>
#include <stdint.h>
#include <stdbool.h>
#include <time.h>
#include <queue.h>
#include <stdio.h>
#include <string.h>
#include <assert.h>
#include <errno.h>
#include <unistd.h>

#include <arch/board/board.h>
#include <net/ethernet.h>
#include <nuttx/arch.h>
#include <nuttx/clk/clk.h>
#include <nuttx/wdog.h>
#include <nuttx/irq.h>
#include <nuttx/net/arp.h>
#include <nuttx/net/netdev.h>

#include "arm_internal.h"
#include "chip.h"
#include "hardware/stm32n6_memorymap.h"
#include "hardware/stm32n6_rcc.h"
#include "stm32n6_gpio.h"
#include "stm32n6_eth.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Configuration ************************************************************/

#if !defined(CONFIG_SCHED_WORKQUEUE)
#  error "Ethernet requires CONFIG_SCHED_WORKQUEUE"
#endif

/* Number of polling attempts before and after reset */

#define STM32_ETH_NTPOLL             500000
#define STM32_ETH_MINDIV_TIMEOUT     500000

/* Clocking *****************************************************************/

#define STM32_ETH_MDC_DIVIDER        42  /* Based on HCLK frequency */

/* PHY timing */

#define STM32_ETH_PHY_MAX_WAIT       0x0004ffff

/* Register values **********************************************************/

/* Clear all interrupts */

#define ETH_DMA_INTCLR               (ETH_DMAISR_TS | ETH_DMAISR_TPS | ETH_DMAISR_TBU | \
                                      ETH_DMAISR_TJT | ETH_DMAISR_RO | ETH_DMAISR_TU | \
                                      ETH_DMAISR_RPS | ETH_DMAISR_RBU | ETH_DMAISR_R | \
                                      ETH_DMAISR_RWT | ETH_DMAISR_ET | ETH_DMAISR_FBE | \
                                      ETH_DMAISR_ER | ETH_DMAISR_AI | ETH_DMAISR_NI)

/* The set of all interrupts handled by this driver */

#define ETH_DMA_INTMASK              (ETH_DMAISR_TS | ETH_DMAISR_TPS | ETH_DMAISR_TBU | \
                                      ETH_DMAISR_TJT | ETH_DMAISR_RO | ETH_DMAISR_TU | \
                                      ETH_DMAISR_RPS | ETH_DMAISR_RBU | ETH_DMAISR_R | \
                                      ETH_DMAISR_RWT | ETH_DMAISR_ET | ETH_DMAISR_FBE | \
                                      ETH_DMAISR_ER | ETH_DMAISR_AI | ETH_DMAISR_NI)

/* Descriptor bit definitions */

#define ETH_DMARXDESC_OWN           (1 << 31)  /* Own bit */
#define ETH_DMARXDESC_FS            (1 << 6)   /* First segment */
#define ETH_DMARXDESC_LS            (1 << 7)   /* Last segment */
#define ETH_DMARXDESC_FL_SHIFT      16         /* Frame length shift */
#define ETH_DMARXDESC_FL_MASK       (0x3FFF << 16)  /* Frame length mask */

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
#define ETH_DMATXDESC_CC_MASK       (0x0f << 3) /* Collision count mask */
#define ETH_DMATXDESC_CC_SHIFT      3
#define ETH_DMATXDESC_ED            (1 << 2)   /* Excessive deferral */
#define ETH_DMATXDESC_UF            (1 << 1)   /* Underflow error */
#define ETH_DMATXDESC_DB            (1 << 0)   /* Deferred bit */

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* A list of all allocated EMAC structures */

static struct stm32n6_eth_s *g_emac[1];

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/* Register operations ******************************************************/

static inline uint32_t stm32n6_eth_getreg(uintptr_t addr, int offset)
{
  return getreg32(addr + offset);
}

static inline void stm32n6_eth_putreg(uintptr_t addr, int offset, uint32_t val)
{
  putreg32(val, addr + offset);
}

/* Buffer management ********************************************************/

static inline uint8_t *stm32n6_eth_descbuffer(int ndx, uint8_t **buffers)
{
  return buffers[ndx];
}

/* Common TX/RX descriptor operations ***************************************/

static inline void stm32n6_eth_txdesc_configure(struct stm32n6_eth_s *priv, int desc,
                                                 uint8_t *buf, int size, bool first,
                                                 bool last)
{
  struct stm32n6_eth_desc_s *txdesc = &priv->txdesc[desc];
  uint32_t ctrl = 0;

  /* Set the buffer address */
  txdesc->des0 = (uint32_t)buf;

  /* Configure control flags */
  ctrl = ETH_DMATXDESC_OWN;  /* Give ownership to DMA */
  if (first) ctrl |= ETH_DMATXDESC_FS;
  if (last)  ctrl |= ETH_DMATXDESC_LS;
  ctrl |= ETH_DMATXDESC_IC;  /* Generate interrupt on completion */

  /* Configure checksum insertion based on frame type */
  ctrl |= ETH_DMATXDESC_CIC_IPPL; /* Insert IP payload checksum */

  txdesc->des1 = size & 0x1FFF;  /* Limit buffer size to 8KB */
  txdesc->des2 = ctrl;
  txdesc->des3 = 0; /* Next descriptor not used in ring mode */
}

static inline void stm32n6_eth_rxdesc_configure(struct stm32n6_eth_s *priv, int desc,
                                                 uint8_t *buf, int size)
{
  struct stm32n6_eth_desc_s *rxdesc = &priv->rxdesc[desc];

  /* Set the buffer address */
  rxdesc->des0 = (uint32_t)buf;
  rxdesc->des1 = (size & 0x1FFF) << 16; /* Buffer size in upper 13 bits */

  /* Set up descriptor control */
  rxdesc->des2 = ETH_DMARXDESC_OWN; /* Give ownership to DMA */
  rxdesc->des3 = 0; /* Next descriptor not used in ring mode */
}

/* Clock management *********************************************************/

static void stm32n6_eth_enable_clk(bool enable)
{
  uint32_t regval;

  /* Enable ETH clock */
  regval = getreg32(STM32_RCC_AHB5ENR);
  if (enable)
    {
      regval |= RCC_AHB5ENR_ETHEN;
      regval |= RCC_AHB5ENR_ETHTXEN;  /* Enable TX clock */
      regval |= RCC_AHB5ENR_ETHRXEN;  /* Enable RX clock */
    }
  else
    {
      regval &= ~RCC_AHB5ENR_ETHEN;
      regval &= ~RCC_AHB5ENR_ETHTXEN;
      regval &= ~RCC_AHB5ENR_ETHRXEN;
    }
  putreg32(regval, STM32_RCC_AHB5ENR);
}

/* PHY Interface ************************************************************/

static int stm32n6_eth_phy_write(uint16_t phyaddr, uint16_t regaddr, uint16_t data)
{
  uint32_t timeout = STM32_ETH_PHY_MAX_WAIT;
  uint32_t regval;

  /* Set up the MACMIIAR register */
  regval = ((phyaddr & 0x1F) << 11) | ((regaddr & 0x1F) << 6) | 
           (STM32_ETH_MDC_DIVIDER << 2) | ETH_MACMIIAR_MB;
  stm32n6_eth_putreg(STM32_ETHMAC_BASE, STM32_ETH_MACMIIAR_OFFSET, regval);

  /* Write the data to the MACMIIDR register */
  stm32n6_eth_putreg(STM32_ETHMAC_BASE, STM32_ETH_MACMIIDR_OFFSET, data);

  /* Wait for completion */
  while (timeout-- > 0)
    {
      regval = stm32n6_eth_getreg(STM32_ETHMAC_BASE, STM32_ETH_MACMIIAR_OFFSET);
      if ((regval & ETH_MACMIIAR_MB) == 0)
        {
          return OK;
        }
    }

  return -ETIMEDOUT;
}

static int stm32n6_eth_phy_read(uint16_t phyaddr, uint16_t regaddr, uint16_t *data)
{
  uint32_t timeout = STM32_ETH_PHY_MAX_WAIT;
  uint32_t regval;

  /* Set up the MACMIIAR register for read */
  regval = ((phyaddr & 0x1F) << 11) | ((regaddr & 0x1F) << 6) | 
           (STM32_ETH_MDC_DIVIDER << 2);
  stm32n6_eth_putreg(STM32_ETHMAC_BASE, STM32_ETH_MACMIIAR_OFFSET, regval);

  /* Start the transaction */
  regval |= ETH_MACMIIAR_MB;
  stm32n6_eth_putreg(STM32_ETHMAC_BASE, STM32_ETH_MACMIIAR_OFFSET, regval);

  /* Wait for completion */
  while (timeout-- > 0)
    {
      regval = stm32n6_eth_getreg(STM32_ETHMAC_BASE, STM32_ETH_MACMIIAR_OFFSET);
      if ((regval & ETH_MACMIIAR_MB) == 0)
        {
          *data = (uint16_t)stm32n6_eth_getreg(STM32_ETHMAC_BASE, STM32_ETH_MACMIIDR_OFFSET);
          return OK;
        }
    }

  return -ETIMEDOUT;
}

/* Low-level Hardware Initialization ****************************************/

static void stm32n6_eth_reset(struct stm32n6_eth_s *priv)
{
  uint32_t regval;
  int i;

  /* Reset the Ethernet MAC */
  regval = stm32n6_eth_getreg(STM32_ETHMAC_BASE, STM32_ETH_DMAMR_OFFSET);
  regval |= ETH_DMAMR_SWR;
  stm32n6_eth_putreg(STM32_ETHMAC_BASE, STM32_ETH_DMAMR_OFFSET, regval);

  /* Wait for the software reset to complete */
  while ((stm32n6_eth_getreg(STM32_ETHMAC_BASE, STM32_ETH_DMAMR_OFFSET) & ETH_DMAMR_SWR) != 0);

  /* Wait for MAC to be ready */
  up_udelay(1000);

  /* Configure MIIAR clock divider */
  regval = stm32n6_eth_getreg(STM32_ETHMAC_BASE, STM32_ETH_MACMIIAR_OFFSET);
  regval &= ~(0x1F << 2);  /* Clear clock range bits */
  regval |= (STM32_ETH_MDC_DIVIDER << 2);  /* Set clock divider */
  stm32n6_eth_putreg(STM32_ETHMAC_BASE, STM32_ETH_MACMIIAR_OFFSET, regval);

  /* Initialize TX descriptors */
  for (i = 0; i < ETH_NTXDESC; i++)
    {
      stm32n6_eth_txdesc_configure(priv, i, priv->txbuf[i], ETH_TXBUF_SIZE, false, false);
    }

  /* Initialize RX descriptors */
  for (i = 0; i < ETH_NRXDESC; i++)
    {
      stm32n6_eth_rxdesc_configure(priv, i, priv->rxbuf[i], ETH_RXBUF_SIZE);
    }

  /* Set up descriptor list addresses */
  stm32n6_eth_putreg(STM32_ETHMAC_BASE, STM32_ETH_DMACHTDR_OFFSET, (uint32_t)priv->txdesc);
  stm32n6_eth_putreg(STM32_ETHMAC_BASE, STM32_ETH_DMACRHDR_OFFSET, (uint32_t)priv->rxdesc);

  /* Configure MAC address */
  regval = (uint32_t)priv->dev.d_mac.ether.ether_addr_octet[5] << 24 |
           (uint32_t)priv->dev.d_mac.ether.ether_addr_octet[4] << 16 |
           (uint32_t)priv->dev.d_mac.ether.ether_addr_octet[3] << 8 |
           (uint32_t)priv->dev.d_mac.ether.ether_addr_octet[2];
  stm32n6_eth_putreg(STM32_ETHMAC_BASE, STM32_ETH_MACA0HR_OFFSET, regval);

  regval = (uint32_t)priv->dev.d_mac.ether.ether_addr_octet[1] << 8 |
           (uint32_t)priv->dev.d_mac.ether.ether_addr_octet[0];
  stm32n6_eth_putreg(STM32_ETHMAC_BASE, STM32_ETH_MACA0LR_OFFSET, regval);

  /* Enable automatic pad/CRC stripping */
  regval = stm32n6_eth_getreg(STM32_ETHMAC_BASE, STM32_ETH_MACCR_OFFSET);
  regval |= ETH_MACCR_APCS;
  stm32n6_eth_putreg(STM32_ETHMAC_BASE, STM32_ETH_MACCR_OFFSET, regval);

  /* Configure frame filter - promiscuous mode for initial implementation */
  regval = ETH_MACFFR_PM;  /* Promiscuous mode */
  stm32n6_eth_putreg(STM32_ETHMAC_BASE, STM32_ETH_MACFFR_OFFSET, regval);

  /* Configure DMA Bus Mode */
  regval = ETH_DMASBMR_FB | ETH_DMASBMR_AAL | ETH_DMASBMR_8PBL;
  stm32n6_eth_putreg(STM32_ETHMAC_BASE, STM32_ETH_DMASBMR_OFFSET, regval);

  /* Enable interrupts */
  regval = ETH_DMAIER_NISE | ETH_DMAIER_RIE | ETH_DMAIER_TIE;
  stm32n6_eth_putreg(STM32_ETHMAC_BASE, STM32_ETH_DMAIER_OFFSET, regval);

  /* Clear any pending interrupts */
  stm32n6_eth_putreg(STM32_ETHMAC_BASE, STM32_ETH_DMAISR_OFFSET, ETH_DMA_INTCLR);
}

/* Network Interface Support ************************************************/

static void stm32n6_eth_setmacaddress(struct stm32n6_eth_s *priv)
{
  uint32_t regval;

  /* Set up the MAC address in the first MAC address register */

  regval = (uint32_t)priv->dev.d_mac.ether.ether_addr_octet[5] << 24 |
           (uint32_t)priv->dev.d_mac.ether.ether_addr_octet[4] << 16 |
           (uint32_t)priv->dev.d_mac.ether.ether_addr_octet[3] << 8 |
           (uint32_t)priv->dev.d_mac.ether.ether_addr_octet[2];
  stm32n6_eth_putreg(STM32_ETHMAC_BASE, STM32_ETH_MACA0HR_OFFSET, regval);

  regval = (uint32_t)priv->dev.d_mac.ether.ether_addr_octet[1] << 8 |
           (uint32_t)priv->dev.d_mac.ether.ether_addr_octet[0];
  stm32n6_eth_putreg(STM32_ETHMAC_BASE, STM32_ETH_MACA0LR_OFFSET, regval);
}

static int stm32n6_eth_init(struct net_driver_s *dev)
{
  struct stm32n6_eth_s *priv = (struct stm32n6_eth_s *)dev->d_private;
  uint32_t regval;
  int i;

  ninfo("Initializing Ethernet\n");

  /* Take the semaphore */
  net_lock();

  /* Enable ethernet clock */
  stm32n6_eth_enable_clk(true);

  /* Reset the MAC and establish the MAC address */
  stm32n6_eth_reset(priv);

  /* Configure MAC address filters */
  stm32n6_eth_setmacaddress(priv);

  /* Enable MAC transmitter and receiver */
  regval = stm32n6_eth_getreg(STM32_ETHMAC_BASE, STM32_ETH_MACCR_OFFSET);
  regval |= ETH_MACCR_TE | ETH_MACCR_RE;
  stm32n6_eth_putreg(STM32_ETHMAC_BASE, STM32_ETH_MACCR_OFFSET, regval);

  /* Configure DMA mode */
  regval = ETH_DMAMR_DA;  /* Arbitration: Fair round robin */
  stm32n6_eth_putreg(STM32_ETHMAC_BASE, STM32_ETH_DMAMR_OFFSET, regval);

  /* Enable DMA transmission and reception */
  regval = ETH_DMAOMR_ST | ETH_DMAOMR_SR;  /* Start transmission and reception */
  stm32n6_eth_putreg(STM32_ETHMAC_BASE, STM32_ETH_DMAOMR_OFFSET, regval);

  /* Initialize the free buffer list */
  for (i = 0; i < ETH_NRXDESC; i++)
    {
      /* Give the descriptor back to the DMA */
      priv->rxdesc[i].des2 = ETH_DMARXDESC_OWN;
    }

  /* Set up the initial TX descriptor state */
  priv->txhead = 0;
  priv->txtail = 0;

  /* Set up the initial RX descriptor state */
  priv->rxndx = 0;

  /* Initialize for poll-based operation */
  priv->reset = false;

  /* Release the semaphore */
  net_unlock();

  return OK;
}

static void stm32n6_eth_txpoll(struct net_driver_s *dev)
{
  struct stm32n6_eth_s *priv = (struct stm32n6_eth_s *)dev->d_private;
  uint32_t regval;
  int i;

  ninfo("TX Poll\n");

  /* Check if the hardware is ready to send another packet */
  if (priv->txhead != priv->txtail)
    {
      /* Are we allowed to send a packet? */
      if ((dev->d_flags & IFF_UP) != 0 && (dev->d_txavail & NET_LL_TXAVAIL) != 0)
        {
          /* Transfer the packet to the hardware */
          i = priv->txhead;
          stm32n6_eth_txdesc_configure(priv, i, dev->d_buf, dev->d_len, true, true);

          /* Increment the head pointer */
          priv->txhead = (priv->txhead + 1) % ETH_NTXDESC;

          /* Start transmission */
          regval = stm32n6_eth_getreg(STM32_ETHMAC_BASE, STM32_ETH_DMATPDR_OFFSET);
          regval |= (1 << 0); /* Poll immediately */
          stm32n6_eth_putreg(STM32_ETHMAC_BASE, STM32_ETH_DMATPDR_OFFSET, regval);

          /* Mark that there is no longer an availability to send a packet */
          dev->d_txavail &= ~NET_LL_TXAVAIL;

          /* Clear OWN bit to indicate transmission started */
          priv->txdesc[i].des2 &= ~ETH_DMATXDESC_OWN;
        }
    }
}

static inline void stm32n6_eth_receive(struct net_driver_s *dev)
{
  struct stm32n6_eth_s *priv = (struct stm32n6_eth_s *)dev->d_private;
  struct stm32n6_eth_desc_s *rxdesc;
  uint32_t des2;
  int i;
  int ndx;

  ninfo("RX Processing\n");

  /* Process received RX packets */
  for (i = 0; i < ETH_NRXDESC; i++)
    {
      ndx = (priv->rxndx + i) % ETH_NRXDESC;
      rxdesc = &priv->rxdesc[ndx];

      /* Check if this descriptor owns the packet */
      if ((rxdesc->des2 & ETH_DMARXDESC_OWN) == 0)
        {
          /* Check if this is the last segment */
          if ((rxdesc->des2 & ETH_DMARXDESC_LS) != 0)
            {
              /* Check for errors */
              des2 = rxdesc->des2;
              if ((des2 & (ETH_DMARXDESC_ES | ETH_DMARXDESC_CE | ETH_DMARXDESC_DE)) == 0)
                {
                  /* Good packet received, get the packet length */
                  uint16_t len = (des2 & ETH_DMARXDESC_FL_MASK) >> 16;

                  /* Copy the data from the buffer to the network buffer */
                  if (len <= ETH_MAX_FRAMELEN && len > 0)
                    {
                      memcpy(dev->d_buf, priv->rxbuf[ndx], len);
                      dev->d_len = len;

                      /* Handle the incoming packet */
                      netdev_rx(dev);

                      /* Update statistics */
                      dev->d_statistics.d_rx_packets++;
                    }
                }
              else
                {
                  /* Packet has errors, update error statistics */
                  dev->d_statistics.d_rx_errors++;
                }

              /* Give the descriptor back to the DMA */
              rxdesc->des2 |= ETH_DMARXDESC_OWN;
            }
        }
      else
        {
          /* If any descriptor still owned by DMA, stop processing */
          break;
        }
    }

  /* Update RX index to next descriptor to process */
  priv->rxndx = ndx;
}

static int stm32n6_eth_ioctl(struct net_driver_s *dev, int cmd, unsigned long arg)
{
  switch (cmd)
    {
      case SIOCMIINX:
        {
          /* Get the MAC address */
          struct ether_addr_s *addr = (struct ether_addr_s *)arg;
          memcpy(addr, &dev->d_mac.ether, sizeof(struct ether_addr_s));
          return OK;
        }

      default:
        return -ENOTTY;
    }
}

static void stm32n6_eth_interrupt(int irq, void *context, void *arg)
{
  struct stm32n6_eth_s *priv = (struct stm32n6_eth_s *)arg;
  uint32_t dmasr;
  bool handled = false;

  /* Get DMA status */
  dmasr = stm32n6_eth_getreg(STM32_ETHMAC_BASE, STM32_ETH_DMAISR_OFFSET);

  /* Clear interrupts */
  stm32n6_eth_putreg(STM32_ETHMAC_BASE, STM32_ETH_DMAISR_OFFSET, dmasr);

  /* Process receive interrupt */
  if ((dmasr & (ETH_DMAISR_R | ETH_DMAISR_RBU)) != 0)
    {
      /* Process received packets */
      stm32n6_eth_receive(&priv->dev);
      handled = true;
    }

  /* Process transmit interrupt */
  if ((dmasr & (ETH_DMAISR_T | ETH_DMAISR_TBU)) != 0)
    {
      /* Check if transmission completed */
      if (priv->txtail != priv->txhead)
        {
          /* Indicate that a packet transmission has completed */
          priv->dev.d_txavail |= NET_LL_TXAVAIL;

          /* Increment the tail pointer */
          priv->txtail = (priv->txtail + 1) % ETH_NTXDESC;
          handled = true;
        }
    }

  if (!handled)
    {
      /* This shouldn't happen... */
      nwarn("WARNING: Unhandled interrupt\n");
    }
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

int stm32n6_eth_initialize(void)
{
  struct stm32n6_eth_s *priv;
  int ret;

  ninfo("Initializing STM32N6 Ethernet\n");

  /* Allocate resources */
  priv = (struct stm32n6_eth_s *)kmm_zalloc(sizeof(struct stm32n6_eth_s));
  if (priv == NULL)
    {
      nerr("ERROR: Failed to allocate adapter structure\n");
      return -ENOMEM;
    }

  /* Initialize the driver structure */
  memset(priv, 0, sizeof(struct stm32n6_eth_s));
  priv->dev.d_ifup    = stm32n6_eth_init;      /* I/F up (new network task) */
  priv->dev.d_ifdown  = NULL;                  /* I/F down */
  priv->dev.d_txpoll  = stm32n6_eth_txpoll;    /* New packet to send */
  priv->dev.d_ioctl   = stm32n6_eth_ioctl;     /* Perform IOCTL */
  priv->dev.d_private = (void *)priv;          /* Private data */

  /* Initialize the MAC address */
  eth_random_ethaddr(&priv->dev.d_mac.ether);

  /* Allocate DMA descriptors and buffers */
  priv->txdesc = (struct stm32n6_eth_desc_s *)
                  kmm_malloc(ETH_NTXDESC * sizeof(struct stm32n6_eth_desc_s));
  if (priv->txdesc == NULL)
    {
      nerr("ERROR: Failed to allocate TX descriptors\n");
      ret = -ENOMEM;
      goto errout_with_priv;
    }

  priv->rxdesc = (struct stm32n6_eth_desc_s *)
                  kmm_malloc(ETH_NRXDESC * sizeof(struct stm32n6_eth_desc_s));
  if (priv->rxdesc == NULL)
    {
      nerr("ERROR: Failed to allocate RX descriptors\n");
      ret = -ENOMEM;
      goto errout_with_txdesc;
    }

  /* Allocate TX buffers */
  for (int i = 0; i < ETH_NTXDESC; i++)
    {
      priv->txbuf[i] = (uint8_t *)kmm_malloc(ETH_TXBUF_SIZE);
      if (priv->txbuf[i] == NULL)
        {
          nerr("ERROR: Failed to allocate TX buffer %d\n", i);
          ret = -ENOMEM;
          goto errout_with_rxdesc;
        }
    }

  /* Allocate RX buffers */
  for (int i = 0; i < ETH_NRXDESC; i++)
    {
      priv->rxbuf[i] = (uint8_t *)kmm_malloc(ETH_RXBUF_SIZE);
      if (priv->rxbuf[i] == NULL)
        {
          nerr("ERROR: Failed to allocate RX buffer %d\n", i);
          ret = -ENOMEM;
          goto errout_with_txbufs;
        }
    }

  /* Set up register base addresses */
  priv->macbase = STM32_ETHMAC_BASE;

  /* Enable ethernet clock */
  stm32n6_eth_enable_clk(true);

  /* Attach the IRQ */
  ret = irq_attach(STM32_IRQ_ETH, stm32n6_eth_interrupt, priv);
  if (ret != OK)
    {
      nerr("ERROR: Failed to attach ETH IRQ\n");
      goto errout_with_rxbufs;
    }

  /* Enable the interrupt */
  up_enable_irq(STM32_IRQ_ETH);

  /* Save the private structure pointer */
  g_emac[0] = priv;

  /* Register the device with the OS */
  ret = netdev_register(&priv->dev, NET_LL_ETHERNET);
  if (ret != OK)
    {
      nerr("ERROR: Failed to register eth device\n");
      goto errout_with_irq;
    }

  ninfo("STM32N6 Ethernet initialized successfully\n");
  return OK;

errout_with_irq:
  up_disable_irq(STM32_IRQ_ETH);
  irq_detach(STM32_IRQ_ETH);

errout_with_rxbufs:
  for (int i = 0; i < ETH_NRXDESC; i++)
    {
      if (priv->rxbuf[i])
        {
          kmm_free(priv->rxbuf[i]);
        }
    }

errout_with_txbufs:
  for (int i = 0; i < ETH_NTXDESC; i++)
    {
      if (priv->txbuf[i])
        {
          kmm_free(priv->txbuf[i]);
        }
    }

errout_with_rxdesc:
  kmm_free(priv->rxdesc);

errout_with_txdesc:
  kmm_free(priv->txdesc);

errout_with_priv:
  kmm_free(priv);

  return ret;
}