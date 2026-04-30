/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_can.c
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

#include <stdbool.h>
#include <errno.h>

#include "chip.h"
#include "stm32n6_can.h"

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32n6_can_initialize
 ****************************************************************************/

int stm32n6_can_initialize(uintptr_t canbase, uint32_t bitrate,
                           uint32_t dbitrate)
{
  return -ENOSYS;
}

/****************************************************************************
 * Name: stm32n6_caninitialize
 ****************************************************************************/

FAR struct can_dev_s *stm32n6_caninitialize(int port)
{
  (void)port;

  return NULL;
}

/****************************************************************************
 * Name: stm32n6_can_enable
 ****************************************************************************/

void stm32n6_can_enable(uintptr_t canbase)
{
}

/****************************************************************************
 * Name: stm32n6_can_disable
 ****************************************************************************/

void stm32n6_can_disable(uintptr_t canbase)
{
}

/****************************************************************************
 * Name: stm32n6_can_transmit
 ****************************************************************************/

int stm32n6_can_transmit(uintptr_t canbase, uint32_t id, bool extended,
                         bool fd, bool brs, uint8_t *data, uint8_t dlc)
{
  return -ENOSYS;
}

/****************************************************************************
 * Name: stm32n6_can_receive
 ****************************************************************************/

int stm32n6_can_receive(uintptr_t canbase, uint32_t *id, bool *extended,
                        uint8_t *data, uint8_t *dlc)
{
  return -ENOSYS;
}

/****************************************************************************
 * Name: stm32n6_can_available
 ****************************************************************************/

bool stm32n6_can_available(uintptr_t canbase)
{
  return false;
}

/****************************************************************************
 * Name: stm32n6_can_reset_error_counters
 ****************************************************************************/

void stm32n6_can_reset_error_counters(uintptr_t canbase)
{
}
