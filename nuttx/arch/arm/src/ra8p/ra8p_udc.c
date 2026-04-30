/****************************************************************************
 * arch/arm/src/ra8p/ra8p_udc.c
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
#include <nuttx/usb/usbdev.h>
#include <nuttx/usb/usbdev_trace.h>
#include <arch/board/board.h>

#include "arm_internal.h"
#include "chip.h"
#include "ra8p_config.h"
#include "ra8p_peripherals.h"
#include "hardware/ra8p_udc.h"

#ifdef CONFIG_RA8P_UDC

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Helper macros for register access */

#define ra8p_udc_putreg16(offset, val) \
  putreg16((val), RA8P_UDC_BASE + (offset))

#define ra8p_udc_getreg16(offset) \
  getreg16(RA8P_UDC_BASE + (offset))

#define ra8p_udc_putreg32(offset, val) \
  putreg32((val), RA8P_UDC_BASE + (offset))

#define ra8p_udc_getreg32(offset) \
  getreg32(RA8P_UDC_BASE + (offset))

#define ra8p_udc_modifyreg16(offset, clrbits, setbits) \
  ra8p_udc_putreg16(offset, \
    (ra8p_udc_getreg16(offset) & ~(clrbits)) | (setbits))

/* USB UDC timeout */

#define RA8P_UDC_TIMEOUT_MS                    (1000)

/* Number of endpoints */

#define RA8P_UDC_NENDPOINTS                   (10)        /* EP0-EP9 */

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* This structure represents one endpoint in the USB device controller */

struct ra8p_udc_endpoint_s
{
  struct usbdev_ep_s ep;              /* USB device endpoint structure */
  struct ra8p_udc_dev_s *dev;         /* Reference to driver structure */
  uint8_t epnum;                      /* Endpoint number */
  uint8_t eptype;                     /* Endpoint type */
  uint16_t maxpacket;                 /* Max packet size */
  bool halted;                        /* Endpoint halt status */
  bool stall;                         /* Stall in progress */
  struct ra8p_udc_req_s *head;        /* Head of request queue */
  struct ra8p_udc_req_s *tail;        /* Tail of request queue */
};

/* This structure represents a request */

struct ra8p_udc_req_s
{
  struct usbdev_req_s req;            /* Standard USB request */
  struct ra8p_udc_endpoint_s *ep;     /* Endpoint associated with request */
  bool inuse;                         /* Request in use */
  struct ra8p_udc_req_s *flink;       /* Forward link in request queue */
};

/* This is the overall device controller driver structure */

struct ra8p_udc_dev_s
{
  struct usbdev_s usbdev;             /* USB device interface */
  uint32_t base;                      /* Base address of UDC registers */
  int irq;                            /* USB interrupt number */
  mutex_t lock;                       /* Thread safe operation */
  struct ra8p_udc_endpoint_s ep[RA8P_UDC_NENDPOINTS]; /* Endpoint structures */
  struct ra8p_udc_req_s reqpool[CONFIG_RA8P_UDC_NREQS]; /* Request pool */
  uint8_t npool;                      /* Number of allocated requests */
  enum ra8p_udc_device_state_e devstate; /* Current device state */
  bool attached;                      /* Device attached */
  bool suspended;                     /* Device suspended */
  bool selfpowered;                   /* Device self-powered */
  bool addressed;                     /* Device addressed */
  uint8_t configuration;              /* Current configuration */
  struct usbdev_req_s *ctrlreq;       /* Allocated control request */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static void ra8p_udc_flush_endpoint(struct ra8p_udc_dev_s *priv,
                                     struct ra8p_udc_endpoint_s *ep);
static void ra8p_udc_ctrl_send_ack(struct ra8p_udc_dev_s *priv);
static void ra8p_udc_ctrl_data_in(struct ra8p_udc_dev_s *priv);
static void ra8p_udc_ctrl_data_out(struct ra8p_udc_dev_s *priv);
static int ra8p_udc_req_complete(struct ra8p_udc_dev_s *priv,
                                 struct ra8p_udc_req_s *req, int result);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_udc_dev_s g_udc;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_udc_set_address
 ****************************************************************************/

static void ra8p_udc_set_address(struct ra8p_udc_dev_s *priv, uint8_t address)
{
  uint16_t regval;

  /* Read current value */

  regval = ra8p_udc_getreg16(RA8P_UDC_DADDR_OFFSET);

  /* Clear current address and set new address */

  regval &= ~(RA8P_UDC_DADDR_USBADDR_MASK);
  regval |= ((address & RA8P_UDC_DADDR_USBADDR_MASK) << RA8P_UDC_DADDR_USBADDR_SHIFT);
  regval |= RA8P_UDC_DADDR_USBADDREN;

  /* Write back the new address */

  ra8p_udc_putreg16(RA8P_UDC_DADDR_OFFSET, regval);
}

/****************************************************************************
 * Name: ra8p_udc_set_test_mode
 ****************************************************************************/

static void ra8p_udc_set_test_mode(struct ra8p_udc_dev_s *priv, uint8_t testmode)
{
  /* Implement test mode functionality if needed */
}

/****************************************************************************
 * Name: ra8p_udc_soft_connect
 ****************************************************************************/

static void ra8p_udc_soft_connect(struct ra8p_udc_dev_s *priv)
{
  /* Enable connection */

  ra8p_udc_modifyreg16(RA8P_UDC_SYSCFG_OFFSET, 0, RA8P_UDC_SYSCFG_CNEN);

  priv->attached = true;
}

/****************************************************************************
 * Name: ra8p_udc_soft_disconnect
 ****************************************************************************/

static void ra8p_udc_soft_disconnect(struct ra8p_udc_dev_s *priv)
{
  /* Disable connection */

  ra8p_udc_modifyreg16(RA8P_UDC_SYSCFG_OFFSET, RA8P_UDC_SYSCFG_CNEN, 0);

  priv->attached = false;
}

/****************************************************************************
 * Name: ra8p_udc_reset
 ****************************************************************************/

static void ra8p_udc_reset(struct ra8p_udc_dev_s *priv)
{
  int i;

  /* Reset device state */

  priv->devstate = RA8P_UDC_DEVICE_POWER;
  priv->addressed = false;
  priv->configuration = 0;
  priv->suspended = false;

  /* Reset endpoints */

  for (i = 0; i < RA8P_UDC_NENDPOINTS; i++)
    {
      struct ra8p_udc_endpoint_s *ep = &priv->ep[i];

      ep->halted = false;
      ep->stall = false;
      ra8p_udc_flush_endpoint(priv, ep);
    }

  /* Clear address */

  ra8p_udc_set_address(priv, 0);
}

/****************************************************************************
 * Name: ra8p_udc_flush_endpoint
 ****************************************************************************/

static void ra8p_udc_flush_endpoint(struct ra8p_udc_dev_s *priv,
                                     struct ra8p_udc_endpoint_s *ep)
{
  /* Flush any pending transfers */

  struct ra8p_udc_req_s *req;

  while (ep->head)
    {
      req = ep->head;
      ep->head = req->flink;
      if (!ep->head)
        {
          ep->tail = NULL;
        }

      ra8p_udc_req_complete(priv, req, -ESHUTDOWN);
    }
}

/****************************************************************************
 * Name: ra8p_udc_req_complete
 ****************************************************************************/

static int ra8p_udc_req_complete(struct ra8p_udc_dev_s *priv,
                                 struct ra8p_udc_req_s *req, int result)
{
  struct ra8p_udc_endpoint_s *ep = req->ep;
  usbtrace(trace_intcomplete(ep->epnum), 0);

  if (req->req.callback)
    {
      usbdbg("Callback ep=%p req=%p result=%d\n", &ep->ep, &req->req, result);
      req->req.callback(&ep->ep, &req->req);
    }

  /* Release the request */

  req->inuse = false;
  req->flink = NULL;

  return OK;
}

/****************************************************************************
 * Name: ra8p_udc_ctrl_send_ack
 ****************************************************************************/

static void ra8p_udc_ctrl_send_ack(struct ra8p_udc_dev_s *priv)
{
  /* Set PID to BUF (acknowledge) for control endpoint */

  ra8p_udc_modifyreg16(RA8P_UDC_DCPCTR_OFFSET, RA8P_UDC_DCPCTR_PID_MASK,
                       RA8P_UDC_DCPCTR_PID_BUF);
}

/****************************************************************************
 * Name: ra8p_udc_ctrl_data_in
 ****************************************************************************/

static void ra8p_udc_ctrl_data_in(struct ra8p_udc_dev_s *priv)
{
  /* Handle control endpoint data IN phase */
  /* This would normally send data from host */

  /* For now, just send ACK */
  ra8p_udc_ctrl_send_ack(priv);
}

/****************************************************************************
 * Name: ra8p_udc_ctrl_data_out
 ****************************************************************************/

static void ra8p_udc_ctrl_data_out(struct ra8p_udc_dev_s *priv)
{
  /* Handle control endpoint data OUT phase */
  /* This would normally receive data from host */

  /* For now, just send ACK */
  ra8p_udc_ctrl_send_ack(priv);
}

/****************************************************************************
 * Name: ra8p_udc_interrupt
 ****************************************************************************/

static int ra8p_udc_interrupt(int irq, void *context, void *arg)
{
  struct ra8p_udc_dev_s *priv = &g_udc;
  uint16_t intsts;
  uint16_t intenb;
  uint8_t i;

  /* Read interrupt status */

  intsts = ra8p_udc_getreg16(RA8P_UDC_INTSTS0_OFFSET);
  intenb = ra8p_udc_getreg16(RA8P_UDC_INTENB0_OFFSET);

  /* Clear interrupt flags */

  ra8p_udc_putreg16(RA8P_UDC_INTSTS0_OFFSET, intsts);

  /* Process enabled interrupts */

  intsts &= intenb;

  /* Handle control transfer */

  if (intsts & RA8P_UDC_INTSTS0_CTRT)
    {
      uint16_t dvsq = ra8p_udc_getreg16(RA8P_UDC_INTSTS0_OFFSET) & RA8P_UDC_INTSTS0_DVSQ_MASK;

      /* Check if control transfer completed */

      if (ra8p_udc_getreg16(RA8P_UDC_DCPCTR_OFFSET) & RA8P_UDC_DCPCTR_CCPL)
        {
          /* Control transfer complete */
        }
    }

  /* Handle device state changes */

  if (intsts & RA8P_UDC_INTSTS0_RST)
    {
      /* USB Reset detected */

      ra8p_udc_reset(priv);
      priv->devstate = RA8P_UDC_DEVICE_DEFAULT;
    }

  /* Handle SOF */

  if (intsts & RA8P_UDC_INTSTS0_SOF)
    {
      /* Start of frame */
    }

  /* Handle resume */

  if (intsts & RA8P_UDC_INTSTS0_RSME)
    {
      /* Resume detected */
      priv->suspended = false;
    }

  /* Handle buffer ready interrupts */

  uint16_t brdy = ra8p_udc_getreg16(RA8P_UDC_BRDYENB_OFFSET);
  for (i = 0; i < RA8P_UDC_NENDPOINTS; i++)
    {
      if (brdy & (1 << i))
        {
          /* Endpoint i buffer ready */
          struct ra8p_udc_endpoint_s *ep = &priv->ep[i];

          if (ep->head)
            {
              struct ra8p_udc_req_s *req = ep->head;
              ep->head = req->flink;
              if (!ep->head)
                {
                  ep->tail = NULL;
                }

              /* Complete the request */
              ra8p_udc_req_complete(priv, req, OK);
            }
        }
    }

  return OK;
}

/****************************************************************************
 * Name: ra8p_udc_ep_configure
 ****************************************************************************/

static int ra8p_udc_ep_configure(struct usbdev_ep_s *ep, 
                                 const struct usb_epdesc_s *desc,
                                 bool last)
{
  struct ra8p_udc_endpoint_s *privep = (struct ra8p_udc_endpoint_s *)ep;
  struct ra8p_udc_dev_s *priv = privep->dev;

  if (!desc || desc->addr >= RA8P_UDC_NENDPOINTS)
    {
      return -EINVAL;
    }

  usbdbg("Configure endpoint %d\n", desc->addr);

  /* Configure endpoint parameters */

  privep->epnum = desc->addr & USB_EPNO_MASK;
  privep->eptype = desc->attr & USB_EPTYPE_MASK;
  privep->maxpacket = le16_to_cpu(desc->mxpacketsize);

  usbdbg("EP%d: type=%d maxpacket=%d\n", 
         privep->epnum, privep->eptype, privep->maxpacket);

  return OK;
}

/****************************************************************************
 * Name: ra8p_udc_ep_disable
 ****************************************************************************/

static void ra8p_udc_ep_disable(struct usbdev_ep_s *ep)
{
  struct ra8p_udc_endpoint_s *privep = (struct ra8p_udc_endpoint_s *)ep;
  struct ra8p_udc_dev_s *priv = privep->dev;

  usbdbg("Disable endpoint %d\n", privep->epnum);

  /* Disable endpoint */

  nxmutex_lock(&priv->lock);
  privep->halted = true;
  ra8p_udc_flush_endpoint(priv, privep);
  nxmutex_unlock(&priv->lock);
}

/****************************************************************************
 * Name: ra8p_udc_ep_allocreq
 ****************************************************************************/

static struct usbdev_req_s *ra8p_udc_ep_allocreq(struct usbdev_ep_s *ep)
{
  struct ra8p_udc_endpoint_s *privep = (struct ra8p_udc_endpoint_s *)ep;
  struct ra8p_udc_dev_s *priv = privep->dev;
  struct ra8p_udc_req_s *ureq;
  int i;

  usbdbg("Allocate request for EP%d\n", privep->epnum);

  /* Find a free request in the pool */

  for (i = 0; i < CONFIG_RA8P_UDC_NREQS; i++)
    {
      ureq = &priv->reqpool[i];
      if (!ureq->inuse)
        {
          ureq->inuse = true;
          ureq->ep = privep;
          ureq->flink = NULL;
          ureq->req.len = 0;
          ureq->req.flags = 0;
          return &ureq->req;
        }
    }

  return NULL;
}

/****************************************************************************
 * Name: ra8p_udc_ep_freereq
 ****************************************************************************/

static void ra8p_udc_ep_freereq(struct usbdev_ep_s *ep,
                                struct usbdev_req_s *req)
{
  struct ra8p_udc_req_s *ureq = (struct ra8p_udc_req_s *)req;

  DEBUGASSERT(ureq);
  ureq->inuse = false;
}

/****************************************************************************
 * Name: ra8p_udc_ep_submit
 ****************************************************************************/

static int ra8p_udc_ep_submit(struct usbdev_ep_s *ep,
                              struct usbdev_req_s *req,
                              usb_complete_t callback)
{
  struct ra8p_udc_endpoint_s *privep = (struct ra8p_udc_endpoint_s *)ep;
  struct ra8p_udc_dev_s *priv = privep->dev;
  struct ra8p_udc_req_s *ureq = (struct ra8p_udc_req_s *)req;

  usbdbg("Submit request for EP%d\n", privep->epnum);

  if (!req || !req->buf || !req->len)
    {
      return -EINVAL;
    }

  req->status = -EINPROGRESS;
  req->actual = 0;

  /* Add to request queue */

  nxmutex_lock(&priv->lock);
  if (!privep->head)
    {
      privep->head = privep->tail = ureq;
    }
  else
    {
      privep->tail->flink = ureq;
      privep->tail = ureq;
    }

  ureq->flink = NULL;

  nxmutex_unlock(&priv->lock);

  /* Process the request if it's the control endpoint */

  if (privep->epnum == 0)
    {
      if (req->flags & USB_REQFLAG_IN)
        {
          ra8p_udc_ctrl_data_in(priv);
        }
      else
        {
          ra8p_udc_ctrl_data_out(priv);
        }
    }

  return OK;
}

/****************************************************************************
 * Name: ra8p_udc_ep_cancel
 ****************************************************************************/

static int ra8p_udc_ep_cancel(struct usbdev_ep_s *ep,
                              struct usbdev_req_s *req)
{
  struct ra8p_udc_endpoint_s *privep = (struct ra8p_udc_endpoint_s *)ep;
  struct ra8p_udc_dev_s *priv = privep->dev;
  struct ra8p_udc_req_s *ureq = (struct ra8p_udc_req_s *)req;
  struct ra8p_udc_req_s *curr;
  struct ra8p_udc_req_s *prev;

  usbdbg("Cancel request for EP%d\n", privep->epnum);

  nxmutex_lock(&priv->lock);

  /* Check if request is still queued */

  if (privep->head == ureq)
    {
      /* Remove from head */

      privep->head = ureq->flink;
      if (!privep->head)
        {
          privep->tail = NULL;
        }

      nxmutex_unlock(&priv->lock);
      ra8p_udc_req_complete(priv, ureq, -ESHUTDOWN);
      return OK;
    }

  /* Search the queue */

  curr = privep->head;
  prev = NULL;

  while (curr && curr != ureq)
    {
      prev = curr;
      curr = curr->flink;
    }

  if (curr)
    {
      /* Found it */

      if (privep->tail == ureq)
        {
          privep->tail = prev;
        }

      prev->flink = ureq->flink;
      nxmutex_unlock(&priv->lock);
      ra8p_udc_req_complete(priv, ureq, -ESHUTDOWN);
      return OK;
    }

  nxmutex_unlock(&priv->lock);
  return -ENXIO;
}

/****************************************************************************
 * Name: ra8p_udc_ep_stall
 ****************************************************************************/

static int ra8p_udc_ep_stall(struct usbdev_ep_s *ep, bool resume)
{
  struct ra8p_udc_endpoint_s *privep = (struct ra8p_udc_endpoint_s *)ep;

  usbdbg("Stall EP%d resume=%d\n", privep->epnum, resume);

  if (!ep)
    {
      return -EINVAL;
    }

  nxmutex_lock(&privep->dev->lock);

  if (resume)
    {
      privep->halted = false;
    }
  else
    {
      privep->halted = true;
    }

  nxmutex_unlock(&privep->dev->lock);
  return OK;
}

/****************************************************************************
 * Name: ra8p_udc_ep_status
 ****************************************************************************/

static int ra8p_udc_ep_status(struct usbdev_ep_s *ep, int status)
{
  struct ra8p_udc_endpoint_s *privep = (struct ra8p_udc_endpoint_s *)ep;

  /* Handle endpoint status changes */

  return OK;
}

/****************************************************************************
 * Name: ra8p_udc_shutdown
 ****************************************************************************/

static void ra8p_udc_shutdown(struct usbdev_s *dev)
{
  struct ra8p_udc_dev_s *priv = (struct ra8p_udc_dev_s *)dev;

  usbdbg("Shutdown UDC\n");

  nxmutex_lock(&priv->lock);

  /* Disconnect */

  ra8p_udc_soft_disconnect(priv);

  /* Disable interrupts */

  ra8p_udc_putreg16(RA8P_UDC_INTENB0_OFFSET, 0);

  nxmutex_unlock(&priv->lock);
}

/****************************************************************************
 * Name: ra8p_udc_activate
 ****************************************************************************/

static int ra8p_udc_activate(struct usbdev_s *dev, bool active)
{
  struct ra8p_udc_dev_s *priv = (struct ra8p_udc_dev_s *)dev;

  usbdbg("Activate UDC active=%d\n", active);

  if (active)
    {
      ra8p_udc_soft_connect(priv);
    }
  else
    {
      ra8p_udc_soft_disconnect(priv);
    }

  return OK;
}

/****************************************************************************
 * Name: ra8p_udc_porttest
 ****************************************************************************/

static int ra8p_udc_porttest(struct usbdev_s *dev, int test)
{
  struct ra8p_udc_dev_s *priv = (struct ra8p_udc_dev_s *)dev;

  usbdbg("Port test %d\n", test);

  nxmutex_lock(&priv->lock);
  ra8p_udc_set_test_mode(priv, test);
  nxmutex_unlock(&priv->lock);

  return OK;
}

/****************************************************************************
 * Name: ra8p_udc_pullup
 ****************************************************************************/

static int ra8p_udc_pullup(struct usbdev_s *dev, bool enable)
{
  struct ra8p_udc_dev_s *priv = (struct ra8p_udc_dev_s *)dev;

  usbdbg("Pull-up enable=%d\n", enable);

  if (enable)
    {
      ra8p_udc_soft_connect(priv);
    }
  else
    {
      ra8p_udc_soft_disconnect(priv);
    }

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_udc_initialize
 *
 * Description:
 *   Initialize the USB device controller driver
 *
 ****************************************************************************/

int ra8p_udc_initialize(void)
{
  struct ra8p_udc_dev_s *priv = &g_udc;
  int i;

  usbdbg("Initialize RA8P UDC\n");

  /* Initialize the driver data structure */

  memset(priv, 0, sizeof(struct ra8p_udc_dev_s));
  priv->base = RA8P_UDC_BASE;
  priv->irq = RA8P_IRQ_USBFS;  /* Using USB Full Speed IRQ */

  nxmutex_init(&priv->lock);

  /* Initialize endpoint structures */

  for (i = 0; i < RA8P_UDC_NENDPOINTS; i++)
    {
      struct ra8p_udc_endpoint_s *ep = &priv->ep[i];

      ep->dev = priv;
      ep->epnum = i;
      ep->ep.ops = &g_udc_epops;

      list_initialize(&ep->reqq);
    }

  /* Initialize the USB device structure */

  priv->usbdev.ops = &g_udc_devops;

  /* Reset and configure USB controller */

  /* Enable USB */

  ra8p_udc_modifyreg16(RA8P_UDC_SYSCFG_OFFSET, 0,
                       RA8P_UDC_SYSCFG_USBE | RA8P_UDC_SYSCFG_DPWRE);

  /* Reset device */

  ra8p_udc_reset(priv);

  /* Enable interrupts */

  ra8p_udc_putreg16(RA8P_UDC_INTENB0_OFFSET,
                    RA8P_UDC_INTENB0_BRDYE | RA8P_UDC_INTENB0_CTRE |
                    RA8P_UDC_INTENB0_DVSE);

  /* Attach interrupt handler */

  int ret = irq_attach(priv->irq, ra8p_udc_interrupt, NULL);
  if (ret < 0)
    {
      usberr("ERROR: Failed to attach IRQ %d\n", priv->irq);
      nxmutex_destroy(&priv->lock);
      return ret;
    }

  up_enable_irq(priv->irq);

  usbdbg("UDC Initialized\n");

  return OK;
}

/* USB endpoint operations */

static const struct usbdev_ep_ops g_udc_epops =
{
  .configure   = ra8p_udc_ep_configure,
  .disable     = ra8p_udc_ep_disable,
  .allocreq    = ra8p_udc_ep_allocreq,
  .freereq     = ra8p_udc_ep_freereq,
  .submit      = ra8p_udc_ep_submit,
  .cancel      = ra8p_udc_ep_cancel,
  .stall       = ra8p_udc_ep_stall,
  .status      = ra8p_udc_ep_status,
};

/* USB device operations */

static const struct usbdev_ops g_udc_devops =
{
  .shutdown   = ra8p_udc_shutdown,
  .activate   = ra8p_udc_activate,
  .porttest   = ra8p_udc_porttest,
  .pullup     = ra8p_udc_pullup,
};

#endif /* CONFIG_RA8P_UDC */