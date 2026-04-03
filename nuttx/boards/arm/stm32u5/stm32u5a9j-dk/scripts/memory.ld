/****************************************************************************
 * boards/arm/stm32u5/stm32u5a9j-dk/scripts/memory.ld
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

/* The STM32U5A9 has 4 MiB of FLASH beginning at address 0x0800:0000 and
 * 2514 KiB of SRAM beginning at address 0x2000:0000.  When booting from
 * FLASH, FLASH memory is aliased to address 0x0000:0000 where the code
 * expects to begin execution by jumping to the entry point in the
 * 0x0800:0000 address range.
 *
 * For MPU support, the kernel-mode NuttX section is assumed to be 128Kb of
 * FLASH and 64Kb of SRAM.  Allowing additional memory permits configuring
 * debug instrumentation to be added to the kernel space without overflowing
 * the partition.
 *
 * Alignment of the user space FLASH partition is also a critical factor:
 * The user space FLASH partition will be spanned with a single region of
 * size 2**n bytes.  The alignment of the user-space region must be the same.
 * As a consequence, as the user-space increases in size, the alignment
 * requirement also increases.
 *
 * This alignment requirement means that the largest user space FLASH region
 * you can have will be 512KB at it would have to be positioned at
 * 0x08000000 (it cannot be positioned at 0x0800000 because vectors power-up
 * reset vectors are places at the beginning of that range).  If you change
 * this address, don't forget to change the CONFIG_NUTTX_USERSPACE
 * configuration setting to match and to modify the check in kernel/userspace.c.
 */

MEMORY
{
  /* 4096Kb FLASH */

  kflash (rx)      : ORIGIN = 0x08000000, LENGTH = 256K
  uflash (rx)      : ORIGIN = 0x08040000, LENGTH = 256K
  xflash (rx)      : ORIGIN = 0x08080000, LENGTH = 3584K

  /* 2514Kb of contiguous SRAM */

  ksram (rwx)      : ORIGIN = 0x20000000, LENGTH = 64K
  usram (rwx)      : ORIGIN = 0x20010000, LENGTH = 64K
  xsram (rwx)      : ORIGIN = 0x20020000, LENGTH = 2386K
}
