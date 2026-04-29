/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_userspace.c
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

#include <stdint.h>
#include <string.h>

#ifdef CONFIG_BUILD_PROTECTED
#  include <nuttx/compiler.h>
#  include <nuttx/cache.h>
#endif

#include "arm_internal.h"
#include "stm32n6_userspace.h"

#ifdef CONFIG_BUILD_PROTECTED

/****************************************************************************
 * Public Data
 ****************************************************************************/

/* These 'addresses' of these values are setup by the linker script.  They are
 * not actual uint32_t storage locations! They are only used meaningfully in
 * the calculation of the initial DATA and BSS addresses.  The linker script
 * must guarantee that __data_load and __data_start are in the same memory
 * bank.
 */

extern uint32_t __data_load[];
extern uint32_t __data_start[];
extern uint32_t __data_end[];
extern uint32_t __bss_start[];
extern uint32_t __bss_end[];

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32n6_userspace
 *
 * Description:
 *   Setup up usermode-related special memory mappings.  This function is
 *   only available when CONFIG_BUILD_PROTECTED=y.
 *
 ****************************************************************************/

void stm32n6_userspace(void)
{
  /* TODO: Implement userspace memory setup for STM32N6 */
}

#endif /* CONFIG_BUILD_PROTECTED */
