/****************************************************************************
 * arch/arm/src/stm32u5/hardware/stm32_dsi.h
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

#ifndef __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32_DSI_H
#define __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32_DSI_H

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Register Offsets *********************************************************/

#define STM32_DSI_VR_OFFSET           0x0000  /* DSI Host Version Register */
#define STM32_DSI_CR_OFFSET           0x0004  /* DSI Host Control Register */
#define STM32_DSI_CCR_OFFSET          0x0008  /* DSI HOST Clock Control Register */
#define STM32_DSI_LVCIDR_OFFSET       0x000C  /* DSI Host Loos VCID Register */
#define STM32_DSI_LCOLCR_OFFSET       0x0010  /* DSI Host Color Coding Register */
#define STM32_DSI_VMCR_OFFSET         0x0014  /* DSI Host Video Mode Configuration Register */
#define STM32_DSI_VPCR_OFFSET         0x0018  /* DSI Host Video Packet Configuration Register */
#define STM32_DSI_VCCR_OFFSET         0x001C  /* DSI Host Video Chunks Configuration Register */
#define STM32_DSI_VNPCR_OFFSET        0x0020  /* DSI Host Video Null Packet Configuration Register */
#define STM32_DSI_VHSACR_OFFSET       0x0024  /* DSI Host Video HSA Configuration Register */
#define STM32_DSI_VHBPCR_OFFSET       0x0028  /* DSI Host Video HBP Configuration Register */
#define STM32_DSI_VLCR_OFFSET         0x002C  /* DSI Host Video Line Configuration Register */
#define STM32_DSI_VVSACR_OFFSET       0x0030  /* DSI Host Video VSA Configuration Register */
#define STM32_DSI_VVBPCR_OFFSET       0x0034  /* DSI Host Video VBP Configuration Register */
#define STM32_DSI_VVFPCR_OFFSET       0x0038  /* DSI Host Video VFP Configuration Register */
#define STM32_DSI_VVACR_OFFSET        0x003C  /* DSI Host Video VA Configuration Register */
#define STM32_DSI_LPCR_OFFSET         0x0040  /* DSI Host LTDC PCR Configuration Register */
#define STM32_DSI_LPMCR_OFFSET        0x0044  /* DSI Host LPM Configuration Register */
#define STM32_DSI_VMCCR_OFFSET        0x004C  /* DSI Host Video Mode Current Configuration Register */
#define STM32_DSI_VMTCCR_OFFSET       0x0050  /* DSI Host Video Mode Threshold Configuration Register */
#define STM32_DSI_VSMCR_OFFSET        0x0054  /* DSI Host Video Stream Mode Configuration Register */
#define STM32_DSI_VSSCCR_OFFSET       0x0058  /* DSI Host Video Stream Stop Configuration Register */
#define STM32_DSI_VBCCR_OFFSET        0x005C  /* DSI Host Video BLL Configuration Register */
#define STM32_DSI_DSMCCR_OFFSET       0x0060  /* DSI Host DSM Configuration Register */
#define STM32_DSI_TCCR0_OFFSET        0x0064  /* DSI Host TO Counte Configuration Register 0 */
#define STM32_DSI_TCCR1_OFFSET        0x0068  /* DSI Host TO Counter Configuration Register 1 */
#define STM32_DSI_TCCR2_OFFSET        0x006C  /* DSI Host TO Counter Configuration Register 2 */
#define STM32_DSI_TCCR3_OFFSET        0x0070  /* DSI Host TO Counter Configuration Register 3 */
#define STM32_DSI_TCCR4_OFFSET        0x0074  /* DSI Host TO Counter Configuration Register 4 */
#define STM32_DSI_TCCR5_OFFSET        0x0078  /* DSI Host TO Counter Configuration Register 5 */
#define STM32_DSI_WCFGR_OFFSET        0x0080  /* DSI Wrapper Configuration Register */
#define STM32_DSI_WCR_OFFSET          0x0084  /* DSI Wrapper Control Register */
#define STM32_DSI_WIER_OFFSET         0x0088  /* DSI Wrapper Interrupt Enable Register */
#define STM32_DSI_WISR_OFFSET         0x008C  /* DSI Wrapper Interrupt and Status Register */
#define STM32_DSI_WIFCR_OFFSET        0x0090  /* DSI Wrapper Interrupt Flag Clear Register */
#define STM32_DSI_WPCR0_OFFSET        0x0094  /* DSI Wrapper PHY Configuration Register 0 */
#define STM32_DSI_WPCR1_OFFSET        0x0098  /* DSI Wrapper PHY Configuration Register 1 */
#define STM32_DSI_WPCR2_OFFSET        0x009C  /* DSI Wrapper PHY Configuration Register 2 */
#define STM32_DSI_WPCR3_OFFSET        0x00A0  /* DSI Wrapper PHY Configuration Register 3 */
#define STM32_DSI_WPCR4_OFFSET        0x00A4  /* DSI Wrapper PHY Configuration Register 4 */
#define STM32_DSI_WRPCR_OFFSET        0x00A8  /* DSI Wrapper Regulator and PLL Control Register */
#define STM32_DSI_WRPCR_OFFSET        0x00AC  /* DSI Wrapper Regulator and Power Control Register */
#define STM32_DSI_PCTLR_OFFSET        0x0100  /* DSI PHY Control Register */
#define STM32_DSI_PCONFR_OFFSET       0x0104  /* DSI PHY Configuration Register */
#define STM32_DSI_PUCR_OFFSET         0x0108  /* DSI PHY ULPS Control Register */
#define STM32_DSI_PTTCR_OFFSET        0x010C  /* DSI PHY TX Triggers Control Register */
#define STM32_DSI_PSR_OFFSET          0x0110  /* DSI PHY Status Register */
#define STM32_DSI_ISR0_OFFSET         0x0114  /* DSI Interrupt & Status Register 0 */
#define STM32_DSI_ISR1_OFFSET         0x0118  /* DSI Interrupt & Status Register 1 */
#define STM32_DSI_IER0_OFFSET         0x011C  /* DSI Interrupt Enable Register 0 */
#define STM32_DSI_IER1_OFFSET         0x0120  /* DSI Interrupt Enable Register 1 */
#define STM32_DSI_FIR0_OFFSET         0x0124  /* DSI Force Interrupt Register 0 */
#define STM32_DSI_FIR1_OFFSET         0x0128  /* DSI Force Interrupt Register 1 */
#define STM32_DSI_VSCR_OFFSET         0x012C  /* DSI Video Shadow Control Register */
#define STM32_DSI_LCVR_OFFSET         0x0130  /* DSI LTDC Current Video Register */
#define STM32_DSI_LCCR_OFFSET         0x0134  /* DSI LTDC Current Color Coding Register */
#define STM32_DSI_CMCR_OFFSET         0x0138  /* DSI HOST Command Mode Configuration Register */
#define STM32_DSI_GHCR_OFFSET         0x013C  /* DSI Generic Header Configuration Register */
#define STM32_DSI_GPDR_OFFSET         0x0140  /* DSI Generic Payload Data Register */
#define STM32_DSI_GPSR_OFFSET         0x0144  /* DSI Generic Packet Status Register */
#define STM32_DSI_TDR_OFFSET          0x0148  /* DSI Tearing Effect Display Register */
#define STM32_DSI_CLCR_OFFSET         0x014C  /* DSI Clock Lane Configuration Register */
#define STM32_DSI_CLTCR_OFFSET        0x0150  /* DSI Clock Lane Timer Configuration Register */
#define STM32_DSI_DLTCR_OFFSET        0x0154  /* DSI Data Lane Timer Configuration Register */
#define STM32_DSI_PCTLR_OFFSET        0x0158  /* DSI PHY Control Register */
#define STM32_DSI_PCONFR_OFFSET       0x015C  /* DSI PHY Configuration Register */
#define STM32_DSI_PUCR_OFFSET         0x0160  /* DSI PHY ULPS Control Register */
#define STM32_DSI_PTTCR_OFFSET        0x0164  /* DSI PHY TX Triggers Control Register */
#define STM32_DSI_PSR_OFFSET          0x0168  /* DSI PHY Status Register */
#define STM32_DSI_ISR0_OFFSET         0x016C  /* DSI Interrupt & Status Register 0 */
#define STM32_DSI_ISR1_OFFSET         0x0170  /* DSI Interrupt & Status Register 1 */
#define STM32_DSI_IER0_OFFSET         0x0174  /* DSI Interrupt Enable Register 0 */
#define STM32_DSI_IER1_OFFSET         0x0178  /* DSI Interrupt Enable Register 1 */
#define STM32_DSI_FIR0_OFFSET         0x017C  /* DSI Force Interrupt Register 0 */
#define STM32_DSI_FIR1_OFFSET         0x0180  /* DSI Force Interrupt Register 1 */

/* Register Addresses *******************************************************/

#define STM32_DSI_VR                   (STM32_DSI_BASE + STM32_DSI_VR_OFFSET)
#define STM32_DSI_CR                   (STM32_DSI_BASE + STM32_DSI_CR_OFFSET)
#define STM32_DSI_CCR                  (STM32_DSI_BASE + STM32_DSI_CCR_OFFSET)
#define STM32_DSI_LVCIDR               (STM32_DSI_BASE + STM32_DSI_LVCIDR_OFFSET)
#define STM32_DSI_LCOLCR               (STM32_DSI_BASE + STM32_DSI_LCOLCR_OFFSET)
#define STM32_DSI_VMCR                 (STM32_DSI_BASE + STM32_DSI_VMCR_OFFSET)
#define STM32_DSI_VPCR                 (STM32_DSI_BASE + STM32_DSI_VPCR_OFFSET)
#define STM32_DSI_VCCR                 (STM32_DSI_BASE + STM32_DSI_VCCR_OFFSET)
#define STM32_DSI_VNPCR                (STM32_DSI_BASE + STM32_DSI_VNPCR_OFFSET)
#define STM32_DSI_VHSACR               (STM32_DSI_BASE + STM32_DSI_VHSACR_OFFSET)
#define STM32_DSI_VHBPCR               (STM32_DSI_BASE + STM32_DSI_VHBPCR_OFFSET)
#define STM32_DSI_VLCR                 (STM32_DSI_BASE + STM32_DSI_VLCR_OFFSET)
#define STM32_DSI_VVSACR               (STM32_DSI_BASE + STM32_DSI_VVSACR_OFFSET)
#define STM32_DSI_VVBPCR               (STM32_DSI_BASE + STM32_DSI_VVBPCR_OFFSET)
#define STM32_DSI_VVFPCR               (STM32_DSI_BASE + STM32_DSI_VVFPCR_OFFSET)
#define STM32_DSI_VVACR                (STM32_DSI_BASE + STM32_DSI_VVACR_OFFSET)
#define STM32_DSI_LPCR                 (STM32_DSI_BASE + STM32_DSI_LPCR_OFFSET)
#define STM32_DSI_LPMCR                (STM32_DSI_BASE + STM32_DSI_LPMCR_OFFSET)
#define STM32_DSI_VMCCR                (STM32_DSI_BASE + STM32_DSI_VMCCR_OFFSET)
#define STM32_DSI_VMTCCR               (STM32_DSI_BASE + STM32_DSI_VMTCCR_OFFSET)
#define STM32_DSI_VSMCR                (STM32_DSI_BASE + STM32_DSI_VSMCR_OFFSET)
#define STM32_DSI_VSSCCR               (STM32_DSI_BASE + STM32_DSI_VSSCCR_OFFSET)
#define STM32_DSI_VBCCR                (STM32_DSI_BASE + STM32_DSI_VBCCR_OFFSET)
#define STM32_DSI_DSMCCR               (STM32_DSI_BASE + STM32_DSI_DSMCCR_OFFSET)
#define STM32_DSI_TCCR0                (STM32_DSI_BASE + STM32_DSI_TCCR0_OFFSET)
#define STM32_DSI_TCCR1                (STM32_DSI_BASE + STM32_DSI_TCCR1_OFFSET)
#define STM32_DSI_TCCR2                (STM32_DSI_BASE + STM32_DSI_TCCR2_OFFSET)
#define STM32_DSI_TCCR3                (STM32_DSI_BASE + STM32_DSI_TCCR3_OFFSET)
#define STM32_DSI_TCCR4                (STM32_DSI_BASE + STM32_DSI_TCCR4_OFFSET)
#define STM32_DSI_TCCR5                (STM32_DSI_BASE + STM32_DSI_TCCR5_OFFSET)
#define STM32_DSI_WCFGR                (STM32_DSI_BASE + STM32_DSI_WCFGR_OFFSET)
#define STM32_DSI_WCR                  (STM32_DSI_BASE + STM32_DSI_WCR_OFFSET)
#define STM32_DSI_WIER                 (STM32_DSI_BASE + STM32_DSI_WIER_OFFSET)
#define STM32_DSI_WISR                 (STM32_DSI_BASE + STM32_DSI_WISR_OFFSET)
#define STM32_DSI_WIFCR                (STM32_DSI_BASE + STM32_DSI_WIFCR_OFFSET)
#define STM32_DSI_WPCR0                (STM32_DSI_BASE + STM32_DSI_WPCR0_OFFSET)
#define STM32_DSI_WPCR1                (STM32_DSI_BASE + STM32_DSI_WPCR1_OFFSET)
#define STM32_DSI_WPCR2                (STM32_DSI_BASE + STM32_DSI_WPCR2_OFFSET)
#define STM32_DSI_WPCR3                (STM32_DSI_BASE + STM32_DSI_WPCR3_OFFSET)
#define STM32_DSI_WPCR4                (STM32_DSI_BASE + STM32_DSI_WPCR4_OFFSET)
#define STM32_DSI_WRPCR                (STM32_DSI_BASE + STM32_DSI_WRPCR_OFFSET)
#define STM32_DSI_WRPCR                (STM32_DSI_BASE + STM32_DSI_WRPCR_OFFSET)

/* Register Bitfield Definitions ********************************************/

/* DSI Host Control Register */
#define DSI_CR_EN                      (1 << 0)   /* Bit 0: DSI Host ON/OFF */
#define DSI_CR_SWRST                   (1 << 1)   /* Bit 1: DSI Host Soft Reset */

/* DSI Host Clock Control Register */
#define DSI_CCR_TXECKDIV_SHIFT         (0)        /* Bits 0-7: TX Escape Clock Division */
#define DSI_CCR_TXECKDIV_MASK          (0xFF << DSI_CCR_TXECKDIV_SHIFT)
#define DSI_CCR_TOCKDIV_SHIFT          (8)        /* Bits 8-15: Timeout Clock Division */
#define DSI_CCR_TOCKDIV_MASK           (0xFF << DSI_CCR_TOCKDIV_SHIFT)

/* DSI Host Video Mode Configuration Register */
#define DSI_VMCR_VMT_SHIFT             (0)        /* Bits 0-1: Video Mode Type */
#define DSI_VMCR_VMT_MASK              (0x3 << DSI_VMCR_VMT_SHIFT)
#  define DSI_VMCR_VMT_NON_BURST_PULSE (0 << DSI_VMCR_VMT_SHIFT)  /* Non-burst with sync pulses */
#  define DSI_VMCR_VMT_NON_BURST_EVENT (1 << DSI_VMCR_VMT_SHIFT)  /* Non-burst with sync events */
#  define DSI_VMCR_VMT_BURST           (2 << DSI_VMCR_VMT_SHIFT)  /* Burst mode */
#define DSI_VMCR_LPVSAE                (1 << 2)   /* Bit 2: Low-Power Vertical Sync Active Enable */
#define DSI_VMCR_LPVBPE                (1 << 3)   /* Bit 3: Low-Power Vertical Back-Porch Enable */
#define DSI_VMCR_LPVFPE                (1 << 4)   /* Bit 4: Low-Power Vertical Front-Porch Enable */
#define DSI_VMCR_LPVAAE                (1 << 5)   /* Bit 5: Low-Power Vertical Active Area Enable */
#define DSI_VMCR_LPHBPE                (1 << 6)   /* Bit 6: Low-Power Horizontal Back-Porch Enable */
#define DSI_VMCR_LPCMDSE               (1 << 7)   /* Bit 7: Low-Power Command Enable */
#define DSI_VMCR_LPCE                  (1 << 8)   /* Bit 8: Low Power Continuous Clock Enable */

/* DSI Wrapper Configuration Register */
#define DSI_WCFGR_DSIMODE                (1 << 0)   /* Bit 0: DSI Mode */
#define DSI_WCFGR_COLMUX_SHIFT           (1)        /* Bits 1-3: Color Multiplexing */
#define DSI_WCFGR_COLMUX_MASK            (0x7 << DSI_WCFGR_COLMUX_SHIFT)
#define DSI_WCFGR_TESRC_SHIFT            (4)        /* Bits 4-5: Tearing Effect Source */
#define DSI_WCFGR_TESRC_MASK             (0x3 << DSI_WCFGR_TESRC_SHIFT)
#define DSI_WCFGR_TEPOL                 (1 << 6)   /* Bit 6: Tearing Effect Polarity */
#define DSI_WCFGR_AR                    (1 << 7)   /* Bit 7: Automatic Refresh */
#define DSI_WCFGR_VSPOL                 (1 << 8)   /* Bit 8: VSYNC Polarity */

/* DSI Wrapper Control Register */
#define DSI_WCR_DSIEN                   (1 << 0)   /* Bit 0: DSI Enable */
#define DSI_WCR_LTDCEN                  (1 << 1)   /* Bit 1: LTDC Enable */
#define DSI_WCR_SHTDN                   (1 << 2)   /* Bit 2: Shutdown */
#define DSI_WCR_RST                     (1 << 3)   /* Bit 3: Reset */

/* DSI Wrapper Interrupt Enable Register */
#define DSI_WIER_TEIE                   (1 << 0)   /* Bit 0: Tearing Effect Interrupt Enable */
#define DSI_WIER_ERIE                   (1 << 1)   /* Bit 1: End of Refresh Interrupt Enable */

/* DSI Wrapper Interrupt and Status Register */
#define DSI_WISR_TEIF                   (1 << 0)   /* Bit 0: Tearing Effect Interrupt Flag */
#define DSI_WISR_ERIF                   (1 << 1)   /* Bit 1: End of Refresh Interrupt Flag */
#define DSI_WISR_BUSY                   (1 << 2)   /* Bit 2: Busy Flag */

#endif /* __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32_DSI_H */