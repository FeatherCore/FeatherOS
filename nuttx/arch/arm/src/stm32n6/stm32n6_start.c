/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_start.c
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
#include <assert.h>
#include <debug.h>

#include <nuttx/cache.h>
#include <nuttx/init.h>
#include <arch/barriers.h>
#include <arch/board/board.h>

#include "arm_internal.h"
#include "nvic.h"
#include "mpu.h"

#include "chip.h"
#include "stm32n6_start.h"
#include "stm32n6_lowsetup.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define HEAP_BASE  ((uintptr_t)_ebss + CONFIG_IDLETHREAD_STACKSIZE)

/****************************************************************************
 * Public Data
 ****************************************************************************/

const uintptr_t g_idle_topstack = HEAP_BASE;

/****************************************************************************
 * Private Function prototypes
 ****************************************************************************/

#ifdef CONFIG_DEBUG_FEATURES
#  define showprogress(c) arm_lowputc(c)
#else
#  define showprogress(c)
#endif

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Public Functions
 ****************************************************************************/

#ifdef CONFIG_ARMV8M_STACKCHECK
void __start(void) noinstrument_function;
#endif

/****************************************************************************
 * Name: __start
 *
 * Description:
 *   This is the reset entry point.
 *
 ****************************************************************************/

void __start(void)
{
  const uint32_t *src;
  uint32_t *dest;

#ifdef CONFIG_ARMV8M_STACKCHECK
  __asm__ volatile("sub r10, sp, %0" : :
                   "r"(CONFIG_IDLETHREAD_STACKSIZE - 64) :);
#endif

  mpu_early_reset();

  showprogress('A');

  stm32n6_clockconfig();
  arm_fpuconfig();
  stm32n6_lowsetup();
  showprogress('B');

  for (dest = (uint32_t *)_START_BSS; dest < (uint32_t *)_END_BSS; )
    {
      *dest++ = 0;
    }

  showprogress('C');

  for (src = (const uint32_t *)_DATA_INIT,
       dest = (uint32_t *)_START_DATA; dest < (uint32_t *)_END_DATA;
      )
    {
      *dest++ = *src++;
    }

  showprogress('D');

#ifdef CONFIG_ARCH_RAMFUNCS
  for (src = (const uint32_t *)_framfuncs,
       dest = (uint32_t *)_sramfuncs; dest < (uint32_t *)_eramfuncs;
      )
    {
      *dest++ = *src++;
    }
#endif

#ifdef CONFIG_ARMV8M_STACKCHECK
  arm_stack_check_init();
#endif

#ifdef CONFIG_ARCH_PERF_EVENTS
  up_perf_init((void *)STM32N6_SYSCLK_FREQUENCY);
#endif

#ifdef CONFIG_ARMV8M_ITMSYSLOG
  itm_syslog_initialize();
#endif

  stm32n6_soc_early_init();

#ifdef USE_EARLYSERIALINIT
  arm_earlyserialinit();
#endif
  showprogress('E');

#ifdef CONFIG_BUILD_PROTECTED
  stm32n6_userspace();
  showprogress('F');
#endif

  stm32_boardinitialize();
  showprogress('G');

#ifdef CONFIG_ARMV8M_ICACHE
  up_enable_icache();
#endif
#ifdef CONFIG_ARMV8M_DCACHE
  up_enable_dcache();
#endif
  showprogress('H');

  showprogress('\r');
  showprogress('\n');

  nx_start();

#ifndef CONFIG_DISABLE_IDLE_LOOP
  for (; ; );
#endif
}
