/****************************************************************************
 * drivers/mtd/mx66uw.c
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
#include <stdint.h>
#include <string.h>

#include <debug.h>
#include <nuttx/kmalloc.h>
#include <nuttx/mtd/mtd.h>
#include <nuttx/spi/qspi.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define MX66UW_MANUFACTURER             0xc2
#define MX66UW_CAPACITY                 0x21

#define MX66UW_CMD_RDID                 0x9f
#define MX66UW_CMD_RDSR                 0x05
#define MX66UW_CMD_WREN                 0x06
#define MX66UW_CMD_READ4B               0x13
#define MX66UW_CMD_PP4B                 0x12
#define MX66UW_CMD_SE4B                 0x21

#define MX66UW_SR_WIP                   (1 << 0)

#define MX66UW_PAGE_SHIFT               8
#define MX66UW_PAGE_SIZE                (1 << MX66UW_PAGE_SHIFT)
#define MX66UW_SECTOR_SHIFT             12
#define MX66UW_SECTOR_SIZE              (1 << MX66UW_SECTOR_SHIFT)
#define MX66UW_NSECTORS                 32768
#define MX66UW_SIZE                     (MX66UW_NSECTORS * MX66UW_SECTOR_SIZE)
#define MX66UW_NPAGES                   (MX66UW_SIZE / MX66UW_PAGE_SIZE)
#define MX66UW_ADDRLEN                  4

#define MX66UW_ID_LEN                   3
#define MX66UW_WAIT_READY_RETRIES       1000000

#ifndef CONFIG_MX66UW_QSPI_FREQUENCY
#  define CONFIG_MX66UW_QSPI_FREQUENCY 24000000
#endif

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct mx66uw_dev_s
{
  struct mtd_dev_s mtd;
  FAR struct qspi_dev_s *qspi;
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int mx66uw_erase(FAR struct mtd_dev_s *dev, off_t startblock,
                        size_t nblocks);
static ssize_t mx66uw_bread(FAR struct mtd_dev_s *dev, off_t startblock,
                            size_t nblocks, FAR uint8_t *buffer);
static ssize_t mx66uw_bwrite(FAR struct mtd_dev_s *dev, off_t startblock,
                             size_t nblocks, FAR const uint8_t *buffer);
static ssize_t mx66uw_read(FAR struct mtd_dev_s *dev, off_t offset,
                           size_t nbytes, FAR uint8_t *buffer);
#ifdef CONFIG_MTD_BYTE_WRITE
static ssize_t mx66uw_write(FAR struct mtd_dev_s *dev, off_t offset,
                            size_t nbytes, FAR const uint8_t *buffer);
#endif
static int mx66uw_ioctl(FAR struct mtd_dev_s *dev, int cmd,
                        unsigned long arg);

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static int mx66uw_cmd(FAR struct mx66uw_dev_s *priv, uint8_t cmd)
{
  struct qspi_cmdinfo_s cmdinfo;

  memset(&cmdinfo, 0, sizeof(cmdinfo));
  cmdinfo.cmd = cmd;

  return QSPI_COMMAND(priv->qspi, &cmdinfo);
}

static int mx66uw_read_status(FAR struct mx66uw_dev_s *priv,
                              FAR uint8_t *status)
{
  struct qspi_cmdinfo_s cmdinfo;

  memset(&cmdinfo, 0, sizeof(cmdinfo));
  cmdinfo.flags = QSPICMD_READDATA;
  cmdinfo.cmd = MX66UW_CMD_RDSR;
  cmdinfo.buflen = 1;
  cmdinfo.buffer = status;

  return QSPI_COMMAND(priv->qspi, &cmdinfo);
}

static int mx66uw_wait_ready(FAR struct mx66uw_dev_s *priv)
{
  uint8_t status;
  int retries;
  int ret;

  for (retries = 0; retries < MX66UW_WAIT_READY_RETRIES; retries++)
    {
      ret = mx66uw_read_status(priv, &status);
      if (ret < 0)
        {
          return ret;
        }

      if ((status & MX66UW_SR_WIP) == 0)
        {
          return OK;
        }
    }

  return -ETIMEDOUT;
}

static int mx66uw_write_enable(FAR struct mx66uw_dev_s *priv)
{
  return mx66uw_cmd(priv, MX66UW_CMD_WREN);
}

static int mx66uw_read_id(FAR struct mx66uw_dev_s *priv, uint8_t id[3])
{
  struct qspi_cmdinfo_s cmdinfo;

  memset(&cmdinfo, 0, sizeof(cmdinfo));
  cmdinfo.flags = QSPICMD_READDATA;
  cmdinfo.cmd = MX66UW_CMD_RDID;
  cmdinfo.buflen = MX66UW_ID_LEN;
  cmdinfo.buffer = id;

  return QSPI_COMMAND(priv->qspi, &cmdinfo);
}

static int mx66uw_read_bytes(FAR struct mx66uw_dev_s *priv, off_t offset,
                             FAR uint8_t *buffer, size_t nbytes)
{
  struct qspi_meminfo_s meminfo;

  if (offset < 0 || offset + nbytes > MX66UW_SIZE)
    {
      return -EINVAL;
    }

  memset(&meminfo, 0, sizeof(meminfo));
  meminfo.cmd = MX66UW_CMD_READ4B;
  meminfo.addrlen = MX66UW_ADDRLEN;
  meminfo.addr = offset;
  meminfo.buflen = nbytes;
  meminfo.buffer = buffer;

  return QSPI_MEMORY(priv->qspi, &meminfo);
}

static int mx66uw_program_page(FAR struct mx66uw_dev_s *priv, off_t offset,
                               FAR const uint8_t *buffer, size_t nbytes)
{
  struct qspi_meminfo_s meminfo;
  size_t pageoff;
  int ret;

  if (offset < 0 || nbytes == 0 || nbytes > MX66UW_PAGE_SIZE ||
      offset + nbytes > MX66UW_SIZE)
    {
      return -EINVAL;
    }

  pageoff = offset & (MX66UW_PAGE_SIZE - 1);
  if (pageoff + nbytes > MX66UW_PAGE_SIZE)
    {
      return -EINVAL;
    }

  ret = mx66uw_write_enable(priv);
  if (ret < 0)
    {
      return ret;
    }

  memset(&meminfo, 0, sizeof(meminfo));
  meminfo.flags = QSPIMEM_WRITE;
  meminfo.cmd = MX66UW_CMD_PP4B;
  meminfo.addrlen = MX66UW_ADDRLEN;
  meminfo.addr = offset;
  meminfo.buflen = nbytes;
  meminfo.buffer = (FAR void *)buffer;

  ret = QSPI_MEMORY(priv->qspi, &meminfo);
  if (ret < 0)
    {
      return ret;
    }

  return mx66uw_wait_ready(priv);
}

static int mx66uw_erase_sector(FAR struct mx66uw_dev_s *priv, off_t offset)
{
  struct qspi_cmdinfo_s cmdinfo;
  int ret;

  if (offset < 0 || offset >= MX66UW_SIZE ||
      (offset & (MX66UW_SECTOR_SIZE - 1)) != 0)
    {
      return -EINVAL;
    }

  ret = mx66uw_write_enable(priv);
  if (ret < 0)
    {
      return ret;
    }

  memset(&cmdinfo, 0, sizeof(cmdinfo));
  cmdinfo.flags = QSPICMD_ADDRESS;
  cmdinfo.cmd = MX66UW_CMD_SE4B;
  cmdinfo.addrlen = MX66UW_ADDRLEN;
  cmdinfo.addr = offset;

  ret = QSPI_COMMAND(priv->qspi, &cmdinfo);
  if (ret < 0)
    {
      return ret;
    }

  return mx66uw_wait_ready(priv);
}

static int mx66uw_erase(FAR struct mtd_dev_s *dev, off_t startblock,
                        size_t nblocks)
{
  FAR struct mx66uw_dev_s *priv = (FAR struct mx66uw_dev_s *)dev;
  off_t block;
  int ret;

  if (startblock < 0 || startblock + nblocks > MX66UW_NSECTORS)
    {
      return -EINVAL;
    }

  ret = QSPI_LOCK(priv->qspi, true);
  if (ret < 0)
    {
      return ret;
    }

  for (block = startblock; block < startblock + nblocks; block++)
    {
      ret = mx66uw_erase_sector(priv, block << MX66UW_SECTOR_SHIFT);
      if (ret < 0)
        {
          break;
        }
    }

  QSPI_LOCK(priv->qspi, false);
  return ret;
}

static ssize_t mx66uw_bread(FAR struct mtd_dev_s *dev, off_t startblock,
                            size_t nblocks, FAR uint8_t *buffer)
{
  FAR struct mx66uw_dev_s *priv = (FAR struct mx66uw_dev_s *)dev;
  off_t offset;
  size_t nbytes;
  int ret;

  if (startblock < 0 || startblock + nblocks > MX66UW_NPAGES)
    {
      return -EINVAL;
    }

  offset = startblock << MX66UW_PAGE_SHIFT;
  nbytes = nblocks << MX66UW_PAGE_SHIFT;

  ret = QSPI_LOCK(priv->qspi, true);
  if (ret < 0)
    {
      return ret;
    }

  ret = mx66uw_read_bytes(priv, offset, buffer, nbytes);
  QSPI_LOCK(priv->qspi, false);

  return ret < 0 ? ret : (ssize_t)nblocks;
}

static ssize_t mx66uw_bwrite(FAR struct mtd_dev_s *dev, off_t startblock,
                             size_t nblocks, FAR const uint8_t *buffer)
{
  FAR struct mx66uw_dev_s *priv = (FAR struct mx66uw_dev_s *)dev;
  off_t block;
  int ret;

  if (startblock < 0 || startblock + nblocks > MX66UW_NPAGES)
    {
      return -EINVAL;
    }

  ret = QSPI_LOCK(priv->qspi, true);
  if (ret < 0)
    {
      return ret;
    }

  for (block = startblock; block < startblock + nblocks; block++)
    {
      ret = mx66uw_program_page(priv, block << MX66UW_PAGE_SHIFT, buffer,
                                MX66UW_PAGE_SIZE);
      if (ret < 0)
        {
          break;
        }

      buffer += MX66UW_PAGE_SIZE;
    }

  QSPI_LOCK(priv->qspi, false);
  return ret < 0 ? ret : (ssize_t)nblocks;
}

static ssize_t mx66uw_read(FAR struct mtd_dev_s *dev, off_t offset,
                           size_t nbytes, FAR uint8_t *buffer)
{
  FAR struct mx66uw_dev_s *priv = (FAR struct mx66uw_dev_s *)dev;
  int ret;

  ret = QSPI_LOCK(priv->qspi, true);
  if (ret < 0)
    {
      return ret;
    }

  ret = mx66uw_read_bytes(priv, offset, buffer, nbytes);
  QSPI_LOCK(priv->qspi, false);

  return ret < 0 ? ret : (ssize_t)nbytes;
}

#ifdef CONFIG_MTD_BYTE_WRITE
static ssize_t mx66uw_write(FAR struct mtd_dev_s *dev, off_t offset,
                            size_t nbytes, FAR const uint8_t *buffer)
{
  FAR struct mx66uw_dev_s *priv = (FAR struct mx66uw_dev_s *)dev;
  size_t chunk;
  size_t pageleft;
  size_t written;
  int ret;

  if (offset < 0 || offset + nbytes > MX66UW_SIZE)
    {
      return -EINVAL;
    }

  ret = QSPI_LOCK(priv->qspi, true);
  if (ret < 0)
    {
      return ret;
    }

  written = 0;
  while (written < nbytes)
    {
      pageleft = MX66UW_PAGE_SIZE -
                 ((offset + written) & (MX66UW_PAGE_SIZE - 1));
      chunk = nbytes - written;
      if (chunk > pageleft)
        {
          chunk = pageleft;
        }

      ret = mx66uw_program_page(priv, offset + written,
                                buffer + written, chunk);
      if (ret < 0)
        {
          break;
        }

      written += chunk;
    }

  QSPI_LOCK(priv->qspi, false);
  return ret < 0 ? ret : (ssize_t)nbytes;
}
#endif

static int mx66uw_ioctl(FAR struct mtd_dev_s *dev, int cmd,
                        unsigned long arg)
{
  FAR struct mx66uw_dev_s *priv = (FAR struct mx66uw_dev_s *)dev;
  int ret;

  ret = -EINVAL;

  switch (cmd)
    {
      case MTDIOC_GEOMETRY:
        {
          FAR struct mtd_geometry_s *geo =
            (FAR struct mtd_geometry_s *)((uintptr_t)arg);

          if (geo != NULL)
            {
              memset(geo, 0, sizeof(*geo));
              geo->blocksize = MX66UW_PAGE_SIZE;
              geo->erasesize = MX66UW_SECTOR_SIZE;
              geo->neraseblocks = MX66UW_NSECTORS;
              ret = OK;
            }
        }
        break;

      case BIOC_PARTINFO:
        {
          FAR struct partition_info_s *info =
            (FAR struct partition_info_s *)((uintptr_t)arg);

          if (info != NULL)
            {
              info->numsectors = MX66UW_NPAGES;
              info->sectorsize = MX66UW_PAGE_SIZE;
              info->startsector = 0;
              info->parent[0] = '\0';
              ret = OK;
            }
        }
        break;

      case BIOC_XIPBASE:
        {
          FAR void **ppv = (FAR void **)((uintptr_t)arg);

          if (ppv != NULL)
            {
              *ppv = NULL;
              ret = OK;
            }
        }
        break;

      case MTDIOC_BULKERASE:
        {
          ret = mx66uw_erase(dev, 0, MX66UW_NSECTORS);
        }
        break;

      case MTDIOC_ERASESTATE:
        {
          FAR uint8_t *result = (FAR uint8_t *)((uintptr_t)arg);

          if (result != NULL)
            {
              *result = 0xff;
              ret = OK;
            }
        }
        break;

      case MTDIOC_SETSPEED:
        {
          ret = QSPI_LOCK(priv->qspi, true);
          if (ret >= 0)
            {
              QSPI_SETFREQUENCY(priv->qspi, (uint32_t)arg);
              QSPI_LOCK(priv->qspi, false);
              ret = OK;
            }
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

/****************************************************************************
 * Name: mx66uw_initialize
 ****************************************************************************/

FAR struct mtd_dev_s *mx66uw_initialize(FAR struct qspi_dev_s *qspi,
                                        bool unprotect)
{
  FAR struct mx66uw_dev_s *priv;
  uint8_t id[MX66UW_ID_LEN];
  int ret;

  if (qspi == NULL)
    {
      return NULL;
    }

  priv = kmm_zalloc(sizeof(*priv));
  if (priv == NULL)
    {
      return NULL;
    }

  priv->mtd.erase = mx66uw_erase;
  priv->mtd.bread = mx66uw_bread;
  priv->mtd.bwrite = mx66uw_bwrite;
  priv->mtd.read = mx66uw_read;
#ifdef CONFIG_MTD_BYTE_WRITE
  priv->mtd.write = mx66uw_write;
#endif
  priv->mtd.ioctl = mx66uw_ioctl;
  priv->mtd.name = "mx66uw";
  priv->qspi = qspi;

  (void)unprotect;

  ret = QSPI_LOCK(qspi, true);
  if (ret < 0)
    {
      kmm_free(priv);
      return NULL;
    }

  QSPI_SETFREQUENCY(qspi, CONFIG_MX66UW_QSPI_FREQUENCY);
  QSPI_SETMODE(qspi, QSPIDEV_MODE0);
  QSPI_SETBITS(qspi, 8);

  ret = mx66uw_read_id(priv, id);
  QSPI_LOCK(qspi, false);

  if (ret < 0)
    {
      ferr("ERROR: MX66UW JEDEC read failed: %d\n", ret);
      kmm_free(priv);
      return NULL;
    }

  if (id[0] != MX66UW_MANUFACTURER || id[2] != MX66UW_CAPACITY)
    {
      ferr("ERROR: Unsupported JEDEC id: %02x %02x %02x\n",
           id[0], id[1], id[2]);
      kmm_free(priv);
      return NULL;
    }

  finfo("MX66UW JEDEC id: %02x %02x %02x\n", id[0], id[1], id[2]);
  return &priv->mtd;
}
