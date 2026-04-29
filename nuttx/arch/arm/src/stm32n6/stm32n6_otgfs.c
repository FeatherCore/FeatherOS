/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_otgfs.c
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
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <errno.h>
#include <unistd.h>

#include <nuttx/arch.h>
#include <nuttx/irq.h>
#include <nuttx/kmalloc.h>
#include <nuttx/usb/usb.h>
#include <nuttx/usb/usbdev.h>
#include <nuttx/usb/usbdev_trace.h>

#include "arm_internal.h"
#include "chip.h"
#include "stm32n6_gpio.h"
#include "hardware/stm32n6_memorymap.h"
#include "hardware/stm32n6_rcc.h"
#include "stm32n6_otgfs.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define USBDEV_MAXPACKET         64   /* Full speed max packet size */

/* Register access macros */

#define stm32n6_getreg32(a)          getreg32(a)
#define stm32n6_putreg32(v,a)        putreg32(v,a)

/* Interrupt mask for device mode */

#define OTG_GINTMSK_DEVINT \
  (OTG_GINTMSK_USBRST | OTG_GINTMSK_ENUMDNEM | OTG_GINTMSK_IEPINT | \
   OTG_GINTMSK_OEPINT | OTG_GINTMSK_RXFLVLM | OTG_GINTMSK_USBSUSPM | \
   OTG_GINTMSK_WUIM)

/* EP0 state definitions */

#define EP0STATE_IDLE              0
#define EP0STATE_SETUP             1
#define EP0STATE_DATA              2
#define EP0STATE_SHORTREAD         3

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* Represents one endpoint */

struct stm32n6_ep_s
{
  struct usbdev_ep_s      ep;        /* USB device endpoint */
  struct stm32n6_usbdev_s *dev;      /* Driver instance */
  uint8_t                 eptype;    /* Endpoint type */
  uint8_t                 epnum;     /* Endpoint number (0-3) */
  uint8_t                 inuse;     /* Non-zero if endpoint is in use */
  uint16_t                maxpacketsize; /* Maximum packet size */
  uint8_t                 stalled;   /* 1: Endpoint is stalled */
  struct stm32n6_req_s   *head;      /* Head of request queue */
  struct stm32n6_req_s   *tail;      /* Tail of request queue */
};

/* Represents one request on an endpoint */

struct stm32n6_req_s
{
  struct usbdev_req_s     req;       /* Standard USB request */
  struct stm32n6_ep_s    *ep;        /* Bound endpoint */
  struct stm32n6_req_s   *flink;     /* Next request in queue */
};

/* Container for a device instance */

struct stm32n6_usbdev_s
{
  struct usbdev_s         usbdev;    /* USB device */
  uint32_t                base;      /* Base address of registers */
  uint8_t                 attached;  /* TRUE: device is attached */
  uint8_t                 suspended; /* TRUE: device is suspended */
  uint8_t                 selfpowered; /* TRUE: device is self-powered */
  uint8_t                 addressed; /* TRUE: device is addressed */
  uint8_t                 configured; /* TRUE: device is configured */
  uint8_t                 setaddress; /* New device address */
  uint8_t                 ep0state;  /* EP0 state */
  uint8_t                 neps;      /* Number of endpoints */
  struct stm32n6_ep_s     epin[STM32N6_OTGFS_NENDPOINTS];   /* IN endpoints */
  struct stm32n6_ep_s     epout[STM32N6_OTGFS_NENDPOINTS];  /* OUT endpoints */
  struct stm32n6_req_s    ep0req;    /* EP0 request structure */
  struct stm32n6_ep_s     ep0;       /* EP0 endpoint structure */
  uint32_t                rxflen;    /* Size of RX FIFO */
  uint32_t                nptxflen;  /* Size of non-periodic TX FIFO */
  uint32_t                ptxflen;   /* Size of periodic TX FIFO */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

/* Register access */

static inline uint32_t stm32n6_otg_getreg(uint32_t addr);
static inline void stm32n6_otg_putreg(uint32_t val, uint32_t addr);

/* Request list operations */

static void stm32n6_rqcancel(struct stm32n6_ep_s *privep, 
                             struct stm32n6_req_s *privreq, int status);

/* Endpoint operations */

static int stm32n6_epconfigure(struct usbdev_ep_s *ep, 
                               const struct usb_epdesc_s *desc);
static int stm32n6_epdisable(struct usbdev_ep_s *ep);
static struct usbdev_req_s *stm32n6_epallocreq(struct usbdev_ep_s *ep);
static void stm32n6_epfreereq(struct usbdev_ep_s *ep, 
                              struct usbdev_req_s *req);
static int stm32n6_epsubmit(struct usbdev_ep_s *ep, 
                            struct usbdev_req_s *req);
static int stm32n6_epcancel(struct usbdev_ep_s *ep, 
                            struct usbdev_req_s *req);
static int stm32n6_epstall(struct usbdev_ep_s *ep, bool resume);

/* USB device operation methods */

static int stm32n6_drvrbind(struct usbdev_s *dev, 
                            struct usbdevdriver_s *driver);
static int stm32n6_drvrunbind(struct usbdev_s *dev, 
                              struct usbdevdriver_s *driver);
static int stm32n6_drvrsetup(struct usbdev_s *dev, 
                             const struct usb_ctrlreq_s *ctrl);
static ssize_t stm32n6_drvrtrace(struct usbdev_s *dev, 
                                 int ntrace, bool verbose);

/* USB controller operations */

static void stm32n6_usbreset(struct stm32n6_usbdev_s *priv);
static void stm32n6_flush_rx(struct stm32n6_usbdev_s *priv);
static void stm32n6_flush_tx(struct stm32n6_usbdev_s *priv, uint32_t txfnum);
static void stm32n6_setaddress(struct stm32n6_usbdev_s *priv, uint16_t address);
static void stm32n6_enableep(struct stm32n6_usbdev_s *priv, int epno);
static void stm32n6_disableep(struct stm32n6_usbdev_s *priv, int epno);
static void stm32n6_seteptype(struct stm32n6_usbdev_s *priv, int epno, uint8_t eptype);
static void stm32n6_setmps(struct stm32n6_usbdev_s *priv, int epno, uint16_t mps);

/* FIFO management */

static void stm32n6_initfifo(struct stm32n6_usbdev_s *priv);

/* Interrupt handling */

static int stm32n6_usbinterrupt(int irq, void *context, void *arg);

/* Power management */

static int stm32n6_usbpullup(struct usbdev_s *dev, bool enable);
static int stm32n6_usbsuspend(struct usbdev_s *dev);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static const struct usbdev_ops_s g_usbdev_ops =
{
  .ep_configure   = stm32n6_epconfigure,
  .ep_disable    = stm32n6_epdisable,
  .ep_allocreq   = stm32n6_epallocreq,
  .ep_freereq    = stm32n6_epfreereq,
  .ep_submit     = stm32n6_epsubmit,
  .ep_cancel     = stm32n6_epcancel,
  .ep_stall      = stm32n6_epstall,
  .pullup        = stm32n6_usbpullup,
  .suspend       = stm32n6_usbsuspend,
  .resume        = NULL,
  .wakeup        = NULL,
  .selfpowered   = NULL,
  .setaddress    = NULL,
};

static struct stm32n6_usbdev_s g_otgdev;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32n6_otg_getreg/stm32n6_otg_putreg
 ****************************************************************************/

static inline uint32_t stm32n6_otg_getreg(uint32_t addr)
{
  return getreg32(addr);
}

static inline void stm32n6_otg_putreg(uint32_t val, uint32_t addr)
{
  putreg32(val, addr);
}

/****************************************************************************
 * Name: stm32n6_rqcancel
 ****************************************************************************/

static void stm32n6_rqcancel(struct stm32n6_ep_s *privep, 
                             struct stm32n6_req_s *privreq, int status)
{
  struct stm32n6_usbdev_s *priv = privep->dev;

  usbdbg("EP%d: cancelling req=%p\n", privep->epnum, privreq);

  /* Remove the request from the endpoint's queue */

  if (privep->head == privreq)
    {
      /* Removing the first request in the queue */

      privep->head = privreq->flink;
      if (privep->tail == privreq)
        {
          /* This is also the last request in the queue */

          privep->tail = NULL;
        }
    }
  else
    {
      /* Remove from somewhere in the middle of the queue */

      struct stm32n6_req_s *prev = privep->head;
      while (prev && prev->flink != privreq)
        {
          prev = prev->flink;
        }

      if (prev)
        {
          prev->flink = privreq->flink;
          if (privep->tail == privreq)
            {
              privep->tail = prev;
            }
        }
    }

  /* Return the request to the free list */

  privreq->flink = priv->free;
  priv->free = privreq;

  /* Callback to the request */

  usbdbg("Callback req=%p status=%d\n", privreq, status);
  if (privreq->req.callback)
    {
      privreq->req.callback(&privep->ep, &privreq->req, status);
    }
}

/****************************************************************************
 * Name: stm32n6_epconfigure
 ****************************************************************************/

static int stm32n6_epconfigure(struct usbdev_ep_s *ep, 
                               const struct usb_epdesc_s *desc)
{
  struct stm32n6_ep_s *privep = (struct stm32n6_ep_s *)ep;
  struct stm32n6_usbdev_s *priv = privep->dev;
  uint32_t regaddr;
  uint32_t regval;
  int epno;

  usbdbg("ep=%p desc=%p\n", ep, desc);

  if (!desc || !privep || !priv)
    {
      return -EINVAL;
    }

  /* Extract endpoint number */

  epno = USB_EPNO(desc->bEndpointAddress);
  if (epno >= STM32N6_OTGFS_NENDPOINTS)
    {
      return -EINVAL;
    }

  /* Set endpoint configuration */

  privep->eptype = desc->bmAttributes & USB_EPTYPE_MASK;
  privep->maxpacketsize = le16toh(desc->wMaxPacketSize);
  privep->epnum = epno;

  /* Configure endpoint register based on direction */

  if (USB_EPDIR(desc->bEndpointAddress) == USB_EPDIR_IN)
    {
      /* Configure IN endpoint */
      regaddr = STM32_OTG_DIEPCTL_OFFSET(epno) + priv->base;
      regval = stm32n6_getreg32(regaddr);

      /* Clear existing configuration */
      regval &= ~(OTG_DEPCTL_MPSIZ_MASK | OTG_DEPCTL_EPTYP_MASK | OTG_DEPCTL_USBAEP);
      
      /* Set max packet size */
      regval |= (privep->maxpacketsize & 0x7FF) << OTG_DEPCTL_MPSIZ_SHIFT;
      
      /* Set endpoint type */
      switch (privep->eptype)
        {
          case USB_EPTYPE_ISOC:
            regval |= OTG_DEPCTL_EPTYP_ISO;
            break;
          case USB_EPTYPE_BULK:
            regval |= OTG_DEPCTL_EPTYP_BULK;
            break;
          case USB_EPTYPE_INTR:
            regval |= OTG_DEPCTL_EPTYP_INTR;
            break;
          default:
            regval |= OTG_DEPCTL_EPTYP_CTRL;
            break;
        }

      /* Set USB active endpoint */
      regval |= OTG_DEPCTL_USBAEP;
      
      /* Set endpoint number */
      regval |= (epno << 22) & (0xF << 22);
      
      stm32n6_putreg32(regval, regaddr);
    }
  else
    {
      /* Configure OUT endpoint */
      regaddr = STM32_OTG_DOEPCTL_OFFSET(epno) + priv->base;
      regval = stm32n6_getreg32(regaddr);

      /* Clear existing configuration */
      regval &= ~(OTG_DEPCTL_MPSIZ_MASK | OTG_DEPCTL_EPTYP_MASK);
      
      /* Set max packet size */
      regval |= (privep->maxpacketsize & 0x7FF) << OTG_DEPCTL_MPSIZ_SHIFT;
      
      /* Set endpoint type */
      switch (privep->eptype)
        {
          case USB_EPTYPE_ISOC:
            regval |= OTG_DEPCTL_EPTYP_ISO;
            break;
          case USB_EPTYPE_BULK:
            regval |= OTG_DEPCTL_EPTYP_BULK;
            break;
          case USB_EPTYPE_INTR:
            regval |= OTG_DEPCTL_EPTYP_INTR;
            break;
          default:
            regval |= OTG_DEPCTL_EPTYP_CTRL;
            break;
        }

      /* Set USB active endpoint */
      regval |= OTG_DEPCTL_USBAEP;
      
      /* Set endpoint number */
      regval |= (epno << 22) & (0xF << 22);
      
      stm32n6_putreg32(regval, regaddr);
    }

  /* Mark endpoint as in use */
  privep->inuse = 1;

  return OK;
}

/****************************************************************************
 * Name: stm32n6_epdisable
 ****************************************************************************/

static int stm32n6_epdisable(struct usbdev_ep_s *ep)
{
  struct stm32n6_ep_s *privep = (struct stm32n6_ep_s *)ep;
  struct stm32n6_usbdev_s *priv;
  irqstate_t flags;
  uint32_t regaddr;
  uint32_t regval;

  usbdbg("ep=%p\n", ep);
  if (!ep || !privep)
    {
      return -EINVAL;
    }

  priv = (struct stm32n6_usbdev_s *)privep->dev;
  if (!priv)
    {
      return -EINVAL;
    }

  flags = enter_critical_section();

  /* Disable endpoint */
  if (privep->epnum == 0 || (privep->epnum > 0 && privep->epnum < 4))
    {
      if (USB_EPDIR(privep->ep.ep_in) == USB_EPDIR_IN)
        {
          regaddr = STM32_OTG_DIEPCTL_OFFSET(privep->epnum) + priv->base;
          regval = stm32n6_getreg32(regaddr);
          regval |= OTG_DEPCTL_EPDIS;
          stm32n6_putreg32(regval, regaddr);
        }
      else
        {
          regaddr = STM32_OTG_DOEPCTL_OFFSET(privep->epnum) + priv->base;
          regval = stm32n6_getreg32(regaddr);
          regval |= OTG_DEPCTL_EPDIS;
          stm32n6_putreg32(regval, regaddr);
        }
    }

  /* Cancel all queued transfers */
  while (privep->head)
    {
      stm32n6_rqcancel(privep, privep->head, -ESHUTDOWN);
    }

  /* Mark endpoint as no longer in use */
  privep->inuse = 0;
  privep->stalled = 0;

  leave_critical_section(flags);
  return OK;
}

/****************************************************************************
 * Name: stm32n6_epallocreq
 ****************************************************************************/

static struct usbdev_req_s *stm32n6_epallocreq(struct usbdev_ep_s *ep)
{
  struct stm32n6_req_s *privreq;

  usbdbg("ep=%p\n", ep);
  privreq = (struct stm32n6_req_s *)kmm_malloc(sizeof(struct stm32n6_req_s));
  if (!privreq)
    {
      usbdbg("Failed to allocate request\n");
      return NULL;
    }

  memset(privreq, 0, sizeof(struct stm32n6_req_s));
  return &privreq->req;
}

/****************************************************************************
 * Name: stm32n6_epfreereq
 ****************************************************************************/

static void stm32n6_epfreereq(struct usbdev_ep_s *ep, struct usbdev_req_s *req)
{
  struct stm32n6_req_s *privreq = (struct stm32n6_req_s *)req;
  DEBUGASSERT(privreq);

  usbdbg("ep=%p req=%p\n", ep, req);
  kmm_free(privreq);
}

/****************************************************************************
 * Name: stm32n6_epsubmit
 ****************************************************************************/

static int stm32n6_epsubmit(struct usbdev_ep_s *ep, struct usbdev_req_s *req)
{
  struct stm32n6_ep_s *privep = (struct stm32n6_ep_s *)ep;
  struct stm32n6_usbdev_s *priv;
  struct stm32n6_req_s *privreq;
  irqstate_t flags;
  uint32_t regaddr;
  uint32_t regval;
  uint32_t xfrsize;

  usbdbg("ep=%p req=%p\n", ep, req);
  if (!req || !ep || !privep || !privep->dev)
    {
      return -EINVAL;
    }

  usbdbg("len=%d flags=%02x\n", req->len, req->flags);

  priv = (struct stm32n6_usbdev_s *)privep->dev;
  privreq = (struct stm32n6_req_s *)req;

  if (!req->callback || !req->buf)
    {
      usbdbg("Invalid request\n");
      return -EINVAL;
    }

  flags = enter_critical_section();

  /* Add the request to the endpoint's queue */

  privreq->flink = NULL;
  privreq->ep = privep;

  if (!privep->head)
    {
      /* First request in the queue */

      privep->head = privreq;
      privep->tail = privreq;

      /* Configure transfer in hardware */
      if (USB_EPDIR(ep->ep_in) == USB_EPDIR_IN)
        {
          /* IN endpoint - data goes from memory to device */
          regaddr = STM32_OTG_DIEPTSIZ_OFFSET(privep->epnum) + priv->base;
          xfrsize = MIN(req->len, privep->maxpacketsize);
          
          regval = stm32n6_getreg32(regaddr);
          regval &= ~OTG_DEPTSIZ_XFRSIZ_MASK;
          regval |= xfrsize & OTG_DEPTSIZ_XFRSIZ_MASK;
          
          /* Set packet count */
          uint32_t pktcnt = (req->len + privep->maxpacketsize - 1) / privep->maxpacketsize;
          regval &= ~OTG_DEPTSIZ_PKTCNT_MASK;
          regval |= (pktcnt << OTG_DEPTSIZ_PKTCNT_SHIFT) & OTG_DEPTSIZ_PKTCNT_MASK;
          
          stm32n6_putreg32(regval, regaddr);

          /* Enable endpoint */
          regaddr = STM32_OTG_DIEPCTL_OFFSET(privep->epnum) + priv->base;
          regval = stm32n6_getreg32(regaddr);
          regval |= OTG_DEPCTL_EPENA | OTG_DEPCTL_CNAK;
          stm32n6_putreg32(regval, regaddr);
        }
      else
        {
          /* OUT endpoint - data comes from device to memory */
          regaddr = STM32_OTG_DOEPTSIZ_OFFSET(privep->epnum) + priv->base;
          xfrsize = MIN(req->len, privep->maxpacketsize);
          
          regval = stm32n6_getreg32(regaddr);
          regval &= ~OTG_DEPTSIZ_XFRSIZ_MASK;
          regval |= xfrsize & OTG_DEPTSIZ_XFRSIZ_MASK;
          
          /* Set packet count */
          uint32_t pktcnt = (req->len + privep->maxpacketsize - 1) / privep->maxpacketsize;
          regval &= ~OTG_DEPTSIZ_PKTCNT_MASK;
          regval |= (pktcnt << OTG_DEPTSIZ_PKTCNT_SHIFT) & OTG_DEPTSIZ_PKTCNT_MASK;
          
          stm32n6_putreg32(regval, regaddr);

          /* Enable endpoint */
          regaddr = STM32_OTG_DOEPCTL_OFFSET(privep->epnum) + priv->base;
          regval = stm32n6_getreg32(regaddr);
          regval |= OTG_DEPCTL_EPENA | OTG_DEPCTL_CNAK;
          stm32n6_putreg32(regval, regaddr);
        }
    }
  else
    {
      /* Add to the end of the queue */
      privep->tail->flink = privreq;
      privep->tail = privreq;
    }

  leave_critical_section(flags);
  return OK;
}

/****************************************************************************
 * Name: stm32n6_drvrsetup
 ****************************************************************************/

static int stm32n6_drvrsetup(struct usbdev_s *dev, const struct usb_ctrlreq_s *ctrl)
{
  struct stm32n6_usbdev_s *priv = (struct stm32n6_usbdev_s *)dev;
  struct stm32n6_ep_s *privep;
  irqstate_t flags;
  int ret = -EIO;

  usbdbg("ctrl=%p\n", ctrl);
  usbdbg("type=%02x req=%02x value=%04x index=%04x len=%04x\n",
         ctrl->type, ctrl->req, ctrl->value, ctrl->index, ctrl->len);

  flags = enter_critical_section();

  /* Forward to the class driver */
  if (priv->usbdev.driver && priv->usbdev.driver->ops->setup)
    {
      ret = priv->usbdev.driver->ops->setup(priv->usbdev.driver, ctrl);
    }

  /* If SETUP was not processed by class driver, handle standard requests */
  if (ret == -EIO)
    {
      /* Handle standard requests */
      switch (ctrl->req)
        {
          case USB_REQ_GETDESCRIPTOR:
            {
              uint8_t type = (ctrl->value >> 8) & 0xff;
              uint8_t index = ctrl->value & 0xff;

              usbdbg("GET_DESCRIPTOR type=%d index=%d\n", type, index);

              switch (type)
                {
                  case USB_CONFIGURATION_DESCRIPTOR_TYPE:
                  case USB_DEVICE_DESCRIPTOR_TYPE:
                  case USB_STRING_DESCRIPTOR_TYPE:
                    /* Forward to class driver */
                    break;
                  default:
                    break;
                }
              break;
            }

          case USB_REQ_SETADDRESS:
            {
              uint16_t address = ctrl->value & 0x7f;
              usbdbg("SET_ADDRESS address=%d\n", address);
              priv->setaddress = address;
              break;
            }

          case USB_REQ_SETCONFIGURATION:
            {
              uint8_t config = ctrl->value & 0xff;
              usbdbg("SET_CONFIG config=%d\n", config);
              priv->configured = (config != 0) ? 1 : 0;
              break;
            }

          case USB_REQ_SETFEATURE:
          case USB_REQ_CLEARFEATURE:
            {
              uint8_t recipient = ctrl->type & USB_REQ_RECIPIENT_MASK;
              uint16_t feature = ctrl->value;

              if (recipient == USB_REQ_RECIPIENT_ENDPOINT && feature == USB_FEATURE_ENDPOINT_HALT)
                {
                  uint8_t epno = USB_EPNO(ctrl->index);
                  uint8_t dir = USB_EPDIR(ctrl->index);

                  usbdbg("SET_FEATURE CLEAR_FEATURE ENDPOINT_HALT ep=%d dir=%d\n", epno, dir);
                  privep = dir == USB_EPDIR_IN ? &priv->epin[epno] : &priv->epout[epno];
                  
                  if (ctrl->req == USB_REQ_SETFEATURE)
                    {
                      stm32n6_epstall(&privep->ep, false);
                    }
                  else
                    {
                      stm32n6_epstall(&privep->ep, true);
                    }
                }
              break;
            }

          default:
            break;
        }
    }

  leave_critical_section(flags);
  return ret;
}

/****************************************************************************
 * Name: stm32n6_drvrbind
 ****************************************************************************/

static int stm32n6_drvrbind(struct usbdev_s *dev, struct usbdevdriver_s *driver)
{
  struct stm32n6_usbdev_s *priv = (struct stm32n6_usbdev_s *)dev;
  irqstate_t flags;

  usbdbg("dev=%p driver=%p\n", dev, driver);
  if (!driver || !dev || !priv)
    {
      return -EINVAL;
    }

  flags = enter_critical_section();

  /* Bind the class driver */
  priv->usbdev.driver = driver;

  /* Enable USB controller */
  stm32n6_usbpullup(dev, true);

  /* Perform driver bind callback */
  if (driver->ops->bind)
    {
      driver->ops->bind(driver, &priv->usbdev);
    }

  leave_critical_section(flags);
  return OK;
}

/****************************************************************************
 * Name: stm32n6_usbpullup
 ****************************************************************************/

static int stm32n6_usbpullup(struct usbdev_s *dev, bool enable)
{
  struct stm32n6_usbdev_s *priv = (struct stm32n6_usbdev_s *)dev;
  uint32_t regval;

  usbdbg("enable=%d\n", enable);
  
  if (enable)
    {
      /* Enable pull-up to connect device */
      regval = stm32n6_getreg32(priv->base + STM32_OTG_GCCFG_OFFSET);
      regval |= OTG_GCCFG_PWRDWN;  /* Power down disable */
      regval |= OTG_GCCFG_VBDEN;   /* Enable VBUS detection */
      stm32n6_putreg32(regval, priv->base + STM32_OTG_GCCFG_OFFSET);

      /* Start USB controller */
      regval = stm32n6_getreg32(priv->base + STM32_OTG_DCTL_OFFSET);
      regval &= ~OTG_DCTL_SDIS;    /* Soft disconnect */
      stm32n6_putreg32(regval, priv->base + STM32_OTG_DCTL_OFFSET);
    }
  else
    {
      /* Disable pull-up to disconnect device */
      regval = stm32n6_getreg32(priv->base + STM32_OTG_DCTL_OFFSET);
      regval |= OTG_DCTL_SDIS;     /* Soft disconnect */
      stm32n6_putreg32(regval, priv->base + STM32_OTG_DCTL_OFFSET);

      regval = stm32n6_getreg32(priv->base + STM32_OTG_GCCFG_OFFSET);
      regval &= ~OTG_GCCFG_PWRDWN; /* Power down enable */
      stm32n6_putreg32(regval, priv->base + STM32_OTG_GCCFG_OFFSET);
    }

  return OK;
}

/****************************************************************************
 * Name: stm32n6_usbreset
 ****************************************************************************/

static void stm32n6_usbreset(struct stm32n6_usbdev_s *priv)
{
  uint32_t regval;
  int i;

  usbdbg("Reset USB\n");

  /* Soft disconnect device */
  regval = stm32n6_getreg32(priv->base + STM32_OTG_DCTL_OFFSET);
  regval |= OTG_DCTL_SDIS;
  stm32n6_putreg32(regval, priv->base + STM32_OTG_DCTL_OFFSET);

  /* Flush TX and RX FIFOs */
  stm32n6_flush_rx(priv);

  for (i = 0; i < STM32N6_OTGFS_NENDPOINTS; i++)
    {
      stm32n6_flush_tx(priv, i);
    }

  /* Clear device address */
  regval = stm32n6_getreg32(priv->base + STM32_OTG_DCFG_OFFSET);
  regval &= ~OTG_DCFG_DAD_MASK;
  stm32n6_putreg32(regval, priv->base + STM32_OTG_DCFG_OFFSET);

  /* Configure device as full-speed */
  regval |= OTG_DCFG_DSPD_FS_PHY_48MHZ;
  stm32n6_putreg32(regval, priv->base + STM32_OTG_DCFG_OFFSET);

  /* Disable soft disconnect */
  regval = stm32n6_getreg32(priv->base + STM32_OTG_DCTL_OFFSET);
  regval &= ~OTG_DCTL_SDIS;
  stm32n6_putreg32(regval, priv->base + STM32_OTG_DCTL_OFFSET);

  /* Reset device state */
  priv->addressed = 0;
  priv->configured = 0;
  priv->setaddress = 0;

  /* Initialize endpoints */
  for (i = 1; i < STM32N6_OTGFS_NENDPOINTS; i++)
    {
      stm32n6_disableep(priv, i);
    }
}

/****************************************************************************
 * Name: stm32n6_flush_rx/stm32n6_flush_tx
 ****************************************************************************/

static void stm32n6_flush_rx(struct stm32n6_usbdev_s *priv)
{
  uint32_t regval;

  regval = OTG_GRSTCTL_RXFFLSH;
  stm32n6_putreg32(regval, priv->base + STM32_OTG_GRSTCTL_OFFSET);

  /* Wait for flush to complete */
  while ((stm32n6_getreg32(priv->base + STM32_OTG_GRSTCTL_OFFSET) & OTG_GRSTCTL_RXFFLSH) != 0);
}

static void stm32n6_flush_tx(struct stm32n6_usbdev_s *priv, uint32_t txfnum)
{
  uint32_t regval;

  regval = OTG_GRSTCTL_TXFFLSH | ((txfnum & 0x1F) << 6);
  stm32n6_putreg32(regval, priv->base + STM32_OTG_GRSTCTL_OFFSET);

  /* Wait for flush to complete */
  while ((stm32n6_getreg32(priv->base + STM32_OTG_GRSTCTL_OFFSET) & OTG_GRSTCTL_TXFFLSH) != 0);
}

/****************************************************************************
 * Name: stm32n6_initfifo
 ****************************************************************************/

static void stm32n6_initfifo(struct stm32n6_usbdev_s *priv)
{
  uint32_t regval;

  /* Set RX FIFO size */
  regval = STM32N6_OTGFS_RXFIFO_SIZE;
  stm32n6_putreg32(regval, priv->base + STM32_OTG_GRXFIFOSIZ_OFFSET);

  /* Set endpoint 0 TX FIFO size and offset */
  regval = (STM32N6_OTGFS_NPTXFIFO_SIZE << 16) | STM32N6_OTGFS_RXFIFO_SIZE; /* Size = 128, Offset = 128 */
  stm32n6_putreg32(regval, priv->base + STM32_OTG_GNPTXFIFOSIZ_OFFSET);

  /* Set other TX FIFOs if needed */
}

/****************************************************************************
 * Name: stm32n6_usbinterrupt
 ****************************************************************************/

static int stm32n6_usbinterrupt(int irq, void *context, void *arg)
{
  struct stm32n6_usbdev_s *priv = (struct stm32n6_usbdev_s *)arg;
  uint32_t intsts;
  uint32_t intmask;
  uint32_t epint;
  int i;

  /* Get interrupt status */
  intsts = stm32n6_getreg32(priv->base + STM32_OTG_GINTSTS_OFFSET);
  intmask = stm32n6_getreg32(priv->base + STM32_OTG_GINTMSK_OFFSET);

  intsts &= intmask;

  /* Clear interrupts */
  stm32n6_putreg32(intsts, priv->base + STM32_OTG_GINTSTS_OFFSET);

  usbvdbg("intsts=%08x\n", intsts);

  /* USB reset */
  if (intsts & OTG_GINTSTS_USBRST)
    {
      usbvdbg("USB Reset\n");
      stm32n6_usbreset(priv);
      stm32n6_initfifo(priv);
      stm32n6_putreg32(0xFF, priv->base + STM32_OTG_DIEPMSK_OFFSET);
      stm32n6_putreg32(0xFF, priv->base + STM32_OTG_DOEPMSK_OFFSET);
      return OK;
    }

  /* Start of frame */
  if (intsts & OTG_GINTSTS_SOF)
    {
      usbvdbg("SOF\n");
      /* Clear SOF interrupt */
      stm32n6_putreg32(OTG_GINTSTS_SOF, priv->base + STM32_OTG_GINTSTS_OFFSET);
    }

  /* Enumeration done */
  if (intsts & OTG_GINTSTS_ENUMDNE)
    {
      usbvdbg("Enumeration Done\n");
      /* Clear enum done interrupt */
      stm32n6_putreg32(OTG_GINTSTS_ENUMDNE, priv->base + STM32_OTG_GINTSTS_OFFSET);
      
      /* Set up default EP0 */
      struct stm32n6_ep_s *ep0 = &priv->epin[0];
      ep0->epnum = 0;
      ep0->maxpacketsize = 64;
      ep0->eptype = USB_EPTYPE_CTRL;
    }

  /* Resume */
  if (intsts & OTG_GINTSTS_WKUINT)
    {
      usbvdbg("Resume\n");
      stm32n6_putreg32(OTG_GINTSTS_WKUINT, priv->base + STM32_OTG_GINTSTS_OFFSET);
      priv->suspended = false;
    }

  /* Suspend */
  if (intsts & OTG_GINTSTS_USBSUSP)
    {
      usbvdbg("Suspend\n");
      stm32n6_putreg32(OTG_GINTSTS_USBSUSP, priv->base + STM32_OTG_GINTSTS_OFFSET);
      priv->suspended = true;
    }

  /* RX FIFO not empty */
  if (intsts & OTG_GINTSTS_RXFLVL)
    {
      uint32_t grxsts;
      uint8_t epnum;
      uint8_t pktsts;
      uint16_t bcnt;
      struct stm32n6_ep_s *privep;

      /* Read RX status */
      grxsts = stm32n6_getreg32(priv->base + STM32_OTG_GRXSTSR_OFFSET);
      epnum = (grxsts >> 0) & 0xF;
      pktsts = (grxsts >> 17) & 0xF;
      bcnt = (grxsts >> 4) & 0x7FF;

      usbvdbg("RXSTS ep=%d pktsts=%d bcnt=%d\n", epnum, pktsts, bcnt);

      /* Pop RX status */
      stm32n6_putreg32(grxsts, priv->base + STM32_OTG_GRXSTSP_OFFSET);

      switch (pktsts)
        {
          case 0x01:  /* Global OUT NAK */
            break;

          case 0x02:  /* Out received */
            {
              if (epnum == 0 && bcnt > 0)
                {
                  /* EP0 OUT packet */
                  privep = &priv->epout[0];
                  if (privep->head)
                    {
                      /* Copy data to request buffer */
                      uint32_t *src = (uint32_t *)(priv->base + STM32_OTG_DFIFO_OFFSET(0));
                      uint32_t *dst = (uint32_t *)privep->head->req.buf;
                      int word_count = (bcnt + 3) / 4;

                      for (int j = 0; j < word_count; j++)
                        {
                          *dst++ = stm32n6_getreg32((uint32_t)src++);
                        }

                      privep->head->req.actual = bcnt;

                      /* Complete the request */
                      stm32n6_rqcancel(privep, privep->head, 0);
                    }
                }
              break;
            }

          case 0x03:  /* Out complete */
            break;

          case 0x04:  /* Setup received */
            {
              if (epnum == 0)
                {
                  struct usb_ctrlreq_s ctrlreq;
                  uint32_t *src = (uint32_t *)(priv->base + STM32_OTG_DFIFO_OFFSET(0));

                  /* Pop SETUP packet */
                  ((uint32_t *)&ctrlreq)[0] = stm32n6_getreg32((uint32_t)src++);
                  ((uint32_t *)&ctrlreq)[1] = stm32n6_getreg32((uint32_t)src++);

                  usbvdbg("SETUP type=%02x req=%02x\n", ctrlreq.type, ctrlreq.req);

                  /* Process SETUP packet */
                  if (priv->usbdev.driver)
                    {
                      int ret = stm32n6_drvrsetup(&priv->usbdev, &ctrlreq);
                      if (ret < 0)
                        {
                          /* Stall EP0 if SETUP wasn't handled */
                          uint32_t regval = stm32n6_getreg32(priv->base + STM32_OTG_DIEPCTL_OFFSET(0));
                          regval |= OTG_DEPCTL_STALL;
                          stm32n6_putreg32(regval, priv->base + STM32_OTG_DIEPCTL_OFFSET(0));
                        }
                    }
                }
              break;
            }

          case 0x06:  /* Setup complete */
            {
              if (epnum == 0)
                {
                  /* EP0 SETUP packet - pop 8 bytes (2 transactions) */
                  for (int j = 0; j < 2; j++)
                    {
                      (void)stm32n6_getreg32(priv->base + STM32_OTG_DFIFO_OFFSET(0));
                    }
                }
              break;
            }

          default:
            break;
        }
    }

  /* IN endpoint interrupt */
  if (intsts & OTG_GINTSTS_IEPINT)
    {
      epint = (stm32n6_getreg32(priv->base + STM32_OTG_DAINT_OFFSET) & 0xFFFF);
      for (i = 0; i < STM32N6_OTGFS_NENDPOINTS && epint; i++, epint >>= 1)
        {
          if ((epint & 1) == 0)
            {
              continue;
            }

          /* Check IN endpoint interrupt status */
          uint32_t diepint = stm32n6_getreg32(priv->base + STM32_OTG_DIEPINT_OFFSET(i));
          uint32_t diepmsk = stm32n6_getreg32(priv->base + STM32_OTG_DIEPMSK_OFFSET);

          usbvdbg("EP%d IN intsts=%08x\n", i, diepint);

          if (diepint & diepmsk)
            {
              /* Clear interrupt flags */
              stm32n6_putreg32(diepint, priv->base + STM32_OTG_DIEPINT_OFFSET(i));

              if (diepint & (1 << 0))  /* Transfer complete */
                {
                  struct stm32n6_ep_s *privep = &priv->epin[i];

                  if (privep->head)
                    {
                      /* Complete the request */
                      stm32n6_rqcancel(privep, privep->head, 0);
                    }
                }
            }
        }
    }

  /* OUT endpoint interrupt */
  if (intsts & OTG_GINTSTS_OEPINT)
    {
      epint = (stm32n6_getreg32(priv->base + STM32_OTG_DAINT_OFFSET) >> 16) & 0xFFFF;
      for (i = 0; i < STM32N6_OTGFS_NENDPOINTS && epint; i++, epint >>= 1)
        {
          if ((epint & 1) == 0)
            {
              continue;
            }

          /* Check OUT endpoint interrupt status */
          uint32_t doepint = stm32n6_getreg32(priv->base + STM32_OTG_DOEPINT_OFFSET(i));
          uint32_t doepmsk = stm32n6_getreg32(priv->base + STM32_OTG_DOEPMSK_OFFSET);

          usbvdbg("EP%d OUT intsts=%08x\n", i, doepint);

          if (doepint & doepmsk)
            {
              /* Clear interrupt flags */
              stm32n6_putreg32(doepint, priv->base + STM32_OTG_DOEPINT_OFFSET(i));

              if (doepint & (1 << 0))  /* Transfer complete */
                {
                  struct stm32n6_ep_s *privep = &priv->epout[i];

                  if (privep->head)
                    {
                      /* Complete the request */
                      stm32n6_rqcancel(privep, privep->head, 0);
                    }
                }
            }
        }
    }

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

int stm32n6_otgfs_initialize(void)
{
  struct stm32n6_usbdev_s *priv;
  uint32_t regval;
  int ret;
  int i;

  usbdbg("Initializing STM32N6 OTG FS\n");

  /* Get driver instance */
  priv = &g_otgdev;
  memset(priv, 0, sizeof(struct stm32n6_usbdev_s));

  /* Set base address */
  priv->base = STM32_OTGFS_BASE;

  /* Enable USB OTG clock */
  regval = getreg32(STM32_RCC_AHB2ENR);
  regval |= RCC_AHB2ENR_OTGFSEN;
  putreg32(regval, STM32_RCC_AHB2ENR);

  /* Configure pin control */
  stm32_configgpio(GPIO_OTG_FS_VBUS);
  stm32_configgpio(GPIO_OTG_FS_DM);
  stm32_configgpio(GPIO_OTG_FS_DP);
  stm32_configgpio(GPIO_OTG_FS_ID);
  stm32_configgpio(GPIO_OTG_FS_SOF);

  /* Reset the USB controller */
  regval = OTG_GRSTCTL_CSRST;
  stm32n6_putreg32(regval, priv->base + STM32_OTG_GRSTCTL_OFFSET);

  /* Wait for reset to complete */
  while ((stm32n6_getreg32(priv->base + STM32_OTG_GRSTCTL_OFFSET) & OTG_GRSTCTL_CSRST) != 0);

  /* Wait for AHB master IDLE state */
  while ((stm32n6_getreg32(priv->base + STM32_OTG_GRSTCTL_OFFSET) & OTG_GRSTCTL_AHBIDL) == 0);

  /* Configure USB core */
  regval = stm32n6_getreg32(priv->base + STM32_OTG_GUSBCFG_OFFSET);
  regval &= ~OTG_GUSBCFG_PHYSEL;  /* FS serial transceiver */
  regval |= OTG_GUSBCFG_SRPCAP;    /* SRP capability */
  regval |= OTG_GUSBCFG_HNPCAP;    /* HNP capability */
  stm32n6_putreg32(regval, priv->base + STM32_OTG_GUSBCFG_OFFSET);

  /* Configure core as device */
  regval = OTG_GUSBCFG_FDMOD | (0x6 << 10);  /* Force device mode, TRDT */
  stm32n6_putreg32(regval, priv->base + STM32_OTG_GUSBCFG_OFFSET);

  /* Configure power and clock gating */
  regval = stm32n6_getreg32(priv->base + STM32_OTG_GCCFG_OFFSET);
  regval |= (1 << 23);  /* NoVBUSSENS */
  regval |= (1 << 21);  /* VBUSASEN */
  regval |= (1 << 20);  /* VBUSBSEN */
  regval |= (1 << 19);  /* SOFOUTEN */
  stm32n6_putreg32(regval, priv->base + STM32_OTG_GCCFG_OFFSET);

  /* Configure device mode */
  regval = OTG_DCFG_DSPD_FS_PHY_48MHZ;  /* Full-speed PHY */
  regval |= (3 << 2);  /* Periodic frame interval */
  stm32n6_putreg32(regval, priv->base + STM32_OTG_DCFG_OFFSET);

  /* Clear device address */
  regval = stm32n6_getreg32(priv->base + STM32_OTG_DCFG_OFFSET);
  regval &= ~0x7F;  /* Clear address */
  stm32n6_putreg32(regval, priv->base + STM32_OTG_DCFG_OFFSET);

  /* Flush TX and RX FIFOs */
  stm32n6_flush_rx(priv);
  for (i = 0; i < STM32N6_OTGFS_NENDPOINTS; i++)
    {
      stm32n6_flush_tx(priv, i);
    }

  /* Initialize FIFOs */
  stm32n6_initfifo(priv);

  /* Configure device interrupt mask */
  regval = OTG_GINTMSK_USBRST |    /* USB reset */
           OTG_GINTMSK_ENUMDNEM |  /* Enumeration done */
           OTG_GINTMSK_IEPINT |    /* IN endpoint interrupt */
           OTG_GINTMSK_OEPINT |    /* OUT endpoint interrupt */
           OTG_GINTMSK_RXFLVLM |   /* RX FIFO not empty */
           OTG_GINTMSK_WUIM |      /* Wakeup */
           OTG_GINTMSK_USBSUSPM;   /* USB suspend */
  stm32n6_putreg32(regval, priv->base + STM32_OTG_GINTMSK_OFFSET);

  /* Disable all endpoints */
  for (i = 0; i < STM32N6_OTGFS_NENDPOINTS; i++)
    {
      /* Disable IN endpoint */
      regval = stm32n6_getreg32(priv->base + STM32_OTG_DIEPCTL_OFFSET(i));
      regval &= ~OTG_DEPCTL_USBAEP;
      stm32n6_putreg32(regval, priv->base + STM32_OTG_DIEPCTL_OFFSET(i));

      /* Disable OUT endpoint */
      regval = stm32n6_getreg32(priv->base + STM32_OTG_DOEPCTL_OFFSET(i));
      regval &= ~OTG_DEPCTL_USBAEP;
      stm32n6_putreg32(regval, priv->base + STM32_OTG_DOEPCTL_OFFSET(i));
    }

  /* Enable USB interrupt */
  ret = irq_attach(STM32_IRQ_OTG_FS, stm32n6_usbinterrupt, priv);
  if (ret < 0)
    {
      usbdbg("Failed to attach OTG_FS IRQ\n");
      return ret;
    }

  up_enable_irq(STM32_IRQ_OTG_FS);

  /* Initialize endpoint structures */
  for (i = 0; i < STM32N6_OTGFS_NENDPOINTS; i++)
    {
      /* Initialize IN endpoint */
      priv->epin[i].dev = priv;
      priv->epin[i].epnum = i;
      priv->epin[i].ep.name = "in";
      priv->epin[i].ep.ops = &g_usbdev_ops;
      priv->epin[i].ep.priv = &priv->epin[i];

      /* Initialize OUT endpoint */
      priv->epout[i].dev = priv;
      priv->epout[i].epnum = i;
      priv->epout[i].ep.name = "out";
      priv->epout[i].ep.ops = &g_usbdev_ops;
      priv->epout[i].ep.priv = &priv->epout[i];
    }

  /* Initialize device structure */
  priv->usbdev.ops = &g_usbdev_ops;
  priv->usbdev.ep0 = &priv->epin[0].ep;

  /* Initialize state */
  priv->attached = 0;
  priv->suspended = 0;
  priv->selfpowered = 1;
  priv->addressed = 0;
  priv->configured = 0;

  /* Register with the USB device class */
  ret = usbdev_register(&priv->usbdev);
  if (ret < 0)
    {
      usbdbg("Failed to register USB device\n");
      return ret;
    }

  usbdbg("STM32N6 OTG FS initialized successfully\n");
  return OK;
}