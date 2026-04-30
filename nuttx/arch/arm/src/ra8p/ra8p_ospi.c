/****************************************************************************
 * arch/arm/src/ra8p/ra8p_ospi.c
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
#include <nuttx/irq.h>
#include <nuttx/kmalloc.h>
#include <nuttx/mutex.h>
#include <nuttx/semaphore.h>
#include <nuttx/mtd/mtd.h>
#include <nuttx/fs/ioctl.h>
#include <arch/board/board.h>

#include "arm_internal.h"
#include "chip.h"
#include "ra8p_config.h"
#include "ra8p_peripherals.h"
#include "hardware/ra8p_ospi.h"

#ifdef CONFIG_RA8P_OSPI0

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Helper macros for register access */

#define ra8p_ospi_putreg32(addr, offset, val) \
  putreg32((val), (addr) + (offset))

#define ra8p_ospi_getreg32(addr, offset) \
  getreg32((addr) + (offset))

#define ra8p_ospi_modifyreg32(addr, offset, clrbits, setbits) \
  ra8p_ospi_putreg32(addr, offset, \
    (ra8p_ospi_getreg32(addr, offset) & ~(clrbits)) | (setbits))

/* OSPI page size and sector size */

#define RA8P_OSPI_PAGE_SIZE                    (256)
#define RA8P_OSPI_SECTOR_SIZE                  (4096)
#define RA8P_OSPI_BLOCK_SIZE                   (65536)

/* OSPI commands */

#define RA8P_OSPI_CMD_READ                     (0x03)
#define RA8P_OSPI_CMD_FAST_READ                (0x0B)
#define RA8P_OSPI_CMD_QUAD_READ                (0x6B)
#define RA8P_OSPI_CMD_OCTAL_READ               (0x13)
#define RA8P_OSPI_CMD_WRITE_ENABLE             (0x06)
#define RA8P_OSPI_CMD_PAGE_PROGRAM             (0x02)
#define RA8P_OSPI_CMD_QUAD_PAGE_PROGRAM        (0x32)
#define RA8P_OSPI_CMD_SECTOR_ERASE             (0x20)
#define RA8P_OSPI_CMD_BLOCK_ERASE              (0xD8)
#define RA8P_OSPI_CMD_CHIP_ERASE               (0xC7)
#define RA8P_OSPI_CMD_READ_STATUS              (0x05)
#define RA8P_OSPI_CMD_READ_ID                  (0x9F)

/* Status register bits */

#define RA8P_OSPI_SR_WIP                       (1 << 0)   /* Write in Progress */
#define RA8P_OSPI_SR_WEL                       (1 << 1)   /* Write Enable Latch */

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct ra8p_ospi_dev_s
{
  struct mtd_dev_s mtd;            /* MTD interface */
  uint32_t base;                   /* Base address of OSPI registers */
  mutex_t lock;                    /* Thread-safe lock */
  size_t size;                     /* Total size of the flash */
  uint32_t sector_size;            /* Sector size */
  uint32_t page_size;              /* Page size */
  bool initialized;                /* True if initialized */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int ra8p_ospi_wait_ready(struct ra8p_ospi_dev_s *priv, uint32_t timeout);
static int ra8p_ospi_write_enable(struct ra8p_ospi_dev_s *priv);
static int ra8p_ospi_read_status(struct ra8p_ospi_dev_s *priv);
static int ra8p_ospi_erase_sector(struct ra8p_ospi_dev_s *priv, off_t sector);
static int ra8p_ospi_program_page(struct ra8p_ospi_dev_s *priv,
                                  off_t offset, const void *data, size_t len);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_ospi_dev_s g_ospi0_priv;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static int ra8p_ospi_wait_ready(struct ra8p_ospi_dev_s *priv, uint32_t timeout)
{
  int ret = OK;
  uint32_t status;

  while (timeout-- > 0)
    {
      status = ra8p_ospi_read_status(priv);
      if ((status & RA8P_OSPI_SR_WIP) == 0)
        {
          break;
        }

      up_udelay(100);
    }

  if (timeout == 0)
    {
      ret = -ETIMEDOUT;
    }

  return ret;
}

static int ra8p_ospi_write_enable(struct ra8p_ospi_dev_s *priv)
{
  uint32_t cmd;

  /* Set write enable command */
  cmd = RA8P_OSPI_CMD_WRITE_ENABLE;
  ra8p_ospi_putreg32(priv->base, RA8P_OSPI_CMDR_OFFSET, cmd);

  /* Issue command */
  ra8p_ospi_modifyreg32(priv->base, RA8P_OSPI_BCR_OFFSET, 0, RA8P_OSPI_BCR_CMD);

  /* Wait for completion */
  return ra8p_ospi_wait_ready(priv, 1000);
}

static int ra8p_ospi_read_status(struct ra8p_ospi_dev_s *priv)
{
  uint32_t cmd;
  uint32_t status;

  /* Set read status command */
  cmd = RA8P_OSPI_CMD_READ_STATUS;
  ra8p_ospi_putreg32(priv->base, RA8P_OSPI_CMDR_OFFSET, cmd);

  /* Issue command */
  ra8p_ospi_modifyreg32(priv->base, RA8P_OSPI_BCR_OFFSET, 0, RA8P_OSPI_BCR_CMD);

  /* Read status register */
  status = ra8p_ospi_getreg32(priv->base, RA8P_OSPI_DR_OFFSET) & 0xFF;

  return status;
}

static int ra8p_ospi_erase_sector(struct ra8p_ospi_dev_s *priv, off_t sector)
{
  uint32_t addr;
  uint32_t cmd;
  int ret;

  /* Calculate address */
  addr = sector * priv->sector_size;

  /* Enable write */
  ret = ra8p_ospi_write_enable(priv);
  if (ret < 0)
    {
      return ret;
    }

  /* Set sector erase command */
  cmd = RA8P_OSPI_CMD_SECTOR_ERASE;
  ra8p_ospi_putreg32(priv->base, RA8P_OSPI_CMDR_OFFSET, cmd);

  /* Set address */
  ra8p_ospi_putreg32(priv->base, RA8P_OSPI_ADDR_OFFSET, addr);

  /* Issue erase command */
  ra8p_ospi_modifyreg32(priv->base, RA8P_OSPI_BCR_OFFSET, 0, RA8P_OSPI_BCR_CMD);

  /* Wait for completion */
  return ra8p_ospi_wait_ready(priv, 100000);
}

static int ra8p_ospi_program_page(struct ra8p_ospi_dev_s *priv,
                                  off_t offset, const void *data, size_t len)
{
  uint32_t cmd;
  uint32_t addr;
  const uint8_t *src = (const uint8_t *)data;
  int ret;
  size_t i;

  /* Enable write */
  ret = ra8p_ospi_write_enable(priv);
  if (ret < 0)
    {
      return ret;
    }

  /* Set page program command */
  cmd = RA8P_OSPI_CMD_PAGE_PROGRAM;
  ra8p_ospi_putreg32(priv->base, RA8P_OSPI_CMDR_OFFSET, cmd);

  /* Set address */
  addr = offset;
  ra8p_ospi_putreg32(priv->base, RA8P_OSPI_ADDR_OFFSET, addr);

  /* Write data */
  for (i = 0; i < len; i++)
    {
      ra8p_ospi_putreg32(priv->base, RA8P_OSPI_DR_OFFSET, src[i]);
    }

  /* Set data length */
  ra8p_ospi_putreg32(priv->base, RA8P_OSPI_DRCR_OFFSET, len);

  /* Issue program command */
  ra8p_ospi_modifyreg32(priv->base, RA8P_OSPI_BCR_OFFSET, 0, RA8P_OSPI_BCR_CMD);

  /* Wait for completion */
  return ra8p_ospi_wait_ready(priv, 10000);
}

/****************************************************************************
 * Name: ra8p_ospi_erase
 ****************************************************************************/

static int ra8p_ospi_erase(struct mtd_dev_s *dev, off_t startblock,
                          size_t nblocks)
{
  struct ra8p_ospi_dev_s *priv = (struct ra8p_ospi_dev_s *)dev;
  int ret = OK;
  off_t block;

  if (!priv || !priv->initialized)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  for (block = startblock; block < startblock + nblocks; block++)
    {
      ret = ra8p_ospi_erase_sector(priv, block);
      if (ret < 0)
        {
          break;
        }
    }

  nxmutex_unlock(&priv->lock);
  return ret;
}

/****************************************************************************
 * Name: ra8p_ospi_bread
 ****************************************************************************/

static ssize_t ra8p_ospi_bread(struct mtd_dev_s *dev, off_t startblock,
                               size_t nblocks, uint8_t *buffer)
{
  struct ra8p_ospi_dev_s *priv = (struct ra8p_ospi_dev_s *)dev;
  uint32_t addr;
  uint32_t cmd;
  size_t i;

  if (!priv || !priv->initialized || !buffer)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  /* Calculate address */
  addr = startblock * priv->sector_size;

  /* Set read command */
  cmd = RA8P_OSPI_CMD_FAST_READ;
  ra8p_ospi_putreg32(priv->base, RA8P_OSPI_CMDR_OFFSET, cmd);

  /* Set address */
  ra8p_ospi_putreg32(priv->base, RA8P_OSPI_ADDR_OFFSET, addr);

  /* Set data length */
  ra8p_ospi_putreg32(priv->base, RA8P_OSPI_DRCR_OFFSET,
                     nblocks * priv->sector_size);

  /* Issue read command */
  ra8p_ospi_modifyreg32(priv->base, RA8P_OSPI_BCR_OFFSET, 0, RA8P_OSPI_BCR_CMD);

  /* Read data */
  for (i = 0; i < nblocks * priv->sector_size; i++)
    {
      buffer[i] = ra8p_ospi_getreg32(priv->base, RA8P_OSPI_DR_OFFSET) & 0xFF;
    }

  nxmutex_unlock(&priv->lock);
  return nblocks;
}

/****************************************************************************
 * Name: ra8p_ospi_bwrite
 ****************************************************************************/

static ssize_t ra8p_ospi_bwrite(struct mtd_dev_s *dev, off_t startblock,
                                size_t nblocks, const uint8_t *buffer)
{
  struct ra8p_ospi_dev_s *priv = (struct ra8p_ospi_dev_s *)dev;
  off_t offset;
  size_t remaining;
  size_t chunk;
  int ret = OK;

  if (!priv || !priv->initialized || !buffer)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  offset = startblock * priv->sector_size;
  remaining = nblocks * priv->sector_size;

  while (remaining > 0)
    {
      chunk = (remaining > priv->page_size) ? priv->page_size : remaining;

      ret = ra8p_ospi_program_page(priv, offset, buffer, chunk);
      if (ret < 0)
        {
          break;
        }

      offset += chunk;
      buffer += chunk;
      remaining -= chunk;
    }

  nxmutex_unlock(&priv->lock);
  return (ret < 0) ? ret : nblocks;
}

/****************************************************************************
 * Name: ra8p_ospi_read
 ****************************************************************************/

static ssize_t ra8p_ospi_read(struct mtd_dev_s *dev, off_t offset,
                              size_t nbytes, uint8_t *buffer)
{
  struct ra8p_ospi_dev_s *priv = (struct ra8p_ospi_dev_s *)dev;
  uint32_t cmd;
  size_t i;

  if (!priv || !priv->initialized || !buffer)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  /* Set read command */
  cmd = RA8P_OSPI_CMD_FAST_READ;
  ra8p_ospi_putreg32(priv->base, RA8P_OSPI_CMDR_OFFSET, cmd);

  /* Set address */
  ra8p_ospi_putreg32(priv->base, RA8P_OSPI_ADDR_OFFSET, offset);

  /* Set data length */
  ra8p_ospi_putreg32(priv->base, RA8P_OSPI_DRCR_OFFSET, nbytes);

  /* Issue read command */
  ra8p_ospi_modifyreg32(priv->base, RA8P_OSPI_BCR_OFFSET, 0, RA8P_OSPI_BCR_CMD);

  /* Read data */
  for (i = 0; i < nbytes; i++)
    {
      buffer[i] = ra8p_ospi_getreg32(priv->base, RA8P_OSPI_DR_OFFSET) & 0xFF;
    }

  nxmutex_unlock(&priv->lock);
  return nbytes;
}

/****************************************************************************
 * Name: ra8p_ospi_ioctl
 ****************************************************************************/

static int ra8p_ospi_ioctl(struct mtd_dev_s *dev, int cmd,
                           unsigned long arg)
{
  struct ra8p_ospi_dev_s *priv = (struct ra8p_ospi_dev_s *)dev;
  int ret = OK;

  if (!priv || !priv->initialized)
    {
      return -EINVAL;
    }

  switch (cmd)
    {
      case MTDIOC_GEOMETRY:
        {
          struct mtd_geometry_s *geo = (struct mtd_geometry_s *)arg;
          if (geo)
            {
              geo->blocksize    = priv->page_size;
              geo->erasesize    = priv->sector_size;
              geo->neraseblocks = priv->size / priv->sector_size;
            }
          else
            {
              ret = -EINVAL;
            }
        }
        break;

      case MTDIOC_BULKERASE:
        {
          nxmutex_lock(&priv->lock);
          /* Implement chip erase if needed */
          nxmutex_unlock(&priv->lock);
        }
        break;

      default:
        ret = -ENOTTY;
        break;
    }

  return ret;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

int ra8p_ospi_initialize(void)
{
  struct ra8p_ospi_dev_s *priv = &g_ospi0_priv;

  /* Initialize private data structure */
  memset(priv, 0, sizeof(struct ra8p_ospi_dev_s));
  priv->base = RA8P_OSPI0_BASE;
  priv->size = 16 * 1024 * 1024; /* 16 MB default */
  priv->sector_size = RA8P_OSPI_SECTOR_SIZE;
  priv->page_size = RA8P_OSPI_PAGE_SIZE;

  nxmutex_init(&priv->lock);

  /* Reset OSPI controller */
  ra8p_ospi_putreg32(priv->base, RA8P_OSPI_SWPR_OFFSET, RA8P_OSPI_SWPR_SWRST);
  up_udelay(10);
  ra8p_ospi_putreg32(priv->base, RA8P_OSPI_SWPR_OFFSET, 0);

  /* Configure OSPI controller */
  ra8p_ospi_putreg32(priv->base, RA8P_OSPI_CCR_OFFSET, RA8P_OSPI_CCR_MOD_EN);
  ra8p_ospi_putreg32(priv->base, RA8P_OSPI_BCR_OFFSET, 0);

  priv->initialized = true;

  /* Register with NuttX MTD subsystem */
  return mtd_register(&priv->mtd, "/dev/ospi0");
}

/****************************************************************************
 * Name: g_ospi_mtd_ops
 *
 * Description:
 *   MTD operations
 *
 ****************************************************************************/

struct mtd_ops_s g_ospi_mtd_ops =
{
  .erase  = ra8p_ospi_erase,
  .bread  = ra8p_ospi_bread,
  .bwrite = ra8p_ospi_bwrite,
  .read   = ra8p_ospi_read,
  .ioctl  = ra8p_ospi_ioctl
};

#endif /* CONFIG_RA8P_OSPI0 */