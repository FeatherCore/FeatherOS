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

#include "chip.h"

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_wdt_initialize
 *
 * Description:
 *   Initialize the WDT driver
 *
 * Return Value:
 *   OK on success, negative error code on failure
 *
 ****************************************************************************/

int ra8p_wdt_initialize(void);

/****************************************************************************
 * Name: ra8p_wdt_enable
 *
 * Description:
 *   Enable the watchdog timer
 *
 * Return Value:
 *   OK on success, negative error code on failure
 *
 ****************************************************************************/

int ra8p_wdt_enable(void);

/****************************************************************************
 * Name: ra8p_wdt_disable
 *
 * Description:
 *   Disable the watchdog timer
 *
 * Return Value:
 *   OK on success, negative error code on failure
 *
 ****************************************************************************/

int ra8p_wdt_disable(void);

/****************************************************************************
 * Name: ra8p_wdt_feed
 *
 * Description:
 *   Feed (refresh) the watchdog timer
 *
 * Return Value:
 *   OK on success, negative error code on failure
 *
 ****************************************************************************/

int ra8p_wdt_feed(void);

/****************************************************************************
 * Name: ra8p_wdt_set_timeout
 *
 * Description:
 *   Set the watchdog timeout value
 *
 * Input Parameters:
 *   timeout_ms - Timeout value in milliseconds
 *
 * Return Value:
 *   OK on success, negative error code on failure
 *
 ****************************************************************************/

int ra8p_wdt_set_timeout(uint32_t timeout_ms);

#endif /* __ARCH_ARM_SRC_RA8P_RA8P_WDT_H */