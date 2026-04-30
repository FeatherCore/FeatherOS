/****************************************************************************
 * arch/arm/src/ra8p/ra8p_hwinfo.c
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
#include <nuttx/arch.h>
#include <nuttx/kmalloc.h>
#include <nuttx/mutex.h>
#include <string.h>
#include <arch/board/board.h>

#include "arm_internal.h"
#include "chip.h"
#include "ra8p_config.h"

#ifdef CONFIG_RA8P_HWINFO

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* RA8P unique ID base address (from datasheet) */

#define RA8P_UNIQUE_ID_BASE                    (0x0100A150)

/* RA8P product number base address */

#define RA8P_PRODUCT_NUMBER_BASE               (0x0100A140)

/* RA8P package type base address */

#define RA8P_PACKAGE_TYPE_BASE                 (0x0100A144)

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct ra8p_chip_info_s
{
  uint32_t unique_id[4];            /* 128-bit unique ID */
  uint32_t product_number;          /* Product number */
  uint32_t package_type;            /* Package type */
  uint8_t revision;                 /* Chip revision */
};

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_chip_info_s g_chip_info;
static bool g_chip_info_initialized = false;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_read_chip_info
 *
 * Description:
 *   Read chip information from hardware registers
 *
 ****************************************************************************/

static void ra8p_read_chip_info(void)
{
  int i;

  if (g_chip_info_initialized)
    {
      return;
    }

  /* Read unique ID (128 bits = 4 x 32-bit words) */

  for (i = 0; i < 4; i++)
    {
      g_chip_info.unique_id[i] = getreg32(RA8P_UNIQUE_ID_BASE + (i * 4));
    }

  /* Read product number */

  g_chip_info.product_number = getreg32(RA8P_PRODUCT_NUMBER_BASE);

  /* Read package type */

  g_chip_info.package_type = getreg32(RA8P_PACKAGE_TYPE_BASE);

  /* Extract revision from product number */

  g_chip_info.revision = (g_chip_info.product_number >> 28) & 0x0F;

  g_chip_info_initialized = true;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_hwinfo_get_unique_id
 *
 * Description:
 *   Get the chip's unique ID
 *
 * Input Parameters:
 *   buffer - Buffer to store unique ID (16 bytes)
 *   len - Length of buffer (must be at least 16 bytes)
 *
 * Return Value:
 *   Number of bytes copied, or negative error code
 *
 ****************************************************************************/

int ra8p_hwinfo_get_unique_id(uint8_t *buffer, size_t len)
{
  if (!buffer || len < 16)
    {
      return -EINVAL;
    }

  ra8p_read_chip_info();

  /* Copy unique ID to buffer (little endian) */

  memcpy(buffer, g_chip_info.unique_id, 16);

  return 16;
}

/****************************************************************************
 * Name: ra8p_hwinfo_get_product_number
 *
 * Description:
 *   Get the chip's product number
 *
 * Return Value:
 *   Product number
 *
 ****************************************************************************/

uint32_t ra8p_hwinfo_get_product_number(void)
{
  ra8p_read_chip_info();

  return g_chip_info.product_number;
}

/****************************************************************************
 * Name: ra8p_hwinfo_get_package_type
 *
 * Description:
 *   Get the chip's package type
 *
 * Return Value:
 *   Package type code
 *
 ****************************************************************************/

uint32_t ra8p_hwinfo_get_package_type(void)
{
  ra8p_read_chip_info();

  return g_chip_info.package_type;
}

/****************************************************************************
 * Name: ra8p_hwinfo_get_revision
 *
 * Description:
 *   Get the chip's revision number
 *
 * Return Value:
 *   Revision number (0-15)
 *
 ****************************************************************************/

uint8_t ra8p_hwinfo_get_revision(void)
{
  ra8p_read_chip_info();

  return g_chip_info.revision;
}

/****************************************************************************
 * Name: ra8p_hwinfo_get_chip_name
 *
 * Description:
 *   Get the chip's name as a string
 *
 * Return Value:
 *   Chip name string
 *
 ****************************************************************************/

const char *ra8p_hwinfo_get_chip_name(void)
{
  ra8p_read_chip_info();

  /* Decode product number to chip name */

  switch (g_chip_info.product_number & 0xFFFF0000)
    {
      case 0x8A010000:
        return "R7KA8P1KFLCAC";

      default:
        return "Unknown";
    }
}

/****************************************************************************
 * Name: ra8p_hwinfo_get_flash_size
 *
 * Description:
 *   Get the chip's flash size in bytes
 *
 * Return Value:
 *   Flash size in bytes
 *
 ****************************************************************************/

uint32_t ra8p_hwinfo_get_flash_size(void)
{
  /* RA8P1 has 768KB MRAM + 256KB for CM33 */

  return (768 * 1024) + (256 * 1024);
}

/****************************************************************************
 * Name: ra8p_hwinfo_get_sram_size
 *
 * Description:
 *   Get the chip's SRAM size in bytes
 *
 * Return Value:
 *   SRAM size in bytes
 *
 ****************************************************************************/

uint32_t ra8p_hwinfo_get_sram_size(void)
{
  /* RA8P1 has 1MB main SRAM + 1404KB SRAM0 + 468KB SRAM1 */

  return (1 * 1024 * 1024) + (1404 * 1024) + (468 * 1024);
}

/****************************************************************************
 * Name: ra8p_hwinfo_get_num_cores
 *
 * Description:
 *   Get the number of CPU cores
 *
 * Return Value:
 *   Number of cores (1 or 2)
 *
 ****************************************************************************/

int ra8p_hwinfo_get_num_cores(void)
{
  /* RA8P1 has dual core (CM85 + CM33) */

  return 2;
}

/****************************************************************************
 * Name: ra8p_hwinfo_get_cpu_frequency
 *
 * Description:
 *   Get the CPU frequency in Hz
 *
 * Return Value:
 *   CPU frequency in Hz
 *
 ****************************************************************************/

uint32_t ra8p_hwinfo_get_cpu_frequency(void)
{
  /* RA8P1 CM85 can run up to 1 GHz */

  return 1000000000;
}

#endif /* CONFIG_RA8P_HWINFO */