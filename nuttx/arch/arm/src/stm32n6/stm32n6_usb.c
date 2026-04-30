/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_usb.c
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

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

#include <sys/types.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>

#include "arm_internal.h"
#include "chip.h"
#include "hardware/stm32n6_memorymap.h"
#include "hardware/stm32n6_rcc.h"
#include "stm32n6_usb.h"

/****************************************************************************
 * Public Functions
 ****************************************************************************/

int stm32n6_usb_initialize(uintptr_t otgbase)
{
  uint32_t regval;

  /* Enable USB OTG HS clock */

  regval = getreg32(STM32_RCC_AHB5ENR);
  regval |= RCC_AHB5ENR_USB1OTGEN;
  putreg32(regval, STM32_RCC_AHB5ENR);

  /* Configure USB PHY */

  regval = getreg32(STM32_RCC_AHB5ENR);
  regval |= RCC_AHB5ENR_USBPHYC1EN;
  putreg32(regval, STM32_RCC_AHB5ENR);

  return OK;
}

void stm32n6_usb_enable(uintptr_t otgbase)
{
  /* Enable USB clock and power */
}

void stm32n6_usb_disable(uintptr_t otgbase)
{
  /* Disable USB clock and power */
}
