/****************************************************************************
 * arch/arm/src/ra8p/ra8p_usbphy.h
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

#ifndef __ARCH_ARM_SRC_RA8P_RA8P_USBPHY_H
#define __ARCH_ARM_SRC_RA8P_RA8P_USBPHY_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

#include <stdint.h>
#include <stdbool.h>

#include "hardware/ra8p_usbphy.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* USBPHY base address */
#define RA8P_USBPHY_BASE                 RA8P_USBPHY_BASE

/* USBPHY timeout in milliseconds */
#define RA8P_USBPHY_TIMEOUT_MS           100

/* USBPHY clock frequencies */
#define RA8P_USBPHY_48MHZ_CLK            48000000  /* 48MHz clock */
#define RA8P_USBPHY_24MHZ_CLK            24000000  /* 24MHz clock */

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* USBPHY configuration structure */
struct ra8p_usbphy_config_s
{
  bool hs_mode;                     /* High-speed mode enabled */
  bool fs_mode;                     /* Full-speed mode enabled */
  bool pll_external;                /* Use external PLL clock */
  bool pullup_enable;               /* Enable D+ pull-up */
  bool pulldown_enable;             /* Enable D- pull-down */
  uint32_t clock_frequency;         /* PHY clock frequency in Hz */
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_usbphy_initialize
 *
 * Description:
 *   Initialize the USBPHY controller based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_initialize(void);

/****************************************************************************
 * Name: ra8p_usbphy_enable
 *
 * Description:
 *   Enable the USBPHY controller based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_enable(void);

/****************************************************************************
 * Name: ra8p_usbphy_disable
 *
 * Description:
 *   Disable the USBPHY controller based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_disable(void);

/****************************************************************************
 * Name: ra8p_usbphy_set_mode
 *
 * Description:
 *   Set USBPHY mode (FS/HS) based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   hs_mode - true for high-speed, false for full-speed
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_set_mode(bool hs_mode);

/****************************************************************************
 * Name: ra8p_usbphy_set_pullup
 *
 * Description:
 *   Set USB D+ pull-up resistor based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_set_pullup(bool enable);

/****************************************************************************
 * Name: ra8p_usbphy_set_pulldown
 *
 * Description:
 *   Set USB D- pull-down resistor based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_set_pulldown(bool enable);

/****************************************************************************
 * Name: ra8p_usbphy_pll_enable
 *
 * Description:
 *   Enable USBPHY PLL based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_pll_enable(void);

/****************************************************************************
 * Name: ra8p_usbphy_pll_disable
 *
 * Description:
 *   Disable USBPHY PLL based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_pll_disable(void);

/****************************************************************************
 * Name: ra8p_usbphy_is_pll_locked
 *
 * Description:
 *   Check if USBPHY PLL is locked based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if PLL is locked, false otherwise
 *
 ****************************************************************************/

bool ra8p_usbphy_is_pll_locked(void);

/****************************************************************************
 * Name: ra8p_usbphy_reset
 *
 * Description:
 *   Reset the USBPHY controller based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_reset(void);

/****************************************************************************
 * Name: ra8p_usbphy_power_down
 *
 * Description:
 *   Put USBPHY in power-down mode based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   enable - true to enter power-down, false to exit
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_power_down(bool enable);

/****************************************************************************
 * Name: ra8p_usbphy_get_status
 *
 * Description:
 *   Get USBPHY status based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Status flags
 *
 ****************************************************************************/

uint8_t ra8p_usbphy_get_status(void);

#endif /* __ARCH_ARM_SRC_RA8P_RA8P_USBPHY_H */