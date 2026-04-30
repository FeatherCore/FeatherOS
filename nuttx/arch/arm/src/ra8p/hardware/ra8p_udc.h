/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_udc.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_UDC_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_UDC_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "chip.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* USB UDC Base Address */

#define RA8P_UDC_BASE                          (0x40250000)

/* USB UDC Register Offsets */

/* System Configuration Control Register (SYSCFG) */

#define RA8P_UDC_SYSCFG_OFFSET                (0x000)
#define RA8P_UDC_SYSCFG_USBE                  (1 << 0)    /* Bit 0: USB Enable */
#define RA8P_UDC_SYSCFG_SCKE                  (1 << 1)    /* Bit 1: SCK Enable */
#define RA8P_UDC_SYSCFG_DPWRE                 (1 << 2)    /* Bit 2: Device Power Enable */
#define RA8P_UDC_SYSCFG_DRPD                  (1 << 3)    /* Bit 3: D+/D- Pull Down */
#define RA8P_UDC_SYSCFG_DCFM                  (1 << 4)    /* Bit 4: Device/Function Mode */
#define RA8P_UDC_SYSCFG_HSE                   (1 << 5)    /* Bit 5: Hi-Speed Enable */
#define RA8P_UDC_SYSCFG_CNEN                  (1 << 6)    /* Bit 6: Connect Enable */
#define RA8P_UDC_SYSCFG_WUPE                  (1 << 7)    /* Bit 7: Wakeup Enable */
#define RA8P_UDC_SYSCFG_WUPSM                 (1 << 8)    /* Bit 8: Wakeup Suspend Mode */

/* System Configuration Status Register (SYSSTS) */

#define RA8P_UDC_SYSSTS_OFFSET                (0x004)
#define RA8P_UDC_SYSSTS_LNST_MASK             (0x3)       /* Bits 0-1: D+/D- Line Status */
#define RA8P_UDC_SYSSTS_LNST_SHIFT            (0)
#define RA8P_UDC_SYSSTS_LNST_SE0              (0x0 << 0)  /* SE0: Both lines pulled low */
#define RA8P_UDC_SYSSTS_LNST_KSTS             (0x1 << 0)  /* K-state: D+ low, D- high */
#define RA8P_UDC_SYSSTS_LNST_JSTS             (0x2 << 0)  /* J-state: D+ high, D- low */
#define RA8P_UDC_SYSSTS_LNST_SE1              (0x3 << 0)  /* SE1: Both lines high */
#define RA8P_UDC_SYSSTS_SOFE                  (1 << 2)    /* Bit 2: SOF Enable */
#define RA8P_UDC_SYSSTS_HTACT                 (1 << 3)    /* Bit 3: Hi-Speed Active */
#define RA8P_UDC_SYSSTS_OVCBI                 (1 << 4)    /* Bit 4: Overcurrent Indication */
#define RA8P_UDC_SYSSTS_PLLACT                (1 << 5)    /* Bit 5: PLL Active */

/* Device Address Register (DADDR) */

#define RA8P_UDC_DADDR_OFFSET                 (0x008)
#define RA8P_UDC_DADDR_USBADDR_MASK           (0x7F)      /* Bits 0-6: USB Address */
#define RA8P_UDC_DADDR_USBADDR_SHIFT          (0)
#define RA8P_UDC_DADDR_USBADDREN              (1 << 7)    /* Bit 7: USB Address Enable */

/* Function Control Register (FUNC) */

#define RA8P_UDC_FUNC_OFFSET                  (0x00C)
#define RA8P_UDC_FUNC_UPPHUB                  (1 << 0)    /* Bit 0: Port of HUB */
#define RA8P_UDC_FUNC_HUBPORT_MASK            (0xF0)      /* Bits 4-7: HUB Port Number */
#define RA8P_UDC_FUNC_HUBPORT_SHIFT           (4)

/* Interrupt Enable Register (INTENB0) */

#define RA8P_UDC_INTENB0_OFFSET               (0x010)
#define RA8P_UDC_INTENB0_BRDYE                (1 << 0)    /* Bit 0: Buffer Ready Enable */
#define RA8P_UDC_INTENB0_NRDYE                (1 << 1)    /* Bit 1: Buffer Not Ready Enable */
#define RA8P_UDC_INTENB0_BEMPE                (1 << 2)    /* Bit 2: Buffer Empty Enable */
#define RA8P_UDC_INTENB0_CTRE                 (1 << 3)    /* Bit 3: Control Transfer Enable */
#define RA8P_UDC_INTENB0_DVSE                 (1 << 4)    /* Bit 4: Device State Enable */
#define RA8P_UDC_INTENB0_SOFE                 (1 << 5)    /* Bit 5: SOF Enable */
#define RA8P_UDC_INTENB0_RSME                 (1 << 6)    /* Bit 6: Resume Enable */
#define RA8P_UDC_INTENB0_VBSE                 (1 << 7)    /* Bit 7: VBUS Enable */
#define RA8P_UDC_INTENB0_EOFERRE              (1 << 8)    /* Bit 8: EOF Error Enable */
#define RA8P_UDC_INTENB0_SIGNE                (1 << 9)    /* Bit 9: Sign Enable */
#define RA8P_UDC_INTENB0_SACKE                (1 << 10)   /* Bit 10: ACK Enable */

/* Interrupt Status Register (INTSTS0) */

#define RA8P_UDC_INTSTS0_OFFSET               (0x014)
#define RA8P_UDC_INTSTS0_BRDY                 (1 << 0)    /* Bit 0: Buffer Ready */
#define RA8P_UDC_INTSTS0_NRDY                 (1 << 1)    /* Bit 1: Buffer Not Ready */
#define RA8P_UDC_INTSTS0_BEMP                 (1 << 2)    /* Bit 2: Buffer Empty */
#define RA8P_UDC_INTSTS0_CTRT                 (1 << 3)    /* Bit 3: Control Transfer */
#define RA8P_UDC_INTSTS0_DVSQ_MASK            (0xE0)      /* Bits 5-7: Device State */
#define RA8P_UDC_INTSTS0_DVSQ_SHIFT           (5)
#define RA8P_UDC_INTSTS0_DVSQ_POWER           (0x0 << 5)  /* Power state */
#define RA8P_UDC_INTSTS0_DVSQ_DEFAULT         (0x1 << 5)  /* Default state */
#define RA8P_UDC_INTSTS0_DVSQ_ADDRESS         (0x2 << 5)  /* Address state */
#define RA8P_UDC_INTSTS0_DVSQ_CONFIGURED      (0x3 << 5)  /* Configured state */
#define RA8P_UDC_INTSTS0_DVSQ_SUSP0           (0x4 << 5)  /* Suspended 0 */
#define RA8P_UDC_INTSTS0_DVSQ_SUSP1           (0x5 << 5)  /* Suspended 1 */
#define RA8P_UDC_INTSTS0_DVSQ_SUSP2           (0x6 << 5)  /* Suspended 2 */
#define RA8P_UDC_INTSTS0_DVSQ_SUSP3           (0x7 << 5)  /* Suspended 3 */
#define RA8P_UDC_INTSTS0_SOF                  (1 << 8)    /* Bit 8: SOF */
#define RA8P_UDC_INTSTS0_RST                  (1 << 9)    /* Bit 9: Reset */
#define RA8P_UDC_INTSTS0_SACK                 (1 << 10)   /* Bit 10: SACK */

/* Buffer Ready Enable Interrupt Register (BRDYENB) */

#define RA8P_UDC_BRDYENB_OFFSET               (0x018)
#define RA8P_UDC_BRDYENB_BRDY0                (1 << 0)    /* Bit 0: Endpoint 0 Buffer Ready */
#define RA8P_UDC_BRDYENB_BRDY1                (1 << 1)    /* Bit 1: Endpoint 1 Buffer Ready */
/* ... up to endpoint 9 */
#define RA8P_UDC_BRDYENB_BRDY9                (1 << 9)    /* Bit 9: Endpoint 9 Buffer Ready */

/* Buffer Not Ready Interrupt Register (NRDYENB) */

#define RA8P_UDC_NRDYENB_OFFSET               (0x01C)
#define RA8P_UDC_NRDYENB_NRDY0                (1 << 0)    /* Bit 0: Endpoint 0 Buffer Not Ready */
/* ... up to endpoint 9 */
#define RA8P_UDC_NRDYENB_NRDY9                (1 << 9)    /* Bit 9: Endpoint 9 Buffer Not Ready */

/* Buffer Empty Enable Interrupt Register (BEMPENB) */

#define RA8P_UDC_BEMPENB_OFFSET               (0x020)
#define RA8P_UDC_BEMPENB_BEMP0                (1 << 0)    /* Bit 0: Endpoint 0 Buffer Empty */
/* ... up to endpoint 9 */
#define RA8P_UDC_BEMPENB_BEMP9                (1 << 9)    /* Bit 9: Endpoint 9 Buffer Empty */

/* Endpoint 0 Control Register (DCPCTR) */

#define RA8P_UDC_DCPCTR_OFFSET                (0x024)
#define RA8P_UDC_DCPCTR_PID_MASK              (0x3)       /* Bits 0-1: PID */
#define RA8P_UDC_DCPCTR_PID_SHIFT             (0)
#define RA8P_UDC_DCPCTR_PID_NAK               (0x0 << 0)  /* NAK */
#define RA8P_UDC_DCPCTR_PID_BUF               (0x1 << 0)  /* BUF */
#define RA8P_UDC_DCPCTR_PID_STALL             (0x2 << 0)  /* STALL */
#define RA8P_UDC_DCPCTR_PID_STALL2            (0x3 << 0)  /* STALL2 */
#define RA8P_UDC_DCPCTR_CCPL                  (1 << 2)    /* Bit 2: Control Transfer Complete */
#define RA8P_UDC_DCPCTR_PBUSY                 (1 << 3)    /* Bit 3: Pipe Busy */
#define RA8P_UDC_DCPCTR_SQMON                 (1 << 4)    /* Bit 4: Sequence Monitor */
#define RA8P_UDC_DCPCTR_SQCLR                 (1 << 5)    /* Bit 5: Sequence Toggle Clear */
#define RA8P_UDC_DCPCTR_CSCLR                 (1 << 6)    /* Bit 6: Control Sequence Clear */

/* Endpoint Register (PIPESEL) */

#define RA8P_UDC_PIPESEL_OFFSET               (0x028)
#define RA8P_UDC_PIPESEL_PIPENB_MASK          (0xF)       /* Bits 0-3: Pipe Number */
#define RA8P_UDC_PIPESEL_PIPENB_SHIFT         (0)

/* Endpoint Control Register (PIPEnCTR) */

#define RA8P_UDC_PIPECTR_OFFSET               (0x02C)  /* Use PIPESEL to select pipe 0-9 */
#define RA8P_UDC_PIPECTR_PID_MASK             (0x3)       /* Bits 0-1: PID */
#define RA8P_UDC_PIPECTR_PID_SHIFT            (0)
#define RA8P_UDC_PIPECTR_PID_NAK              (0x0 << 0)  /* NAK */
#define RA8P_UDC_PIPECTR_PID_BUF              (0x1 << 0)  /* BUF */
#define RA8P_UDC_PIPECTR_PID_STALL            (0x2 << 0)  /* STALL */
#define RA8P_UDC_PIPECTR_PID_STALL2           (0x3 << 0)  /* STALL2 */
#define RA8P_UDC_PIPECTR_PBUSY                (1 << 3)    /* Bit 3: Pipe Busy */
#define RA8P_UDC_PIPECTR_ATREPM               (1 << 4)    /* Bit 4: Auto Repsonse Mode */
#define RA8P_UDC_PIPECTR_ACLRM                (1 << 5)    /* Bit 5: Auto Buffer Clear */
#define RA8P_UDC_PIPECTR_SQCLR                (1 << 6)    /* Bit 6: Sequence Toggle Clear */
#define RA8P_UDC_PIPECTR_SQR                  (1 << 7)    /* Bit 7: Sequence Toggle */

/* Endpoint Transaction Counter Register (PIPE_TRN) */

#define RA8P_UDC_PIPE_TRN_OFFSET              (0x030)
#define RA8P_UDC_PIPE_TRN_TRNCNT_MASK         (0xFFFF)    /* Bits 0-15: Transaction Counter */
#define RA8P_UDC_PIPE_TRN_TRNCNT_SHIFT        (0)

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* USB device state */

enum ra8p_udc_device_state_e
{
  RA8P_UDC_DEVICE_POWER = 0,        /* Power state */
  RA8P_UDC_DEVICE_DEFAULT,          /* Default state */
  RA8P_UDC_DEVICE_ADDRESS,          /* Address state */
  RA8P_UDC_DEVICE_CONFIGURED,       /* Configured state */
  RA8P_UDC_DEVICE_SUSPENDED,        /* Suspended state */
};

/* USB endpoint types */

enum ra8p_udc_endpoint_type_e
{
  RA8P_UDC_EP_CONTROL = 0,          /* Control endpoint */
  RA8P_UDC_EP_ISOCHRONOUS,          /* Isochronous endpoint */
  RA8P_UDC_EP_BULK,                 /* Bulk endpoint */
  RA8P_UDC_EP_INTERRUPT,            /* Interrupt endpoint */
};

/* USB endpoint structure */

struct ra8p_udc_endpoint_s
{
  uint8_t ep_num;                   /* Endpoint number (0-9) */
  enum ra8p_udc_endpoint_type_e type; /* Endpoint type */
  uint16_t maxpacket;               /* Maximum packet size */
  bool dir_in;                      /* Direction (true = IN, false = OUT) */
  uint8_t *buffer;                  /* Buffer address */
  size_t buflen;                    /* Buffer length */
  bool enabled;                     /* Endpoint enabled flag */
};

/* USB UDC configuration */

struct ra8p_udc_config_s
{
  uint32_t base;                    /* USB UDC base address */
  int irq;                          /* USB UDC interrupt number */
  struct ra8p_udc_endpoint_s eps[10]; /* Endpoint configurations */
  uint8_t num_eps;                  /* Number of endpoints (1-10) */
  bool hi_speed;                    /* Hi-Speed mode enabled */
  bool connected;                   /* Device connected */
  bool suspended;                   /* Device suspended */
};

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_UDC_H */