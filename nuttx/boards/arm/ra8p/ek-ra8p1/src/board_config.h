/****************************************************************************
 * boards/arm/ra8p/ek-ra8p1/src/board_config.h
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

#ifndef __BOARDS_ARM_RA8P_EK_RA8P1_SRC_BOARD_CONFIG_H
#define __BOARDS_ARM_RA8P_EK_RA8P1_SRC_BOARD_CONFIG_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <arch/ra8p/chip.h>

#include "ra8p_gpio.h"
#include "ra8p_userswitch.h"
#include "ra8p_userled.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Configuration ************************************************************/

/* How many SCKDIVCR dividers to configure */

#if defined(CONFIG_ARCH_FAMILY_RA8M1) || defined(CONFIG_ARCH_FAMILY_RA8D1) || \
    defined(CONFIG_ARCH_FAMILY_RA8M2) || defined(CONFIG_ARCH_FAMILY_RA8P1)
#  define SCKDIVCR_DIVIDER_MAX  7
#elif defined(CONFIG_ARCH_FAMILY_RA8M3)
#  define SCKDIVCR_DIVIDER_MAX  6
#else
#  error "Unknown chip family for SCKDIVCR divider"
#endif

/* LED Configuration */

#ifdef CONFIG_ARCH_LEDS
#  define LED_STARTED         0  /* LED off */
#  define LED_HEAPALLOCATE    1  /* LED off */
#  define LED_IRQSENABLED     2  /* LED on for 1s */
#  define LED_STACKCREATED    3  /* LED on for 1s then off */
#endif

/* Can't support SDRAM features if SDRAM is not enabled */

#ifndef CONFIG_RA8P_SDRAM
#  undef CONFIG_RA8P_SDRAM_DMA
#endif

/* Clock Configuration */

#define RA_XTAL_FREQUENCY     24000000  /* 24MHz external crystal */

/* System Clock Dividers */

#define RA_ICLK_DIV           3         /* Divide by 3 for 200MHz */
#define RA_FCLK_DIV           2         /* Divide by 2 for 100MHz */
#define RA_PCLKA_DIV          2         /* Divide by 2 for 100MHz */
#define RA_PCLKB_DIV          4         /* Divide by 4 for 50MHz */
#define RA_PCLKC_DIV          4         /* Divide by 4 for 50MHz */
#define RA_PCLKD_DIV          4         /* Divide by 4 for 50MHz */

/****************************************************************************
 * Public Data
 ****************************************************************************/

#ifndef __ASSEMBLY__

#undef EXTERN
#if defined(__cplusplus)
#define EXTERN extern "C"
extern "C"
{
#else
#define EXTERN extern
#endif

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_bringup
 *
 * Description:
 *   Perform architecture-specific initialization
 *
 ****************************************************************************/

int ra8p_bringup(void);

/****************************************************************************
 * Name: ra8p_boardinitialize
 *
 * Description:
 *   All RA8P architectures must provide the following entry point.  This
 *   entry point is called early in the initialization -- after all memory
 *   has been configured and mapped but before any devices have been
 *   initialized.
 *
 ****************************************************************************/

void ra8p_boardinitialize(void);

/****************************************************************************
 * Name: ra8p_userswitch_initialize
 *
 * Description:
 *   Initialize the user switches
 *
 ****************************************************************************/

uint32_t board_userswitch_initialize(void);

/****************************************************************************
 * Name: board_userswitch
 *
 * Description:
 *   Return the state of a single user switch
 *
 ****************************************************************************/

bool board_userswitch(int id);

/****************************************************************************
 * Name: board_userswitch_all
 *
 * Description:
 *   Return the state of all user switches
 *
 ****************************************************************************/

uint32_t board_userswitch_all(void);

/****************************************************************************
 * Name: board_userled_initialize
 *
 * Description:
 *   Initialize the user LEDs
 *
 ****************************************************************************/

uint32_t board_userled_initialize(void);

/****************************************************************************
 * Name: board_userled
 *
 * Description:
 *   Set the state of a single user LED
 *
 ****************************************************************************/

void board_userled(int led, bool ledon);

/****************************************************************************
 * Name: board_userled_all
 *
 * Description:
 *   Set the state of all user LEDs
 *
 ****************************************************************************/

void board_userled_all(uint32_t ledset);

#ifdef CONFIG_BOARD_LATE_INITIALIZE
/****************************************************************************
 * Name: board_late_initialize
 *
 * Description:
 *   If CONFIG_BOARD_LATE_INITIALIZE is selected, then an additional
 *   initialization call will be performed in the boot-up sequence to a
 *   function called board_late_initialize().  board_late_initialize() will
 *   be called immediately after up_initialize().  If CONFIG_BOARD_LATE_INITIALIZE
 *   is not selected, then the board_app_initialize() must be called from
 *   the IDLE thread at the beginning of the application.
 *
 ****************************************************************************/

void board_late_initialize(void);
#endif

#ifdef CONFIG_BOARDCTL_RESET
/****************************************************************************
 * Name: board_reset
 *
 * Description:
 *   Reset the board.  This function may or may not perform the actual
 *   reset operation.  In any event, the board will be left in a state
 *   as-if a reset occurred.
 *
 ****************************************************************************/

void board_reset(void);
#endif

#undef EXTERN
#if defined(__cplusplus)
}
#endif

#endif /* __ASSEMBLY__ */
#endif /* __BOARDS_ARM_RA8P_EK_RA8P1_SRC_BOARD_CONFIG_H */