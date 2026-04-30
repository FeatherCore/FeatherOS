/****************************************************************************
 * boards/arm/ra8p/ek-ra8p1/include/board.h
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

#ifndef __BOARDS_ARM_RA8P_EK_RA8P1_INCLUDE_BOARD_H
#define __BOARDS_ARM_RA8P_EK_RA8P1_INCLUDE_BOARD_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Clocking *****************************************************************/

/* This is the canonical configuration for RA8P1:
 *   System Clock source      : PLL (1 GHz)
 *   ICLK(Hz)                 : 200000000
 *   PCLKA(Hz)                : 120000000
 *   PCLKB(Hz)                : 60000000
 *   PCLKC(Hz)                : 60000000
 *   PCLKD(Hz)                : 60000000
 *   FCLK(Hz)                 : 200000000
 *   BCLK(Hz)                 : 60000000
 *   UCLK(Hz)                 : 48000000
 */

/* Clock selection HOCO, MOCO, LOCO, PLL */

#define RA_CKSEL  R_SYSTEM_SCKSCR_CKSEL_PLL

/* XTAL frequency (24MHz as per Zephyr board configuration) */
#define RA_XTAL_FREQUENCY  24000000

/* PLL settings for 1GHz CPU clock */
#define RA_PLL_MUL          50  /* 24MHz * 50 = 1200MHz VCO */
#define RA_PLL_DIV          2   /* 1200MHz / 2 = 600MHz CPU clock */
#define RA_PLL2_MUL         25  /* For PLL2 */

/* System clock division settings */
#define RA_ICK_DIV          R_SYSTEM_SCKDIVCR_ICK_DIV_3  /* 600/3 = 200MHz ICLK */
#define RA_ICLK_FREQUENCY   200000000
#define RA_FCK_DIV          R_SYSTEM_SCKDIVCR_FCK_DIV_2  /* 200/2 = 100MHz FCLK */
#define RA_FCK_FREQUENCY    100000000
#define RA_PCKA_DIV         R_SYSTEM_SCKDIVCR_PCKA_DIV_2 /* 200/2 = 100MHz PCLKA */
#define RA_PCKA_FREQUENCY   100000000
#define RA_PCKB_DIV         R_SYSTEM_SCKDIVCR_PCKB_DIV_4 /* 200/4 = 50MHz PCLKB */
#define RA_PCKB_FREQUENCY   50000000
#define RA_PCKC_DIV         R_SYSTEM_SCKDIVCR_PCKC_DIV_4 /* 200/4 = 50MHz PCLKC */
#define RA_PCKC_FREQUENCY   50000000
#define RA_PCKD_DIV         R_SYSTEM_SCKDIVCR_PCKD_DIV_4 /* 200/4 = 50MHz PCLKD */
#define RA_PCKD_FREQUENCY   50000000

/* USB clock settings */
#define RA_UCK_DIV          R_SYSTEM_SCKDIVCR3_UCK_DIV_10 /* 600/10 = 60MHz */
#define RA_UCK_FREQUENCY    60000000

/* Alternate function pin selections for UART/SCI */

/* UART2 - Console/NSH */
#define GPIO_SCI2_RX        GPIO_RXD2_MISO2_SCL2_2  /* P201 */
#define GPIO_SCI2_TX        GPIO_TXD2_MOSI2_SDA2_2  /* P200 */

/* UART8 - Secondary UART */
#define GPIO_SCI8_RX        GPIO_RXD8_MISO8_SCL8_2  /* P713 */
#define GPIO_SCI8_TX        GPIO_TXD8_MOSI8_SDA8_2  /* P712 */

/* UART9 - Secondary UART */
#define GPIO_SCI9_RX        GPIO_RXD9_MISO9_SCL9_2  /* P308 */
#define GPIO_SCI9_TX        GPIO_TXD9_MOSI9_SDA9_2  /* P309 */

/* LED pin selections */
/* On EK-RA8P1 board: LED1 (P600), LED2 (P303), LED3 (PA07) */
#define GPIO_LED1           (gpio_pinset_t){ PORT6, PIN0, (GPIO_OUTPUT | GPIO_OUTPUT_ONE) }   /* P600 */
#define GPIO_LED2           (gpio_pinset_t){ PORT3, PIN3, (GPIO_OUTPUT | GPIO_OUTPUT_ONE) }   /* P303 */
#define GPIO_LED3           (gpio_pinset_t){ PORTA, PIN7, (GPIO_OUTPUT | GPIO_OUTPUT_ONE) }   /* PA07 */

#define LED_DRIVER_PATH     "/dev/userleds"

/* LED index values for use with board_userled() */
#define BOARD_LED1          0
#define BOARD_LED2          1
#define BOARD_LED3          2
#define BOARD_NLEDS         3

/* LED bits for use with board_userled_all() */
#define BOARD_LED1_BIT      (1 << BOARD_LED1)
#define BOARD_LED2_BIT      (1 << BOARD_LED2)
#define BOARD_LED3_BIT      (1 << BOARD_LED3)

/* These LEDs are not used by the board port unless CONFIG_ARCH_LEDS is
 * defined.  In that case, the usage by the board port is defined in
 * include/board.h and src/ek-ra8p1_leds.c. The LEDs are used to encode
 * OS-related events as follows:
 *
 *  SYMBOL                MEANING                         LED STATE
 *                                                         LED1  LED2  LED3
 *  -------------------  --------------------------      ----- ----- -----
 *  LED_STARTED          NuttX has been started           OFF   OFF   OFF
 *  LED_HEAPALLOCATE     Heap has been allocated          OFF   OFF   OFF
 *  LED_IRQSENABLED      Interrupts enabled               OFF   OFF   OFF
 *  LED_STACKCREATED     Idle stack created                ON   OFF   OFF
 *  LED_INIRQ            In an interrupt                 N/C   GLOW  OFF
 *  LED_SIGNAL           In a signal handler             N/C   GLOW  OFF
 *  LED_ASSERTION        An assertion failed               ON   GLOW  OFF
 *  LED_PANIC            The system has crashed           N/C   N/C   Blinking
 */

#define LED_STARTED         0
#define LED_HEAPALLOCATE    0
#define LED_IRQSENABLED     0
#define LED_STACKCREATED    1
#define LED_INIRQ           2
#define LED_SIGNAL          2
#define LED_ASSERTION       2
#define LED_PANIC           3

/* Button/switch selections */
#define GPIO_SW1            (gpio_pinset_t){ PORT0, PIN9, (GPIO_INPUT | GPIO_PULLUP) } /* P009 */
#define GPIO_SW2            (gpio_pinset_t){ PORT0, PIN8, (GPIO_INPUT | GPIO_PULLUP) } /* P008 */

/* Switch index values for use with board_userswitch() */
#define BUTTON_SW1          0
#define BUTTON_SW2          1
#define NUM_BUTTONS         2

/* USB Pins */
#define GPIO_USB_FS_VBUS    GPIO_USB_FS_VBUS_2    /* P407 */
#define GPIO_USB_FS_DP      GPIO_USB_FS_DP_2      /* P814 */
#define GPIO_USB_FS_DM      GPIO_USB_FS_DM_2      /* P815 */
#define GPIO_USB_HS_VBUS    GPIO_USB_HS_VBUS_2    /* P408 */
#define GPIO_USB_HS_PWREN   GPIO_USB_HS_PWREN_2   /* P408 */
#define GPIO_USB_HS_DP      GPIO_USB_HS_DP_2      /* P400 */
#define GPIO_USB_HS_DM      GPIO_USB_HS_DM_2      /* P401 */

#endif /* __BOARDS_ARM_RA8P_EK_RA8P1_INCLUDE_BOARD_H */