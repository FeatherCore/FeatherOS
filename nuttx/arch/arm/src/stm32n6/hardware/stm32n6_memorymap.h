/****************************************************************************
 * arch/arm/src/stm32n6/hardware/stm32n6_memorymap.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_MEMORYMAP_H
#define __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_MEMORYMAP_H

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Reference: Based on Zephyr dts/arm/st/n6/stm32n6.dtsi */

/* Peripherals Base Address - Secure Access (Default) */
#define STM32N6_PERIPH_BASE        0x50000000

/* System Control Space */
#define NVIC_BASE                  0xe000e000
#define SCB_BASE                   0xe000ed00
#define MPU_BASE                   0xe000ed90
#define FPU_BASE                   0xe000ef30

/* Cortex-M55 Debug Registers */
#define DCB_BASE                   0xe000edf0
#define DWT_BASE                   0xe0001000
#define TPIU_BASE                  0xe0040000
#define ITM_BASE                   0xe0000000

/* RCC (Reset and Clock Control) */
#define STM32_RCC_BASE             (STM32N6_PERIPH_BASE + 0x0028000)
#define STM32_RCC_CR               (STM32_RCC_BASE + 0x00)
#define STM32_RCC_ICSCR            (STM32_RCC_BASE + 0x04)
#define STM32_RCC_CFGR1            (STM32_RCC_BASE + 0x20)
#define STM32_RCC_CFGR2            (STM32_RCC_BASE + 0x24)
#define STM32_RCC_PLL1CFGR         (STM32_RCC_BASE + 0x28)
#define STM32_RCC_PLL2CFGR         (STM32_RCC_BASE + 0x2C)
#define STM32_RCC_PLL3CFGR         (STM32_RCC_BASE + 0x30)
#define STM32_RCC_PLL4CFGR         (STM32_RCC_BASE + 0x34)
#define STM32_RCC_DLYCFGR          (STM32_RCC_BASE + 0x38)
#define STM32_RCC_PLL1DIVR         (STM32_RCC_BASE + 0x40)
#define STM32_RCC_PLL2DIVR         (STM32_RCC_BASE + 0x44)
#define STM32_RCC_PLL3DIVR         (STM32_RCC_BASE + 0x48)
#define STM32_RCC_PLL4DIVR         (STM32_RCC_BASE + 0x4C)
#define STM32_RCC_PLL1FRACR        (STM32_RCC_BASE + 0x50)
#define STM32_RCC_PLL2FRACR        (STM32_RCC_BASE + 0x54)
#define STM32_RCC_PLL3FRACR        (STM32_RCC_BASE + 0x58)
#define STM32_RCC_PLL4FRACR        (STM32_RCC_BASE + 0x5C)
#define STM32_RCC_OCENSETR         (STM32_RCC_BASE + 0x60)
#define STM32_RCC_OSPICFGR         (STM32_RCC_BASE + 0x64)
#define STM32_RCC_DSICFGR          (STM32_RCC_BASE + 0x68)
#define STM32_RCC_CKGAENR          (STM32_RCC_BASE + 0x6C)
#define STM32_RCC_AHBPRESC         (STM32_RCC_BASE + 0x70)
#define STM32_RCC_DHCPRESC         (STM32_RCC_BASE + 0x74)
#define STM32_RCC_D1CPRESC         (STM32_RCC_BASE + 0x78)
#define STM32_RCC_D2HPRESC         (STM32_RCC_BASE + 0x7C)
#define STM32_RCC_D1PPRESC         (STM32_RCC_BASE + 0x80)
#define STM32_RCC_D2PPRESC1        (STM32_RCC_BASE + 0x84)
#define STM32_RCC_D2PPRESC2        (STM32_RCC_BASE + 0x88)
#define STM32_RCC_CKPERSELR        (STM32_RCC_BASE + 0x8C)
#define STM32_RCC_D1CFGR           (STM32_RCC_BASE + 0x90)
#define STM32_RCC_D2CFGR           (STM32_RCC_BASE + 0x94)
#define STM32_RCC_SRDCFGR          (STM32_RCC_BASE + 0x98)
#define STM32_RCC_MSSRDCFGR        (STM32_RCC_BASE + 0x9C)
#define STM32_RCC_PLL34CFGR        (STM32_RCC_BASE + 0xA0)
#define STM32_RCC_AHB1ENR          (STM32_RCC_BASE + 0x100)
#define STM32_RCC_AHB2ENR          (STM32_RCC_BASE + 0x104)
#define STM32_RCC_AHB3ENR          (STM32_RCC_BASE + 0x108)
#define STM32_RCC_AHB4ENR          (STM32_RCC_BASE + 0x10C)
#define STM32_RCC_AHB5ENR          (STM32_RCC_BASE + 0x110)
#define STM32_RCC_AHB1LPENR        (STM32_RCC_BASE + 0x140)
#define STM32_RCC_AHB2LPENR        (STM32_RCC_BASE + 0x144)
#define STM32_RCC_AHB3LPENR        (STM32_RCC_BASE + 0x148)
#define STM32_RCC_AHB4LPENR        (STM32_RCC_BASE + 0x14C)
#define STM32_RCC_AHB5LPENR        (STM32_RCC_BASE + 0x150)
#define STM32_RCC_APB1ENR          (STM32_RCC_BASE + 0x180)
#define STM32_RCC_APB2ENR          (STM32_RCC_BASE + 0x184)
#define STM32_RCC_APB3ENR          (STM32_RCC_BASE + 0x188)
#define STM32_RCC_APB4ENR          (STM32_RCC_BASE + 0x18C)
#define STM32_RCC_APB5ENR          (STM32_RCC_BASE + 0x190)
#define STM32_RCC_APB1LPENR        (STM32_RCC_BASE + 0x1C0)
#define STM32_RCC_APB2LPENR        (STM32_RCC_BASE + 0x1C4)
#define STM32_RCC_APB3LPENR        (STM32_RCC_BASE + 0x1C8)
#define STM32_RCC_APB4LPENR        (STM32_RCC_BASE + 0x1CC)
#define STM32_RCC_APB5LPENR        (STM32_RCC_BASE + 0x1D0)
#define STM32_RCC_AHB1RSTR         (STM32_RCC_BASE + 0x200)
#define STM32_RCC_AHB2RSTR         (STM32_RCC_BASE + 0x204)
#define STM32_RCC_AHB3RSTR         (STM32_RCC_BASE + 0x208)
#define STM32_RCC_AHB4RSTR         (STM32_RCC_BASE + 0x20C)
#define STM32_RCC_AHB5RSTR         (STM32_RCC_BASE + 0x210)
#define STM32_RCC_APB1RSTR         (STM32_RCC_BASE + 0x240)
#define STM32_RCC_APB2RSTR         (STM32_RCC_BASE + 0x244)
#define STM32_RCC_APB3RSTR         (STM32_RCC_BASE + 0x248)
#define STM32_RCC_APB4RSTR         (STM32_RCC_BASE + 0x24C)
#define STM32_RCC_APB5RSTR         (STM32_RCC_BASE + 0x250)
#define STM32_RCC_AHB1SMENR        (STM32_RCC_BASE + 0x280)
#define STM32_RCC_AHB2SMENR        (STM32_RCC_BASE + 0x284)
#define STM32_RCC_AHB3SMENR        (STM32_RCC_BASE + 0x288)
#define STM32_RCC_AHB4SMENR        (STM32_RCC_BASE + 0x28C)
#define STM32_RCC_AHB5SMENR        (STM32_RCC_BASE + 0x290)
#define STM32_RCC_APB1SMENR        (STM32_RCC_BASE + 0x2C0)
#define STM32_RCC_APB2SMENR        (STM32_RCC_BASE + 0x2C4)
#define STM32_RCC_APB3SMENR        (STM32_RCC_BASE + 0x2C8)
#define STM32_RCC_APB4SMENR        (STM32_RCC_BASE + 0x2CC)
#define STM32_RCC_APB5SMENR        (STM32_RCC_BASE + 0x2D0)
#define STM32_RCC_CCIPR1           (STM32_RCC_BASE + 0x144)
#define STM32_RCC_CCIPR2           (STM32_RCC_BASE + 0x148)
#define STM32_RCC_CCIPR3           (STM32_RCC_BASE + 0x14C)
#define STM32_RCC_CCIPR4           (STM32_RCC_BASE + 0x150)
#define STM32_RCC_CCIPR5           (STM32_RCC_BASE + 0x154)
#define STM32_RCC_CCIPR6           (STM32_RCC_BASE + 0x158)
#define STM32_RCC_CCIPR7           (STM32_RCC_BASE + 0x15C)
#define STM32_RCC_CCIPR8           (STM32_RCC_BASE + 0x160)
#define STM32_RCC_CCIPR9           (STM32_RCC_BASE + 0x164)
#define STM32_RCC_CCIPR12          (STM32_RCC_BASE + 0x170)
#define STM32_RCC_CCIPR13          (STM32_RCC_BASE + 0x174)
#define STM32_RCC_CCIPR14          (STM32_RCC_BASE + 0x178)
#define STM32_RCC_ICC1             (STM32_RCC_BASE + 0x0C4)
#define STM32_RCC_ICC2             (STM32_RCC_BASE + 0x0C8)
#define STM32_RCC_ICC3             (STM32_RCC_BASE + 0x0CC)
#define STM32_RCC_ICC4             (STM32_RCC_BASE + 0x0D0)
#define STM32_RCC_ICC5             (STM32_RCC_BASE + 0x0D4)
#define STM32_RCC_ICC6             (STM32_RCC_BASE + 0x0D8)
#define STM32_RCC_ICC7             (STM32_RCC_BASE + 0x0DC)
#define STM32_RCC_ICC8             (STM32_RCC_BASE + 0x0E0)
#define STM32_RCC_ICC9             (STM32_RCC_BASE + 0x0E4)
#define STM32_RCC_ICC10            (STM32_RCC_BASE + 0x0E8)
#define STM32_RCC_ICC11            (STM32_RCC_BASE + 0x0EC)
#define STM32_RCC_ICC12            (STM32_RCC_BASE + 0x0F0)
#define STM32_RCC_ICC13            (STM32_RCC_BASE + 0x0F4)
#define STM32_RCC_ICC14            (STM32_RCC_BASE + 0x0F8)
#define STM32_RCC_ICC15            (STM32_RCC_BASE + 0x0FC)
#define STM32_RCC_ICC16            (STM32_RCC_BASE + 0x100)
#define STM32_RCC_ICC17            (STM32_RCC_BASE + 0x104)
#define STM32_RCC_ICC18            (STM32_RCC_BASE + 0x108)
#define STM32_RCC_ICC19            (STM32_RCC_BASE + 0x10C)
#define STM32_RCC_ICC20            (STM32_RCC_BASE + 0x110)

/* PWR (Power Control) */
#define STM32_PWR_BASE             (STM32N6_PERIPH_BASE + 0x0002C00)
#define STM32_PWR_CR1              (STM32_PWR_BASE + 0x00)
#define STM32_PWR_CSR1             (STM32_PWR_BASE + 0x04)
#define STM32_PWR_CR2              (STM32_PWR_BASE + 0x08)
#define STM32_PWR_CR3              (STM32_PWR_BASE + 0x0C)
#define STM32_PWR_CR4              (STM32_PWR_BASE + 0x10)
#define STM32_PWR_CPUCR            (STM32_PWR_BASE + 0x14)
#define STM32_PWR_D3CR             (STM32_PWR_BASE + 0x18)
#define STM32_PWR_WUCR             (STM32_PWR_BASE + 0x20)
#define STM32_PWR_WUSCR            (STM32_PWR_BASE + 0x24)
#define STM32_PWR_WUSR             (STM32_PWR_BASE + 0x28)
#define STM32_PWR_SECCFGR          (STM32_PWR_BASE + 0x30)
#define STM32_PWR_PRIVCFGR         (STM32_PWR_BASE + 0x34)

/* EXTI (External Interrupt) */
#define STM32_EXTI_BASE            (STM32N6_PERIPH_BASE + 0x0025000)
#define STM32_EXTI_IMR1            (STM32_EXTI_BASE + 0x00)
#define STM32_EXTI_IMR2            (STM32_EXTI_BASE + 0x04)
#define STM32_EXTI_IMR3            (STM32_EXTI_BASE + 0x08)
#define STM32_EXTI_EMR1            (STM32_EXTI_BASE + 0x20)
#define STM32_EXTI_EMR2            (STM32_EXTI_BASE + 0x24)
#define STM32_EXTI_EMR3            (STM32_EXTI_BASE + 0x28)
#define STM32_EXTI_RTSR1           (STM32_EXTI_BASE + 0x40)
#define STM32_EXTI_RTSR2           (STM32_EXTI_BASE + 0x44)
#define STM32_EXTI_RTSR3           (STM32_EXTI_BASE + 0x48)
#define STM32_EXTI_FTSR1           (STM32_EXTI_BASE + 0x60)
#define STM32_EXTI_FTSR2           (STM32_EXTI_BASE + 0x64)
#define STM32_EXTI_FTSR3           (STM32_EXTI_BASE + 0x68)
#define STM32_EXTI_SWIER1          (STM32_EXTI_BASE + 0x80)
#define STM32_EXTI_SWIER2          (STM32_EXTI_BASE + 0x84)
#define STM32_EXTI_SWIER3          (STM32_EXTI_BASE + 0x88)
#define STM32_EXTI_PR1             (STM32_EXTI_BASE + 0xA0)
#define STM32_EXTI_PR2             (STM32_EXTI_BASE + 0xA4)
#define STM32_EXTI_PR3             (STM32_EXTI_BASE + 0xA8)
#define STM32_EXTI_CPUIMR1         (STM32_EXTI_BASE + 0xD0)
#define STM32_EXTI_CPUIMR2         (STM32_EXTI_BASE + 0xD4)
#define STM32_EXTI_CPUIMR3         (STM32_EXTI_BASE + 0xD8)
#define STM32_EXTI_CPUEMR1         (STM32_EXTI_BASE + 0xF0)
#define STM32_EXTI_CPUEMR2         (STM32_EXTI_BASE + 0xF4)
#define STM32_EXTI_CPUEMR3         (STM32_EXTI_BASE + 0xF8)
#define STM32_EXTI_C1IMR1          (STM32_EXTI_BASE + 0x100)
#define STM32_EXTI_C1IMR2          (STM32_EXTI_BASE + 0x104)
#define STM32_EXTI_C1IMR3          (STM32_EXTI_BASE + 0x108)
#define STM32_EXTI_C1EMR1          (STM32_EXTI_BASE + 0x120)
#define STM32_EXTI_C1EMR2          (STM32_EXTI_BASE + 0x124)
#define STM32_EXTI_C1EMR3          (STM32_EXTI_BASE + 0x128)

/* GPIO Ports */
#define STM32_GPIOA_BASE           (STM32N6_PERIPH_BASE + 0x00200000)
#define STM32_GPIOB_BASE           (STM32N6_PERIPH_BASE + 0x00200400)
#define STM32_GPIOC_BASE           (STM32N6_PERIPH_BASE + 0x00200800)
#define STM32_GPIOD_BASE           (STM32N6_PERIPH_BASE + 0x00200C00)
#define STM32_GPIOE_BASE           (STM32N6_PERIPH_BASE + 0x00201000)
#define STM32_GPIOF_BASE           (STM32N6_PERIPH_BASE + 0x00201400)
#define STM32_GPIOG_BASE           (STM32N6_PERIPH_BASE + 0x00201800)
#define STM32_GPIOH_BASE           (STM32N6_PERIPH_BASE + 0x00201C00)
#define STM32_GPION_BASE           (STM32N6_PERIPH_BASE + 0x00203400)
#define STM32_GPIOO_BASE           (STM32N6_PERIPH_BASE + 0x00203800)
#define STM32_GPIOP_BASE           (STM32N6_PERIPH_BASE + 0x00203C00)
#define STM32_GPIOQ_BASE           (STM32N6_PERIPH_BASE + 0x00204000)

/* USART/UART */
#define STM32_USART1_BASE          (STM32N6_PERIPH_BASE + 0x00010000)
#define STM32_USART2_BASE          (STM32N6_PERIPH_BASE + 0x00004400)
#define STM32_USART3_BASE          (STM32N6_PERIPH_BASE + 0x00004800)
#define STM32_UART4_BASE           (STM32N6_PERIPH_BASE + 0x00004C00)
#define STM32_UART5_BASE           (STM32N6_PERIPH_BASE + 0x00005000)
#define STM32_USART6_BASE          (STM32N6_PERIPH_BASE + 0x00010400)
#define STM32_UART7_BASE           (STM32N6_PERIPH_BASE + 0x00007800)
#define STM32_UART8_BASE           (STM32N6_PERIPH_BASE + 0x00007C00)
#define STM32_UART9_BASE           (STM32N6_PERIPH_BASE + 0x00010800)
#define STM32_USART10_BASE         (STM32N6_PERIPH_BASE + 0x00010C00)

/* I2C */
#define STM32_I2C1_BASE            (STM32N6_PERIPH_BASE + 0x00005400)
#define STM32_I2C2_BASE            (STM32N6_PERIPH_BASE + 0x00005800)
#define STM32_I2C3_BASE            (STM32N6_PERIPH_BASE + 0x00005C00)
#define STM32_I2C4_BASE            (STM32N6_PERIPH_BASE + 0x00001C00)

/* SPI */
#define STM32_SPI1_BASE            (STM32N6_PERIPH_BASE + 0x00013000)
#define STM32_SPI2_BASE            (STM32N6_PERIPH_BASE + 0x00003800)
#define STM32_SPI3_BASE            (STM32N6_PERIPH_BASE + 0x00003C00)
#define STM32_SPI4_BASE            (STM32N6_PERIPH_BASE + 0x00013400)
#define STM32_SPI5_BASE            (STM32N6_PERIPH_BASE + 0x00015000)
#define STM32_SPI6_BASE            (STM32N6_PERIPH_BASE + 0x00011400)

/* I3C */
#define STM32_I3C1_BASE            (STM32N6_PERIPH_BASE + 0x00006000)
#define STM32_I3C2_BASE            (STM32N6_PERIPH_BASE + 0x00006400)

/* FDCAN */
#define STM32_FDCAN1_BASE          (STM32N6_PERIPH_BASE + 0x0000A000)
#define STM32_FDCAN2_BASE          (STM32N6_PERIPH_BASE + 0x0000A400)
#define STM32_FDCAN3_BASE          (STM32N6_PERIPH_BASE + 0x0000E800)

/* ADC */
#define STM32_ADC1_BASE            (STM32N6_PERIPH_BASE + 0x00022000)
#define STM32_ADC2_BASE            (STM32N6_PERIPH_BASE + 0x00022100)

/* GPDMA */
#define STM32_GPDMA1_BASE          (STM32N6_PERIPH_BASE + 0x00021000)

/* LTDC (LCD-TFT Display Controller) */
#define STM32_LTDC_BASE            (STM32N6_PERIPH_BASE + 0x00011000)

/* DMA2D (2D Graphics Accelerator) */
#define STM32_DMA2D_BASE           (STM32N6_PERIPH_BASE + 0x00011400)

/* DCMIPP (Digital Camera Interface Parallel Port) */
#define STM32_DCMIPP_BASE          (STM32N6_PERIPH_BASE + 0x00802000)

/* Ethernet */
#define STM32_ETH_BASE             (STM32N6_PERIPH_BASE + 0x008036000)

/* SDMMC */
#define STM32_SDMMC1_BASE          (STM32N6_PERIPH_BASE + 0x008027000)
#define STM32_SDMMC2_BASE          (STM32N6_PERIPH_BASE + 0x008026800)

/* XSPI */
#define STM32_XSPI1_BASE           (STM32N6_PERIPH_BASE + 0x008025000)
#define STM32_XSPI2_BASE           (STM32N6_PERIPH_BASE + 0x00802A000)
#define STM32_XSPI3_BASE           (STM32N6_PERIPH_BASE + 0x00802D000)

/* NPU (Neural-ART Neural Processing Unit) */
#define STM32_NPU_BASE             (STM32N6_PERIPH_BASE + 0x0080E0000)
#define STM32_NPU_SIZE             (128 * 1024)  /* 128 KB */

/* NPU Cache */
#define STM32_NPU_CACHE_BASE       (STM32N6_PERIPH_BASE + 0x0080DFC00)

/* JPEG Codec */
#define STM32_JPEG_BASE            (STM32N6_PERIPH_BASE + 0x008023000)

/* VENC (Video Encoder) */
#define STM32_VENC_BASE            (STM32N6_PERIPH_BASE + 0x008005000)

/* USB OTG HS */
#define STM32_OTG_HS1_BASE         (STM32N6_PERIPH_BASE + 0x008040000)
#define STM32_OTG_HS2_BASE         (STM32N6_PERIPH_BASE + 0x008080000)

/* USB PHY */
#define STM32_USBPHYC1_BASE        (STM32N6_PERIPH_BASE + 0x00803FC00)
#define STM32_USBPHYC2_BASE        (STM32N6_PERIPH_BASE + 0x0080C0000)

/* RNG (Random Number Generator) */
#define STM32_RNG_BASE             (STM32N6_PERIPH_BASE + 0x00020000)

/* CRC (Cyclic Redundancy Check) */
#define STM32_CRC_BASE             (STM32N6_PERIPH_BASE + 0x0024C00)

/* BSEC (Boot Security) */
#define STM32_BSEC_BASE            (STM32N6_PERIPH_BASE + 0x0009000)

/* IWDG/WWDG */
#define STM32_IWDG_BASE            (STM32N6_PERIPH_BASE + 0x0004800)
#define STM32_WWDG_BASE            (STM32N6_PERIPH_BASE + 0x00002C00)

/* RAMCFG (RAM Configuration) */
#define STM32_RAMCFG_SRAM3_BASE    (STM32N6_PERIPH_BASE + 0x002023100)
#define STM32_RAMCFG_SRAM4_BASE    (STM32N6_PERIPH_BASE + 0x002023180)
#define STM32_RAMCFG_SRAM5_BASE    (STM32N6_PERIPH_BASE + 0x002023200)
#define STM32_RAMCFG_SRAM6_BASE    (STM32N6_PERIPH_BASE + 0x002023280)

/* Memory Regions */

/* Internal Flash */
#define STM32_FLASH_BASE           0x08000000
#define STM32_FLASH_SIZE           0x01000000  /* 16 MB (Typical for N657) */

/* Embedded SRAM */
#define STM32_SRAM_BASE            0x20000000
#define STM32_SRAM_SIZE            0x00100000  /* 1 MB (Typical) */

/* AXISRAM Memory (AXI SRAM) - 6 banks, total 2.75 MB */
#define STM32_AXISRAM1_BASE        0x34000000  /* 512 KB */
#define STM32_AXISRAM1_SIZE        (512 * 1024)
#define STM32_AXISRAM2_BASE        0x34180000  /* 512 KB */
#define STM32_AXISRAM2_SIZE        (512 * 1024)
#define STM32_AXISRAM3_BASE        0x34200000  /* 448 KB */
#define STM32_AXISRAM3_SIZE        (448 * 1024)
#define STM32_AXISRAM4_BASE        0x34270000  /* 448 KB */
#define STM32_AXISRAM4_SIZE        (448 * 1024)
#define STM32_AXISRAM5_BASE        0x342E0000  /* 448 KB */
#define STM32_AXISRAM5_SIZE        (448 * 1024)
#define STM32_AXISRAM6_BASE        0x34350000  /* 448 KB */
#define STM32_AXISRAM6_SIZE        (448 * 1024)

/* Total AXISRAM: 2.75 MB */

/* External Memory (via XSPI) */
#define STM32_XSPI1_MEM_BASE       0x90000000  /* PSRAM mapped area */
#define STM32_XSPI2_MEM_BASE       0x70000000  /* Flash memory area */
#define STM32_XSPI3_MEM_BASE       0x80000000  /* Additional memory area */

#endif /* __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_MEMORYMAP_H */
