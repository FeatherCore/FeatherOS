/****************************************************************************
 * arch/arm/src/ra8p/ra8p_ethernet.c
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
#include <nuttx/arch.h>
#include <nuttx/irq.h>
#include <nuttx/kmalloc.h>
#include <nuttx/mutex.h>
#include <nuttx/net/netdev.h>
#include <nuttx/net/ethernet.h>
#include <arch/board/board.h>

#include "arm_internal.h"
#include "chip.h"
#include "ra8p_config.h"
#include "ra8p_peripherals.h"
#include "hardware/ra8p_eth.h"

#ifdef CONFIG_RA8P_ETHERNET

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Helper macros for register access */

#define ra8p_eth_putreg32(base, offset, val) \
  putreg32((val), (base) + (offset))

#define ra8p_eth_getreg32(base, offset) \
  getreg32((base) + (offset))

#define ra8p_eth_modifyreg32(base, offset, clrbits, setbits) \
  ra8p_eth_putreg32(base, offset, \
    (ra8p_eth_getreg32(base, offset) & ~(clrbits)) | (setbits))

/* Ethernet buffer sizes */

#define RA8P_ETH_FRAME_SIZE                   (1518)
#define RA8P_ETH_RX_BUFFER_SIZE              (2048)
#define RA8P_ETH_TX_BUFFER_SIZE              (2048)
#define RA8P_ETH_RX_DESCRIPTORS              (8)
#define RA8P_ETH_TX_DESCRIPTORS              (8)

/* PHY address */

#define RA8P_ETH_PHY_ADDR                    (0x00)

/* MII management clock division */

#define RA8P_ETH_MDC_DIV                     (0x0F)

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* TX/RX Descriptor */

struct ra8p_eth_desc_s
{
  volatile uint32_t tdes0;        /* Descriptor word 0 */
  volatile uint32_t tdes1;        /* Descriptor word 1 */
  volatile uint32_t tdes2;        /* Descriptor word 2 (buffer address low) */
  volatile uint32_t tdes3;        /* Descriptor word 3 (buffer address high) */
};

/* TX/RX Buffer */

struct ra8p_eth_buf_s
{
  uint8_t data[RA8P_ETH_RX_BUFFER_SIZE];
};

/* Private device structure */

struct ra8p_eth_dev_s
{
  /* NuttX network device interface */

  struct netdev_s dev;

  /* Hardware base addresses */

  uint32_t ethbase;               /* ETHERC base address */
  uint32_t edmabase;              /* EDMAC base address */
  uint32_t mdiobase;              /* MDIO base address */

  /* Interrupt number */

  int irq;

  /* Mutex for thread safety */

  mutex_t lock;

  /* TX/RX descriptors */

  struct ra8p_eth_desc_s *txdesc;
  struct ra8p_eth_desc_s *rxdesc;

  /* TX/RX buffers */

  struct ra8p_eth_buf_s *txbuf;
  struct ra8p_eth_buf_s *rxbuf;

  /* Current descriptor indices */

  uint16_t tx_head;
  uint16_t tx_tail;
  uint16_t rx_head;

  /* Device flags */

  bool initialized;
  bool linkup;
  uint8_t speed;                  /* 0 = 10Mbps, 1 = 100Mbps, 2 = 1000Mbps */
  uint8_t duplex;                 /* 0 = Half, 1 = Full */

  /* Statistics */

  uint32_t rx_packets;
  uint32_t tx_packets;
  uint32_t rx_errors;
  uint32_t tx_errors;
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static void ra8p_eth_reset(struct ra8p_eth_dev_s *priv);
static int ra8p_eth_mdio_read(struct ra8p_eth_dev_s *priv,
                               uint8_t phyaddr, uint8_t regaddr, uint16_t *value);
static int ra8p_eth_mdio_write(struct ra8p_eth_dev_s *priv,
                                uint8_t phyaddr, uint8_t regaddr, uint16_t value);
static int ra8p_eth_phy_init(struct ra8p_eth_dev_s *priv);
static void ra8p_eth_tx_setup(struct ra8p_eth_dev_s *priv);
static void ra8p_eth_rx_setup(struct ra8p_eth_dev_s *priv);
static int ra8p_eth_hw_init(struct ra8p_eth_dev_s *priv);
static void ra8p_eth_start(struct ra8p_eth_dev_s *priv);
static void ra8p_eth_stop(struct ra8p_eth_dev_s *priv);
static int ra8p_eth_tx_packet(struct ra8p_eth_dev_s *priv,
                              struct netdev_var_s *netvar);
static void ra8p_eth_rx_packet(struct ra8p_eth_dev_s *priv);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_eth_dev_s g_eth0_priv;

/* TX descriptor base - must be aligned */

static struct ra8p_eth_desc_s g_txdesc[RA8P_ETH_TX_DESCRIPTORS]
  __attribute__((aligned(32)));

/* RX descriptor base - must be aligned */

static struct ra8p_eth_desc_s g_rxdesc[RA8P_ETH_RX_DESCRIPTORS]
  __attribute__((aligned(32)));

/* TX buffers */

static struct ra8p_eth_buf_s g_txbuf[RA8P_ETH_TX_DESCRIPTORS];

/* RX buffers */

static struct ra8p_eth_buf_s g_rxbuf[RA8P_ETH_RX_DESCRIPTORS];

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static void ra8p_eth_reset(struct ra8p_eth_dev_s *priv)
{
  /* Reset ETHERC */

  ra8p_eth_modifyreg32(priv->ethbase, RA8P_ETHERC_ECMR_OFFSET,
                        0, RA8P_ETHERC_ECMR_RST);

  up_udelay(10);

  /* Reset EDMAC */

  ra8p_eth_modifyreg32(priv->edmabase, RA8P_EDMAC_EDMR_OFFSET,
                        0, RA8P_EDMAC_EDMR_SWR);
}

static int ra8p_eth_mdio_read(struct ra8p_eth_dev_s *priv,
                               uint8_t phyaddr, uint8_t regaddr, uint16_t *value)
{
  uint32_t mpsm;
  uint32_t timeout;

  /* Wait for previous operation to complete */

  timeout = 1000;
  do
    {
      mpsm = ra8p_eth_getreg32(priv->mdiobase, RA8P_MDIO_MPSM_OFFSET);
      if ((mpsm & (RA8P_MDIO_MPSM_MPR | RA8P_MDIO_MPSM_MPW)) == 0)
        {
          break;
        }
      up_udelay(1);
    }
  while (--timeout > 0);

  if (timeout == 0)
    {
      return -ETIMEDOUT;
    }

  /* Set PHY address and register address */

  mpsm = (phyaddr << 22) | (regaddr << 18) | RA8P_MDIO_MPSM_CLS |
         (RA8P_ETH_MDC_DIV << 0) | RA8P_MDIO_MPSM_MPE | RA8P_MDIO_MPSM_MMR;

  ra8p_eth_putreg32(priv->mdiobase, RA8P_MDIO_MPSM_OFFSET, mpsm);

  /* Wait for read completion */

  timeout = 1000;
  do
    {
      mpsm = ra8p_eth_getreg32(priv->mdiobase, RA8P_MDIO_MPSM_OFFSET);
      if ((mpsm & RA8P_MDIO_MPSM_MPR) == 0)
        {
          break;
        }
      up_udelay(1);
    }
  while (--timeout > 0);

  if (timeout == 0)
    {
      return -ETIMEDOUT;
    }

  /* Read data */

  *value = ra8p_eth_getreg32(priv->mdiobase, RA8P_MDIO_MPDRC_OFFSET) & 0xFFFF;

  return OK;
}

static int ra8p_eth_mdio_write(struct ra8p_eth_dev_s *priv,
                                uint8_t phyaddr, uint8_t regaddr, uint16_t value)
{
  uint32_t mpsm;
  uint32_t timeout;

  /* Wait for previous operation to complete */

  timeout = 1000;
  do
    {
      mpsm = ra8p_eth_getreg32(priv->mdiobase, RA8P_MDIO_MPSM_OFFSET);
      if ((mpsm & (RA8P_MDIO_MPSM_MPR | RA8P_MDIO_MPSM_MPW)) == 0)
        {
          break;
        }
      up_udelay(1);
    }
  while (--timeout > 0);

  if (timeout == 0)
    {
      return -ETIMEDOUT;
    }

  /* Write data */

  ra8p_eth_putreg32(priv->mdiobase, RA8P_MDIO_MPDWC_OFFSET, value);

  /* Set PHY address and register address */

  mpsm = (phyaddr << 22) | (regaddr << 18) | RA8P_MDIO_MPSM_CLS |
         (RA8P_ETH_MDC_DIV << 0) | RA8P_MDIO_MPSM_MPE | RA8P_MDIO_MPSM_MPW;

  ra8p_eth_putreg32(priv->mdiobase, RA8P_MDIO_MPSM_OFFSET, mpsm);

  /* Wait for write completion */

  timeout = 1000;
  do
    {
      mpsm = ra8p_eth_getreg32(priv->mdiobase, RA8P_MDIO_MPSM_OFFSET);
      if ((mpsm & RA8P_MDIO_MPSM_MPW) == 0)
        {
          break;
        }
      up_udelay(1);
    }
  while (--timeout > 0);

  if (timeout == 0)
    {
      return -ETIMEDOUT;
    }

  return OK;
}

static int ra8p_eth_phy_init(struct ra8p_eth_dev_s *priv)
{
  uint16_t phyid;
  uint16_t status;
  int ret;

  /* Read PHY ID */

  ret = ra8p_eth_mdio_read(priv, RA8P_ETH_PHY_ADDR, 0x02, &phyid);
  if (ret < 0)
    {
      nerr("ERROR: Failed to read PHY ID: %d\n", ret);
      return ret;
    }

  ninfo("PHY ID: 0x%04x\n", phyid);

  /* Read PHY status */

  ret = ra8p_eth_mdio_read(priv, RA8P_ETH_PHY_ADDR, 0x01, &status);
  if (ret < 0)
    {
      nerr("ERROR: Failed to read PHY status: %d\n", ret);
      return ret;
    }

  /* Check link status */

  if (status & 0x0004) /* Link is up */
    {
      priv->linkup = true;

      /* Determine speed and duplex */

      if (status & 0x2000) /* 100BASE-TX Full Duplex */
        {
          priv->speed = 1;
          priv->duplex = 1;
        }
      else if (status & 0x1000) /* 100BASE-TX */
        {
          priv->speed = 1;
          priv->duplex = 0;
        }
      else if (status & 0x0100) /* 10BASE-T Full Duplex */
        {
          priv->speed = 0;
          priv->duplex = 1;
        }
      else /* 10BASE-T */
        {
          priv->speed = 0;
          priv->duplex = 0;
        }

      ninfo("Link up: %s Mbps %s duplex\n",
            priv->speed == 2 ? "1000" : (priv->speed == 1 ? "100" : "10"),
            priv->duplex ? "full" : "half");
    }
  else
    {
      priv->linkup = false;
      ninfo("Link down\n");
    }

  return OK;
}

static void ra8p_eth_tx_setup(struct ra8p_eth_dev_s *priv)
{
  int i;

  /* Initialize TX descriptors */

  for (i = 0; i < RA8P_ETH_TX_DESCRIPTORS; i++)
    {
      g_txdesc[i].tdes0 = 0;
      g_txdesc[i].tdes1 = 0;
      g_txdesc[i].tdes2 = (uint32_t)&g_txbuf[i].data;
      g_txdesc[i].tdes3 = 0;
    }

  /* Mark last descriptor */

  g_txdesc[RA8P_ETH_TX_DESCRIPTORS - 1].tdes1 = (1 << 30); /* Set OWN and TCH */

  /* Set descriptor list address */

  ra8p_eth_putreg32(priv->edmabase, RA8P_EDMAC_TDLAR_OFFSET,
                     (uint32_t)g_txdesc);

  priv->tx_head = 0;
  priv->tx_tail = 0;
}

static void ra8p_eth_rx_setup(struct ra8p_eth_dev_s *priv)
{
  int i;

  /* Initialize RX descriptors */

  for (i = 0; i < RA8P_ETH_RX_DESCRIPTORS; i++)
    {
      g_rxdesc[i].tdes0 = (1 << 31); /* Set OWN */
      g_rxdesc[i].tdes1 = (1 << 30) | RA8P_ETH_RX_BUFFER_SIZE; /* Set TCH and RBS1 */
      g_rxdesc[i].tdes2 = (uint32_t)&g_rxbuf[i].data;
      g_rxdesc[i].tdes3 = 0;
    }

  /* Mark last descriptor */

  g_rxdesc[RA8P_ETH_RX_DESCRIPTORS - 1].tdes1 |= (1 << 24); /* Set LDE */

  /* Set descriptor list address */

  ra8p_eth_putreg32(priv->edmabase, RA8P_EDMAC_RDLAR_OFFSET,
                     (uint32_t)g_rxdesc);

  priv->rx_head = 0;
}

static int ra8p_eth_hw_init(struct ra8p_eth_dev_s *priv)
{
  /* Reset hardware */

  ra8p_eth_reset(priv);

  /* Initialize PHY */

  ra8p_eth_phy_init(priv);

  /* Set MAC address */

  ra8p_eth_putreg32(priv->ethbase, RA8P_ETHERC_MAHR_OFFSET,
                     ((uint32_t)priv->dev.dev_addr[0] << 24) |
                     ((uint32_t)priv->dev.dev_addr[1] << 16) |
                     ((uint32_t)priv->dev.dev_addr[2] << 8) |
                     ((uint32_t)priv->dev.dev_addr[3]));

  ra8p_eth_putreg32(priv->ethbase, RA8P_ETHERC_MALR_OFFSET,
                     ((uint32_t)priv->dev.dev_addr[4] << 24) |
                     ((uint32_t)priv->dev.dev_addr[5] << 16));

  /* Configure ETHERC */

  uint32_t ecmr = RA8P_ETHERC_ECMR_RE | RA8P_ETHERC_ECMR_TE;

  if (priv->duplex == 1)
    {
      ecmr |= RA8P_ETHERC_ECMR_DM; /* Full duplex */
    }

  ra8p_eth_putreg32(priv->ethbase, RA8P_ETHERC_ECMR_OFFSET, ecmr);

  /* Set receive frame length */

  ra8p_eth_putreg32(priv->ethbase, RA8P_ETHERC_RFLR_OFFSET,
                     RA8P_ETH_FRAME_SIZE);

  /* Configure EDMAC */

  /* Set receive mode */

  ra8p_eth_putreg32(priv->edmabase, RA8P_EDMAC_RMCR_OFFSET, 0);

  /* Clear status flags */

  ra8p_eth_putreg32(priv->edmabase, RA8P_EDMAC_EESR_OFFSET, 0xFFFFFFFF);

  /* Set transmit FIFO threshold */

  ra8p_eth_putreg32(priv->edmabase, RA8P_EDMAC_TFTR_OFFSET, 0);

  /* Set FIFO depth */

  ra8p_eth_putreg32(priv->edmabase, RA8P_EDMAC_FDR_OFFSET, 0x0F);

  /* Setup TX/RX descriptors */

  ra8p_eth_tx_setup(priv);
  ra8p_eth_rx_setup(priv);

  return OK;
}

static void ra8p_eth_start(struct ra8p_eth_dev_s *priv)
{
  /* Enable receive */

  ra8p_eth_modifyreg32(priv->ethbase, RA8P_ETHERC_ECMR_OFFSET,
                       0, RA8P_ETHERC_ECMR_RE);

  /* Enable transmit */

  ra8p_eth_modifyreg32(priv->ethbase, RA8P_ETHERC_ECMR_OFFSET,
                       0, RA8P_ETHERC_ECMR_TE);

  /* Enable EDMAC receive */

  ra8p_eth_putreg32(priv->edmabase, RA8P_EDMAC_EDRRR_OFFSET,
                    RA8P_EDMAC_EDRRR_RR);

  /* Enable EDMAC transmit */

  ra8p_eth_putreg32(priv->edmabase, RA8P_EDMAC_EDTRR_OFFSET,
                    RA8P_EDMAC_EDTRR_TR);
}

static void ra8p_eth_stop(struct ra8p_eth_dev_s *priv)
{
  /* Disable receive and transmit */

  ra8p_eth_modifyreg32(priv->ethbase, RA8P_ETHERC_ECMR_OFFSET,
                        RA8P_ETHERC_ECMR_RE | RA8P_ETHERC_ECMR_TE, 0);

  /* Disable EDMAC */

  ra8p_eth_putreg32(priv->edmabase, RA8P_EDMAC_EDRRR_OFFSET, 0);
  ra8p_eth_putreg32(priv->edmabase, RA8P_EDMAC_EDTRR_OFFSET, 0);
}

static int ra8p_eth_tx_packet(struct ra8p_eth_dev_s *priv,
                              struct netdev_var_s *netvar)
{
  struct ra8p_eth_desc_s *desc;
  uint32_t status;
  uint16_t len;
  int ret = OK;

  /* Check if TX descriptor is available */

  desc = &g_txdesc[priv->tx_head];

  /* Wait for descriptor to be available */

  uint32_t timeout = 10000;
  while ((desc->tdes0 & (1 << 31)) != 0) /* OWN bit set */
    {
      up_udelay(1);
      if (--timeout == 0)
        {
          nerr("ERROR: TX timeout\n");
          return -ETIMEDOUT;
        }
    }

  /* Copy data to TX buffer */

  len = netvar->var_len;
  if (len > RA8P_ETH_TX_BUFFER_SIZE)
    {
      len = RA8P_ETH_TX_BUFFER_SIZE;
    }

  memcpy(g_txbuf[priv->tx_head].data, netvar->var_buf, len);

  /* Setup TX descriptor */

  desc->tdes1 = (len & 0x3FFF) | (1 << 30); /* Set TCH and RBS1 */
  desc->tdes0 = (1 << 28) | (1 << 31); /* Set CIC and OWN */

  /* Advance TX head */

  priv->tx_head = (priv->tx_head + 1) % RA8P_ETH_TX_DESCRIPTORS;

  /* Start transmission */

  ra8p_eth_putreg32(priv->edmabase, RA8P_EDMAC_EDTRR_OFFSET,
                    RA8P_EDMAC_EDTRR_TR);

  return ret;
}

static void ra8p_eth_rx_packet(struct ra8p_eth_dev_s *priv)
{
  struct ra8p_eth_desc_s *desc;
  uint16_t len;

  /* Check for received packets */

  desc = &g_rxdesc[priv->rx_head];

  if ((desc->tdes0 & (1 << 31)) == 0) /* OWN bit cleared */
    {
      /* Check for errors */

      if ((desc->tdes0 & (1 << 15)) == 0) /* No errors */
        {
          /* Get frame length */

          len = (desc->tdes0 >> 16) & 0x3FFF;

          if (len > 0 && len <= RA8P_ETH_RX_BUFFER_SIZE)
            {
              /* Process received packet */

              priv->rx_packets++;

              /* Give packet to network stack */

              netdev_input(&priv->dev, g_rxbuf[priv->rx_head].data,
                           len, 0);
            }
        }
      else
        {
          /* Frame has errors */

          priv->rx_errors++;
        }

      /* Reinitialize descriptor */

      desc->tdes0 = (1 << 31); /* Set OWN */
      desc->tdes1 = (1 << 30) | RA8P_ETH_RX_BUFFER_SIZE; /* Set TCH and RBS1 */

      /* Advance RX head */

      priv->rx_head = (priv->rx_head + 1) % RA8P_ETH_RX_DESCRIPTORS;
    }
}

/****************************************************************************
 * Name: ra8p_eth_interrupt
 ****************************************************************************/

static int ra8p_eth_interrupt(int irq, void *context, void *arg)
{
  struct ra8p_eth_dev_s *priv = (struct ra8p_eth_dev_s *)arg;
  uint32_t status;

  /* Read interrupt status */

  status = ra8p_eth_getreg32(priv->edmabase, RA8P_EDMAC_EESR_OFFSET);

  /* Clear interrupt flags */

  ra8p_eth_putreg32(priv->edmabase, RA8P_EDMAC_EESR_OFFSET, status);

  /* Handle TX complete */

  if (status & RA8P_EDMAC_EESR_TC)
    {
      priv->tx_packets++;
    }

  /* Handle RX complete */

  if (status & RA8P_EDMAC_EESR_FR)
    {
      /* Process all received packets */

      while ((g_rxdesc[priv->rx_head].tdes0 & (1 << 31)) == 0)
        {
          ra8p_eth_rx_packet(priv);
        }
    }

  /* Handle errors */

  if (status & (RA8P_EDMAC_EESR_RFOF | RA8P_EDMAC_EESR_TFUF))
    {
      nerr("ERROR: FIFO overflow/underflow\n");
    }

  return OK;
}

/****************************************************************************
 * Name: ra8p_eth_txnotify
 *
 * Description:
 *   Notify the Ethernet driver that new TX data is available
 *
 ****************************************************************************/

static void ra8p_eth_txnotify(FAR struct netdev_s *dev)
{
  struct ra8p_eth_dev_s *priv = (struct ra8p_eth_dev_s *)dev;

  /* Check if device is ready for transmission */

  if (priv->initialized && priv->linkup)
    {
      /* Start transmission */

      ra8p_eth_putreg32(priv->edmabase, RA8P_EDMAC_EDTRR_OFFSET,
                        RA8P_EDMAC_EDTRR_TR);
    }
}

/****************************************************************************
 * Name: ra8p_eth_ifup
 *
 * Description:
 *   Bring up the Ethernet interface
 *
 ****************************************************************************/

static int ra8p_eth_ifup(FAR struct netdev_s *dev)
{
  struct ra8p_eth_dev_s *priv = (struct ra8p_eth_dev_s *)dev;

  nxmutex_lock(&priv->lock);

  /* Initialize hardware */

  int ret = ra8p_eth_hw_init(priv);
  if (ret < 0)
    {
      nxmutex_unlock(&priv->lock);
      return ret;
    }

  /* Start device */

  ra8p_eth_start(priv);

  priv->initialized = true;

  nxmutex_unlock(&priv->lock);
  return OK;
}

/****************************************************************************
 * Name: ra8p_eth_ifdown
 *
 * Description:
 *   Take down the Ethernet interface
 *
 ****************************************************************************/

static void ra8p_eth_ifdown(FAR struct netdev_s *dev)
{
  struct ra8p_eth_dev_s *priv = (struct ra8p_eth_dev_s *)dev;

  nxmutex_lock(&priv->lock);

  /* Stop device */

  ra8p_eth_stop(priv);

  priv->initialized = false;

  nxmutex_unlock(&priv->lock);
}

/****************************************************************************
 * Name: ra8p_eth_send
 *
 * Description:
 *   Send a packet
 *
 ****************************************************************************/

static int ra8p_eth_send(FAR struct netdev_s *dev, FAR struct netdev_var_s *netvar)
{
  struct ra8p_eth_dev_s *priv = (struct ra8p_eth_dev_s *)dev;
  int ret;

  nxmutex_lock(&priv->lock);

  /* Check if link is up */

  if (!priv->linkup)
    {
      nxmutex_unlock(&priv->lock);
      return -ENETUNREACH;
    }

  /* Transmit packet */

  ret = ra8p_eth_tx_packet(priv, netvar);

  nxmutex_unlock(&priv->lock);
  return ret;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

int ra8p_ethernet_initialize(void)
{
  struct ra8p_eth_dev_s *priv = &g_eth0_priv;
  int ret;

  /* Initialize private data */

  memset(priv, 0, sizeof(struct ra8p_eth_dev_s));
  priv->ethbase = RA8P_ETHERC0_BASE;
  priv->edmabase = RA8P_ETHERC0_BASE + 0x1000; /* EDMAC offset */
  priv->mdiobase = RA8P_MDIO0_BASE;
  priv->irq = RA8P_IRQ_ETH;

  nxmutex_init(&priv->lock);

  /* Set default MAC address */

  priv->dev.dev_addr[0] = 0x74;
  priv->dev.dev_addr[1] = 0x90;
  priv->dev.dev_addr[2] = 0x50;
  priv->dev.dev_addr[3] = 0x01;
  priv->dev.dev_addr[4] = 0x02;
  priv->dev.dev_addr[5] = 0x03;
  priv->dev.dev_addr_len = 6;

  /* Set network device operations */

  priv->dev.ifup    = ra8p_eth_ifup;
  priv->dev.ifdown  = ra8p_eth_ifdown;
  priv->dev.send    = ra8p_eth_send;
  priv->dev.txnotify = ra8p_eth_txnotify;

  /* Attach interrupt handler */

  ret = irq_attach(priv->irq, ra8p_eth_interrupt, priv);
  if (ret < 0)
    {
      goto errout_with_mutex;
    }

  /* Enable interrupt */

  up_enable_irq(priv->irq);

  /* Register network device */

  ret = netdev_register(&priv->dev, NET_LL_ETHERNET);
  if (ret < 0)
    {
      goto errout_with_irq;
    }

  return OK;

errout_with_irq:
  up_disable_irq(priv->irq);
  irq_detach(priv->irq);
errout_with_mutex:
  nxmutex_destroy(&priv->lock);
  return ret;
}

#endif /* CONFIG_RA8P_ETHERNET */