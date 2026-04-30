/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_memorymap.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_MEMORYMAP_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_MEMORYMAP_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Memory Map for RA8P1 (R7KA8P1KFLCAC) based on Zephyr RA8P1 device tree */

/* Code Memory Area */
#define RA8P_CODE_REGION_BASE          0x00000000  /* Code MRAM/Flash Base */
#define RA8P_CODE_REGION_SIZE          0x00400000  /* 4 MB */

/* Data Memory Area */
#define RA8P_DATA_REGION_BASE          0x20000000  /* Data MRAM Base */
#define RA8P_DATA_REGION_SIZE          0x00080000  /* 512 KB */

/* Main SRAM */
#define RA8P_SRAM_BASE                 0x22000000  /* Main SRAM Base */
#define RA8P_SRAM_SIZE                 0x00200000  /* 2 MB */
#define RA8P_SRAM0_BASE                0x22000000  /* 1.4 MB SRAM0 */
#define RA8P_SRAM0_SIZE                0x00160000
#define RA8P_SRAM1_BASE                0x22160000  /* 0.6 MB SRAM1 */
#define RA8P_SRAM1_SIZE                0x000A0000

/* External Memory Areas */
#define RA8P_DDR_BASE                  0x60000000  /* External DDR Base */
#define RA8P_DDR_SIZE                  0x10000000  /* 256 MB */
#define RA8P_SDRAM_BASE                0x68000000  /* External SDRAM Base */
#define RA8P_SDRAM_SIZE                0x04000000  /* 64 MB */

/* Peripheral Register Areas */
#define RA8P_PERIPHERAL_BASE           0x40000000  /* Peripheral Base */
#define RA8P_PERIPHERAL_SIZE           0x08000000  /* 128 MB */

/* System Control and Reset */
#define RA8P_SYSTEM_BASE               0x4001E000  /* System Control */
#define RA8P_SYSTEM_SIZE               0x00001000

/* Clock Generation Control */
#define RA8P_CGC_BASE                  0x40023000  /* CGC Base */
#define RA8P_CGC_SIZE                  0x00001000

/* ICU (External Interrupt Controller) */
#define RA8P_ICU_BASE                  0x40024000  /* ICU Base */
#define RA8P_ICU_SIZE                  0x00001000

/* DMAC (Direct Memory Access Controller) */
#define RA8P_DMAC_BASE                 0x4000A000  /* DMAC Base */
#define RA8P_DMAC_SIZE                 0x00001000

/* ICU (Internal Interrupt Controller) */
#define RA8P_ICU_BASE                  0x40024000  /* ICU Base */
#define RA8P_ICU_SIZE                  0x00001000

/* Port Function Control */
#define RA8P_PFC_BASE                  0x40040000  /* PFC Base */
#define RA8P_PFC_SIZE                  0x00001000

/* Port Function Select */
#define RA8P_PFS_BASE                  0x40400000  /* PFS Base */
#define RA8P_PFS_SIZE                  0x00000400

/* I/O Ports */
#define RA8P_IOPORT0_BASE              0x40400000  /* IOPORT0 Base */
#define RA8P_IOPORT0_SIZE              0x00000020
#define RA8P_IOPORT1_BASE              0x40400020  /* IOPORT1 Base */
#define RA8P_IOPORT1_SIZE              0x00000020
#define RA8P_IOPORT2_BASE              0x40400040  /* IOPORT2 Base */
#define RA8P_IOPORT2_SIZE              0x00000020
#define RA8P_IOPORT3_BASE              0x40400060  /* IOPORT3 Base */
#define RA8P_IOPORT3_SIZE              0x00000020
#define RA8P_IOPORT4_BASE              0x40400080  /* IOPORT4 Base */
#define RA8P_IOPORT4_SIZE              0x00000020
#define RA8P_IOPORT5_BASE              0x404000A0  /* IOPORT5 Base */
#define RA8P_IOPORT5_SIZE              0x00000020
#define RA8P_IOPORT6_BASE              0x404000C0  /* IOPORT6 Base */
#define RA8P_IOPORT6_SIZE              0x00000020
#define RA8P_IOPORT7_BASE              0x404000E0  /* IOPORT7 Base */
#define RA8P_IOPORT7_SIZE              0x00000020
#define RA8P_IOPORT8_BASE              0x40400100  /* IOPORT8 Base */
#define RA8P_IOPORT8_SIZE              0x00000020
#define RA8P_IOPORT9_BASE              0x40400120  /* IOPORT9 Base */
#define RA8P_IOPORT9_SIZE              0x00000020
#define RA8P_IOPORTA_BASE              0x40400140  /* IOPORTA Base */
#define RA8P_IOPORTA_SIZE              0x00000020
#define RA8P_IOPORTB_BASE              0x40400160  /* IOPORTB Base */
#define RA8P_IOPORTB_SIZE              0x00000020
#define RA8P_IOPORTC_BASE              0x40400180  /* IOPORTC Base */
#define RA8P_IOPORTC_SIZE              0x00000020
#define RA8P_IOPORTD_BASE              0x404001A0  /* IOPORTD Base */
#define RA8P_IOPORTD_SIZE              0x00000020

/* UART/SCI_B Peripherals */
#define RA8P_SCI0_BASE                 0x40358000  /* SCI0 Base */
#define RA8P_SCI1_BASE                 0x40358100  /* SCI1 Base */
#define RA8P_SCI2_BASE                 0x40358200  /* SCI2 Base */
#define RA8P_SCI3_BASE                 0x40358300  /* SCI3 Base */
#define RA8P_SCI4_BASE                 0x40358400  /* SCI4 Base */
#define RA8P_SCI5_BASE                 0x40358500  /* SCI5 Base */
#define RA8P_SCI6_BASE                 0x40358600  /* SCI6 Base */
#define RA8P_SCI7_BASE                 0x40358700  /* SCI7 Base */
#define RA8P_SCI8_BASE                 0x40358800  /* SCI8 Base */
#define RA8P_SCI9_BASE                 0x40358900  /* SCI9 Base */

/* SPI Peripherals */
#define RA8P_SPI0_BASE                 0x4035C000  /* SPI0 Base */
#define RA8P_SPI1_BASE                 0x4035C100  /* SPI1 Base */
#define RA8P_SPI2_BASE                 0x4035C200  /* SPI2 Base */
#define RA8P_SPI3_BASE                 0x4035C300  /* SPI3 Base */

/* IIC Peripherals */
#define RA8P_IIC0_BASE                 0x4025E000  /* IIC0 Base */
#define RA8P_IIC1_BASE                 0x4025E100  /* IIC1 Base */
#define RA8P_IIC2_BASE                 0x4025E200  /* IIC2 Base */

/* GPT Peripherals */
#define RA8P_GPT0_BASE                 0x40322000  /* GPT0 Base */
#define RA8P_GPT1_BASE                 0x40322100  /* GPT1 Base */
#define RA8P_GPT2_BASE                 0x40322200  /* GPT2 Base */
#define RA8P_GPT3_BASE                 0x40322300  /* GPT3 Base */
#define RA8P_GPT4_BASE                 0x40322400  /* GPT4 Base */
#define RA8P_GPT5_BASE                 0x40322500  /* GPT5 Base */
#define RA8P_GPT6_BASE                 0x40322600  /* GPT6 Base */
#define RA8P_GPT7_BASE                 0x40322700  /* GPT7 Base */
#define RA8P_GPT8_BASE                 0x40322800  /* GPT8 Base */
#define RA8P_GPT9_BASE                 0x40322900  /* GPT9 Base */
#define RA8P_GPT10_BASE                0x40322A00  /* GPT10 Base */
#define RA8P_GPT11_BASE                0x40322B00  /* GPT11 Base */
#define RA8P_GPT12_BASE                0x40322C00  /* GPT12 Base */
#define RA8P_GPT13_BASE                0x40322D00  /* GPT13 Base */

/* CANFD Peripherals */
#define RA8P_CANFD0_BASE               0x40380000  /* CANFD0 Base */
#define RA8P_CANFD1_BASE               0x40390000  /* CANFD1 Base */

/* SDHC Peripherals */
#define RA8P_SDHC0_BASE                0x40252000  /* SDHC0 Base */
#define RA8P_SDHC1_BASE                0x40252400  /* SDHC1 Base */

/* USB Peripherals */
#define RA8P_USBFS_BASE                0x40250000  /* USBFS Base */
#define RA8P_USBHS_BASE                0x40351000  /* USBHS Base */

/* USBPHY Peripheral */
#define RA8P_USBPHY_BASE               0x40254000  /* USBPHY Base */

/* RTC Peripheral */
#define RA8P_RTC_BASE                  0x40202000  /* RTC Base */

/* WDT Peripherals */
#define RA8P_WDT_BASE                  0x40202600  /* WDT Base */

/* ELC (Event Link Controller) */
#define RA8P_ELC_BASE                  0x40025000  /* ELC Base */

/* MTU (Multi-Function Timer Unit) */
#define RA8P_MTU_BASE                  0x40328000  /* MTU Base */

/* POEG (Port Output Enable for GPT) */
#define RA8P_POEG_BASE                 0x4032A000  /* POEG Base */

/* A/D Converter */
#define RA8P_ADC0_BASE                 0x40330000  /* ADC0 Base */
#define RA8P_ADC1_BASE                 0x40332000  /* ADC1 Base */

/* D/A Converter */
#define RA8P_DAC_BASE                  0x40333000  /* DAC Base */

/* COMP (Comparator) */
#define RA8P_COMP_HS_BASE              0x40235000  /* COMP_HS Base */
#define RA8P_COMP_LP_BASE              0x40236000  /* COMP_LP Base */

/* LCDC (LCD Controller) */
#define RA8P_LCDC_BASE                 0x40340000  /* LCDC Base */

/* GLCDC (Graphics LCD Controller) */
#define RA8P_GLCDC_BASE                0x40342000  /* GLCDC Base */

/* MIPI DSI */
#define RA8P_MIPI_DSI_BASE             0x40346000  /* MIPI DSI Base */

/* ETHERC (Ethernet Controller) */
#define RA8P_ETHERC_BASE               0x4010C000  /* ETHERC Base */

/* EDMAC (Ethernet DMA Controller) */
#define RA8P_EDMAC_BASE                0x40110000  /* EDMAC Base */

/* USBHS_D0FIFO and USBHS_D1FIFO */
#define RA8P_USBHS_FIFO_BASE           0x40354000  /* USBHS FIFO Base */

/* Option Setting Areas */
#define RA8P_OFS_BASE                  0x02C9F000  /* Option Setting Base */
#define RA8P_OFS_SIZE                  0x00001000

#define RA8P_OTP_BASE                  0x02E07000  /* OTP Base */
#define RA8P_OTP_SIZE                  0x0012B000

/* Cache Controller */
#define RA8P_CACHE_BASE                0x40021000  /* Cache Controller Base */

/* MPC (Memory Protection Controller) */
#define RA8P_MPC_BASE                  0x40022000  /* MPC Base */

/* MPU (Memory Protection Unit) */
#define RA8P_MPU_BASE                  0xE000ED90  /* MPU Base (Cortex-M85) */

/* SysTick */
#define RA8P_SYSTICK_BASE              0xE000E010  /* SysTick Base (Cortex-M85) */

/* NVIC */
#define RA8P_NVIC_BASE                 0xE000E100  /* NVIC Base (Cortex-M85) */

/* Debug ITM */
#define RA8P_ITM_BASE                  0xE0000000  /* ITM Base (Cortex-M85) */

/* Core Debug */
#define RA8P_COREDEBUG_BASE            0xE000EDF0  /* Core Debug Base (Cortex-M85) */

/* Memory Tagging Unit */
#define RA8P_MTB_BASE                  0xE0043000  /* MTB Base (Cortex-M85) */

/* Memory Protection Unit (CM33 core) */
#define RA8P_CM33_MPU_BASE             0xE000ED90  /* CM33 MPU Base */

/* Core Debug (CM33 core) */
#define RA8P_CM33_COREDEBUG_BASE       0xE000EDF0  /* CM33 Core Debug Base */

/* NPU (Neural Processing Unit) */
#define RA8P_NPU_BASE                  0x40140000  /* NPU Base */

/* SDRAM Controller */
#define RA8P_SDRAMC_BASE               0x40003C00  /* SDRAM Controller Base */

/* OSPI (OctaSerial Peripheral Interface) */
#define RA8P_OSPI_BASE                 0x4026C000  /* OSPI Base */

/* QSPI (QuadSerial Peripheral Interface) */
#define RA8P_QSPI_BASE                 0x40268000  /* QSPI Base */

/* CEC (Consumer Electronics Control) */
#define RA8P_CEC_BASE                  0x40272000  /* CEC Base */

/* CRC (Cyclic Redundancy Check) */
#define RA8P_CRC_BASE                  0x40310000  /* CRC Base */

/* RSA Accelerator */
#define RA8P_RSA_BASE                  0x40138000  /* RSA Base */

/* AES Accelerator */
#define RA8P_AES_BASE                  0x40134000  /* AES Base */

/* TRNG (True Random Number Generator) */
#define RA8P_TRNG_BASE                 0x4013A000  /* TRNG Base */

/* SLCDC (Segment LCD Controller) */
#define RA8P_SLCDC_BASE                0x40270000  /* SLCDC Base */

/* I3C (Improved Inter-Integrated Circuit) */
#define RA8P_I3C0_BASE                 0x4035F000  /* I3C0 Base */

/* I2S (Inter-IC Sound) */
#define RA8P_I2S0_BASE                 0x4025D000  /* I2S0 Base */
#define RA8P_I2S1_BASE                 0x4025D100  /* I2S1 Base */

/* SSIF (Sampled Sound Interface) */
#define RA8P_SSIF0_BASE                0x4025D000  /* SSIF0 Base */
#define RA8P_SSIF1_BASE                0x4025D100  /* SSIF1 Base */

/* QSPI ROM */
#define RA8P_QSPI_ROM_BASE             0x68000000  /* QSPI ROM Base */

/* SDHI (Secure Digital Host Interface) */
#define RA8P_SDHI0_BASE                0x40252000  /* SDHI0 Base */
#define RA8P_SDHI1_BASE                0x40252400  /* SDHI1 Base */

/* CEU (Camera Engine Unit) */
#define RA8P_CEU_BASE                  0x40348000  /* CEU Base */

/* DU (Display Unit) */
#define RA8P_DU_BASE                   0x40344000  /* DU Base */

/* MTB (Micro Trace Buffer) */
#define RA8P_MTB_BASE                  0xE0043000  /* MTB Base */

/* PMU (Performance Monitoring Unit) */
#define RA8P_PMU_BASE                  0xE0041000  /* PMU Base */

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_MEMORYMAP_H */