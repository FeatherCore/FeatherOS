/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_flash.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_STM32N6_FLASH_H
#define __ARCH_ARM_SRC_STM32N6_STM32N6_FLASH_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <stdint.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_FLASH_BASE             0x08000000
#define STM32_FLASH_SIZE            0x01000000

#define FLASH_KEYR_OFFSET           0x08
#define FLASH_OPTKEYR_OFFSET       0x0C
#define FLASH_CR_OFFSET            0x10
#define FLASH_SR_OFFSET           0x14
#define FLASH_CR_PG              (1 << 0)
#define FLASH_CR_PER             (1 << 1)
#define FLASH_CR_MER1            (1 << 2)
#define FLASH_CR_PAGE_PG          (1 << 3)
#define FLASH_CR_BKER            (1 << 4)
#define FLASH_CR_MER2            (1 << 15)
#define FLASH_CR_STRT            (1 << 5)
#define FLASH_CR_OPTSTRT         (1 << 17)
#define FLASH_CR_OBL_LAUNCH      (1 << 27)
#define FLASH_CR_OPTLOCK         (1 << 30)
#define FLASH_CR_LOCK            (1 << 31)

#define FLASH_SR_BSY             (1 << 0)
#define FLASH_SR_WBNE            (1 << 1)
#define FLASH_SR_QW              (1 << 2)
#define FLASH_SR_CRC_BUSY        (1 << 3)
#define FLASH_SR_EOP             (1 << 16)
#define FLASH_SR_WRPERR          (1 << 17)
#define FLASH_SR_PGSERR          (1 << 18)
#define FLASH_SR_STRBERR         (1 << 19)
#define FLASH_SR_INCERR          (1 << 21)
#define FLASH_SR_OPERR           (1 << 22)
#define FLASH_SR_RDPERR          (1 << 23)
#define FLASH_SR_RDERR           (1 << 24)
#define FLASH_SR_BOOT_HDMISS     (1 << 25)
#define FLASH_SR_BOOT_CURRMISS   (1 << 26)

#define FLASH_KEY1               0x45670123
#define FLASH_KEY2               0xcdef89ab

#define FLASH_OPTKEY1            0x08192a3b
#define FLASH_OPTKEY2            0x4c5d6e7f

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

void stm32n6_flash_init(void);
int stm32n6_flash_unlock(void);
int stm32n6_flash_lock(void);
int stm32n6_flash_erase_page(uint32_t pageaddr);
int stm32n6_flash_program(uint32_t addr, uint32_t data);
int stm32n6_flash_read(uint32_t addr, uint32_t *data);

#endif /* __ARCH_ARM_SRC_STM32N6_STM32N6_FLASH_H */