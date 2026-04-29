/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_usb.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_STM32N6_USB_H
#define __ARCH_ARM_SRC_STM32N6_STM32N6_USB_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <stdint.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* USB register offsets */
#define USB_OTG_GOTGCTL_OFFSET     0x000
#define USB_OTG_GOTGINT_OFFSET     0x004
#define USB_OTG_GAHBCFG_OFFSET     0x008
#define USB_OTG_GUSBCFG_OFFSET     0x00C
#define USB_OTG_GRSTCTL_OFFSET     0x010
#define USB_OTG_GINTSTS_OFFSET     0x014
#define USB_OTG_GINTMSK_OFFSET     0x018
#define USB_OTG_GRXSTSR_OFFSET     0x01C
#define USB_OTG_GRXSTSP_OFFSET     0x020
#define USB_OTG_GRXFIFO_OFFSET     0x024
#define USB_OTG_GNPTXFSIZ_OFFSET   0x028
#define USB_OTG_GNPTXSTS_OFFSET    0x02C
#define USB_OTG_DIEPCTL_OFFSET     0x140
#define USB_OTG_DOEPCTL_OFFSET     0x540

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

int stm32n6_usb_initialize(uintptr_t otgbase);
void stm32n6_usb_enable(uintptr_t otgbase);
void stm32n6_usb_disable(uintptr_t otgbase);

#endif /* __ARCH_ARM_SRC_STM32N6_STM32N6_USB_H */