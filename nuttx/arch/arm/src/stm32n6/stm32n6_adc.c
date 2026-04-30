/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_adc.c
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
#include "stm32n6_adc.h"

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32n6_adc_initialize
 ****************************************************************************/

int stm32n6_adc_initialize(void)
{
  return -ENOSYS;
}

/****************************************************************************
 * Name: stm32n6_adcinitialize
 ****************************************************************************/

FAR struct adc_dev_s *stm32n6_adcinitialize(int intf,
                                            FAR const uint8_t *chanlist,
                                            int nchannels)
{
  (void)intf;
  (void)chanlist;
  (void)nchannels;

  return NULL;
}
