/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_sci_b.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_SCI_B_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_SCI_B_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* SCI_B Register offsets ****************************************************/

#define RA8P_SCI_B_CCR0_OFFSET        0x0000  /* Control Register 0 */
#define RA8P_SCI_B_CCR1_OFFSET        0x0004  /* Control Register 1 */
#define RA8P_SCI_B_CCR2_OFFSET        0x0008  /* Control Register 2 */
#define RA8P_SCI_B_CCR3_OFFSET        0x000c  /* Control Register 3 */
#define RA8P_SCI_B_CCR4_OFFSET        0x0010  /* Control Register 4 */
#define RA8P_SCI_B_CCR5_OFFSET        0x0014  /* Control Register 5 */
#define RA8P_SCI_B_CSR_OFFSET         0x0018  /* Status Register */
#define RA8P_SCI_B_CFCLR_OFFSET       0x001c  /* Flag Clear Register */
#define RA8P_SCI_B_FFCLR_OFFSET       0x0020  /* FIFO Flag Clear Register */
#define RA8P_SCI_B_FCR_OFFSET         0x0024  /* FIFO Control Register */
#define RA8P_SCI_B_FDR_OFFSET         0x0028  /* FIFO Data Count Register */
#define RA8P_SCI_B_FTSR_OFFSET        0x002c  /* FIFO Transmit Status Register */
#define RA8P_SCI_B_FRSR_OFFSET        0x0030  /* FIFO Receive Status Register */
#define RA8P_SCI_B_CESR_OFFSET        0x0034  /* Communication Enable Status Register */
#define RA8P_SCI_B_TDR_OFFSET         0x0038  /* Transmit Data Register */
#define RA8P_SCI_B_RDR_OFFSET         0x003c  /* Receive Data Register */
#define RA8P_SCI_B_SEMR_OFFSET        0x0040  /* Serial Extended Mode Register */
#define RA8P_SCI_B_SCMR_OFFSET        0x0044  /* Smart Card Mode Register */
#define RA8P_SCI_B_BRR_OFFSET         0x0048  /* Bit Rate Register */
#define RA8P_SCI_B_MDDR_OFFSET        0x004c  /* Modulation Duty Register */
#define RA8P_SCI_B_SPTR_OFFSET        0x0050  /* Serial Port Register */
#define RA8P_SCI_B_DCCR_OFFSET        0x0054  /* Data Control Register */
#define RA8P_SCI_B_SPCR_OFFSET        0x0058  /* SPI Control Register */
#define RA8P_SCI_B_SPCR2_OFFSET       0x005c  /* SPI Control Register 2 */
#define RA8P_SCI_B_SPCMD_OFFSET(n)    (0x0060 + ((n) * 4))  /* SPI Command Register */
#define RA8P_SCI_B_SPSCR_OFFSET       0x0080  /* SPI Status Register */
#define RA8P_SCI_B_SPSSR_OFFSET       0x0084  /* SPI Status Select Register */
#define RA8P_SCI_B_SPTSR_OFFSET       0x0088  /* SPI Transmit Status Register */
#define RA8P_SCI_B_SPRSR_OFFSET       0x008c  /* SPI Receive Status Register */

/* CCR0 - Control Register 0 ************************************************/

#define RA8P_SCI_B_CCR0_TIE           (1 << 0)   /* Transmit Interrupt Enable */
#define RA8P_SCI_B_CCR0_RIE           (1 << 1)   /* Receive Interrupt Enable */
#define RA8P_SCI_B_CCR0_TE            (1 << 2)   /* Transmit Enable */
#define RA8P_SCI_B_CCR0_RE            (1 << 3)   /* Receive Enable */
#define RA8P_SCI_B_CCR0_TEIE          (1 << 4)   /* Transmit End Interrupt Enable */
#define RA8P_SCI_B_CCR0_MPIE          (1 << 5)   /* Multi-Processor Interrupt Enable */
#define RA8P_SCI_B_CCR0_REIE          (1 << 6)   /* Receive Error Interrupt Enable */

/* CCR1 - Control Register 1 ************************************************/

#define RA8P_SCI_B_CCR1_TEIE          (1 << 0)   /* Transmit End Interrupt Enable */
#define RA8P_SCI_B_CCR1_TIEE          (1 << 1)   /* Transmit Interrupt Enable Extended */
#define RA8P_SCI_B_CCR1_RIEE          (1 << 2)   /* Receive Interrupt Enable Extended */

/* CCR2 - Control Register 2 ************************************************/

#define RA8P_SCI_B_CCR2_FM           (1 << 0)    /* FIFO Mode */
#define RA8P_SCI_B_CCR2_TFRST        (1 << 1)    /* Transmit FIFO Reset */
#define RA8P_SCI_B_CCR2_RFRST        (1 << 2)    /* Receive FIFO Reset */
#define RA8P_SCI_B_CCR2_RDFE         (1 << 3)    /* Receive Data Full Enable */
#define RA8P_SCI_B_CCR2_RDREQE       (1 << 4)    /* Receive Data Request Enable */
#define RA8P_SCI_B_CCR2_TDFE         (1 << 5)    /* Transmit Data Empty Enable */
#define RA8P_SCI_B_CCR2_TDREQE       (1 << 6)    /* Transmit Data Request Enable */

/* CCR3 - Control Register 3 ************************************************/

#define RA8P_SCI_B_CCR3_BPEN         (1 << 0)    /* Baud Rate Generator Output Enable */
#define RA8P_SCI_B_CCR3_ADTRG        (1 << 1)    /* A/D Trigger Output Enable */
#define RA8P_SCI_B_CCR3_CKE_MASK     (0x03 << 2) /* Clock Enable Mask */
#define RA8P_SCI_B_CCR3_CKE_SHIFT    2
#define RA8P_SCI_B_CCR3_CKE_INT      (0 << 2)    /* Internal Clock */
#define RA8P_SCI_B_CCR3_CKE_INT_OUT  (1 << 2)    /* Internal Clock with Output */
#define RA8P_SCI_B_CCR3_CKE_EXT      (2 << 2)    /* External Clock */

/* CCR4 - Control Register 4 ************************************************/

#define RA8P_SCI_B_CCR4_CKS_MASK     (0x03 << 0) /* Clock Select Mask */
#define RA8P_SCI_B_CCR4_CKS_SHIFT    0
#define RA8P_SCI_B_CCR4_CKS_PCLK     (0 << 0)    /* PCLK */
#define RA8P_SCI_B_CCR4_CKS_PCLK_4   (1 << 0)    /* PCLK/4 */
#define RA8P_SCI_B_CCR4_CKS_PCLK_16  (2 << 0)    /* PCLK/16 */
#define RA8P_SCI_B_CCR4_CKS_PCLK_64  (3 << 0)    /* PCLK/64 */
#define RA8P_SCI_B_CCR4_BGDM         (1 << 2)    /* Baud Rate Generator Double-Speed Mode */
#define RA8P_SCI_B_CCR4_ABCS         (1 << 3)    /* Asynchronous Mode Base Clock Select */
#define RA8P_SCI_B_CCR4_ABCS_16      (0 << 3)    /* 16 base clock cycles */
#define RA8P_SCI_B_CCR4_ABCS_8       (1 << 3)    /* 8 base clock cycles */
#define RA8P_SCI_B_CCR4_BRME         (1 << 4)    /* Bit Rate Modulation Enable */
#define RA8P_SCI_B_CCR4_MFF          (1 << 5)    /* MDD Register Function Select */

/* CSR - Status Register ****************************************************/

#define RA8P_SCI_B_CSR_TDRE          (1 << 0)    /* Transmit Data Register Empty */
#define RA8P_SCI_B_CSR_RDRF          (1 << 1)    /* Receive Data Register Full */
#define RA8P_SCI_B_CSR_ORER          (1 << 2)    /* Overrun Error */
#define RA8P_SCI_B_CSR_FER           (1 << 3)    /* Framing Error */
#define RA8P_SCI_B_CSR_PER           (1 << 4)    /* Parity Error */
#define RA8P_SCI_B_CSR_TEND          (1 << 5)    /* Transmit End */
#define RA8P_SCI_B_CSR_MPB           (1 << 6)    /* Multi-Processor Bit */
#define RA8P_SCI_B_CSR_MPBID_MASK    (0x0f << 8) /* Multi-Processor Bit ID Mask */
#define RA8P_SCI_B_CSR_MPBID_SHIFT   8
#define RA8P_SCI_B_CSR_ER           (1 << 12)    /* Error Flag */
#define RA8P_SCI_B_CSR_DR           (1 << 13)    /* Data Ready */
#define RA8P_SCI_B_CSR_DRC          (1 << 14)    /* Data Ready Clear */
#define RA8P_SCI_B_CSR_TDFE         (1 << 15)    /* Transmit FIFO Data Empty */
#define RA8P_SCI_B_CSR_TDREQ        (1 << 16)    /* Transmit Data Request */
#define RA8P_SCI_B_CSR_RDFE         (1 << 17)    /* Receive FIFO Data Full */
#define RA8P_SCI_B_CSR_RDREQ        (1 << 18)    /* Receive Data Request */

/* CFCLR - Flag Clear Register **********************************************/

#define RA8P_SCI_B_CFCLR_TDRFC       (1 << 0)    /* Transmit Data Register Full Clear */
#define RA8P_SCI_B_CFCLR_RDRFC       (1 << 1)    /* Receive Data Register Full Clear */
#define RA8P_SCI_B_CFCLR_ORERC       (1 << 2)    /* Overrun Error Clear */
#define RA8P_SCI_B_CFCLR_FERC        (1 << 3)    /* Framing Error Clear */
#define RA8P_SCI_B_CFCLR_PERC        (1 << 4)    /* Parity Error Clear */
#define RA8P_SCI_B_CFCLR_MPBFC       (1 << 6)    /* Multi-Processor Bit Flag Clear */
#define RA8P_SCI_B_CFCLR_DRC         (1 << 14)   /* Data Ready Clear */

/* FFCLR - FIFO Flag Clear Register ****************************************/

#define RA8P_SCI_B_FFCLR_DRC         (1 << 0)    /* Data Ready Clear */
#define RA8P_SCI_B_FFCLR_TDRFC       (1 << 1)    /* Transmit Data Register Full Clear */
#define RA8P_SCI_B_FFCLR_RDRFC       (1 << 2)    /* Receive Data Register Full Clear */

/* FCR - FIFO Control Register **********************************************/

#define RA8P_SCI_B_FCR_RFRST         (1 << 0)    /* Receive FIFO Reset */
#define RA8P_SCI_B_FCR_TFRST         (1 << 1)    /* Transmit FIFO Reset */
#define RA8P_SCI_B_FCR_DREQ          (1 << 2)    /* Data Request Enable */
#define RA8P_SCI_B_FCR_TFRST2        (1 << 3)    /* Transmit FIFO Reset 2 */
#define RA8P_SCI_B_FCR_RFRST2        (1 << 4)    /* Receive FIFO Reset 2 */
#define RA8P_SCI_B_FCR_RTRG_MASK     (0x07 << 5) /* Receive Trigger Mask */
#define RA8P_SCI_B_FCR_RTRG_SHIFT    5
#define RA8P_SCI_B_FCR_RTRG_1        (0 << 5)    /* 1 data */
#define RA8P_SCI_B_FCR_RTRG_4        (1 << 5)    /* 4 data */
#define RA8P_SCI_B_FCR_RTRG_8        (2 << 5)    /* 8 data */
#define RA8P_SCI_B_FCR_RTRG_14       (3 << 5)    /* 14 data */
#define RA8P_SCI_B_FCR_TTRG_MASK     (0x07 << 8) /* Transmit Trigger Mask */
#define RA8P_SCI_B_FCR_TTRG_SHIFT    8
#define RA8P_SCI_B_FCR_TTRG_0        (0 << 8)    /* 0 data */
#define RA8P_SCI_B_FCR_TTRG_4        (1 << 8)    /* 4 data */
#define RA8P_SCI_B_FCR_TTRG_8        (2 << 8)    /* 8 data */
#define RA8P_SCI_B_FCR_TTRG_14       (3 << 8)    /* 14 data */

/* FDR - FIFO Data Count Register *******************************************/

#define RA8P_SCI_B_FDR_R_MASK        (0x0f << 0) /* Receive Count Mask */
#define RA8P_SCI_B_FDR_R_SHIFT       0
#define RA8P_SCI_B_FDR_T_MASK        (0x0f << 8) /* Transmit Count Mask */
#define RA8P_SCI_B_FDR_T_SHIFT       8

/* FTSR - FIFO Transmit Status Register *************************************/

#define RA8P_SCI_B_FTSR_TDREQ        (1 << 0)    /* Transmit Data Request */
#define RA8P_SCI_B_FTSR_TDFE         (1 << 1)    /* Transmit FIFO Data Empty */
#define RA8P_SCI_B_FTSR_TDRF         (1 << 2)    /* Transmit Data Register Full */

/* FRSR - FIFO Receive Status Register **************************************/

#define RA8P_SCI_B_FRSR_RDREQ        (1 << 0)    /* Receive Data Request */
#define RA8P_SCI_B_FRSR_RDFE         (1 << 1)    /* Receive FIFO Data Full */
#define RA8P_SCI_B_FRSR_RDRF         (1 << 2)    /* Receive Data Register Full */
#define RA8P_SCI_B_FRSR_R_MASK       (0x0f << 8) /* Receive Count Mask */
#define RA8P_SCI_B_FRSR_R_SHIFT      8

/* CESR - Communication Enable Status Register ******************************/

#define RA8P_SCI_B_CESR_TIST        (1 << 0)     /* Transmit In Progress Status */
#define RA8P_SCI_B_CESR_RIST        (1 << 1)     /* Receive In Progress Status */

/* SEMR - Serial Extended Mode Register *************************************/

#define RA8P_SCI_B_SEMR_BRME        (1 << 0)     /* Bit Rate Modulation Enable */
#define RA8P_SCI_B_SEMR_ABCS        (1 << 1)     /* Asynchronous Mode Base Clock Select */
#define RA8P_SCI_B_SEMR_NFEN        (1 << 2)     /* Noise Filter Enable */
#define RA8P_SCI_B_SEMR_BGDM        (1 << 3)     /* Baud Rate Generator Double-Speed Mode */
#define RA8P_SCI_B_SEMR_RXDESEL     (1 << 4)     /* Receive Data Sampling Select */
#define RA8P_SCI_B_SEMR_ACS         (1 << 5)     /* Asynchronous Clock Select */
#define RA8P_SCI_B_SEMR_MDDRS       (1 << 6)     /* MDDR Select */
#define RA8P_SCI_B_SEMR_MDDRS_BRR   (0 << 6)     /* Use BRR */
#define RA8P_SCI_B_SEMR_MDDRS_MDDR  (1 << 6)     /* Use MDDR */
#define RA8P_SCI_B_SEMR_SDIR        (1 << 7)     /* Serial Data Transfer Direction */
#define RA8P_SCI_B_SEMR_SDIR_LSB    (0 << 7)     /* LSB first */
#define RA8P_SCI_B_SEMR_SDIR_MSB    (1 << 7)     /* MSB first */
#define RA8P_SCI_B_SEMR_SZ          (1 << 8)     /* Smart Card Mode Select */

/* SCMR - Smart Card Mode Register ******************************************/

#define RA8P_SCI_B_SCMR_SINV        (1 << 0)     /* Smart Card Invert Data */
#define RA8P_SCI_B_SCMR_SINV_NORMAL (0 << 0)     /* Normal data */
#define RA8P_SCI_B_SCMR_SINV_INVERT (1 << 0)     /* Inverted data */
#define RA8P_SCI_B_SCMR_SDIR        (1 << 1)     /* Smart Card Data Transfer Direction */
#define RA8P_SCI_B_SCMR_SDIR_LSB    (0 << 1)     /* LSB first */
#define RA8P_SCI_B_SCMR_SDIR_MSB    (1 << 1)     /* MSB first */
#define RA8P_SCI_B_SCMR_BCP2        (1 << 2)     /* Base Clock Pulse 2 */
#define RA8P_SCI_B_SCMR_CHR         (1 << 3)     /* Character Length */
#define RA8P_SCI_B_SCMR_CHR_8       (0 << 3)     /* 8-bit */
#define RA8P_SCI_B_SCMR_CHR_7       (1 << 3)     /* 7-bit */
#define RA8P_SCI_B_SCMR_PE          (1 << 4)     /* Parity Enable */
#define RA8P_SCI_B_SCMR_PM          (1 << 5)     /* Parity Mode */
#define RA8P_SCI_B_SCMR_PM_EVEN     (0 << 5)     /* Even parity */
#define RA8P_SCI_B_SCMR_PM_ODD      (1 << 5)     /* Odd parity */
#define RA8P_SCI_B_SCMR_STOP        (1 << 6)     /* Stop Bit Length */
#define RA8P_SCI_B_SCMR_STOP_1      (0 << 6)     /* 1 stop bit */
#define RA8P_SCI_B_SCMR_STOP_2      (1 << 6)     /* 2 stop bits */
#define RA8P_SCI_B_SCMR_BLK         (1 << 7)     /* Block Transfer Mode */
#define RA8P_SCI_B_SCMR_BCP_MASK    (0x07 << 8)  /* Base Clock Pulse Mask */
#define RA8P_SCI_B_SCMR_BCP_SHIFT   8

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* SCI_B Register Map *******************************************************/

struct ra8p_sci_b_s
{
  volatile uint32_t ccr0;      /* Control Register 0 */
  volatile uint32_t ccr1;      /* Control Register 1 */
  volatile uint32_t ccr2;      /* Control Register 2 */
  volatile uint32_t ccr3;      /* Control Register 3 */
  volatile uint32_t ccr4;      /* Control Register 4 */
  volatile uint32_t ccr5;      /* Control Register 5 */
  volatile uint32_t csr;       /* Status Register */
  volatile uint32_t cfclr;     /* Flag Clear Register */
  volatile uint32_t ffclr;     /* FIFO Flag Clear Register */
  volatile uint32_t fcr;       /* FIFO Control Register */
  volatile uint32_t fdr;       /* FIFO Data Count Register */
  volatile uint32_t ftsr;      /* FIFO Transmit Status Register */
  volatile uint32_t frsr;      /* FIFO Receive Status Register */
  volatile uint32_t cesr;      /* Communication Enable Status Register */
  volatile uint32_t tdr;       /* Transmit Data Register */
  volatile uint32_t rdr;       /* Receive Data Register */
  volatile uint32_t semr;      /* Serial Extended Mode Register */
  volatile uint32_t scmr;      /* Smart Card Mode Register */
  volatile uint32_t brr;       /* Bit Rate Register */
  volatile uint32_t mddr;      /* Modulation Duty Register */
  volatile uint32_t sptr;      /* Serial Port Register */
  volatile uint32_t dccr;      /* Data Control Register */
  volatile uint32_t spcr;      /* SPI Control Register */
  volatile uint32_t spcr2;     /* SPI Control Register 2 */
  volatile uint32_t spcmd[8];  /* SPI Command Register 0-7 */
  volatile uint32_t spscr;     /* SPI Status Register */
  volatile uint32_t spssr;     /* SPI Status Select Register */
  volatile uint32_t sptsr;     /* SPI Transmit Status Register */
  volatile uint32_t sprsr;     /* SPI Receive Status Register */
};

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_SCI_B_H */