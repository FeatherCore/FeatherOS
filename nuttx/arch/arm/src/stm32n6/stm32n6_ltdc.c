/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6 ltdc.c
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
#include <debug.h>

#include <nuttx/arch.h>
#include <nuttx/irq.h>
#include <nuttx/kmalloc.h>

#include "arm_internal.h"
#include "chip.h"
#include "hardware/stm32n6_memorymap.h"
#include "hardware/stm32n6_rcc.h"
#include "stm32n6_gpio.h"
#include "stm32n6_ltdc.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* LTDC register access macros */

#define ltdc_getreg32(offset)        getreg32(STM32_LTDC_BASE + (offset))
#define ltdc_putreg32(val, offset)   putreg32(val, STM32_LTDC_BASE + (offset))

/* Register bit manipulation */

#define LTDC_GCR_POLARITY_MASK       (LTDC_GCR_PCPOL | LTDC_GCR_DEPOL | LTDC_GCR_VSPOL | LTDC_GCR_HSPOL)

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct stm32n6_ltdc_s g_ltdc;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static void ltdc_wait_sof(void)
{
  /* Wait for start of frame */
  while ((ltdc_getreg32(STM32_LTDC_CDSR_OFFSET) & (1 << 1)) == 0);
}

static void ltdc_enable_clock(void)
{
  uint32_t regval;

  /* Enable LTDC clock */
  regval = getreg32(STM32_RCC_APB5ENR);
  regval |= RCC_APB5ENR_LTDCEN;
  putreg32(regval, STM32_RCC_APB5ENR);

  /* Wait for clock to be enabled */
  while (!(getreg32(STM32_RCC_APB5ENR) & RCC_APB5ENR_LTDCEN));
}

static void ltdc_enable_irq(uint32_t irqmask)
{
  uint32_t regval;

  regval = ltdc_getreg32(STM32_LTDC_IER_OFFSET);
  regval |= irqmask;
  ltdc_putreg32(regval, STM32_LTDC_IER_OFFSET);
}

static void ltdc_disable_irq(uint32_t irqmask)
{
  uint32_t regval;

  regval = ltdc_getreg32(STM32_LTDC_IER_OFFSET);
  regval &= ~irqmask;
  ltdc_putreg32(regval, STM32_LTDC_IER_OFFSET);
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

int stm32n6_ltdc_initialize(void)
{
  uint32_t regval;

  ltdcdbg("Initializing STM32N6 LTDC\n");

  /* Enable LTDC clock */
  ltdc_enable_clock();

  /* Reset LTDC */
  regval = getreg32(STM32_RCC_APB5RSTR);
  regval |= RCC_APB5RSTR_LTDCRST;
  putreg32(regval, STM32_RCC_APB5RSTR);

  regval &= ~RCC_APB5RSTR_LTDCRST;
  putreg32(regval, STM32_RCC_APB5RSTR);

  /* Initialize global variables */
  memset(&g_ltdc, 0, sizeof(struct stm32n6_ltdc_s));
  g_ltdc.base = STM32_LTDC_BASE;

  return OK;
}

int stm32n6_ltdc_configure(struct stm32n6_ltdc_s *ltdc)
{
  uint32_t regval;

  ltdcdbg("Configuring LTDC\n");

  /* Disable LTDC during configuration */
  regval = ltdc_getreg32(STM32_LTDC_GCR_OFFSET);
  regval &= ~LTDC_GCR_LTDCEN;
  ltdc_putreg32(regval, STM32_LTDC_GCR_OFFSET);

  /* Configure synchronization parameters */
  regval = ((ltdc->hsync - 1) << LTDC_SSCR_HSW_SHIFT) |
           ((ltdc->vsync - 1) << LTDC_SSCR_VSH_SHIFT);
  ltdc_putreg32(regval, STM32_LTDC_SSCR_OFFSET);

  /* Configure back porch */
  regval = ((ltdc->hbp - 1) << LTDC_BPCR_AHBP_SHIFT) |
           ((ltdc->vbp - 1) << LTDC_BPCR_AVBP_SHIFT);
  ltdc_putreg32(regval, STM32_LTDC_BPCR_OFFSET);

  /* Configure active width */
  regval = ((ltdc->width - 1) << LTDC_AWCR_AAW_SHIFT) |
           ((ltdc->height - 1) << LTDC_AWCR_AAH_SHIFT);
  ltdc_putreg32(regval, STM32_LTDC_AWCR_OFFSET);

  /* Configure total width */
  regval = ((ltdc->width + ltdc->hbp + ltdc->hfp - 1) << LTDC_TWCR_TOTALW_SHIFT) |
           ((ltdc->height + ltdc->vbp + ltdc->vfp - 1) << LTDC_TWCR_TOTALH_SHIFT);
  ltdc_putreg32(regval, STM32_LTDC_TWCR_OFFSET);

  /* Configure global control register */
  regval = ltdc_getreg32(STM32_LTDC_GCR_OFFSET);
  regval &= ~LTDC_GCR_POLARITY_MASK;

  /* Set signal polarities based on display requirements */
  if (ltdc->hsync < 0) regval |= LTDC_GCR_HSPOL;  /* Active high */
  if (ltdc->vsync < 0) regval |= LTDC_GCR_VSPOL;  /* Active high */
  if (ltdc->depol < 0)  regval |= LTDC_GCR_DEPOL;  /* Active high */
  if (ltdc->pcpol < 0)  regval |= LTDC_GCR_PCPOL;  /* Active high */

  ltdc_putreg32(regval, STM32_LTDC_GCR_OFFSET);

  /* Configure background color */
  ltdc_putreg32(0x00000000, STM32_LTDC_BCCR_OFFSET);  /* Black background */

  /* Configure layer properties */
  for (int i = 0; i < LTDC_MAX_LAYERS; i++)
    {
      if (ltdc->layers[i].framebuff != 0)
        {
          /* Configure layer */
          stm32n6_ltdc_set_layer_format(i, ltdc->layers[i].pf);
          stm32n6_ltdc_set_layer_alpha(i, ltdc->layers[i].alpha);
          stm32n6_ltdc_set_layer_address(i, ltdc->layers[i].framebuff);
          stm32n6_ltdc_set_layer_window(i, ltdc->layers[i].hoffset,
                                       ltdc->layers[i].hoffset + ltdc->layers[i].hspan,
                                       ltdc->layers[i].voffset,
                                       ltdc->layers[i].voffset + ltdc->layers[i].vspan);
        }
    }

  /* Enable LTDC */
  regval = ltdc_getreg32(STM32_LTDC_GCR_OFFSET);
  regval |= LTDC_GCR_LTDCEN;
  ltdc_putreg32(regval, STM32_LTDC_GCR_OFFSET);

  /* Reload shadow registers */
  ltdc_reload_config();

  return OK;
}

int stm32n6_ltdc_enable_layer(int layer)
{
  uint32_t regval;

  if (layer < 0 || layer >= LTDC_MAX_LAYERS)
    {
      return -EINVAL;
    }

  ltdcdbg("Enabling LTDC layer %d\n", layer);

  /* Get layer control register */
  if (layer == 0)
    {
      regval = ltdc_getreg32(STM32_LTDC_L1CR_OFFSET);
      regval |= LTDC_LxCR_LEN;
      ltdc_putreg32(regval, STM32_LTDC_L1CR_OFFSET);
    }
  else
    {
      regval = ltdc_getreg32(STM32_LTDC_L2CR_OFFSET);
      regval |= LTDC_LxCR_LEN;
      ltdc_putreg32(regval, STM32_LTDC_L2CR_OFFSET);
    }

  /* Reload shadow registers */
  ltdc_reload_config();

  return OK;
}

int stm32n6_ltdc_disable_layer(int layer)
{
  uint32_t regval;

  if (layer < 0 || layer >= LTDC_MAX_LAYERS)
    {
      return -EINVAL;
    }

  ltdcdbg("Disabling LTDC layer %d\n", layer);

  /* Get layer control register */
  if (layer == 0)
    {
      regval = ltdc_getreg32(STM32_LTDC_L1CR_OFFSET);
      regval &= ~LTDC_LxCR_LEN;
      ltdc_putreg32(regval, STM32_LTDC_L1CR_OFFSET);
    }
  else
    {
      regval = ltdc_getreg32(STM32_LTDC_L2CR_OFFSET);
      regval &= ~LTDC_LxCR_LEN;
      ltdc_putreg32(regval, STM32_LTDC_L2CR_OFFSET);
    }

  /* Reload shadow registers */
  ltdc_reload_config();

  return OK;
}

int stm32n6_ltdc_set_layer_format(int layer, uint32_t format)
{
  uint32_t regval;

  if (layer < 0 || layer >= LTDC_MAX_LAYERS)
    {
      return -EINVAL;
    }

  /* Configure pixel format */
  if (layer == 0)
    {
      regval = ltdc_getreg32(STM32_LTDC_L1PFCR_OFFSET);
      regval &= ~LTDC_LxPFCR_PF_MASK;
      regval |= (format & LTDC_LxPFCR_PF_MASK);
      ltdc_putreg32(regval, STM32_LTDC_L1PFCR_OFFSET);
    }
  else
    {
      regval = ltdc_getreg32(STM32_LTDC_L2PFCR_OFFSET);
      regval &= ~LTDC_LxPFCR_PF_MASK;
      regval |= (format & LTDC_LxPFCR_PF_MASK);
      ltdc_putreg32(regval, STM32_LTDC_L2PFCR_OFFSET);
    }

  return OK;
}

int stm32n6_ltdc_set_layer_alpha(int layer, uint8_t alpha)
{
  uint32_t regval;

  if (layer < 0 || layer >= LTDC_MAX_LAYERS)
    {
      return -EINVAL;
    }

  /* Set constant alpha value */
  if (layer == 0)
    {
      regval = ltdc_getreg32(STM32_LTDC_L1CACR_OFFSET);
      regval &= ~0xFF;
      regval |= alpha;
      ltdc_putreg32(regval, STM32_LTDC_L1CACR_OFFSET);
    }
  else
    {
      regval = ltdc_getreg32(STM32_LTDC_L2CACR_OFFSET);
      regval &= ~0xFF;
      regval |= alpha;
      ltdc_putreg32(regval, STM32_LTDC_L2CACR_OFFSET);
    }

  return OK;
}

int stm32n6_ltdc_set_layer_address(int layer, uint32_t address)
{
  if (layer < 0 || layer >= LTDC_MAX_LAYERS)
    {
      return -EINVAL;
    }

  /* Set frame buffer address */
  if (layer == 0)
    {
      ltdc_putreg32(address, STM32_LTDC_L1CFBAR_OFFSET);
    }
  else
    {
      ltdc_putreg32(address, STM32_LTDC_L2CFBAR_OFFSET);
    }

  return OK;
}

int stm32n6_ltdc_set_layer_window(int layer, uint16_t hstart, uint16_t hstop,
                                  uint16_t vstart, uint16_t vstop)
{
  uint32_t regval;

  if (layer < 0 || layer >= LTDC_MAX_LAYERS)
    {
      return -EINVAL;
    }

  /* Set horizontal position */
  regval = ((hstop - 1) << 16) | (hstart & 0xFFFF);
  if (layer == 0)
    {
      ltdc_putreg32(regval, STM32_LTDC_L1WHPCR_OFFSET);
    }
  else
    {
      ltdc_putreg32(regval, STM32_LTDC_L2WHPCR_OFFSET);
    }

  /* Set vertical position */
  regval = ((vstop - 1) << 16) | (vstart & 0xFFFF);
  if (layer == 0)
    {
      ltdc_putreg32(regval, STM32_LTDC_L1WVPCR_OFFSET);
    }
  else
    {
      ltdc_putreg32(regval, STM32_LTDC_L2WVPCR_OFFSET);
    }

  return OK;
}

void stm32n6_ltdc_reload_config(void)
{
  uint32_t regval;

  /* Request shadow reload */
  regval = ltdc_getreg32(STM32_LTDC_SRCR_OFFSET);
  regval |= LTDC_SRCR_IMR;  /* Immediate reload */
  ltdc_putreg32(regval, STM32_LTDC_SRCR_OFFSET);
}

/* Interrupt handler */
int ltdc_interrupt(int irq, void *context, void *arg)
{
  uint32_t isr = ltdc_getreg32(STM32_LTDC_ISR_OFFSET);
  uint32_t pending = isr & ltdc_getreg32(STM32_LTDC_IER_OFFSET);

  /* Clear interrupts */
  ltdc_putreg32(pending, STM32_LTDC_ICR_OFFSET);

  if (pending & (1 << 0))  /* Line interrupt */
    {
      ltdcdbg("LTDC Line Interrupt\n");
    }

  if (pending & (1 << 1))  /* FIFO underrun */
    {
      ltdcerr("LTDC FIFO Underrun Error\n");
    }

  if (pending & (1 << 2))  /* Transfer error */
    {
      ltdcerr("LTDC Transfer Error\n");
    }

  if (pending & (1 << 3))  /* Register reload */
    {
      ltdcdbg("LTDC Register Reload Complete\n");
    }

  return OK;
}