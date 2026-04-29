/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_otgfs.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_STM32N6_OTGFS_H
#define __ARCH_ARM_SRC_STM32N6_STM32N6_OTGFS_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <nuttx/usb/usb.h>
#include <nuttx/usb/usbdev.h>
#include <nuttx/usb/usbhost.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_OTGFS_BASE             (STM32N6_PERIPH_BASE + 0x08040000)
#define STM32_OTGHS1_BASE           (STM32N6_PERIPH_BASE + 0x08040000)  /* USBOTG_HS1 */
#define STM32_OTGHS2_BASE           (STM32N6_PERIPH_BASE + 0x08080000)  /* USBOTG_HS2 */

/* Register Offsets */
#define STM32_OTG_GOTGCTL_OFFSET     0x0000  /* Control and Status Register */
#define STM32_OTG_GOTGINT_OFFSET     0x0004  /* Interrupt Register */
#define STM32_OTG_GAHBCFG_OFFSET     0x0008  /* AHB Configuration Register */
#define STM32_OTG_GUSBCFG_OFFSET     0x000C  /* USB Configuration Register */
#define STM32_OTG_GRSTCTL_OFFSET     0x0010  /* Reset Register */
#define STM32_OTG_GINTSTS_OFFSET     0x0014  /* Interrupt Status Register */
#define STM32_OTG_GINTMSK_OFFSET     0x0018  /* Interrupt Mask Register */
#define STM32_OTG_GRXSTSR_OFFSET     0x001C  /* Receive Status Debug Read Register */
#define STM32_OTG_GRXSTSP_OFFSET     0x0020  /* Receive Status Read/Pop Register */
#define STM32_OTG_GRXFIFOSIZ_OFFSET  0x0024  /* Receive FIFO Size Register */
#define STM32_OTG_GNPTXFIFOSIZ_OFFSET 0x0028 /* Non-Periodic TX FIFO Size Register */
#define STM32_OTG_GNPTXSTS_OFFSET    0x0028  /* Non-Periodic TX FIFO Status */
#define STM32_OTG_GCCFG_OFFSET       0x0038  /* General Core Configuration Register */
#define STM32_OTG_CID_OFFSET         0x003C  /* Core ID Register */
#define STM32_OTG_GLPMCFG_OFFSET     0x0540  /* LPM Configuration Register */
#define STM32_OTG_GPWRDN_OFFSET      0x0544  /* Power Down Register */
#define STM32_OTG_GDFIFOCFG_OFFSET   0x0548  /* DFIFO Configuration Register */
#define STM32_OTG_HPTXFSIZ_OFFSET    0x0100  /* Host Periodic TX FIFO Size Register */

/* Device Configuration Registers */
#define STM32_OTG_DCFG_OFFSET        0x0800  /* Device Configuration Register */
#define STM32_OTG_DCTL_OFFSET        0x0804  /* Device Control Register */
#define STM32_OTG_DSTS_OFFSET        0x0808  /* Device Status Register */
#define STM32_OTG_DIEPMSK_OFFSET     0x0810  /* Device IN Endpoint Common Interrupt Mask */
#define STM32_OTG_DOEPMSK_OFFSET     0x0814  /* Device OUT Endpoint Common Interrupt Mask */
#define STM32_OTG_DAINT_OFFSET       0x0818  /* Device All Endpoints Interrupt Register */
#define STM32_OTG_DAINTMSK_OFFSET    0x081C  /* Device All Endpoints Interrupt Mask */

/* Device IN Endpoint Specific Registers */
#define STM32_OTG_DIEPCTL_OFFSET(ep) (0x0900 + (ep) * 0x20)  /* Device EP Control */
#define STM32_OTG_DIEPTSIZ_OFFSET(ep) (0x0910 + (ep) * 0x20) /* Device EP Transfer Size */
#define STM32_OTG_DIEPDMA_OFFSET(ep) (0x0914 + (ep) * 0x20)  /* Device EP DMA Address */

/* Device OUT Endpoint Specific Registers */
#define STM32_OTG_DOEPCTL_OFFSET(ep) (0x0B00 + (ep) * 0x20)  /* Device EP Control */
#define STM32_OTG_DOEPTSIZ_OFFSET(ep) (0x0B10 + (ep) * 0x20) /* Device EP Transfer Size */
#define STM32_OTG_DOEPDMA_OFFSET(ep) (0x0B14 + (ep) * 0x20)  /* Device EP DMA Address */

/* Host Mode Registers */
#define STM32_OTG_HCFG_OFFSET        0x0400  /* Host Configuration Register */
#define STM32_OTG_HFIR_OFFSET        0x0404  /* Host Frame Interval Register */
#define STM32_OTG_HFNUM_OFFSET       0x0408  /* Host Frame Number/Frame Remaining */
#define STM32_OTG_HPTXSTS_OFFSET     0x0410  /* Host Periodic TX Status */
#define STM32_OTG_HAINT_OFFSET       0x0414  /* Host All Channels Interrupt Register */
#define STM32_OTG_HAINTMSK_OFFSET    0x0418  /* Host All Channels Interrupt Mask */

/* Host Channel Specific Registers */
#define STM32_OTG_HCCHAR_OFFSET(ch)  (0x0500 + (ch) * 0x20)  /* Host Channel Characteristics */
#define STM32_OTG_HCINT_OFFSET(ch)   (0x0508 + (ch) * 0x20)  /* Host Channel Interrupt */
#define STM32_OTG_HCINTMSK_OFFSET(ch) (0x050C + (ch) * 0x20) /* Host Channel Interrupt Mask */
#define STM32_OTG_HCTSIZ_OFFSET(ch)  (0x0510 + (ch) * 0x20)  /* Host Channel Transfer Size */

/* Core Register Bit Definitions */

/* GOTGCTL Register */
#define OTG_GOTGCTL_SRQSCS          (1 << 0)   /* Session request success */
#define OTG_GOTGCTL_SRQ             (1 << 1)   /* Session request */
#define OTG_GOTGCTL_HNGSCS          (1 << 8)   /* HNP request */
#define OTG_GOTGCTL_HNPRQ           (1 << 9)   /* HNP request */
#define OTG_GOTGCTL_HSHNPEN         (1 << 10)  /* Host set HNP enable */
#define OTG_GOTGCTL_DHNPEN          (1 << 11)  /* Device HNP enable */
#define OTG_GOTGCTL_CIDSTS          (1 << 16)  /* Connector ID status */
#define OTG_GOTGCTL_DBCT            (1 << 17)  /* Long/short debounce time */
#define OTG_GOTGCTL_ASVLD           (1 << 18)  /* A-session valid */
#define OTG_GOTGCTL_BSVLD           (1 << 19)  /* B-session valid */
#define OTG_GOTGCTL_OTGVER          (1 << 20)  /* OTG version */

/* GAHBCFG Register */
#define OTG_GAHBCFG_GLBINT          (1 << 0)   /* Global interrupt mask */
#define OTG_GAHBCFG_HBSTLEN_SHIFT   1          /* Burst length/type */
#define OTG_GAHBCFG_HBSTLEN_MASK    (0x0F << 1)
#define OTG_GAHBCFG_HBSTLEN_INCR    (0 << 1)   /* Single incr. burst */
#define OTG_GAHBCFG_HBSTLEN_INCR4   (3 << 1)   /* 4-beat incr. burst */
#define OTG_GAHBCFG_HBSTLEN_INCR8   (5 << 1)   /* 8-beat incr. burst */
#define OTG_GAHBCFG_HBSTLEN_INCR16  (7 << 1)   /* 16-beat incr. burst */
#define OTG_GAHBCFG_DMAEN           (1 << 5)   /* DMA enable */
#define OTG_GAHBCFG_TXFELVL         (1 << 7)   /* TxFIFO empty level */
#define OTG_GAHBCFG_PTXFELVL        (1 << 8)   /* Periodic TxFIFO empty level */

/* GUSBCFG Register */
#define OTG_GUSBCFG_TOCAL_SHIFT     0          /* FS timeout calibration */
#define OTG_GUSBCFG_TOCAL_MASK      (7 << 0)
#define OTG_GUSBCFG_PHYSEL          (1 << 6)   /* USB 2.0 high-speed ULPI PHY or USB 1.1 full-speed serial transceiver select */
#define OTG_GUSBCFG_SRPCAP          (1 << 8)   /* SRP capability */
#define OTG_GUSBCFG_HNPCAP          (1 << 9)   /* HNP capability */
#define OTG_GUSBCFG_USBTRDTIM_SHIFT 10         /* USB turnaround time */
#define OTG_GUSBCFG_USBTRDTIM_MASK  (0x0F << 10)
#define OTG_GUSBCFG_PHYLPCS         (1 << 15)  /* PHY Low-power clock select */
#define OTG_GUSBCFG_ULPIFSLS        (1 << 17)  /* ULPI FS/LS select */
#define OTG_GUSBCFG_ULPIAUTORES     (1 << 18)  /* ULPI Auto-resume */
#define OTG_GUSBCFG_ULPICSM         (1 << 19)  /* ULPI Clock SuspendM */
#define OTG_GUSBCFG_ULPIEVBUSD      (1 << 20)  /* ULPI External VBUS Drive */
#define OTG_GUSBCFG_ULPIEVBUSI      (1 << 21)  /* ULPI External VBUS Indicator */
#define OTG_GUSBCFG_TSDPS           (1 << 22)  /* TermSel DLine pulsing selection */
#define OTG_GUSBCFG_PCCI            (1 << 23)  /* Indicator complement */
#define OTG_GUSBCFG_PTCI            (1 << 24)  /* Indicator pass through */
#define OTG_GUSBCFG_ULPIIPD         (1 << 25)  /* ULPI interface protect disable */
#define OTG_GUSBCFG_FHMOD           (1 << 29)  /* Force host mode */
#define OTG_GUSBCFG_FDMOD           (1 << 30)  /* Force device mode */
#define OTG_GUSBCFG_CTXPKT          (1 << 31)  /* Corrupt Tx packet */

/* GRSTCTL Register */
#define OTG_GRSTCTL_CSRST           (1 << 0)   /* Core soft reset */
#define OTG_GRSTCTL_HSRST           (1 << 1)   /* HCLK soft reset */
#define OTG_GRSTCTL_FCRST           (1 << 2)   /* Host frame counter reset */
#define OTG_GRSTCTL_RXFFLSH         (1 << 4)   /* RxFIFO flush */
#define OTG_GRSTCTL_TXFFLSH_SHIFT   5          /* TxFIFO flush */
#define OTG_GRSTCTL_TXFFLSH_MASK    (0x1F << 5)
#define OTG_GRSTCTL_TXFNUM_SHIFT    6          /* TxFIFO number */
#define OTG_GRSTCTL_TXFNUM_MASK     (0x1F << 6)
#define OTG_GRSTCTL_AHBIDL          (1 << 31)  /* AHB Master idle */

/* GINTSTS Register */
#define OTG_GINTSTS_CMOD            (1 << 0)   /* Current mode of operation */
#define OTG_GINTSTS_MMIS            (1 << 1)   /* Mode mismatch interrupt */
#define OTG_GINTSTS_OTGINT          (1 << 2)   /* OTG interrupt */
#define OTG_GINTSTS_SOF             (1 << 3)   /* Start of frame */
#define OTG_GINTSTS_RXFLVL          (1 << 4)   /* RxFIFO non-empty */
#define OTG_GINTSTS_NPTXFE          (1 << 5)   /* Non-periodic TxFIFO empty */
#define OTG_GINTSTS_GINNAKEFF       (1 << 6)   /* Global IN non-periodic NAK effective */
#define OTG_GINTSTS_BOUTNAKEFF      (1 << 7)   /* Global OUT NAK effective */
#define OTG_GINTSTS_ESUSP           (1 << 10)  /* Early suspend */
#define OTG_GINTSTS_USBSUSP         (1 << 11)  /* USB suspend */
#define OTG_GINTSTS_USBRST          (1 << 12)  /* USB reset */
#define OTG_GINTSTS_ENUMDNE         (1 << 13)  /* Enumeration done */
#define OTG_GINTSTS_ISOODRP         (1 << 14)  /* Isochronous OUT packet dropped interrupt */
#define OTG_GINTSTS_EOPF            (1 << 15)  /* End of periodic frame interrupt */
#define OTG_GINTSTS_IEPINT          (1 << 18)  /* IN endpoint interrupt */
#define OTG_GINTSTS_OEPINT          (1 << 19)  /* OUT endpoint interrupt */
#define OTG_GINTSTS_IISOIXFR        (1 << 20)  /* Incomplete isochronous IN transfer */
#define OTG_GINTSTS_PXFR            (1 << 21)  /* Incomplete periodic transfer */
#define OTG_GINTSTS_FSUSP           (1 << 22)  /* Data fetch suspended */
#define OTG_GINTSTS_RSTDET          (1 << 23)  /* Reset detected interrupt */
#define OTG_GINTSTS_HPRTINT         (1 << 24)  /* Host port interrupt */
#define OTG_GINTSTS_HCINT           (1 << 25)  /* Host channels interrupt */
#define OTG_GINTSTS_PTXFE           (1 << 26)  /* Periodic TxFIFO empty */
#define OTG_GINTSTS_LPM             (1 << 27)  /* LPM interrupt */
#define OTG_GINTSTS_CIDSCHG         (1 << 28)  /* Connector ID status change */
#define OTG_GINTSTS_DISCINT         (1 << 29)  /* Disconnect detected interrupt */
#define OTG_GINTSTS_SRQINT          (1 << 30)  /* Session request/new session detected interrupt */
#define OTG_GINTSTS_WKUINT          (1 << 31)  /* Resume/remote wakeup detected interrupt */

/* Device Configuration Register (DCFG) */
#define OTG_DCFG_DSPD_SHIFT         0          /* Device speed */
#define OTG_DCFG_DSPD_MASK          (3 << 0)
#define OTG_DCFG_DSPD_FS_PHY_48MHZ  (3 << 0)   /* Full speed on PHY clock at 48MHz */
#define OTG_DCFG_NZLSOHSK           (1 << 2)   /* Non-zero-length status OUT handshake */
#define OTG_DCFG_DAD_SHIFT          4          /* Device address */
#define OTG_DCFG_DAD_MASK           (0x7F << 4)
#define OTG_DCFG_PFIVL_SHIFT        11         /* Periodic frame interval */
#define OTG_DCFG_PFIVL_MASK         (3 << 11)

/* Device Control Register (DCTL) */
#define OTG_DCTL_RWUSIG             (1 << 0)   /* Remote wakeup signaling */
#define OTG_DCTL_SDIS               (1 << 1)   /* Soft disconnect */
#define OTG_DCTL_GINSTS             (1 << 2)   /* Global IN NAK status */
#define OTG_DCTL_GONSTS             (1 << 3)   /* Global OUT NAK status */
#define OTG_DCTL_TCTL_SHIFT         4          /* Test control */
#define OTG_DCTL_TCTL_MASK          (7 << 4)
#define OTG_DCTL_SGINAK             (1 << 7)   /* Set global IN NAK */
#define OTG_DCTL_CGINAK             (1 << 8)   /* Clear global IN NAK */
#define OTG_DCTL_SGONAK             (1 << 9)   /* Set global OUT NAK */
#define OTG_DCTL_CGONAK             (1 << 10)  /* Clear global OUT NAK */
#define OTG_DCTL_POPRGDNE           (1 << 11)  /* Power-on programming done */

/* Device Endpoint Control Register (DIEPCTL/DOEPCTL) */
#define OTG_DEPCTL_MPSIZ_SHIFT      0          /* Maximum packet size */
#define OTG_DEPCTL_MPSIZ_MASK       (0x7FF << 0)
#define OTG_DEPCTL_USBAEP           (1 << 15)  /* USB active endpoint */
#define OTG_DEPCTL_EONUM_DPID       (1 << 16)  /* Even/odd frame number */
#define OTG_DEPCTL_NAKSTS           (1 << 17)  /* NAK status */
#define OTG_DEPCTL_EPTYP_SHIFT      18         /* Endpoint type */
#define OTG_DEPCTL_EPTYP_MASK       (3 << 18)
#define OTG_DEPCTL_EPTYP_CTRL       (0 << 18)  /* Control */
#define OTG_DEPCTL_EPTYP_ISO        (1 << 18)  /* Isochronous */
#define OTG_DEPCTL_EPTYP_BULK       (2 << 18)  /* Bulk */
#define OTG_DEPCTL_EPTYP_INTR       (3 << 18)  /* Interrupt */
#define OTG_DEPCTL_STALL            (1 << 21)  /* STALL handshake */
#define OTG_DEPCTL_TXFNUM_SHIFT     22         /* TxFIFO number */
#define OTG_DEPCTL_TXFNUM_MASK      (0xF << 22)
#define OTG_DEPCTL_CNAK             (1 << 26)  /* Clear NAK */
#define OTG_DEPCTL_SNAK             (1 << 27)  /* Set NAK */
#define OTG_DEPCTL_SD0PID_SEVNFRM   (1 << 28)  /* Set DATA0 PID/SEVENFRM */
#define OTG_DEPCTL_SODDFRM          (1 << 29)  /* Set odd frame/SD0PID */
#define OTG_DEPCTL_EPDIS            (1 << 30)  /* Endpoint disable */
#define OTG_DEPCTL_EPENA            (1 << 31)  /* Endpoint enable */

/* Endpoint Transfer Size Register (DIEPTSIZ/DOEPTSIZ) */
#define OTG_DEPTSIZ_XFRSIZ_SHIFT    0          /* Transfer size */
#define OTG_DEPTSIZ_XFRSIZ_MASK     (0x7FFFF << 0)
#define OTG_DEPTSIZ_PKTCNT_SHIFT    19         /* Packet count */
#define OTG_DEPTSIZ_PKTCNT_MASK     (0x3FF << 19)
#define OTG_DEPTSIZ_MULCNT_SHIFT    29         /* Packet count (multi-count) */
#define OTG_DEPTSIZ_MULCNT_MASK     (3 << 29)

/* Host Channel Characteristic Register (HCCHAR) */
#define OTG_HCCHAR_MPSIZ_SHIFT      0          /* Maximum packet size */
#define OTG_HCCHAR_MPSIZ_MASK       (0x7FF << 0)
#define OTG_HCCHAR_EPNUM_SHIFT      11         /* Endpoint number */
#define OTG_HCCHAR_EPNUM_MASK       (0xF << 11)
#define OTG_HCCHAR_EPDIR            (1 << 15)  /* Endpoint direction */
#define OTG_HCCHAR_LSDEV            (1 << 17)  /* Low-speed device */
#define OTG_HCCHAR_EPTYP_SHIFT      18         /* Endpoint type */
#define OTG_HCCHAR_EPTYP_MASK       (3 << 18)
#define OTG_HCCHAR_EPTYP_CTRL       (0 << 18)  /* Control */
#define OTG_HCCHAR_EPTYP_ISO        (1 << 18)  /* Isochronous */
#define OTG_HCCHAR_EPTYP_BULK       (2 << 18)  /* Bulk */
#define OTG_HCCHAR_EPTYP_INTR       (3 << 18)  /* Interrupt */
#define OTG_HCCHAR_MC_SHIFT         20         /* Multi Count */
#define OTG_HCCHAR_MC_MASK          (3 << 20)
#define OTG_HCCHAR_DAD_SHIFT        22         /* Device address */
#define OTG_HCCHAR_DAD_MASK         (0x7F << 22)
#define OTG_HCCHAR_ODDFRM           (1 << 29)  /* Odd frame */
#define OTG_HCCHAR_CHDIS            (1 << 30)  /* Channel disable */
#define OTG_HCCHAR_CHENA            (1 << 31)  /* Channel enable */

/* Maximum endpoints and channels */
#define STM32N6_OTGFS_NENDPOINTS    4
#define STM32N6_OTGHS_NENDPOINTS    6
#define STM32N6_OTGFS_NCHANNELS     8
#define STM32N6_OTGHS_NCHANNELS     12

/* Default FIFO sizes */
#define STM32N6_OTGFS_RX_FIFO_SIZE  512
#define STM32N6_OTGFS_TX0_FIFO_SIZE 64
#define STM32N6_OTGFS_TX1_FIFO_SIZE 128
#define STM32N6_OTGFS_TX2_FIFO_SIZE 128
#define STM32N6_OTGFS_TX3_FIFO_SIZE 128

#define STM32N6_OTGHS_RX_FIFO_SIZE  1024
#define STM32N6_OTGHS_TX0_FIFO_SIZE 512
#define STM32N6_OTGHS_TX1_FIFO_SIZE 512
#define STM32N6_OTGHS_TX2_FIFO_SIZE 512
#define STM32N6_OTGHS_TX3_FIFO_SIZE 512
#define STM32N6_OTGHS_TX4_FIFO_SIZE 512
#define STM32N6_OTGHS_TX5_FIFO_SIZE 512

/****************************************************************************
 * Public Types
 ****************************************************************************/

struct stm32n6_otgfs_dev_s
{
  /* Core driver state */
  struct usbdev_s         usbdev;
  struct stm32n6_otgfs_dev_s *next;

  /* Driver state */
  uint8_t                 ep0state;      /* State of EP0 */
  uint8_t                 neps;          /* Number of endpoints */
  bool                    attached;      /* Connected to host */
  bool                    suspended;     /* Suspended */
  bool                    selfpowered;   /* Self-powered */

  /* Register base address */
  uint32_t                base;
  uint32_t                hcoutsize;     /* Size of HCOUT register set */
  int                     hcd_irq;       /* Host mode interrupt */
  int                     dev_irq;       /* Device mode interrupt */

  /* Endpoint list */
  struct stm32n6_ep_s     epin[STM32N6_OTGFS_NENDPOINTS];
  struct stm32n6_ep_s     epout[STM32N6_OTGFS_NENDPOINTS];

  /* Request queue */
  struct stm32n6_req_s    *ep0req;       /* EP0 request */
};

struct stm32n6_otghs_dev_s
{
  /* Core driver state */
  struct usbdev_s         usbdev;
  struct stm32n6_otghs_dev_s *next;

  /* Driver state */
  uint8_t                 ep0state;      /* State of EP0 */
  uint8_t                 neps;          /* Number of endpoints */
  bool                    attached;      /* Connected to host */
  bool                    suspended;     /* Suspended */
  bool                    selfpowered;   /* Self-powered */

  /* Register base address */
  uint32_t                base;
  uint32_t                hcoutsize;     /* Size of HCOUT register set */
  int                     hcd_irq;       /* Host mode interrupt */
  int                     dev_irq;       /* Device mode interrupt */

  /* Endpoint list */
  struct stm32n6_ep_s     epin[STM32N6_OTGHS_NENDPOINTS];
  struct stm32n6_ep_s     epout[STM32N6_OTGHS_NENDPOINTS];

  /* Request queue */
  struct stm32n6_req_s    *ep0req;       /* EP0 request */
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

int stm32n6_otgfs_initialize(void);
int stm32n6_otghs_initialize(void);

#endif /* __ARCH_ARM_SRC_STM32N6_STM32N6_OTGFS_H */