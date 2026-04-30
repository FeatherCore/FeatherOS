/****************************************************************************
 * boards/arm/stm32n6/stm32n6570-dk/src/stm32_xspi.c
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
#include <syslog.h>

#include <nuttx/fs/fs.h>
#include <nuttx/mtd/mtd.h>
#include <nuttx/spi/qspi.h>

#include "stm32n6_xspi.h"

#include "stm32n6570-dk.h"

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32_xspi_initialize
 ****************************************************************************/

int stm32_xspi_initialize(void)
{
  FAR struct qspi_dev_s *qspi;
  FAR struct mtd_dev_s *mtd;
  int ret;

  qspi = stm32n6_xspi_initialize(2);
  if (qspi == NULL)
    {
      syslog(LOG_ERR, "ERROR: Failed to initialize XSPI2\n");
      return -ENODEV;
    }

  mtd = mx66uw_initialize(qspi, true);
  if (mtd == NULL)
    {
      syslog(LOG_ERR, "ERROR: Failed to probe MX66UW on XSPI2\n");
      return -ENODEV;
    }

  ret = register_mtddriver("/dev/xspi2flash0", mtd, 0755, NULL);
  if (ret < 0)
    {
      syslog(LOG_ERR, "ERROR: register_mtddriver failed: %d\n", ret);
    }

  return ret;
}
