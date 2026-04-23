/****************************************************************************
 * arch/arm/src/ra8p/ra8p_lowsetup.c
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

#include <stdint.h>

#include "ra8p_lowsetup.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* SCI_B UART register definitions */
#define RA8P_SCI_BASE(n)        (0x40070000ul + (n) * 0x1000)
#define RA8P_SCI_SCISR_OFFSET   0x00
#define RA8P_SCI_SCITDR_OFFSET  0x24

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_lowsetup
 *
 * Description:
 *   Called before arm_earlyserialinit to set up SCI/UART pins.
 *
 ****************************************************************************/

void ra8p_lowsetup(void)
{
  /* Configure SCI_B pins for UART function.
   * Default UART pins for RA8P:
   * - SCI_B2: P110 (TX), P111 (RX) - Default console
   *
   * Note: Board-specific code should override these defaults if needed.
   */

#ifdef CONFIG_RA8P_SCI_B_UART2
  /* SCI_B2 is typically used as console on RA8P EK boards */
  /* Pin configuration would be done here via port multiplexer registers */
#endif
}

/****************************************************************************
 * Name: arm_lowputc
 *
 * Description:
 *   Output one character to the console UART (for debug).
 *
 ****************************************************************************/

#ifdef CONFIG_DEBUG_FEATURES
void arm_lowputc(char ch)
{
#ifdef CONFIG_RA8P_SCI_B_UART2
  volatile uint32_t *sci_sr  = (uint32_t *)(RA8P_SCI_BASE(2) + RA8P_SCI_SCISR_OFFSET);
  volatile uint32_t *sci_tdr = (uint32_t *)(RA8P_SCI_BASE(2) + RA8P_SCI_SCITDR_OFFSET);

  /* Wait for transmit buffer empty (TDRE flag) */
  while (((*sci_sr) & (1 << 7)) == 0);

  /* Send character */
  *sci_tdr = (uint32_t)ch;
#endif
}
#endif
