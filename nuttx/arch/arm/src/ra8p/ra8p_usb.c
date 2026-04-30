/****************************************************************************
 * arch/arm/src/ra8p/ra8p_usb.c
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
#include <errno.h>
#include <debug.h>

#include "chip.h"
#include "arm_internal.h"
#include "hardware/ra8p_usb.h"
#include "hardware/ra8p_memorymap.h"

#ifdef CONFIG_RA8P_USBFS

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define RA8P_USBFS_BASE                RA8P_USBFS_BASE
#define RA8P_USBHS_BASE                RA8P_USBHS_BASE

/* USB timeout in milliseconds */
#define RA8P_USB_TIMEOUT_MS            1000

/* USB endpoint maximum number */
#define RA8P_USB_MAX_ENDPOINTS         10

/* USB control endpoint (endpoint 0) */
#define RA8P_USB_EP0                   0

/* USB PID (Packet ID) values */
#define RA8P_USB_PID_NAK               0x01
#define RA8P_USB_PID_BUF               0x02
#define RA8P_USB_PID_STALL             0x03

/* USB FIFO size */
#define RA8P_USB_FIFO_SIZE             512

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* RA8P1 USB driver state structure */

struct ra8p_usb_priv_s
{
  uint32_t base;                              /* Base address of USB registers */
  uint8_t channel;                            /* USB channel (0=USBFS, 1=USBHS) */
  bool initialized;                           /* Initialization flag */
  bool enabled;                               /* Enable flag */
  bool suspended;                             /* Suspend state */
  bool vbus_present;                          /* VBUS presence */
  uint8_t devaddr;                            /* Current device address */
  uint8_t configuration;                       /* Current configuration */
  uint8_t power_state;                         /* Power state */
  uint16_t max_packet[RA8P_USB_MAX_ENDPOINTS]; /* Max packet size per endpoint */
  uint8_t ep_type[RA8P_USB_MAX_ENDPOINTS];     /* Endpoint type */
  uint8_t ep_stall[RA8P_USB_MAX_ENDPOINTS];    /* Stall status per endpoint */
  sem_t wait_sem;                             /* Wait semaphore for transfers */
  struct usb_dev_s *driver;                   /* USB device driver */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int usb_wait_ready(struct ra8p_usb_priv_s *priv);
static int usb_set_address(struct ra8p_usb_priv_s *priv, uint8_t addr);
static int usb_set_configuration(struct ra8p_usb_priv_s *priv, uint8_t config);
static void usb_putreg32(struct ra8p_usb_priv_s *priv, uint32_t offset, uint32_t value);
static uint32_t usb_getreg32(struct ra8p_usb_priv_s *priv, uint32_t offset);
static void usb_putreg16(struct ra8p_usb_priv_s *priv, uint32_t offset, uint16_t value);
static uint16_t usb_getreg16(struct ra8p_usb_priv_s *priv, uint32_t offset);
static void usb_putreg8(struct ra8p_usb_priv_s *priv, uint32_t offset, uint8_t value);
static uint8_t usb_getreg8(struct ra8p_usb_priv_s *priv, uint32_t offset);
static int usb_set_endpoint_type(struct ra8p_usb_priv_s *priv, uint8_t ep, uint8_t type);
static int usb_set_endpoint_maxpacket(struct ra8p_usb_priv_s *priv, uint8_t ep, uint16_t maxpacketsize);
static int usb_reset_controller(struct ra8p_usb_priv_s *priv);
static int usb_enable_controller(struct ra8p_usb_priv_s *priv);
static int usb_wait_vbus_ready(struct ra8p_usb_priv_s *priv);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_usb_priv_s g_usb[2];  /* USBFS and USBHS */

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: usb_putreg32
 ****************************************************************************/

static inline void usb_putreg32(struct ra8p_usb_priv_s *priv, uint32_t offset, uint32_t value)
{
  putreg32(value, priv->base + offset);
}

/****************************************************************************
 * Name: usb_getreg32
 ****************************************************************************/

static inline uint32_t usb_getreg32(struct ra8p_usb_priv_s *priv, uint32_t offset)
{
  return getreg32(priv->base + offset);
}

/****************************************************************************
 * Name: usb_putreg16
 ****************************************************************************/

static inline void usb_putreg16(struct ra8p_usb_priv_s *priv, uint32_t offset, uint16_t value)
{
  putreg16(value, priv->base + offset);
}

/****************************************************************************
 * Name: usb_getreg16
 ****************************************************************************/

static inline uint16_t usb_getreg16(struct ra8p_usb_priv_s *priv, uint32_t offset)
{
  return getreg16(priv->base + offset);
}

/****************************************************************************
 * Name: usb_putreg8
 ****************************************************************************/

static inline void usb_putreg8(struct ra8p_usb_priv_s *priv, uint32_t offset, uint8_t value)
{
  putreg8(value, priv->base + offset);
}

/****************************************************************************
 * Name: usb_getreg8
 ****************************************************************************/

static inline uint8_t usb_getreg8(struct ra8p_usb_priv_s *priv, uint32_t offset)
{
  return getreg8(priv->base + offset);
}

/****************************************************************************
 * Name: usb_wait_vbus_ready
 ****************************************************************************/

static int usb_wait_vbus_ready(struct ra8p_usb_priv_s *priv)
{
  volatile int timeout = RA8P_USB_TIMEOUT_MS * 1000;  /* microseconds */

  /* Wait for VBUS to be detected */
  while (!(usb_getreg16(priv, RA8P_USB_SYSSTS0_OFFSET) & RA8P_USB_SYSSTS0_OVRC0) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  priv->vbus_present = true;
  return OK;
}

/****************************************************************************
 * Name: usb_reset_controller
 ****************************************************************************/

static int usb_reset_controller(struct ra8p_usb_priv_s *priv)
{
  uint16_t syscfg;

  /* Disable USB module first */
  syscfg = usb_getreg16(priv, RA8P_USB_SYSCFG_OFFSET);
  syscfg &= ~RA8P_USB_SYSCFG_USBEN;
  usb_putreg16(priv, RA8P_USB_SYSCFG_OFFSET, syscfg);

  /* Wait for module to disable */
  volatile int timeout = RA8P_USB_TIMEOUT_MS * 1000;
  while ((usb_getreg16(priv, RA8P_USB_SYSCFG_OFFSET) & RA8P_USB_SYSCFG_USBEN) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  /* Reset the USB module */
  syscfg = usb_getreg16(priv, RA8P_USB_SYSCFG_OFFSET);
  syscfg |= RA8P_USB_SYSCFG_SCKE;  /* Enable USB clock */
  syscfg &= ~RA8P_USB_SYSCFG_DCFM;  /* Device mode */
  syscfg |= RA8P_USB_SYSCFG_DPRPU;  /* Enable D+ pull-up */
  usb_putreg16(priv, RA8P_USB_SYSCFG_OFFSET, syscfg);

  return OK;
}

/****************************************************************************
 * Name: usb_enable_controller
 ****************************************************************************/

static int usb_enable_controller(struct ra8p_usb_priv_s *priv)
{
  uint16_t syscfg;

  /* Read current configuration */
  syscfg = usb_getreg16(priv, RA8P_USB_SYSCFG_OFFSET);

  /* Enable USB module */
  syscfg |= RA8P_USB_SYSCFG_USBEN;

  /* Set to device mode */
  syscfg |= RA8P_USB_SYSCFG_DCFM;

  /* Enable D+ pull-up */
  syscfg |= RA8P_USB_SYSCFG_DPRPU;

  /* Write back to register */
  usb_putreg16(priv, RA8P_USB_SYSCFG_OFFSET, syscfg);

  /* Wait for USB module to be enabled */
  volatile int timeout = RA8P_USB_TIMEOUT_MS * 1000;
  while (!(usb_getreg16(priv, RA8P_USB_SYSSTS0_OFFSET) & RA8P_USB_SYSSTS0_LNST) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  return OK;
}

/****************************************************************************
 * Name: usb_set_endpoint_type
 ****************************************************************************/

static int usb_set_endpoint_type(struct ra8p_usb_priv_s *priv, uint8_t ep, uint8_t type)
{
  uint16_t pipemode;
  uint16_t pipenm;

  if (ep >= RA8P_USB_MAX_ENDPOINTS)
    {
      return -EINVAL;
    }

  /* Select pipe */
  usb_putreg16(priv, RA8P_USB_PIPESEL_OFFSET, ep);

  /* Configure pipe mode register based on endpoint type */
  pipemode = usb_getreg16(priv, RA8P_USB_PIPEMOD_OFFSET);
  pipemode &= ~RA8P_USB_PIPEMOD_TYP_MASK;
  
  switch (type)
    {
      case USB_EP_ATTR_XFER_CONTROL:
        pipemode |= RA8P_USB_PIPEMOD_TYP_CNTL;  /* Control */
        break;
      case USB_EP_ATTR_XFER_ISOC:
        pipemode |= RA8P_USB_PIPEMOD_TYP_ISO;   /* Isochronous */
        break;
      case USB_EP_ATTR_XFER_BULK:
        pipemode |= RA8P_USB_PIPEMOD_TYP_BULK;  /* Bulk */
        break;
      case USB_EP_ATTR_XFER_INT:
        pipemode |= RA8P_USB_PIPEMOD_TYP_INT;   /* Interrupt */
        break;
      default:
        return -EINVAL;
    }

  usb_putreg16(priv, RA8P_USB_PIPEMOD_OFFSET, pipemode);

  /* Configure pipe number mode register */
  pipenm = usb_getreg16(priv, RA8P_USB_PIPENM_OFFSET);
  if (ep == 0)  /* Control endpoint */
    {
      pipenm |= RA8P_USB_PIPENM_BVLD;  /* Buffer valid */
      pipenm |= RA8P_USB_PIPENM_DBLB;  /* Double buffer */
    }
  else
    {
      pipenm |= RA8P_USB_PIPENM_BVLD;  /* Buffer valid */
    }
  usb_putreg16(priv, RA8P_USB_PIPENM_OFFSET, pipenm);

  g_usb[priv->channel].ep_type[ep] = type;

  return OK;
}

/****************************************************************************
 * Name: usb_set_endpoint_maxpacket
 ****************************************************************************/

static int usb_set_endpoint_maxpacket(struct ra8p_usb_priv_s *priv, uint8_t ep, uint16_t maxpacketsize)
{
  uint16_t pipemaxp;

  if (ep >= RA8P_USB_MAX_ENDPOINTS)
    {
      return -EINVAL;
    }

  /* Select pipe */
  usb_putreg16(priv, RA8P_USB_PIPESEL_OFFSET, ep);

  /* Set maximum packet size */
  pipemaxp = usb_getreg16(priv, RA8P_USB_PIPEMAXP_OFFSET);
  pipemaxp &= ~RA8P_USB_PIPEMAXP_MXPS_MASK;
  pipemaxp |= (maxpacketsize & RA8P_USB_PIPEMAXP_MXPS_MASK);
  usb_putreg16(priv, RA8P_USB_PIPEMAXP_OFFSET, pipemaxp);

  g_usb[priv->channel].max_packet[ep] = maxpacketsize;

  return OK;
}

/****************************************************************************
 * Name: usb_select_pipe
 ****************************************************************************/

static int usb_select_pipe(struct ra8p_usb_priv_s *priv, uint8_t ep)
{
  uint16_t pipesel;

  if (ep >= RA8P_USB_MAX_ENDPOINTS)
    {
      return -EINVAL;
    }

  usb_putreg16(priv, RA8P_USB_PIPESEL_OFFSET, ep);

  /* Wait for pipe selection to complete */
  volatile int timeout = 1000;
  while (timeout--)
    {
      pipesel = usb_getreg16(priv, RA8P_USB_PIPESEL_OFFSET);
      if ((pipesel & 0x0F) == ep)
        {
          break;
        }
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  return OK;
}

/****************************************************************************
 * Name: usb_set_pid
 ****************************************************************************/

static int usb_set_pid(struct ra8p_usb_priv_s *priv, uint8_t ep, uint8_t pid)
{
  int ret;

  ret = usb_select_pipe(priv, ep);
  if (ret != OK)
    {
      return ret;
    }

  uint16_t pipectr = usb_getreg16(priv, RA8P_USB_PIPECTR_OFFSET);
  pipectr &= ~RA8P_USB_PIPECTR_PID_MASK;
  pipectr |= (pid << RA8P_USB_PIPECTR_PID_SHIFT);
  usb_putreg16(priv, RA8P_USB_PIPECTR_OFFSET, pipectr);

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_usb_initialize
 *
 * Description:
 *   Initialize the USB controller based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   config - Pointer to configuration structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_initialize(const struct ra8p_usb_config_s *config)
{
  struct ra8p_usb_priv_s *priv;
  uint16_t regval;
  int ret;
  int i;

  if (config == NULL || config->channel >= 2)
    {
      return -EINVAL;
    }

  priv = &g_usb[config->channel];
  priv->channel = config->channel;
  priv->base = (config->channel == 0) ? RA8P_USBFS_BASE : RA8P_USBHS_BASE;
  priv->devaddr = 0;
  priv->configuration = 0;
  priv->initialized = false;
  priv->enabled = false;
  priv->suspended = false;
  priv->vbus_present = false;

  /* Reset USB controller */
  ret = usb_reset_controller(priv);
  if (ret != OK)
    {
      return ret;
    }

  /* Wait for VBUS to be ready */
  ret = usb_wait_vbus_ready(priv);
  if (ret != OK)
    {
      usberr("VBUS not ready for USB%d\n", config->channel);
      return ret;
    }

  /* Enable USB module */
  ret = usb_enable_controller(priv);
  if (ret != OK)
    {
      return ret;
    }

  /* Configure interrupt settings */
  regval = RA8P_USB_INTENB0_BRDYE |    /* Buffer ready interrupt enable */
           RA8P_USB_INTENB0_NRDYE |    /* Buffer not ready interrupt enable */
           RA8P_USB_INTENB0_BEMPE |    /* Buffer empty interrupt enable */
           RA8P_USB_INTENB0_CTRT |     /* Control transfer stage transition interrupt enable */
           RA8P_USB_INTENB0_DVSE |     /* Device state transition interrupt enable */
           RA8P_USB_INTENB0_SOFE |     /* SOF reception interrupt enable */
           RA8P_USB_INTENB0_RESE;      /* USB reset interrupt enable */

  usb_putreg16(priv, RA8P_USB_INTENB0_OFFSET, regval);

  /* Enable additional interrupt sources */
  regval = RA8P_USB_INTENB1_SACKE |   /* Setup acknowledge interrupt enable */
           RA8P_USB_INTENB1_EOFERRE | /* EOF error interrupt enable */
           RA8P_USB_INTENB1_NRDYEE |  /* Not ready interrupt enable */
           RA8P_USB_INTENB1_BRDYE0 |  /* Buffer ready EP0 interrupt enable */
           RA8P_USB_INTENB1_BRDYE1 |  /* Buffer ready EP1 interrupt enable */
           RA8P_USB_INTENB1_BRDYE2 |  /* Buffer ready EP2 interrupt enable */
           RA8P_USB_INTENB1_BRDYE3 |  /* Buffer ready EP3 interrupt enable */
           RA8P_USB_INTENB1_BRDYE4 |  /* Buffer ready EP4 interrupt enable */
           RA8P_USB_INTENB1_BRDYE5 |  /* Buffer ready EP5 interrupt enable */
           RA8P_USB_INTENB1_BRDYE6 |  /* Buffer ready EP6 interrupt enable */
           RA8P_USB_INTENB1_BRDYE7 |  /* Buffer ready EP7 interrupt enable */
           RA8P_USB_INTENB1_BRDYE8 |  /* Buffer ready EP8 interrupt enable */
           RA8P_USB_INTENB1_BRDYE9;   /* Buffer ready EP9 interrupt enable */
  usb_putreg16(priv, RA8P_USB_INTENB1_OFFSET, regval);

  /* Clear interrupt status */
  usb_putreg16(priv, RA8P_USB_INTSTS0_OFFSET, 0xFFFF);
  usb_putreg16(priv, RA8P_USB_INTSTS1_OFFSET, 0xFFFF);
  usb_putreg16(priv, RA8P_USB_INTSTS2_OFFSET, 0xFFFF);

  /* Initialize endpoint settings */
  for (i = 0; i < RA8P_USB_MAX_ENDPOINTS; i++)
    {
      priv->max_packet[i] = 64;  /* Default to 64 bytes */
      priv->ep_type[i] = USB_EP_ATTR_XFER_CONTROL;  /* Default to control */
      priv->ep_stall[i] = 0;     /* Not stalled */
    }

  /* Configure control endpoint (EP0) */
  ret = usb_set_endpoint_type(priv, 0, USB_EP_ATTR_XFER_CONTROL);
  if (ret != OK)
    {
      return ret;
    }

  ret = usb_set_endpoint_maxpacket(priv, 0, 64);  /* Control endpoint max 64 bytes */
  if (ret != OK)
    {
      return ret;
    }

  /* Configure FIFOs */
  /* Configure CFIFO */
  usb_putreg16(priv, RA8P_USB_CFIFOSEL_OFFSET, 0);
  usb_putreg16(priv, RA8P_USB_CFIFOCTR_OFFSET, 0);

  /* Set device state to default */
  regval = usb_getreg16(priv, RA8P_USB_DVSTCTR0_OFFSET);
  regval &= ~RA8P_USB_DVSTCTR0_WKUP;  /* Clear wake-up */
  regval |= RA8P_USB_DVSTCTR0_WUE;    /* Enable wake-up function */
  usb_putreg16(priv, RA8P_USB_DVSTCTR0_OFFSET, regval);

  priv->initialized = true;
  priv->enabled = true;

  usbinfo("USB%d initialized at 0x%08x\n", config->channel, priv->base);
  return OK;
}

/****************************************************************************
 * Name: ra8p_usb_ep_configure
 *
 * Description:
 *   Configure a USB endpoint based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   epdesc - Endpoint descriptor
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_ep_configure(uint8_t channel, const struct usb_epdesc_s *epdesc)
{
  struct ra8p_usb_priv_s *priv;
  uint8_t epnum;
  uint8_t dir;
  uint8_t eptype;
  uint16_t maxpacketsize;
  int ret;

  if (channel >= 2 || epdesc == NULL || !g_usb[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_usb[channel];
  epnum = USB_EPNO(epdesc->addr);
  dir = (epdesc->addr & USB_DIR_IN) != 0;
  eptype = epdesc->attr & USB_EP_ATTR_XFERTYPE_MASK;
  maxpacketsize = (uint16_t)epdesc->mxpacketsize;

  if (epnum >= RA8P_USB_MAX_ENDPOINTS)
    {
      return -EINVAL;
    }

  /* Select pipe for this endpoint */
  usb_putreg16(priv, RA8P_USB_PIPESEL_OFFSET, epnum);

  /* Configure endpoint type */
  ret = usb_set_endpoint_type(priv, epnum, eptype);
  if (ret != OK)
    {
      return ret;
    }

  /* Set maximum packet size */
  ret = usb_set_endpoint_maxpacket(priv, epnum, maxpacketsize);
  if (ret != OK)
    {
      return ret;
    }

  /* Configure pipe direction */
  uint16_t pipenm = usb_getreg16(priv, RA8P_USB_PIPENM_OFFSET);
  if (dir)  /* IN endpoint */
    {
      pipenm |= RA8P_USB_PIPENM_DIR;  /* Set direction to IN */
    }
  else      /* OUT endpoint */
    {
      pipenm &= ~RA8P_USB_PIPENM_DIR; /* Clear direction to OUT */
    }
  usb_putreg16(priv, RA8P_USB_PIPENM_OFFSET, pipenm);

  /* Enable pipe */
  uint16_t pipectr = usb_getreg16(priv, RA8P_USB_PIPECTR_OFFSET);
  pipectr &= ~RA8P_USB_PIPECTR_PID_MASK;  /* Clear PID bits */
  pipectr |= RA8P_USB_PIPECTR_PID_BUF;    /* Set PID to BUF (ready for transfer) */
  usb_putreg16(priv, RA8P_USB_PIPECTR_OFFSET, pipectr);

  usbinfo("USB%d EP%d configured: maxpacket=%u, type=%u, dir=%s\n", 
          channel, epnum, maxpacketsize, eptype, dir ? "IN" : "OUT");
  return OK;
}

/****************************************************************************
 * Name: ra8p_usb_ep_disable
 *
 * Description:
 *   Disable a USB endpoint based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   ep - Endpoint address
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_ep_disable(uint8_t channel, uint8_t ep)
{
  struct ra8p_usb_priv_s *priv;
  uint8_t epnum;

  if (channel >= 2 || !g_usb[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_usb[channel];
  epnum = USB_EPNO(ep);

  if (epnum >= RA8P_USB_MAX_ENDPOINTS)
    {
      return -EINVAL;
    }

  /* Don't disable EP0 (control endpoint) */
  if (epnum == 0)
    {
      return -EPERM;
    }

  /* Select pipe */
  usb_putreg16(priv, RA8P_USB_PIPESEL_OFFSET, epnum);

  /* Set pipe to NAK state */
  uint16_t pipectr = usb_getreg16(priv, RA8P_USB_PIPECTR_OFFSET);
  pipectr &= ~RA8P_USB_PIPECTR_PID_MASK;
  pipectr |= RA8P_USB_PIPECTR_PID_NAK;
  usb_putreg16(priv, RA8P_USB_PIPECTR_OFFSET, pipectr);

  usbinfo("USB%d EP%d disabled\n", channel, epnum);
  return OK;
}

/****************************************************************************
 * Name: ra8p_usb_ep_stall
 *
 * Description:
 *   Stall a USB endpoint based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   ep - Endpoint address
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_ep_stall(uint8_t channel, uint8_t ep)
{
  struct ra8p_usb_priv_s *priv;
  uint8_t epnum;

  if (channel >= 2 || !g_usb[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_usb[channel];
  epnum = USB_EPNO(ep);

  if (epnum >= RA8P_USB_MAX_ENDPOINTS)
    {
      return -EINVAL;
    }

  /* Select pipe */
  usb_putreg16(priv, RA8P_USB_PIPESEL_OFFSET, epnum);

  /* Set pipe to STALL state */
  uint16_t pipectr = usb_getreg16(priv, RA8P_USB_PIPECTR_OFFSET);
  pipectr &= ~RA8P_USB_PIPECTR_PID_MASK;
  pipectr |= RA8P_USB_PIPECTR_PID_STALL;
  usb_putreg16(priv, RA8P_USB_PIPECTR_OFFSET, pipectr);

  g_usb[channel].ep_stall[epnum] = 1;

  usbinfo("USB%d EP%d stalled\n", channel, epnum);
  return OK;
}

/****************************************************************************
 * Name: ra8p_usb_ep_resume
 *
 * Description:
 *   Resume a stalled USB endpoint based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   ep - Endpoint address
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_ep_resume(uint8_t channel, uint8_t ep)
{
  struct ra8p_usb_priv_s *priv;
  uint8_t epnum;

  if (channel >= 2 || !g_usb[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_usb[channel];
  epnum = USB_EPNO(ep);

  if (epnum >= RA8P_USB_MAX_ENDPOINTS)
    {
      return -EINVAL;
    }

  /* Select pipe */
  usb_putreg16(priv, RA8P_USB_PIPESEL_OFFSET, epnum);

  /* Set pipe to BUF state (ready) */
  uint16_t pipectr = usb_getreg16(priv, RA8P_USB_PIPECTR_OFFSET);
  pipectr &= ~RA8P_USB_PIPECTR_PID_MASK;
  pipectr |= RA8P_USB_PIPECTR_PID_BUF;
  usb_putreg16(priv, RA8P_USB_PIPECTR_OFFSET, pipectr);

  g_usb[channel].ep_stall[epnum] = 0;

  usbinfo("USB%d EP%d resumed\n", channel, epnum);
  return OK;
}

/****************************************************************************
 * Name: ra8p_usb_ep_write
 *
 * Description:
 *   Write data to a USB endpoint based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   ep - Endpoint number (0-9)
 *   buffer - Data buffer to write
 *   buflen - Number of bytes to write
 *
 * Returned Value:
 *   Number of bytes written on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_ep_write(uint8_t channel, uint8_t ep, const uint8_t *buffer, uint32_t buflen)
{
  struct ra8p_usb_priv_s *priv;
  uint8_t epnum;
  uint32_t remaining;
  uint32_t copied = 0;
  uint32_t chunk;
  int timeout;
  uint32_t fifo_addr;
  int i;

  if (channel >= 2 || !g_usb[channel].enabled || buffer == NULL || buflen == 0)
    {
      return -EINVAL;
    }

  priv = &g_usb[channel];
  epnum = USB_EPNO(ep);

  if (epnum >= RA8P_USB_MAX_ENDPOINTS)
    {
      return -EINVAL;
    }

  /* Check if endpoint is ready for transmission */
  uint16_t pipectr = usb_getreg16(priv, RA8P_USB_PIPECTR_OFFSET);
  if ((pipectr & RA8P_USB_PIPECTR_PID_MASK) == RA8P_USB_PIPECTR_PID_STALL)
    {
      return -EBUSY;  /* Endpoint is stalled */
    }

  /* Calculate FIFO address based on endpoint */
  if (epnum == 0)  /* EP0 uses CFIFO */
    {
      fifo_addr = priv->base + RA8P_USB_CFIFO_OFFSET;
    }
  else if (epnum & 1)  /* Odd endpoints use D1FIFO */
    {
      fifo_addr = priv->base + RA8P_USB_D1FIFO_OFFSET;
    }
  else  /* Even endpoints use D0FIFO */
    {
      fifo_addr = priv->base + RA8P_USB_D0FIFO_OFFSET;
    }

  remaining = buflen;

  while (remaining > 0)
    {
      /* Wait for FIFO to be ready */
      timeout = RA8P_USB_TIMEOUT_MS * 1000;
      while (!(usb_getreg16(priv, RA8P_USB_FIFOCTR_OFFSET) & RA8P_USB_FIFOCTR_FRDY) && timeout--)
        {
          up_udelay(1);
        }

      if (timeout <= 0)
        {
          return -ETIMEDOUT;
        }

      /* Calculate chunk size */
      chunk = remaining;
      uint16_t maxps = priv->max_packet[epnum];
      if (chunk > maxps)
        {
          chunk = maxps;
        }

      /* Write data to FIFO in 32-bit words */
      const uint32_t *buf32 = (const uint32_t *)&buffer[copied];
      for (i = 0; i < (chunk + 3) / 4; i++)
        {
          putreg32(buf32[i], fifo_addr);
        }

      copied += chunk;
      remaining -= chunk;

      /* Clear ready flag for next transfer */
      usb_putreg16(priv, RA8P_USB_FIFOCTR_OFFSET, RA8P_USB_FIFOCTR_BCLR);
    }

  /* Send data by setting PID to INBUF */
  pipectr = usb_getreg16(priv, RA8P_USB_PIPECTR_OFFSET);
  pipectr &= ~RA8P_USB_PIPECTR_PID_MASK;
  pipectr |= RA8P_USB_PIPECTR_PID_INBUF;
  usb_putreg16(priv, RA8P_USB_PIPECTR_OFFSET, pipectr);

  usbinfo("USB%d EP%d wrote %u bytes\n", channel, epnum, buflen);
  return copied;
}

/****************************************************************************
 * Name: ra8p_usb_ep_read
 *
 * Description:
 *   Read data from a USB endpoint based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   ep - Endpoint number (0-9)
 *   buffer - Buffer to read data into
 *   buflen - Size of buffer
 *
 * Returned Value:
 *   Number of bytes read on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_ep_read(uint8_t channel, uint8_t ep, uint8_t *buffer, uint32_t buflen)
{
  struct ra8p_usb_priv_s *priv;
  uint8_t epnum;
  uint32_t remaining;
  uint32_t copied = 0;
  uint32_t chunk;
  uint16_t pipenm;
  uint32_t fifo_addr;
  int i;
  int timeout;

  if (channel >= 2 || !g_usb[channel].enabled || buffer == NULL || buflen == 0)
    {
      return -EINVAL;
    }

  priv = &g_usb[channel];
  epnum = USB_EPNO(ep);

  if (epnum >= RA8P_USB_MAX_ENDPOINTS)
    {
      return -EINVAL;
    }

  /* Check if data is available */
  pipenm = usb_getreg16(priv, RA8P_USB_PIPENM_OFFSET);
  if (!(pipenm & RA8P_USB_PIPENM_BSTS))  /* No data available */
    {
      return -EAGAIN;
    }

  /* Calculate FIFO address based on endpoint */
  if (epnum == 0)  /* EP0 uses CFIFO */
    {
      fifo_addr = priv->base + RA8P_USB_CFIFO_OFFSET;
    }
  else if (epnum & 1)  /* Odd endpoints use D1FIFO */
    {
      fifo_addr = priv->base + RA8P_USB_D1FIFO_OFFSET;
    }
  else  /* Even endpoints use D0FIFO */
    {
      fifo_addr = priv->base + RA8P_USB_D0FIFO_OFFSET;
    }

  /* Get data length */
  uint32_t dtln = (pipenm & RA8P_USB_PIPENM_DTLN_MASK) >> RA8P_USB_PIPENM_DTLN_SHIFT;
  chunk = dtln < buflen ? dtln : buflen;
  remaining = chunk;

  /* Read data from FIFO in 32-bit words */
  uint32_t *buf32 = (uint32_t *)buffer;
  for (i = 0; i < (chunk + 3) / 4; i++)
    {
      buf32[i] = getreg32(fifo_addr);
    }

  /* Update copied count */
  copied = chunk;

  /* Clear buffer status */
  uint16_t pipectr = usb_getreg16(priv, RA8P_USB_PIPECTR_OFFSET);
  pipectr |= RA8P_USB_PIPECTR_BCLR;  /* Clear buffer */
  usb_putreg16(priv, RA8P_USB_PIPECTR_OFFSET, pipectr);

  usbinfo("USB%d EP%d read %u bytes\n", channel, epnum, copied);
  return copied;
}

/****************************************************************************
 * Name: ra8p_usb_set_address
 *
 * Description:
 *   Set USB device address based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   address - Device address to set (0-127)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_set_address(uint8_t channel, uint8_t address)
{
  struct ra8p_usb_priv_s *priv;
  uint16_t regval;

  if (channel >= 2 || address > 127 || !g_usb[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_usb[channel];

  /* Set device address */
  regval = usb_getreg16(priv, RA8P_USB_DVSTCTR0_OFFSET);
  regval &= ~RA8P_USB_DVSTCTR0_UADDR_MASK;
  regval |= (address << RA8P_USB_DVSTCTR0_UADDR_SHIFT);
  usb_putreg16(priv, RA8P_USB_DVSTCTR0_OFFSET, regval);

  priv->devaddr = address;

  usbinfo("USB%d address set to %u\n", channel, address);
  return OK;
}

/****************************************************************************
 * Name: ra8p_usb_connect
 *
 * Description:
 *   Connect USB device (enable pull-up) based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_connect(uint8_t channel)
{
  struct ra8p_usb_priv_s *priv;
  uint16_t syscfg;

  if (channel >= 2 || !g_usb[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_usb[channel];

  /* Enable D+ pull-up */
  syscfg = usb_getreg16(priv, RA8P_USB_SYSCFG_OFFSET);
  syscfg |= RA8P_USB_SYSCFG_DPRPU;  /* Enable D+ pull-up */
  usb_putreg16(priv, RA8P_USB_SYSCFG_OFFSET, syscfg);

  usbinfo("USB%d connected (D+ pull-up enabled)\n", channel);
  return OK;
}

/****************************************************************************
 * Name: ra8p_usb_disconnect
 *
 * Description:
 *   Disconnect USB device (disable pull-up) based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_disconnect(uint8_t channel)
{
  struct ra8p_usb_priv_s *priv;
  uint16_t syscfg;

  if (channel >= 2 || !g_usb[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_usb[channel];

  /* Disable D+ pull-up */
  syscfg = usb_getreg16(priv, RA8P_USB_SYSCFG_OFFSET);
  syscfg &= ~RA8P_USB_SYSCFG_DPRPU;  /* Disable D+ pull-up */
  usb_putreg16(priv, RA8P_USB_SYSCFG_OFFSET, syscfg);

  usbinfo("USB%d disconnected (D+ pull-up disabled)\n", channel);
  return OK;
}

/****************************************************************************
 * Name: ra8p_usb_suspend
 *
 * Description:
 *   Suspend USB device based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_suspend(uint8_t channel)
{
  struct ra8p_usb_priv_s *priv;
  uint16_t dvstctr;

  if (channel >= 2 || !g_usb[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_usb[channel];

  /* Set suspend mode */
  dvstctr = usb_getreg16(priv, RA8P_USB_DVSTCTR0_OFFSET);
  dvstctr |= RA8P_USB_DVSTCTR0_UACT;  /* Disable USB activity */
  usb_putreg16(priv, RA8P_USB_DVSTCTR0_OFFSET, dvstctr);

  /* Additional suspend operations */
  uint16_t syscfg = usb_getreg16(priv, RA8P_USB_SYSCFG_OFFSET);
  syscfg |= RA8P_USB_SYSCFG_SLPMD;  /* Enter sleep mode */
  usb_putreg16(priv, RA8P_USB_SYSCFG_OFFSET, syscfg);

  priv->suspended = true;

  usbinfo("USB%d suspended\n", channel);
  return OK;
}

/****************************************************************************
 * Name: ra8p_usb_resume
 *
 * Description:
 *   Resume USB device from suspend based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_resume(uint8_t channel)
{
  struct ra8p_usb_priv_s *priv;
  uint16_t dvstctr;

  if (channel >= 2 || !g_usb[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_usb[channel];

  /* Exit sleep mode */
  uint16_t syscfg = usb_getreg16(priv, RA8P_USB_SYSCFG_OFFSET);
  syscfg &= ~RA8P_USB_SYSCFG_SLPMD;  /* Exit sleep mode */
  usb_putreg16(priv, RA8P_USB_SYSCFG_OFFSET, syscfg);

  /* Resume USB activity */
  dvstctr = usb_getreg16(priv, RA8P_USB_DVSTCTR0_OFFSET);
  dvstctr |= RA8P_USB_DVSTCTR0_UACT;  /* Enable USB activity */
  usb_putreg16(priv, RA8P_USB_DVSTCTR0_OFFSET, dvstctr);

  priv->suspended = false;

  usbinfo("USB%d resumed\n", channel);
  return OK;
}

/****************************************************************************
 * Name: ra8p_usb_is_connected
 *
 * Description:
 *   Check if USB device is connected based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *
 * Returned Value:
 *   true if connected, false otherwise
 *
 ****************************************************************************/

bool ra8p_usb_is_connected(uint8_t channel)
{
  struct ra8p_usb_priv_s *priv;

  if (channel >= 2 || !g_usb[channel].initialized)
    {
      return false;
    }

  priv = &g_usb[channel];

  /* Check connection status in system status register */
  uint16_t syssts = usb_getreg16(priv, RA8P_USB_SYSSTS0_OFFSET);
  uint16_t lnst = (syssts & RA8P_USB_SYSSTS0_LNST_MASK) >> RA8P_USB_SYSSTS0_LNST_SHIFT;

  /* Connection is established when line state is neither SE0 nor unknown */
  return (lnst != RA8P_USB_SYSSTS0_LNST_SE0 && lnst != RA8P_USB_SYSSTS0_LNST_UNDEF);
}

/****************************************************************************
 * Name: ra8p_usb_is_suspended
 *
 * Description:
 *   Check if USB device is suspended based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *
 * Returned Value:
 *   true if suspended, false otherwise
 *
 ****************************************************************************/

bool ra8p_usb_is_suspended(uint8_t channel)
{
  struct ra8p_usb_priv_s *priv;

  if (channel >= 2 || !g_usb[channel].initialized)
    {
      return false;
    }

  priv = &g_usb[channel];
  return priv->suspended;
}

/****************************************************************************
 * Name: ra8p_usb_get_status
 *
 * Description:
 *   Get USB controller status flags based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *
 * Returned Value:
 *   Status flags
 *
 ****************************************************************************/

uint32_t ra8p_usb_get_status(uint8_t channel)
{
  struct ra8p_usb_priv_s *priv;

  if (channel >= 2 || !g_usb[channel].initialized)
    {
      return 0;
    }

  priv = &g_usb[channel];

  uint16_t intsts0 = usb_getreg16(priv, RA8P_USB_INTSTS0_OFFSET);
  uint16_t intsts1 = usb_getreg16(priv, RA8P_USB_INTSTS1_OFFSET);
  uint16_t intsts2 = usb_getreg16(priv, RA8P_USB_INTSTS2_OFFSET);

  return ((uint32_t)intsts2 << 16) | (uint32_t)intsts1 << 8 | (uint32_t)intsts0;
}

/****************************************************************************
 * Name: ra8p_usb_clear_status
 *
 * Description:
 *   Clear USB controller status flags based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   flags - Flags to clear
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_clear_status(uint8_t channel, uint32_t flags)
{
  struct ra8p_usb_priv_s *priv;

  if (channel >= 2 || !g_usb[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_usb[channel];

  /* Clear status flags */
  uint16_t intsts0 = flags & 0xFF;
  uint16_t intsts1 = (flags >> 8) & 0xFF;
  uint16_t intsts2 = (flags >> 16) & 0xFF;

  usb_putreg16(priv, RA8P_USB_INTSTS0_OFFSET, intsts0);
  usb_putreg16(priv, RA8P_USB_INTSTS1_OFFSET, intsts1);
  usb_putreg16(priv, RA8P_USB_INTSTS2_OFFSET, intsts2);

  return OK;
}

#endif /* CONFIG_RA8P_USBFS */