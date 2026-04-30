/****************************************************************************
 * arch/arm/src/ra8p/ra8p_wdt.h
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

#ifndef __ARCH_ARM_SRC_RA8P_RA8P_WDT_H
#define __ARCH_ARM_SRC_RA8P_RA8P_WDT_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

#include <stdint.h>
#include <stdbool.h>

#include "hardware/ra8p_wdt.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* WDT timeout range in milliseconds */
#define RA8P_WDT_MIN_TIMEOUT_MS          1      /* Minimum timeout */
#define RA8P_WDT_MAX_TIMEOUT_MS          32768  /* Maximum timeout based on 2^15 cycles */

/* WDT refresh sequences */
#define RA8P_WDT_REFRESH_SEQ1            0xAC
#define RA8P_WDT_REFRESH_SEQ2            0x53

/* Default timeout in milliseconds */
#define RA8P_WDT_DEFAULT_TIMEOUT_MS      1000   /* 1 second default */

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* WDT configuration structure */
struct ra8p_wdt_config_s
{
  uint32_t timeout_ms;              /* Timeout period in milliseconds */
  bool reset_enabled;               /* Enable reset on timeout */
  bool interrupt_only;              /* Interrupt-only mode (no reset) */
  uint8_t clock_divider;            /* Clock divider (0-15) */
  bool low_power_enable;            /* Enable low power mode */
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_wdt_initialize
 *
 * Description:
 *   Initialize the WDT based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   timeout_ms - Timeout period in milliseconds
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_initialize(uint32_t timeout_ms);

/****************************************************************************
 * Name: ra8p_wdt_enable
 *
 * Description:
 *   Enable the WDT based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_enable(void);

/****************************************************************************
 * Name: ra8p_wdt_disable
 *
 * Description:
 *   Disable the WDT based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_disable(void);

/****************************************************************************
 * Name: ra8p_wdt_refresh
 *
 * Description:
 *   Refresh the WDT based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_refresh(void);

/****************************************************************************
 * Name: ra8p_wdt_set_timeout
 *
 * Description:
 *   Set WDT timeout period based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   timeout_ms - Timeout period in milliseconds
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_set_timeout(uint32_t timeout_ms);

/****************************************************************************
 * Name: ra8p_wdt_is_enabled
 *
 * Description:
 *   Check if WDT is enabled based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if enabled, false otherwise
 *
 ****************************************************************************/

bool ra8p_wdt_is_enabled(void);

/****************************************************************************
 * Name: ra8p_wdt_is_overflow_occurred
 *
 * Description:
 *   Check if WDT overflow has occurred based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if overflow occurred, false otherwise
 *
 ****************************************************************************/

bool ra8p_wdt_is_overflow_occurred(void);

/****************************************************************************
 * Name: ra8p_wdt_clear_overflow_flag
 *
 * Description:
 *   Clear WDT overflow flag based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_clear_overflow_flag(void);

/****************************************************************************
 * Name: ra8p_wdt_get_timeout
 *
 * Description:
 *   Get current WDT timeout period based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Current timeout period in milliseconds
 *
 ****************************************************************************/

uint32_t ra8p_wdt_get_timeout(void);

/****************************************************************************
 * Name: ra8p_wdt_feed
 *
 * Description:
 *   Alias for wdt_refresh based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_feed(void);

/****************************************************************************
 * Name: ra8p_wdt_reset_count
 *
 * Description:
 *   Reset the watchdog counter based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_reset_count(void);

/****************************************************************************
 * Name: ra8p_wdt_get_reset_count
 *
 * Description:
 *   Get the number of resets caused by WDT timeout based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Reset count
 *
 ****************************************************************************/

int ra8p_wdt_get_reset_count(void);

#endif /* __ARCH_ARM_SRC_RA8P_RA8P_WDT_H */