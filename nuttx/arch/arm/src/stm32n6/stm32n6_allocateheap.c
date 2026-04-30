/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_allocateheap.c
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
#include <assert.h>
#include <debug.h>

#include <nuttx/arch.h>
#include <nuttx/board.h>
#include <nuttx/mm/mm.h>

#include "chip.h"
#include "arm_internal.h"

#ifdef CONFIG_BUILD_PROTECTED
#  include "stm32n6_userspace.h"
#endif

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define HEAP_BASE  ((uintptr_t)_ebss + CONFIG_IDLETHREAD_STACKSIZE)
#define HEAP_END   (CONFIG_RAM_START + CONFIG_RAM_SIZE)

/****************************************************************************
 * Public Functions
 ****************************************************************************/

void up_allocate_heap(FAR void **heap_start, size_t *heap_size)
{
  *heap_start = (FAR void *)HEAP_BASE;
  *heap_size  = HEAP_END - HEAP_BASE;
}

#ifdef CONFIG_ARCH_USE_MMU
void up_malloc_initialize(void)
{
  uintptr_t base;
  size_t size;

  base = HEAP_BASE;
  size = HEAP_END - HEAP_BASE;

  umm_initialize((FAR void *)base, size);
}
#endif

void board_autoled_on(int led)
{
}

void board_autoled_off(int led)
{
}
