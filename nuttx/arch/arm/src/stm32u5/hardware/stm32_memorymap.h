/****************************************************************************
 * arch/arm/src/stm32u5/hardware/stm32_memorymap.h
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

#ifndef __ARCH_ARM_SRC_STM32U5_STM32_MEMORYMAP_H
#define __ARCH_ARM_SRC_STM32U5_STM32_MEMORYMAP_H

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* STM32U5XXX Address Blocks ************************************************/

#define STM32_CODE_BASE         0x00000000  /* 0x00000000-0x1fffffff: 512Mb code block */
#define STM32_SRAM_BASE         0x20000000  /* 0x20000000-0x3fffffff: 512Mb sram block */
#define STM32_PERIPH_BASE       0x40000000  /* 0x40000000-0x5fffffff: 512Mb peripheral block */
#define STM32_FMC_BASE          0x60000000  /* 0x60000000-0x7fffffff: 512Mb FMC memories */
#define STM32_FMC_BANK1         0x60000000  /* 0x60000000-0x6fffffff: 256Mb NOR/SRAM */
#define STM32_OCTOSPI2_BANK     0x70000000  /* 0x70000000-0x7fffffff: 256Mb OCTOSPI2 memories */
#define STM32_FMC_BANK3         0x80000000  /* 0x80000000-0x8fffffff: 256Mb NAND FLASH */
#define STM32_OCTOSPI1_BANK     0x90000000  /* 0x90000000-0x9fffffff: 256Mb OCTOSPI1 memories */
#define STM32_HSPI1_BANK        0xA0000000  /* 0xa0000000-0xafffffff: 256Mb HSPI1 memories */
#define STM32_CORTEX_BASE       0xE0000000  /* 0xe0000000-0xffffffff: 512Mb Cortex-M33 block */

#define STM32_REGION_MASK       0xF0000000
#define STM32_IS_SRAM(a)        ((((uint32_t)(a)) & STM32_REGION_MASK) == STM32_SRAM_BASE)
#define STM32_IS_EXTSRAM(a)     ((((uint32_t)(a)) & STM32_REGION_MASK) == STM32_FMC_BANK1)
#define STM32_IS_FMC(a)         ((((uint32_t)(a)) & STM32_REGION_MASK) == STM32_FMC_BASE)
#define STM32_IS_OCTOSPI2(a)    ((((uint32_t)(a)) & STM32_REGION_MASK) == STM32_OCTOSPI2_BANK)
#define STM32_IS_OCTOSPI1(a)    ((((uint32_t)(a)) & STM32_REGION_MASK) == STM32_OCTOSPI1_BANK)
#define STM32_IS_HSPI1(a)       ((((uint32_t)(a)) & STM32_REGION_MASK) == STM32_HSPI1_BANK)

/* Code Base Addresses ******************************************************/

#if defined(CONFIG_STM32U5_STM32U535XX) || defined(CONFIG_STM32U5_STM32U545XX)
#  define STM32_BOOT_BASE         0x00000000  /* 0x00000000-0x000fffff: Aliased boot memory */
#  define STM32_FLASH_BASE        0x08000000  /* 0x08000000-0x081fffff: FLASH memory */
#  define STM32_SRAM1_BASE        0x20000000  /* 0x20000000-0x2002ffff: 192k SRAM1 */
#  define STM32_SRAM2_BASE        0x20030000  /* 0x20030000-0x2003ffff:  64k SRAM2 */

#elif defined(CONFIG_STM32U5_STM32U575XX) || defined(CONFIG_STM32U5_STM32U585XX)

#  define STM32_BOOT_BASE         0x00000000  /* 0x00000000-0x000fffff: Aliased boot memory */
#  define STM32_FLASH_BASE        0x08000000  /* 0x08000000-0x081fffff: FLASH memory */
#  define STM32_SRAM1_BASE        0x20000000  /* 0x20000000-0x2002ffff: 192k SRAM1 */
#  define STM32_SRAM2_BASE        0x20030000  /* 0x20030000-0x2003ffff:  64k SRAM2 */
#  define STM32_SRAM3_BASE        0x20040000  /* 0x20040000-0x200bffff: 512k SRAM3 */

#elif defined(CONFIG_STM32U5_STM32U59XX) || defined(CONFIG_STM32U5_STM32U59AXX) || \
      defined(CONFIG_STM32U5_STM32U5A5XX) || defined(CONFIG_STM32U5_STM32U5A9XX)

#  define STM32_BOOT_BASE         0x00000000  /* 0x00000000-0x000fffff: Aliased boot memory */
#  define STM32_FLASH_BASE        0x08000000  /* 0x08000000-0x081fffff: FLASH memory */
#  define STM32_SRAM1_BASE        0x20000000  /* 0x20000000-0x200bffff: 768k SRAM1 */
#  define STM32_SRAM2_BASE        0x200c0000  /* 0x200c0000-0x200cffff:  64k SRAM2 */
#  define STM32_SRAM3_BASE        0x200d0000  /* 0x200d0000-0x2019ffff: 832k SRAM3 */
#  define STM32_SRAM4_BASE        0x28000000  /* 0x28000000-0x28003fff: 16k SRAM4 */
#  define STM32_SRAM5_BASE        0x201a0000  /* 0x201a0000-0x2026ffff: 832k SRAM5 */

#elif defined(CONFIG_STM32U5_STM32U5FXX) || defined(CONFIG_STM32U5_STM32U5GXX)

#  define STM32_BOOT_BASE         0x00000000  /* 0x00000000-0x000fffff: Aliased boot memory */
#  define STM32_FLASH_BASE        0x08000000  /* 0x08000000-0x081fffff: FLASH memory */
#  define STM32_SRAM1_BASE        0x20000000  /* 0x20000000-0x200bffff: 768k SRAM1 */
#  define STM32_SRAM2_BASE        0x200c0000  /* 0x200c0000-0x200cffff:  64k SRAM2 */
#  define STM32_SRAM3_BASE        0x200d0000  /* 0x200d0000-0x2019ffff: 832k SRAM3 */
#  define STM32_SRAM4_BASE        0x28000000  /* 0x28000000-0x28003fff: 16k SRAM4 */
#  define STM32_SRAM5_BASE        0x201a0000  /* 0x201a0000-0x2026ffff: 832k SRAM5 */
#  define STM32_SRAM6_BASE        0x20270000  /* 0x20270000-0x202effff: 512k SRAM6 */

#else
#  error "stm32_memorymap: unsupported STM32U5 memory map"
#endif

/* System Memory Addresses **************************************************/

#define STM32_SYSMEM_UID        0x0BFA0700  /* The 96-bit unique device identifier */
#define STM32_SYSMEM_FSIZE      0x0BFA07A0  /* Size of Flash memory in Kbytes. */
#define STM32_SYSMEM_PACKAGE    0x0BFA0500  /* Indicates the device's package type. */

/* Peripheral Base Addresses ************************************************/

#define STM32_APB1_BASE         0x40000000  /* 0x40000000-0x4000ffff: APB1 */
#define STM32_APB2_BASE         0x40010000  /* 0x40010000-0x4001ffff: APB2 */
#define STM32_AHB1_BASE         0x40020000  /* 0x40020000-0x4201ffff: AHB1 */
#define STM32_AHB2_BASE         0x42020000  /* 0x42020000-0x460003ff: AHB2 */
#define STM32_APB3_BASE         0x46000000  /* 0x46000000-0x4601ffff: APB3 */
#define STM32_AHB3_BASE         0x46020000  /* 0x46020000-0x4fffffff: AHB3 */

/* APB1 Base Addresses ******************************************************/

#define STM32_TIM2_BASE         (STM32_APB1_BASE + 0x00000000)
#define STM32_TIM3_BASE         (STM32_APB1_BASE + 0x00000400)
#define STM32_TIM4_BASE         (STM32_APB1_BASE + 0x00000800)
#define STM32_TIM5_BASE         (STM32_APB1_BASE + 0x00000c00)
#define STM32_TIM6_BASE         (STM32_APB1_BASE + 0x00001000)
#define STM32_TIM7_BASE         (STM32_APB1_BASE + 0x00001400)
#define STM32_WWDG_BASE         (STM32_APB1_BASE + 0x00002c00)
#define STM32_IWDG_BASE         (STM32_APB1_BASE + 0x00003000)
#define STM32_SPI2_BASE         (STM32_APB1_BASE + 0x00003800)
#define STM32_USART2_BASE       (STM32_APB1_BASE + 0x00004400)
#define STM32_USART3_BASE       (STM32_APB1_BASE + 0x00004800)
#define STM32_UART4_BASE        (STM32_APB1_BASE + 0x00004c00)
#define STM32_UART5_BASE        (STM32_APB1_BASE + 0x00005000)
#define STM32_I2C1_BASE         (STM32_APB1_BASE + 0x00005400)
#define STM32_I2C2_BASE         (STM32_APB1_BASE + 0x00005800)
#define STM32_CRS_BASE          (STM32_APB1_BASE + 0x00006000)
#define STM32_USART6_BASE       (STM32_APB1_BASE + 0x00006400)
#define STM32_I2C4_BASE         (STM32_APB1_BASE + 0x00008400)
#define STM32_LPTIM2_BASE       (STM32_APB1_BASE + 0x00009400)
#define STM32_I2C5_BASE         (STM32_APB1_BASE + 0x00009800)
#define STM32_I2C6_BASE         (STM32_APB1_BASE + 0x00009c00)
#define STM32_FDCAN1_BASE       (STM32_APB1_BASE + 0x0000a400)
#define STM32_FDCAN_RAM_BASE    (STM32_APB1_BASE + 0x0000ac00)
#define STM32_UCPD1_BASE        (STM32_APB1_BASE + 0x0000dc00)

/* APB2 Base Addresses ******************************************************/

#define STM32_TIM1_BASE         (STM32_APB2_BASE + 0x00002c00)
#define STM32_SPI1_BASE         (STM32_APB2_BASE + 0x00003000)
#define STM32_TIM8_BASE         (STM32_APB2_BASE + 0x00003400)
#define STM32_USART1_BASE       (STM32_APB2_BASE + 0x00003800)
#define STM32_TIM15_BASE        (STM32_APB2_BASE + 0x00004000)
#define STM32_TIM16_BASE        (STM32_APB2_BASE + 0x00004400)
#define STM32_TIM17_BASE        (STM32_APB2_BASE + 0x00004800)
#define STM32_SAI1_BASE         (STM32_APB2_BASE + 0x00005400)
#define STM32_SAI2_BASE         (STM32_APB2_BASE + 0x00005800)
#define STM32_USB_BASE          (STM32_APB2_BASE + 0x00006000)
#define STM32_USBRAM_BASE       (STM32_APB2_BASE + 0x00006400)
#define STM32_GFXTIM_BASE       (STM32_APB2_BASE + 0x00006400)
#define STM32_LTDC_BASE         (STM32_APB2_BASE + 0x00006800)
#define STM32_DSI_BASE          (STM32_APB2_BASE + 0x00006c00)

/* AHB1 Base Addresses ******************************************************/

#define STM32_GPDMA1_BASE       (STM32_AHB1_BASE + 0x00000000)
#define STM32_CORDIC_BASE       (STM32_AHB1_BASE + 0x00001000)
#define STM32_FMAC_BASE         (STM32_AHB1_BASE + 0x00001400)
#define STM32_FLASHIF_BASE      (STM32_AHB1_BASE + 0x00002000)
#define STM32_CRC_BASE          (STM32_AHB1_BASE + 0x00003000)
#define STM32_TSC_BASE          (STM32_AHB1_BASE + 0x00004000)
#define STM32_MDF1_BASE         (STM32_AHB1_BASE + 0x00005000)
#define STM32_RAMCFG_BASE       (STM32_AHB1_BASE + 0x00006000)
#define STM32_JPEG_BASE         (STM32_AHB1_BASE + 0x0000a000)
#define STM32_DMA2D_BASE        (STM32_AHB1_BASE + 0x0000b000)
#define STM32_GFXMMU_BASE       (STM32_AHB1_BASE + 0x0000c000)
#define STM32_GPU2D_BASE        (STM32_AHB1_BASE + 0x0000f000)
#define STM32_ICACHE_BASE       (STM32_AHB1_BASE + 0x00010400)
#define STM32_DCACHE1_BASE      (STM32_AHB1_BASE + 0x00011400)
#define STM32_DCACHE2_BASE      (STM32_AHB1_BASE + 0x00011800)
#define STM32_GTZC1_TZSC_BASE   (STM32_AHB1_BASE + 0x00012400)
#define STM32_GTZC1_TZIC_BASE   (STM32_AHB1_BASE + 0x00012800)
#define STM32_GTZC1_MPCBB1_BASE (STM32_AHB1_BASE + 0x00012c00)
#define STM32_GTZC1_MPCBB2_BASE (STM32_AHB1_BASE + 0x00013000)
#define STM32_GTZC1_MPCBB3_BASE (STM32_AHB1_BASE + 0x00013400)
#define STM32_GTZC1_MPCBB5_BASE (STM32_AHB1_BASE + 0x00013800)
#define STM32_GTZC1_MPCBB6_BASE (STM32_AHB1_BASE + 0x00013c00)
#define STM32_BKPSRAM_BASE      (STM32_AHB1_BASE + 0x00016400)

/* AHB2 Base Addresses ******************************************************/

#define STM32_GPIOA_BASE        (STM32_AHB2_BASE + 0x00000000)
#define STM32_GPIOB_BASE        (STM32_AHB2_BASE + 0x00000400)
#define STM32_GPIOC_BASE        (STM32_AHB2_BASE + 0x00000800)
#define STM32_GPIOD_BASE        (STM32_AHB2_BASE + 0x00000c00)
#define STM32_GPIOE_BASE        (STM32_AHB2_BASE + 0x00001000)
#define STM32_GPIOF_BASE        (STM32_AHB2_BASE + 0x00001400)
#define STM32_GPIOG_BASE        (STM32_AHB2_BASE + 0x00001800)
#define STM32_GPIOH_BASE        (STM32_AHB2_BASE + 0x00001c00)
#define STM32_GPIOI_BASE        (STM32_AHB2_BASE + 0x00002000)
#define STM32_GPIOJ_BASE        (STM32_AHB2_BASE + 0x00002400)
#define STM32_ADC1_BASE         (STM32_AHB2_BASE + 0x00008000)
#define STM32_DCMI_BASE         (STM32_AHB2_BASE + 0x0000c000)
#define STM32_PSSI_BASE         (STM32_AHB2_BASE + 0x0000c400)
#define STM32_OTG_FS_BASE       (STM32_AHB2_BASE + 0x00020000)
#define STM32_USBOTGHS_BASE     (STM32_AHB2_BASE + 0x00020000) /* HS */
#define STM32_AES_BASE          (STM32_AHB2_BASE + 0x000a0000)
#define STM32_HASH_BASE         (STM32_AHB2_BASE + 0x000a0400)
#define STM32_RNG_BASE          (STM32_AHB2_BASE + 0x000a0800)
#define STM32U5_RNG_BASE        STM32_RNG_BASE
#define STM32_SAES_BASE         (STM32_AHB2_BASE + 0x000a0c00)
#define STM32_PKA_BASE          (STM32_AHB2_BASE + 0x000a2000)
#define STM32_OCTOSPIM_BASE     (STM32_AHB2_BASE + 0x000a4000)
#define STM32_OTFDEC1_BASE      (STM32_AHB2_BASE + 0x000a5000)
#define STM32_OTFDEC2_BASE      (STM32_AHB2_BASE + 0x000a5400)
#define STM32_SDMMC1_BASE       (STM32_AHB2_BASE + 0x000a8000)
#define STM32_DLYBSD1_BASE      (STM32_AHB2_BASE + 0x000a8400)
#define STM32_DLYBSD2_BASE      (STM32_AHB2_BASE + 0x000a8800)
#define STM32_SDMMC2_BASE       (STM32_AHB2_BASE + 0x000a8c00)
#define STM32_DLYBOS1_BASE      (STM32_AHB2_BASE + 0x000af000)
#define STM32_DLYBOS2_BASE      (STM32_AHB2_BASE + 0x000af400)
#define STM32_FSMC_BASE         (STM32_AHB2_BASE + 0x000b0400)
#define STM32_FSMCR_BASE        (STM32_AHB2_BASE + 0x000b0400)
#define STM32_OCTOSPI1_BASE     (STM32_AHB2_BASE + 0x000b1400)
#define STM32_OCTOSPI1R_BASE    (STM32_AHB2_BASE + 0x000b1400)
#define STM32_OCTOSPI2_BASE     (STM32_AHB2_BASE + 0x000b2400)
#define STM32_OCTOSPI2R_BASE    (STM32_AHB2_BASE + 0x000b2400)
#define STM32_HSPI1_BASE         (STM32_AHB2_BASE + 0x000b3400)
#define STM32_HSPI1R_BASE        (STM32_AHB2_BASE + 0x000b3400)

/* APB3 Base Addresses ******************************************************/

#define STM32_SYSCFG_BASE       (STM32_APB3_BASE + 0x00000400)
#define STM32_SPI3_BASE         (STM32_APB3_BASE + 0x00002000)
#define STM32_LPUART1_BASE      (STM32_APB3_BASE + 0x00002400)
#define STM32_I2C3_BASE         (STM32_APB3_BASE + 0x00002800)
#define STM32_LPTIM1_BASE       (STM32_APB3_BASE + 0x00004400)
#define STM32_LPTIM3_BASE       (STM32_APB3_BASE + 0x00004800)
#define STM32_LPTIM4_BASE       (STM32_APB3_BASE + 0x00004c00)
#define STM32_OPAMP_BASE        (STM32_APB3_BASE + 0x00005000)
#define STM32_COMP_BASE         (STM32_APB3_BASE + 0x00005400)
#define STM32_VREFBUF_BASE      (STM32_APB3_BASE + 0x00007400)
#define STM32_RTC_BASE          (STM32_APB3_BASE + 0x00007800)
#define STM32U5_RTC_BASE        STM32_RTC_BASE
#define STM32_TAMP_BASE         (STM32_APB3_BASE + 0x00007c00)

/* AHB3 Base Addresses ******************************************************/

#define STM32_LPGPIO1_BASE      (STM32_AHB3_BASE + 0x00000000)
#define STM32_PWR_BASE          (STM32_AHB3_BASE + 0x00000800)
#define STM32_RCC_BASE          (STM32_AHB3_BASE + 0x00000c00)
#define STM32_ADC4_BASE         (STM32_AHB3_BASE + 0x00001000)
#define STM32_DAC1_BASE         (STM32_AHB3_BASE + 0x00001800)
#define STM32_EXTI_BASE         (STM32_AHB3_BASE + 0x00002000)
#define STM32_GTZC2_TZSC_BASE   (STM32_AHB3_BASE + 0x00003000)
#define STM32_GTZC2_TZIC_BASE   (STM32_AHB3_BASE + 0x00003400)
#define STM32_GTZC2_MPCBB4_BASE (STM32_AHB3_BASE + 0x00003800)
#define STM32_ADF1_BASE         (STM32_AHB3_BASE + 0x00004000)
#define STM32_LPDMA1_BASE       (STM32_AHB3_BASE + 0x00005000)

#endif /* __ARCH_ARM_SRC_STM32U5_STM32_MEMORYMAP_H */
