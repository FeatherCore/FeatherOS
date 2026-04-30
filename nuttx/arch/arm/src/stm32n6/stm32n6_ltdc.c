/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_ltdc.c
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

#include <errno.h>

#include "chip.h"
#include "stm32n6_ltdc.h"

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32n6_ltdc_initialize
 ****************************************************************************/

int stm32n6_ltdc_initialize(void)
{
  return -ENOSYS;
}

/****************************************************************************
 * Name: stm32n6_ltdc_configure
 ****************************************************************************/

int stm32n6_ltdc_configure(struct stm32n6_ltdc_s *ltdc)
{
  return -ENOSYS;
}

/****************************************************************************
 * Name: stm32n6_ltdc_enable_layer
 ****************************************************************************/

int stm32n6_ltdc_enable_layer(int layer)
{
  return -ENOSYS;
}

/****************************************************************************
 * Name: stm32n6_ltdc_disable_layer
 ****************************************************************************/

int stm32n6_ltdc_disable_layer(int layer)
{
  return -ENOSYS;
}

/****************************************************************************
 * Name: stm32n6_ltdc_set_layer_format
 ****************************************************************************/

int stm32n6_ltdc_set_layer_format(int layer, uint32_t format)
{
  return -ENOSYS;
}

/****************************************************************************
 * Name: stm32n6_ltdc_set_layer_alpha
 ****************************************************************************/

int stm32n6_ltdc_set_layer_alpha(int layer, uint8_t alpha)
{
  return -ENOSYS;
}

/****************************************************************************
 * Name: stm32n6_ltdc_set_layer_address
 ****************************************************************************/

int stm32n6_ltdc_set_layer_address(int layer, uint32_t address)
{
  return -ENOSYS;
}

/****************************************************************************
 * Name: stm32n6_ltdc_set_layer_window
 ****************************************************************************/

int stm32n6_ltdc_set_layer_window(int layer, uint16_t hstart, uint16_t hstop,
                                  uint16_t vstart, uint16_t vstop)
{
  return -ENOSYS;
}

/****************************************************************************
 * Name: stm32n6_ltdc_reload_config
 ****************************************************************************/

void stm32n6_ltdc_reload_config(void)
{
}
