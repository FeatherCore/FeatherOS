/****************************************************************************
 * arch/arm/src/ra8p/ra8p_start.c
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

#include <nuttx/init.h>
#include <nuttx/arch.h>
#include <arch/barriers.h>

#include "arm_internal.h"
#include "nvic.h"
#include "ra8p_clockconfig.h"
#include "ra8p_lowsetup.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define HEAP_BASE  ((uintptr_t)_ebss + CONFIG_IDLETHREAD_STACKSIZE)

/****************************************************************************
 * Public Data
 ****************************************************************************/

/* g_idle_topstack: _sbss is the start of the BSS region as defined by the
 * linker script. _ebss lies at the end of the BSS region. The idle task
 * stack starts at the end of BSS and is of size CONFIG_IDLETHREAD_STACKSIZE.
 */

const uintptr_t g_idle_topstack = HEAP_BASE;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: showprogress
 *
 * Description:
 *   Print a character on the CONSOLE USART to show boot status.
 *
 ****************************************************************************/

#ifdef CONFIG_DEBUG_FEATURES
#  define showprogress(c)  arm_lowputc(c)
#else
#  define showprogress(c)
#endif

/****************************************************************************
 * Name: ra8p_dcache_enable
 ****************************************************************************/

#ifdef CONFIG_ARMV8M_DCACHE
static inline void ra8p_dcache_enable(void)
{
  uint32_t ccr;

  up_invalidate_dcache_all();

  ccr = getreg32(NVIC_CCR);
  ccr |= NVIC_CCR_DCACHE;
  putreg32(ccr, NVIC_CCR);

  ARM_DMB();
}
#else
#  define ra8p_dcache_enable()
#endif

/****************************************************************************
 * Name: ra8p_icache_enable
 ****************************************************************************/

#ifdef CONFIG_ARMV8M_ICACHE
static inline void ra8p_icache_enable(void)
{
  uint32_t ccr;

  ccr = getreg32(NVIC_CCR);
  ccr |= NVIC_CCR_ICACHE;
  putreg32(ccr, NVIC_CCR);

  ARM_ISB();
}
#else
#  define ra8p_icache_enable()
#endif

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: __start
 ****************************************************************************/

void __start(void)
{
#ifndef CONFIG_BUILD_PIC
  const uint32_t *src;
  uint32_t *dest;
#endif

  /* Clear .bss */
  for (dest = (uint32_t *)_sbss; dest < (uint32_t *)_ebss; )
    {
      *dest++ = 0;
    }

#ifndef CONFIG_BUILD_PIC
  /* Copy the initialized data section from FLASH to SRAM */
  for (src = (const uint32_t *)_eronly, dest = (uint32_t *)_sdata;
       dest < (uint32_t *)_edata;)
    {
      *dest++ = *src++;
    }
#endif

  /* Configure clocks and FPU */
  ra8p_clockconfig();
  arm_fpuconfig();

  /* Configure serial pins and early serial */
  ra8p_lowsetup();

  showprogress('A');

#ifdef USE_EARLYSERIALINIT
  arm_earlyserialinit();
#endif

  showprogress('B');

  /* Enable caches */
  ra8p_icache_enable();
  ra8p_dcache_enable();

  showprogress('C');

#ifdef CONFIG_BOARD_INITIALIZE
  ra8p_boardinitialize();
#endif

  showprogress('D');

  /* Start NuttX */
  showprogress('\r');
  showprogress('\n');

  nx_start();

  /* Shouldn't get here */
  for (; ; );
}
