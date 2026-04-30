/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_pinctrl.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_PINCTRL_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_PINCTRL_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "ra8p_memorymap.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* PFS (Port Function Select) Register Offsets */
#define RA8P_PFS_PNPCR_OFFSET(port)  (((port) < 8) ? 0x20 : 0x60)
#define RA8P_PFS_PNPCR_OFFSET(port)  (((port) < 8) ? 0x24 : 0x64)

/* PFS Register bit definitions */
#define RA8P_PFS_MASK              0x9FFFFFFF
#define RA8P_PFS_PSEL_SHIFT        24
#define RA8P_PFS_PSEL_MASK         (0x1F << 24)
#define RA8P_PFS_PSEL(n)           ((n) << 24)

#define RA8P_PFS_ISEL              (1 << 22)
#define RA8P_PFS_ODEN              (1 << 21)
#define RA8P_PFS_PUEN              (1 << 20)
#define RA8P_PFS_PDEN              (1 << 19)
#define RA8P_PFS_DS_SHIFT          16
#define RA8P_PFS_DS_MASK           (3 << 16)
#define RA8P_PFS_DS(n)             ((n) << 16)

#define RA8P_PFS_DS_LOW            0
#define RA8P_PFS_DS_MEDIUM         1
#define RA8P_PFS_DS_HIGH           2
#define RA8P_PFS_DS_HIGHSPEED      3

#define RA8P_PFS_EID               (1 << 15)
#define RA8P_PFS_NODER             (1 << 13)
#define RA8P_PFS_ASEL              (1 << 9)
#define RA8P_PFS_PMC               (1 << 8)

/* Pin Assignment Selection Values - Based on Zephyr RA8P1 pinctrl definitions */
#define RA8P_PSEL_SCI_8            0x01
#define RA8P_PSEL_SCI_9            0x02
#define RA8P_PSEL_SPI              0x05
#define RA8P_PSEL_I2C              0x0B
#define RA8P_PSEL_GPT1             0x01
#define RA8P_PSEL_USBFS            0x01
#define RA8P_PSEL_USBHS            0x01
#define RA8P_PSEL_CEU              0x01
#define RA8P_PSEL_BUS              0x01

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_pinctrl_configure
 *
 * Description:
 *   Configure a pin with the given function.
 *
 * Input Parameters:
 *   port - GPIO port number (0-13)
 *   pin  - Pin number (0-15)
 *   psel - Peripheral select value
 *
 ****************************************************************************/

void ra8p_pinctrl_configure(uint8_t port, uint8_t pin, uint8_t psel);

/****************************************************************************
 * Name: ra8p_pinctrl_set_drive_strength
 *
 * Description:
 *   Set the drive strength for a pin.
 *
 * Input Parameters:
 *   port - GPIO port number (0-13)
 *   pin  - Pin number (0-15)
 *   ds   - Drive strength (0=low, 1=medium, 2=high, 3=high-speed)
 *
 ****************************************************************************/

void ra8p_pinctrl_set_drive_strength(uint8_t port, uint8_t pin, uint8_t ds);

/****************************************************************************
 * Name: ra8p_pinctrl_enable_pull
 *
 * Description:
 *   Enable pull-up or pull-down for a pin.
 *
 * Input Parameters:
 *   port - GPIO port number (0-13)
 *   pin  - Pin number (0-15)
 *   pupd - 0=none, 1=pull-up, 2=pull-down
 *
 ****************************************************************************/

void ra8p_pinctrl_enable_pull(uint8_t port, uint8_t pin, uint8_t pupd);

/****************************************************************************
 * Name: ra8p_pinctrl_enable_open_drain
 *
 * Description:
 *   Enable open-drain mode for a pin.
 *
 * Input Parameters:
 *   port - GPIO port number (0-13)
 *   pin  - Pin number (0-15)
 *   enable - true to enable, false to disable
 *
 ****************************************************************************/

void ra8p_pinctrl_enable_open_drain(uint8_t port, uint8_t pin, bool enable);

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_PINCTRL_H */