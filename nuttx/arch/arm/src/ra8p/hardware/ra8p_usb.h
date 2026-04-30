/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_usb.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_USB_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_USB_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* USB Register Offsets - Based on RA8P Hardware Manual and Zephyr definitions */
#define RA8P_USB_SYSCFG_OFFSET           0x0000  /* System Configuration Register */
#define RA8P_USB_SYSSTS0_OFFSET          0x0004  /* System Status Register 0 */
#define RA8P_USB_DVSTCTR0_OFFSET         0x0008  /* Device Control Register 0 */
#define RA8P_USB_CFIFO_OFFSET            0x0010  /* Control Transfer FIFO Register */
#define RA8P_USB_D0FIFO_OFFSET           0x0014  /* Data Transfer FIFO Register 0 */
#define RA8P_USB_D1FIFO_OFFSET           0x0018  /* Data Transfer FIFO Register 1 */
#define RA8P_USB_CFIFOSEL_OFFSET         0x0020  /* Control Transfer FIFO Select Register */
#define RA8P_USB_CFIFOCTR_OFFSET         0x0022  /* Control Transfer FIFO Control Register */
#define RA8P_USB_D0FIFOSEL_OFFSET        0x0024  /* Data Transfer FIFO Select Register 0 */
#define RA8P_USB_D0FIFOCTR_OFFSET        0x0026  /* Data Transfer FIFO Control Register 0 */
#define RA8P_USB_D1FIFOSEL_OFFSET        0x0028  /* Data Transfer FIFO Select Register 1 */
#define RA8P_USB_D1FIFOCTR_OFFSET        0x002A  /* Data Transfer FIFO Control Register 1 */
#define RA8P_USB_INTENB0_OFFSET          0x0030  /* Interrupt Enable Register 0 */
#define RA8P_USB_INTENB1_OFFSET          0x0032  /* Interrupt Enable Register 1 */
#define RA8P_USB_INTENB2_OFFSET          0x0034  /* Interrupt Enable Register 2 */
#define RA8P_USB_INTSTS0_OFFSET          0x0038  /* Interrupt Status Register 0 */
#define RA8P_USB_INTSTS1_OFFSET          0x003A  /* Interrupt Status Register 1 */
#define RA8P_USB_INTSTS2_OFFSET          0x003C  /* Interrupt Status Register 2 */
#define RA8P_USB_FIFOSEL_OFFSET          0x0044  /* FIFO Select Register */
#define RA8P_USB_FIFOCTR_OFFSET          0x0046  /* FIFO Control Register */
#define RA8P_USB_PIPESEL_OFFSET          0x0048  /* Pipe Select Register */
#define RA8P_USB_PIPECFG_OFFSET          0x004A  /* Pipe Configuration Register */
#define RA8P_USB_PIPEBUF_OFFSET          0x004C  /* Pipe Buffer Register */
#define RA8P_USB_PIPETRN_OFFSET          0x004E  /* Pipe Transaction Counter */
#define RA8P_USB_PIPEPERI_OFFSET         0x0050  /* Pipe Periodic Interval */
#define RA8P_USB_PIPE1CTR_OFFSET         0x0058  /* Pipe 1 Control Register */
#define RA8P_USB_PIPE2CTR_OFFSET         0x005A  /* Pipe 2 Control Register */
#define RA8P_USB_PIPE3CTR_OFFSET         0x005C  /* Pipe 3 Control Register */
#define RA8P_USB_PIPE4CTR_OFFSET         0x005E  /* Pipe 4 Control Register */
#define RA8P_USB_PIPE5CTR_OFFSET         0x0060  /* Pipe 5 Control Register */
#define RA8P_USB_PIPE6CTR_OFFSET         0x0062  /* Pipe 6 Control Register */
#define RA8P_USB_PIPE7CTR_OFFSET         0x0064  /* Pipe 7 Control Register */
#define RA8P_USB_PIPE8CTR_OFFSET         0x0066  /* Pipe 8 Control Register */
#define RA8P_USB_PIPE9CTR_OFFSET         0x0068  /* Pipe 9 Control Register */
#define RA8P_USB_PIPECTR_OFFSET          0x0070  /* Pipe Control Register */
#define RA8P_USB_PIPEMAXP_OFFSET         0x0072  /* Pipe Maximum Packet Size Register */
#define RA8P_USB_BRDYENB_OFFSET          0x0078  /* BRDY Interrupt Enable Register */
#define RA8P_USB_NRDYENB_OFFSET          0x007A  /* NRDY Interrupt Enable Register */
#define RA8P_USB_BEMPENB_OFFSET          0x007C  /* BEMP Interrupt Enable Register */
#define RA8P_USB_BRDYSTS_OFFSET          0x0080  /* BRDY Interrupt Status Register */
#define RA8P_USB_NRDYSTS_OFFSET          0x0082  /* NRDY Interrupt Status Register */
#define RA8P_USB_BEMPSTS_OFFSET          0x0084  /* BEMP Interrupt Status Register */
#define RA8P_USB_DFIFO0_OFFSET           0x00C0  /* DFIFO0 Register */
#define RA8P_USB_DFIFO1_OFFSET           0x00E0  /* DFIFO1 Register */
#define RA8P_USB_DFIFOSEL_OFFSET         0x0100  /* DFIFO Select Register */
#define RA8P_USB_DFIFOSELB_OFFSET        0x0104  /* DFIFO Select B Register */
#define RA8P_USB_DFIFOCFG_OFFSET         0x0108  /* DFIFO Configuration Register */
#define RA8P_USB_DFIFOCTR_OFFSET         0x010C  /* DFIFO Control Register */

/* SYSCFG - System Configuration Register */
#define RA8P_USB_SYSCFG_USBEN            (1 << 0)   /* USB Enable */
#define RA8P_USB_SYSCFG_USBE             (1 << 1)   /* USB Clock Enable */
#define RA8P_USB_SYSCFG_DCFM             (1 << 4)   /* Device/Function Mode */
#define RA8P_USB_SYSCFG_DRPD             (1 << 5)   /* D+/D- Pull-down Control */
#define RA8P_USB_SYSCFG_DPRPU            (1 << 6)   /* D+ Pull-up Control */
#define RA8P_USB_SYSCFG_SCKE             (1 << 8)   /* System Clock Enable */
#define RA8P_USB_SYSCFG_CNEN             (1 << 9)   /* Cable Connection Enable */
#define RA8P_USB_SYSCFG_HSE              (1 << 10)  /* Hi-Speed Enable */
#define RA8P_USB_SYSCFG_DVSQ_MASK        (0x07 << 12) /* Device State */
#define RA8P_USB_SYSCFG_DVSQ_SHIFT       12

/* SYSSTS0 - System Status Register 0 */
#define RA8P_USB_SYSSTS0_LNST_MASK       (0x03 << 0)  /* USB Data Line Status */
#define RA8P_USB_SYSSTS0_LNST_SHIFT      0
#define RA8P_USB_SYSSTS0_LNST_SE0        0            /* SE0 */
#define RA8P_USB_SYSSTS0_LNST_KSTS       1            /* K State */
#define RA8P_USB_SYSSTS0_LNST_JSTS       2            /* J State */
#define RA8P_USB_SYSSTS0_LNST_UNDEF      3            /* Undefined */
#define RA8P_USB_SYSSTS0_OVRC0           (1 << 4)     /* Overcurrent Input 0 */
#define RA8P_USB_SYSSTS0_HTACT           (1 << 8)     /* USB Host Control Transfer Active */

/* DVSTCTR0 - Device Control Register 0 */
#define RA8P_USB_DVSTCTR0_UACT           (1 << 0)     /* USB Activate */
#define RA8P_USB_DVSTCTR0_RESUME         (1 << 2)     /* Resume Output */
#define RA8P_USB_DVSTCTR0_USBRST         (1 << 3)     /* USB Reset */
#define RA8P_USB_DVSTCTR0_RWUPE          (1 << 4)     /* Wake-up Enable */
#define RA8P_USB_DVSTCTR0_USBADDR_MASK   (0x7F << 8)  /* USB Address */
#define RA8P_USB_DVSTCTR0_USBADDR_SHIFT  8
#define RA8P_USB_DVSTCTR0_WKUP           (1 << 15)    /* Wake-up Interrupt */
#define RA8P_USB_DVSTCTR0_VBSE           (1 << 16)    /* VBUS Interrupt Enable */
#define RA8P_USB_DVSTCTR0_RSME           (1 << 17)    /* Resume Interrupt Enable */
#define RA8P_USB_DVSTCTR0_SOFE           (1 << 18)    /* SOF Interrupt Enable */
#define RA8P_USB_DVSTCTR0_DVSE           (1 << 19)    /* Device State Interrupt Enable */
#define RA8P_USB_DVSTCTR0_CTRE           (1 << 20)    /* Control Transfer Interrupt Enable */
#define RA8P_USB_DVSTCTR0_BEMPE          (1 << 21)    /* Buffer Empty Interrupt Enable */
#define RA8P_USB_DVSTCTR0_NRDYE          (1 << 22)    /* Not Ready Interrupt Enable */
#define RA8P_USB_DVSTCTR0_BRDYE          (1 << 23)    /* Buffer Ready Interrupt Enable */

/* CFIFOCTR - Control Transfer FIFO Control Register */
#define RA8P_USB_CFIFOCTR_DTLN_MASK      (0x1FF << 0) /* Data Transfer Length */
#define RA8P_USB_CFIFOCTR_DTLN_SHIFT     0
#define RA8P_USB_CFIFOCTR_FRDY           (1 << 16)    /* FIFO Ready */
#define RA8P_USB_CFIFOCTR_BCLR           (1 << 17)    /* Buffer Clear */
#define RA8P_USB_CFIFOCTR_BVAL           (1 << 18)    /* Buffer Valid */
#define RA8P_USB_CFIFOCTR_CTSQ_MASK      (0x07 << 19) /* Control Transfer Stage */
#define RA8P_USB_CFIFOCTR_CTSQ_SHIFT     19
#define RA8P_USB_CFIFOCTR_CTSQ_SETUP0    0            /* Setup 0 */
#define RA8P_USB_CFIFOCTR_CTSQ_SETUP1    1            /* Setup 1 */
#define RA8P_USB_CFIFOCTR_CTSQ_IN0       2            /* In 0 */
#define RA8P_USB_CFIFOCTR_CTSQ_IN1       3            /* In 1 */
#define RA8P_USB_CFIFOCTR_CTSQ_OUT0      4            /* Out 0 */
#define RA8P_USB_CFIFOCTR_CTSQ_OUT1      5            /* Out 1 */
#define RA8P_USB_CFIFOCTR_CTSQ_STATUSIN  6            /* Status In */
#define RA8P_USB_CFIFOCTR_CTSQ_STATUSOUT 7            /* Status Out */

/* INTSTS0 - Interrupt Status Register 0 */
#define RA8P_USB_INTSTS0_BRDY            (1 << 0)     /* Buffer Ready */
#define RA8P_USB_INTSTS0_NRDY            (1 << 1)     /* Buffer Not Ready */
#define RA8P_USB_INTSTS0_BEMP            (1 << 2)     /* Buffer Empty */
#define RA8P_USB_INTSTS0_CTRT            (1 << 3)     /* Control Transfer Stage Transition */
#define RA8P_USB_INTSTS0_DVST            (1 << 4)     /* Device State Transition */
#define RA8P_USB_INTSTS0_SOFR            (1 << 5)     /* SOF Reception */
#define RA8P_USB_INTSTS0_RESE            (1 << 6)     /* USB Reset */
#define RA8P_USB_INTSTS0_VBSE            (1 << 7)     /* VBUS Interrupt */
#define RA8P_USB_INTSTS0_RSME            (1 << 8)     /* Resume */
#define RA8P_USB_INTSTS0_SOFD            (1 << 9)     /* SOF Detection */
#define RA8P_USB_INTSTS0_TRNC            (1 << 10)    /* Transaction Completion */
#define RA8P_USB_INTSTS0_CNCT            (1 << 11)    /* Connection Detection */
#define RA8P_USB_INTSTS0_DCT             (1 << 12)    /* Disconnection Detection */
#define RA8P_USB_INTSTS0_ATTCH           (1 << 13)    /* Attachment Detection */
#define RA8P_USB_INTSTS0_DTCH            (1 << 14)    /* Detachment Detection */

/* INTENB0 - Interrupt Enable Register 0 */
#define RA8P_USB_INTENB0_BRDYE           (1 << 0)     /* Buffer Ready Interrupt Enable */
#define RA8P_USB_INTENB0_NRDYE           (1 << 1)     /* Buffer Not Ready Interrupt Enable */
#define RA8P_USB_INTENB0_BEMPE           (1 << 2)     /* Buffer Empty Interrupt Enable */
#define RA8P_USB_INTENB0_CTRTE           (1 << 3)     /* Control Transfer Stage Transition IE */
#define RA8P_USB_INTENB0_DVSE            (1 << 4)     /* Device State Transition Interrupt Enable */
#define RA8P_USB_INTENB0_SOFE            (1 << 5)     /* SOF Reception Interrupt Enable */
#define RA8P_USB_INTENB0_RESE            (1 << 6)     /* USB Reset Interrupt Enable */
#define RA8P_USB_INTENB0_VBSE            (1 << 7)     /* VBUS Interrupt Enable */
#define RA8P_USB_INTENB0_RSME            (1 << 8)     /* Resume Interrupt Enable */
#define RA8P_USB_INTENB0_SOFDE           (1 << 9)     /* SOF Detection Interrupt Enable */
#define RA8P_USB_INTENB0_TRNCE           (1 << 10)    /* Transaction Completion Interrupt Enable */
#define RA8P_USB_INTENB0_CNCTE           (1 << 11)    /* Connection Detection Interrupt Enable */
#define RA8P_USB_INTENB0_DCTE            (1 << 12)    /* Disconnection Detection Interrupt Enable */
#define RA8P_USB_INTENB0_ATTCHIE         (1 << 13)    /* Attachment Detection Interrupt Enable */
#define RA8P_USB_INTENB0_DTCHIE          (1 << 14)    /* Detachment Detection Interrupt Enable */

/* PIPECFG - Pipe Configuration Register */
#define RA8P_USB_PIPECFG_EPNUM_MASK      (0x0F << 0)  /* Endpoint Number */
#define RA8P_USB_PIPECFG_EPNUM_SHIFT     0
#define RA8P_USB_PIPECFG_DIR             (1 << 4)     /* Direction */
#define RA8P_USB_PIPECFG_SHTNAK          (1 << 5)     /* Short Packet Transfer NAK */
#define RA8P_USB_PIPECFG_CNTMD           (1 << 6)     /* Continuous Transfer Mode */
#define RA8P_USB_PIPECFG_DBLB            (1 << 7)     /* Double Buffer Mode */
#define RA8P_USB_PIPECFG_BFRE            (1 << 8)     /* Buffer Ready Interrupt */
#define RA8P_USB_PIPECFG_BFRE_CLR        (1 << 9)     /* Buffer Ready Interrupt Clear */
#define RA8P_USB_PIPECFG_TYP_MASK        (0x03 << 10) /* Transfer Type */
#define RA8P_USB_PIPECFG_TYP_SHIFT       10
#define RA8P_USB_PIPECFG_TYP_CNTL        0            /* Control */
#define RA8P_USB_PIPECFG_TYP_ISO         1            /* Isochronous */
#define RA8P_USB_PIPECFG_TYP_BULK        2            /* Bulk */
#define RA8P_USB_PIPECFG_TYP_INT         3            /* Interrupt */
#define RA8P_USB_PIPECFG_BIGEND          (1 << 12)    /* Big Endian */
#define RA8P_USB_PIPECFG_ISOACC          (1 << 13)    /* Isochronous Transfer Accept */
#define RA8P_USB_PIPECFG_SQSET           (1 << 14)    /* Sequence Set */
#define RA8P_USB_PIPECFG_SQCLR           (1 << 15)    /* Sequence Clear */

/* PIPECTR - Pipe Control Register */
#define RA8P_USB_PIPECTR_PID_MASK        (0x03 << 0)  /* Response PID */
#define RA8P_USB_PIPECTR_PID_SHIFT       0
#define RA8P_USB_PIPECTR_PID_NAK         0            /* NAK */
#define RA8P_USB_PIPECTR_PID_BUF         1            /* BUF */
#define RA8P_USB_PIPECTR_PID_STALL       2            /* STALL */
#define RA8P_USB_PIPECTR_PID_INBUF       3            /* IN BUF */
#define RA8P_USB_PIPECTR_CCPL            (1 << 2)     /* Complete Pipe Control */
#define RA8P_USB_PIPECTR_PBUSY           (1 << 3)     /* Pipe Busy */
#define RA8P_USB_PIPECTR_INBUFM          (1 << 5)     /* IN Buffer Monitor */
#define RA8P_USB_PIPECTR_ACLRM           (1 << 6)     /* ACK/STALL Clear Mode */
#define RA8P_USB_PIPECTR_SQMON           (1 << 7)     /* Sequence Monitor */
#define RA8P_USB_PIPECTR_SQTGL           (1 << 8)     /* Sequence Toggle */
#define RA8P_USB_PIPECTR_CSCLR           (1 << 15)    /* CS Clear */

/* USB Base Addresses */
#define RA8P_USBFS_BASE                  0x40250000
#define RA8P_USBHS_BASE                  0x40351000

/* USBPHY Base Address */
#define RA8P_USBPHY_BASE                 0x40254000

/* Maximum number of pipes */
#define RA8P_USB_MAX_PIPES               10

/* FIFO sizes */
#define RA8P_USB_FIFO_SIZE               512

/* Interrupt numbers */
#define RA8P_IRQ_USBFS                   92
#define RA8P_IRQ_USBHS                   93
#define RA8P_IRQ_USBPHY                  96

/* USB default timeout in milliseconds */
#define RA8P_USB_TIMEOUT_MS              1000

/* USB endpoint types */
#define RA8P_USB_EP_CNTL                 0
#define RA8P_USB_EP_ISO                  1
#define RA8P_USB_EP_BULK                 2
#define RA8P_USB_EP_INT                  3

/* USB direction definitions */
#define RA8P_USB_DIR_OUT                 0
#define RA8P_USB_DIR_IN                  1

/* USB PID (Packet ID) definitions */
#define RA8P_USB_PID_OUT                 0x01
#define RA8P_USB_PID_IN                  0x09
#define RA8P_USB_PID_SOF                 0x05
#define RA8P_USB_PID_SETUP               0x0D
#define RA8P_USB_PID_DATA0               0x03
#define RA8P_USB_PID_DATA1               0x0B
#define RA8P_USB_PID_DATA2               0x07
#define RA8P_USB_PID_MDATA               0x0F
#define RA8P_USB_PID_ACK                 0x02
#define RA8P_USB_PID_NAK                 0x0A
#define RA8P_USB_PID_STALL               0x0E
#define RA8P_USB_PID_NYET                0x06
#define RA8P_USB_PID_PRE                 0x0C
#define RA8P_USB_PID_ERR                 0x0C
#define RA8P_USB_PID_SPLIT               0x08
#define RA8P_USB_PID_PING                0x04

/* USB command definitions */
#define RA8P_USB_CMD0_GO_IDLE_STATE      0
#define RA8P_USB_CMD1_SEND_OP_COND      1
#define RA8P_USB_CMD2_ALL_SEND_CID      2
#define RA8P_USB_CMD3_SEND_RELATIVE_ADDR 3
#define RA8P_USB_CMD7_SELECT_CARD       7
#define RA8P_USB_CMD8_SEND_IF_COND      8
#define RA8P_USB_CMD9_SEND_CSD          9
#define RA8P_USB_CMD12_STOP_TRANSMISSION 12
#define RA8P_USB_CMD16_SET_BLOCKLEN     16
#define RA8P_USB_CMD17_READ_SINGLE_BLOCK 17
#define RA8P_USB_CMD18_READ_MULTIPLE_BLOCK 18
#define RA8P_USB_CMD24_WRITE_SINGLE_BLOCK 24
#define RA8P_USB_CMD25_WRITE_MULTIPLE_BLOCK 25
#define RA8P_USB_CMD55_APP_CMD          55
#define RA8P_USB_ACMD41_SD_SEND_OP_COND 41

/* Maximum USB payload */
#define RA8P_USB_MAX_PAYLOAD             64

/* USB endpoint maximum number */
#define RA8P_USB_MAX_ENDPOINTS           10

/* USB standard frequency values */
#define RA8P_USB_FREQ_48MHZ              48000000
#define RA8P_USB_FREQ_24MHZ              24000000
#define RA8P_USB_FREQ_12MHZ              12000000

/* USB control endpoint (endpoint 0) */
#define RA8P_USB_EP0                     0

/* USB DMA channel select */
#define RA8P_USB_DMA_CH0                 0
#define RA8P_USB_DMA_CH1                 1
#define RA8P_USB_DMA_NONE                0xFF

/* USB interrupt enable flags */
#define RA8P_USB_INT_BRDY                (1 << 0)
#define RA8P_USB_INT_NRDY                (1 << 1)
#define RA8P_USB_INT_BEMP                (1 << 2)
#define RA8P_USB_INT_CTRT                (1 << 3)
#define RA8P_USB_INT_DVST                (1 << 4)
#define RA8P_USB_INT_SOFR                (1 << 5)
#define RA8P_USB_INT_RESE                (1 << 6)
#define RA8P_USB_INT_VBSE                (1 << 7)
#define RA8P_USB_INT_RSME                (1 << 8)
#define RA8P_USB_INT_SOFD                (1 << 9)
#define RA8P_USB_INT_TRNC                (1 << 10)
#define RA8P_USB_INT_CNCT                (1 << 11)
#define RA8P_USB_INT_DCT                 (1 << 12)
#define RA8P_USB_INT_ATTCH               (1 << 13)
#define RA8P_USB_INT_DTCH                (1 << 14)

/* USB transfer modes */
#define RA8P_USB_TRNS_READ               0
#define RA8P_USB_TRNS_WRITE              1
#define RA8P_USB_TRNS_MULTIPLE           2
#define RA8P_USB_TRNS_SINGLE             0

/* USB device states */
#define RA8P_USB_DEVSTATE_DETACHED       0
#define RA8P_USB_DEVSTATE_ATTACHED       1
#define RA8P_USB_DEVSTATE_POWERED        2
#define RA8P_USB_DEVSTATE_DEFAULT        3
#define RA8P_USB_DEVSTATE_ADDRESS        4
#define RA8P_USB_DEVSTATE_CONFIGURED     5
#define RA8P_USB_DEVSTATE_SUSPENDED      6

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* USB configuration structure */
struct ra8p_usb_config_s
{
  uint8_t channel;                    /* USB channel (0=USBFS, 1=USBHS) */
  uint32_t frequency;                 /* USB clock frequency in Hz */
  bool high_speed;                    /* Enable high-speed mode */
  bool pullup_enabled;                /* Enable D+ pull-up resistor */
  bool vbus_detection;                /* Enable VBUS detection */
  bool remote_wakeup;                 /* Enable remote wakeup */
  uint8_t max_endpoints;              /* Maximum number of endpoints */
  bool power_managed;                 /* Enable power management */
  bool enable_dma;                    /* Enable DMA for transfers */
  uint8_t endpoint_types[RA8P_USB_MAX_ENDPOINTS]; /* Endpoint types */
  uint16_t max_packet_sizes[RA8P_USB_MAX_ENDPOINTS]; /* Maximum packet sizes */
};

/* USB endpoint configuration structure */
struct ra8p_usb_epconfig_s
{
  uint8_t ep_num;                    /* Endpoint number (0-9) */
  uint8_t dir;                       /* Direction (0=OUT, 1=IN) */
  uint8_t type;                      /* Endpoint type */
  uint16_t max_packet_size;          /* Maximum packet size */
  bool enabled;                      /* Enable flag */
  bool stall;                        /* Stall flag */
  uint8_t buffer_size;               /* Buffer size */
  bool double_buffer;                /* Double buffer mode */
  bool continuous_transfer;          /* Continuous transfer mode */
};

/* USB transfer structure */
struct ra8p_usb_transfer_s
{
  uint8_t ep_num;                    /* Endpoint number */
  uint8_t *buffer;                   /* Data buffer */
  uint32_t length;                   /* Data length */
  bool direction;                    /* Direction (true=IN, false=OUT) */
  bool blocking;                     /* Blocking vs non-blocking */
  uint32_t timeout;                  /* Timeout in ms */
  bool use_dma;                      /* Use DMA for transfer */
  bool auto_zlp;                     /* Auto Zero-Length Packet */
};

/* USB device information structure */
struct ra8p_usb_deviceinfo_s
{
  uint8_t address;                   /* USB device address */
  uint8_t configuration;             /* Configuration value */
  uint8_t interface;                 /* Interface value */
  bool connected;                    /* Connection status */
  bool suspended;                    /* Suspend status */
  bool remote_wakeup;                /* Remote wakeup enabled */
  uint32_t max_speed;                /* Maximum supported speed */
  bool high_speed_capable;           /* High speed capable */
  uint32_t status;                   /* Device status flags */
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_usb_initialize
 *
 * Description:
 *   Initialize the USB controller based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   config - Pointer to USB configuration structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_initialize(const struct ra8p_usb_config_s *config);

/****************************************************************************
 * Name: ra8p_usb_enable
 *
 * Description:
 *   Enable the USB controller based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_enable(uint8_t channel);

/****************************************************************************
 * Name: ra8p_usb_disable
 *
 * Description:
 *   Disable the USB controller based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_disable(uint8_t channel);

/****************************************************************************
 * Name: ra8p_usb_ep_configure
 *
 * Description:
 *   Configure a USB endpoint based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   epdesc - Pointer to endpoint descriptor
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_ep_configure(uint8_t channel, const struct usb_epdesc_s *epdesc);

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

int ra8p_usb_ep_disable(uint8_t channel, uint8_t ep);

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

int ra8p_usb_ep_stall(uint8_t channel, uint8_t ep);

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

int ra8p_usb_ep_resume(uint8_t channel, uint8_t ep);

/****************************************************************************
 * Name: ra8p_usb_ep_write
 *
 * Description:
 *   Write data to a USB endpoint based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   ep - Endpoint number
 *   buffer - Data buffer to write
 *   length - Number of bytes to write
 *
 * Returned Value:
 *   Number of bytes written on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_ep_write(uint8_t channel, uint8_t ep, const uint8_t *buffer, uint32_t length);

/****************************************************************************
 * Name: ra8p_usb_ep_read
 *
 * Description:
 *   Read data from a USB endpoint based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   ep - Endpoint number
 *   buffer - Buffer to read data into
 *   length - Size of buffer
 *
 * Returned Value:
 *   Number of bytes read on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_ep_read(uint8_t channel, uint8_t ep, uint8_t *buffer, uint32_t length);

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

int ra8p_usb_set_address(uint8_t channel, uint8_t address);

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

int ra8p_usb_connect(uint8_t channel);

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

int ra8p_usb_disconnect(uint8_t channel);

/****************************************************************************
 * Name: ra8p_usb_get_device_info
 *
 * Description:
 *   Get USB device information based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   info - Pointer to device info structure to fill
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_get_device_info(uint8_t channel, struct ra8p_usb_deviceinfo_s *info);

/****************************************************************************
 * Name: ra8p_usb_wakeup
 *
 * Description:
 *   Generate USB remote wakeup based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_wakeup(uint8_t channel);

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

int ra8p_usb_suspend(uint8_t channel);

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

int ra8p_usb_resume(uint8_t channel);

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

bool ra8p_usb_is_connected(uint8_t channel);

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

bool ra8p_usb_is_suspended(uint8_t channel);

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

uint32_t ra8p_usb_get_status(uint8_t channel);

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

int ra8p_usb_clear_status(uint8_t channel, uint32_t flags);

/****************************************************************************
 * Name: ra8p_usb_enable_interrupts
 *
 * Description:
 *   Enable USB interrupts based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_enable_interrupts(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_usb_reset_controller
 *
 * Description:
 *   Reset USB controller based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_reset_controller(uint8_t channel);

/****************************************************************************
 * Name: ra8p_usb_get_error_flags
 *
 * Description:
 *   Get USB controller error flags based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *
 * Returned Value:
 *   Error flags
 *
 ****************************************************************************/

uint32_t ra8p_usb_get_error_flags(uint8_t channel);

/****************************************************************************
 * Name: ra8p_usb_clear_errors
 *
 * Description:
 *   Clear USB controller error flags based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_clear_errors(uint8_t channel);

/****************************************************************************
 * Name: ra8p_usb_enable_dma
 *
 * Description:
 *   Enable/disable DMA for USB transfers based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   enable - true to enable DMA, false to use PIO
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_enable_dma(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_usb_set_test_mode
 *
 * Description:
 *   Set USB test mode based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   test_mode - Test mode value (0-15)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_set_test_mode(uint8_t channel, uint8_t test_mode);

/****************************************************************************
 * Name: ra8p_usb_get_frame_number
 *
 * Description:
 *   Get USB frame number based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *
 * Returned Value:
 *   Current frame number
 *
 ****************************************************************************/

uint32_t ra8p_usb_get_frame_number(uint8_t channel);

/****************************************************************************
 * Name: ra8p_usb_get_packet_count
 *
 * Description:
 *   Get packet count for endpoint based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   ep - Endpoint number
 *
 * Returned Value:
 *   Packet count
 *
 ****************************************************************************/

uint16_t ra8p_usb_get_packet_count(uint8_t channel, uint8_t ep);

/****************************************************************************
 * Name: ra8p_usb_get_data_count
 *
 * Description:
 *   Get data count for endpoint based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   ep - Endpoint number
 *
 * Returned Value:
 *   Data count
 *
 ****************************************************************************/

uint32_t ra8p_usb_get_data_count(uint8_t channel, uint8_t ep);

/****************************************************************************
 * Name: ra8p_usb_set_power_mode
 *
 * Description:
 *   Set USB power mode based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   low_power - true for low power mode, false for normal
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_set_power_mode(uint8_t channel, bool low_power);

/****************************************************************************
 * Name: ra8p_usb_is_powered
 *
 * Description:
 *   Check if USB is powered based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *
 * Returned Value:
 *   true if powered, false otherwise
 *
 ****************************************************************************/

bool ra8p_usb_is_powered(uint8_t channel);

/****************************************************************************
 * Name: ra8p_usb_get_device_state
 *
 * Description:
 *   Get USB device state based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *
 * Returned Value:
 *   Device state
 *
 ****************************************************************************/

uint8_t ra8p_usb_get_device_state(uint8_t channel);

/****************************************************************************
 * Name: ra8p_usb_get_endpoint_status
 *
 * Description:
 *   Get USB endpoint status based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   ep - Endpoint number
 *
 * Returned Value:
 *   Endpoint status flags
 *
 ****************************************************************************/

uint32_t ra8p_usb_get_endpoint_status(uint8_t channel, uint8_t ep);

/****************************************************************************
 * Name: ra8p_usb_set_endpoint_status
 *
 * Description:
 *   Set USB endpoint status based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   ep - Endpoint number
 *   status - Status flags to set
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_set_endpoint_status(uint8_t channel, uint8_t ep, uint32_t status);

/****************************************************************************
 * Name: ra8p_usb_enable_remote_wakeup
 *
 * Description:
 *   Enable/disable remote wakeup capability based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   enable - true to enable remote wakeup, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_enable_remote_wakeup(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_usb_get_max_packet
 *
 * Description:
 *   Get endpoint's maximum packet size based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   ep - Endpoint number
 *
 * Returned Value:
 *   Maximum packet size, or 0 if error
 *
 ****************************************************************************/

uint16_t ra8p_usb_get_max_packet(uint8_t channel, uint8_t ep);

/****************************************************************************
 * Name: ra8p_usb_is_enabled
 *
 * Description:
 *   Check if USB controller is enabled based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *
 * Returned Value:
 *   true if enabled, false otherwise
 *
 ****************************************************************************/

bool ra8p_usb_is_enabled(uint8_t channel);

/****************************************************************************
 * Name: ra8p_usb_is_initialized
 *
 * Description:
 *   Check if USB controller is initialized based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *
 * Returned Value:
 *   true if initialized, false otherwise
 *
 ****************************************************************************/

bool ra8p_usb_is_initialized(uint8_t channel);

/****************************************************************************
 * Name: ra8p_usb_get_response
 *
 * Description:
 *   Get response from last USB command based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   index - Response register index (0-3)
 *
 * Returned Value:
 *   Response register value
 *
 ****************************************************************************/

uint32_t ra8p_usb_get_response(uint8_t channel, uint8_t index);

/****************************************************************************
 * Name: ra8p_usb_set_transfer_mode
 *
 * Description:
 *   Set USB transfer mode based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   mode - Transfer mode flags
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_set_transfer_mode(uint8_t channel, uint32_t mode);

/****************************************************************************
 * Name: ra8p_usb_get_fifo_ptr
 *
 * Description:
 *   Get current FIFO pointer based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   ep - Endpoint number
 *
 * Returned Value:
 *   FIFO pointer address
 *
 ****************************************************************************/

uint32_t ra8p_usb_get_fifo_ptr(uint8_t channel, uint8_t ep);

/****************************************************************************
 * Name: ra8p_usb_set_autoretry
 *
 * Description:
 *   Set USB auto-retry mode based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   enable - true to enable auto-retry, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_set_autoretry(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_usb_set_burst_size
 *
 * Description:
 *   Set USB burst size based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   channel - USB channel (0=USBFS, 1=USBHS)
 *   size - Burst size in packets (0-15)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usb_set_burst_size(uint8_t channel, uint8_t size);

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_USB_H */