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
 * Pre-processor Definitions
 ****************************************************************************/

/* RA8P Memory Map - Based on RA8P1 (R7KA8P1KFLCAC) */

/* Code Flash (ROM) */
#define RA8P_CODE_FLASH_BASE    0x00000000
#define RA8P_CODE_FLASH_SIZE    0x00400000

/* SRAM */
#define RA8P_SRAM_BASE          0x20000000
#define RA8P_SRAM_SIZE          0x00200000

/* CCM SRAM */
#define RA8P_CCMSRAM_BASE       0x21000000
#define RA8P_CCMSRAM_SIZE       0x00004000

/* System Control */
#define RA8P_SYSTEM_CONTROL_BASE  0x40010000
#define RA8P_SYSTEM_CONTROL_SIZE  0x00010000

/* SCI_B UART peripherals - 0x40070000 + n*0x1000 */
#define RA8P_SCI_B0_BASE        0x40070000
#define RA8P_SCI_B1_BASE        0x40071000
#define RA8P_SCI_B2_BASE        0x40072000
#define RA8P_SCI_B3_BASE        0x40073000
#define RA8P_SCI_B4_BASE        0x40074000

/* SPI_B peripherals */
#define RA8P_SPI_B0_BASE        0x40080000
#define RA8P_SPI_B1_BASE        0x40081000

/* I2C_B peripherals */
#define RA8P_I2C_B0_BASE        0x40050000
#define RA8P_I2C_B1_BASE        0x40051000

/* GPIO */
#define RA8P_GPIO0_BASE         0x40080000
#define RA8P_GPIO1_BASE         0x40081000
#define RA8P_GPIO2_BASE         0x40082000
#define RA8P_GPIO3_BASE         0x40083000
#define RA8P_GPIO4_BASE         0x40084000

/* Ethernet RMAC */
#define RA8P_ETHERNET_BASE      0x40110000

/* USB FS/HS */
#define RA8P_USBFS_BASE         0x40090000
#define RA8P_USBHS_BASE         0x40091000

/* LCD Controller (GLCDC) */
#define RA8P_LCD_BASE           0x400cc000

/* CRC */
#define RA8P_CRC_BASE           0x40012000

/* RTC */
#define RA8P_RTC_BASE           0x40041000

/* WDT */
#define RA8P_WDT_BASE           0x40043000

/* ICU (Interrupt Controller Unit) */
#define RA8P_ICU_BASE           0x40024000

/* DTC */
#define RA8P_DTC_BASE           0x40015000

/* GPT (General PWM Timer) */
#define RA8P_GPT0_BASE         0x40038000
#define RA8P_GPT1_BASE         0x40038100
#define RA8P_GPT2_BASE         0x40038200
#define RA8P_GPT3_BASE         0x40038300

/* OSTM (OS Timer) */
#define RA8P_OSTM0_BASE        0x40040000
#define RA8P_OSTM1_BASE        0x40040100
#define RA8P_OSTM2_BASE        0x40040200

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_MEMORYMAP_H */
