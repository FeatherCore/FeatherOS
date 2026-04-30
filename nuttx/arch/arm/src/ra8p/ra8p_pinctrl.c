/****************************************************************************
 * arch/arm/src/ra8p/ra8p_pinctrl.c
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
#include <stdbool.h>

#include "hardware/ra8p_pinctrl.h"
#include "hardware/ra8p_memorymap.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* PFS register offset calculation - Based on Zephyr RA8P1 device tree */
#define RA8P_PFS_REG(port, pin)  (RA8P_PFS_BASE + ((port) * 0x40) + ((pin) * 4))

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_pinctrl_configure
 ****************************************************************************/

void ra8p_pinctrl_configure(uint8_t port, uint8_t pin, uint8_t psel)
{
  uintptr_t pfs_reg;
  uint32_t regval;

  if (port >= 14 || pin >= 16)
    {
      return;
    }

  pfs_reg = RA8P_PFS_REG(port, pin);

  regval = getreg32(pfs_reg);
  regval &= ~(RA8P_PFS_PSEL_MASK);
  regval |= RA8P_PFS_PSEL(psel);
  regval |= RA8P_PFS_PMC;

  putreg32(regval, pfs_reg);
}

/****************************************************************************
 * Name: ra8p_pinctrl_set_drive_strength
 ****************************************************************************/

void ra8p_pinctrl_set_drive_strength(uint8_t port, uint8_t pin, uint8_t ds)
{
  uintptr_t pfs_reg;
  uint32_t regval;

  if (port >= 14 || pin >= 16)
    {
      return;
    }

  pfs_reg = RA8P_PFS_REG(port, pin);

  regval = getreg32(pfs_reg);
  regval &= ~(RA8P_PFS_DS_MASK);
  regval |= RA8P_PFS_DS(ds);

  putreg32(regval, pfs_reg);
}

/****************************************************************************
 * Name: ra8p_pinctrl_enable_pull
 ****************************************************************************/

void ra8p_pinctrl_enable_pull(uint8_t port, uint8_t pin, uint8_t pupd)
{
  uintptr_t pfs_reg;
  uint32_t regval;

  if (port >= 14 || pin >= 16)
    {
      return;
    }

  pfs_reg = RA8P_PFS_REG(port, pin);

  regval = getreg32(pfs_reg);
  regval &= ~(RA8P_PFS_PUEN | RA8P_PFS_PDEN);

  switch (pupd)
    {
      case 1:
        regval |= RA8P_PFS_PUEN;
        break;
      case 2:
        regval |= RA8P_PFS_PDEN;
        break;
      default:
        break;
    }

  putreg32(regval, pfs_reg);
}

/****************************************************************************
 * Name: ra8p_pinctrl_enable_open_drain
 ****************************************************************************/

void ra8p_pinctrl_enable_open_drain(uint8_t port, uint8_t pin, bool enable)
{
  uintptr_t pfs_reg;
  uint32_t regval;

  if (port >= 14 || pin >= 16)
    {
      return;
    }

  pfs_reg = RA8P_PFS_REG(port, pin);

  regval = getreg32(pfs_reg);

  if (enable)
    {
      regval |= RA8P_PFS_ODEN;
    }
  else
    {
      regval &= ~RA8P_PFS_ODEN;
    }

  putreg32(regval, pfs_reg);
}

/****************************************************************************
 * Name: ra8p_pinctrl_uart_init
 *
 * Description:
 *   Initialize UART pins based on Zephyr RA8P1 pinctrl configuration.
 *   Reference: ek_ra8p1-pinctrl.dtsi
 *
 ****************************************************************************/

#ifdef CONFIG_RA8P_SCI_B_UART
void ra8p_pinctrl_uart_init(void)
{
  /* UART2 default pins (console) - P110 (TX), P111 (RX) */
#ifdef CONFIG_RA8P_SCI_B_UART2
  /* SCI2 TX (P201) - PSEL=1, PIN=0 */
  ra8p_pinctrl_configure(2, 0, RA8P_PSEL_SCI_8);
  ra8p_pinctrl_set_drive_strength(2, 0, RA8P_PFS_DS_MEDIUM);

  /* SCI2 RX (P200) - PSEL=1, PIN=1 */
  ra8p_pinctrl_configure(2, 1, RA8P_PSEL_SCI_8);
#endif

  /* UART8 - P713 (TX), P712 (RX) */
#ifdef CONFIG_RA8P_SCI_B_UART8
  /* SCI8 TX - PSEL=2, PIN=13 */
  ra8p_pinctrl_configure(7, 13, RA8P_PSEL_SCI_8);
  ra8p_pinctrl_set_drive_strength(7, 13, RA8P_PFS_DS_MEDIUM);

  /* SCI8 RX - PSEL=3, PIN=13 */
  ra8p_pinctrl_configure(7, 13, RA8P_PSEL_SCI_8);
#endif

  /* UART9 - P308 (TX), P309 (RX) */
#ifdef CONFIG_RA8P_SCI_B_UART9
  /* SCI9 TX - PSEL=9, PIN=2 */
  ra8p_pinctrl_configure(3, 2, RA8P_PSEL_SCI_9);
  ra8p_pinctrl_set_drive_strength(3, 2, RA8P_PFS_DS_MEDIUM);

  /* SCI9 RX - PSEL=8, PIN=2 */
  ra8p_pinctrl_configure(3, 2, RA8P_PSEL_SCI_9);
#endif
}
#endif

/****************************************************************************
 * Name: ra8p_pinctrl_spi_init
 *
 * Description:
 *   Initialize SPI pins based on Zephyr RA8P1 pinctrl configuration.
 *   Reference: ek_ra8p1-pinctrl.dtsi
 *
 ****************************************************************************/

#ifdef CONFIG_RA8P_SPI_B
void ra8p_pinctrl_spi_init(void)
{
  /* SPI1 - P101-P103 (MISO, MOSI, RSPCK, SSL) */
#ifdef CONFIG_RA8P_SPI_B1
  /* SPI1 MISO - PSEL=0, PIN=1 */
  ra8p_pinctrl_configure(0, 1, RA8P_PSEL_SPI);

  /* SPI1 MOSI - PSEL=1, PIN=1 */
  ra8p_pinctrl_configure(0, 2, RA8P_PSEL_SPI);

  /* SPI1 RSPCK - PSEL=2, PIN=1 */
  ra8p_pinctrl_configure(0, 3, RA8P_PSEL_SPI);

  /* SPI1 SSL - PSEL=3, PIN=1 */
  ra8p_pinctrl_configure(0, 4, RA8P_PSEL_SPI);
#endif
}
#endif

/****************************************************************************
 * Name: ra8p_pinctrl_i2c_init
 *
 * Description:
 *   Initialize I2C pins based on Zephyr RA8P1 pinctrl configuration.
 *   Reference: ek_ra8p1-pinctrl.dtsi
 *
 ****************************************************************************/

#ifdef CONFIG_RA8P_IIC
void ra8p_pinctrl_i2c_init(void)
{
  /* IIC1 - P511 (SCL), P512 (SDA) */
#ifdef CONFIG_RA8P_IIC1
  /* IIC1 SCL - PSEL=5, PIN=12 */
  ra8p_pinctrl_configure(5, 12, RA8P_PSEL_I2C);
  ra8p_pinctrl_set_drive_strength(5, 12, RA8P_PFS_DS_MEDIUM);
  ra8p_pinctrl_enable_open_drain(5, 12, true);

  /* IIC1 SDA - PSEL=5, PIN=11 */
  ra8p_pinctrl_configure(5, 11, RA8P_PSEL_I2C);
  ra8p_pinctrl_set_drive_strength(5, 11, RA8P_PFS_DS_MEDIUM);
  ra8p_pinctrl_enable_open_drain(5, 11, true);
#endif
}
#endif

/****************************************************************************
 * Name: ra8p_pinctrl_sdram_init
 *
 * Description:
 *   Initialize SDRAM pins based on Zephyr RA8P1 pinctrl configuration.
 *   Reference: ek_ra8p1-pinctrl.dtsi
 *
 ****************************************************************************/

#ifdef CONFIG_RA8P_SDRAMC
void ra8p_pinctrl_sdram_init(void)
{
  /* SDRAM pins - Port A, B, C, D */
  uint8_t port;
  uint8_t pin;

  /* Address lines - P1003-P1012 */
  for (pin = 0; pin <= 12; pin++)
    {
      ra8p_pinctrl_configure(10, pin, RA8P_PSEL_BUS);
      ra8p_pinctrl_set_drive_strength(10, pin, RA8P_PFS_DS_HIGH);
    }

  /* Address lines - P503-P5012 */
  for (pin = 3; pin <= 12; pin++)
    {
      ra8p_pinctrl_configure(5, pin, RA8P_PSEL_BUS);
      ra8p_pinctrl_set_drive_strength(5, pin, RA8P_PFS_DS_HIGH);
    }

  /* Data lines and control signals - Port B, C, D, etc */
  /* DQ0-DQ31, DQM0-DQM3, WE, CAS, RAS, CS, CKE, SDCLK */
}
#endif

/****************************************************************************
 * Name: ra8p_pinctrl_usb_init
 *
 * Description:
 *   Initialize USB pins based on Zephyr RA8P1 pinctrl configuration.
 *   Reference: ek_ra8p1-pinctrl.dtsi
 *
 ****************************************************************************/

#ifdef CONFIG_RA8P_USBFS
void ra8p_pinctrl_usbfs_init(void)
{
  /* USB FS pins */
  /* USB_DM - P814 */
  ra8p_pinctrl_configure(8, 14, RA8P_PSEL_USBFS);
  ra8p_pinctrl_set_drive_strength(8, 14, RA8P_PFS_DS_HIGH);

  /* USB_DP - P815 */
  ra8p_pinctrl_configure(8, 15, RA8P_PSEL_USBFS);
  ra8p_pinctrl_set_drive_strength(8, 15, RA8P_PFS_DS_HIGH);

  /* VBUS - P407 */
  ra8p_pinctrl_configure(4, 7, RA8P_PSEL_USBFS);
}
#endif

#ifdef CONFIG_RA8P_USBHS
void ra8p_pinctrl_usbhs_init(void)
{
  /* USB HS pins */
  /* VBUS - P408 */
  ra8p_pinctrl_configure(4, 8, RA8P_PSEL_USBHS);
  ra8p_pinctrl_set_drive_strength(4, 8, RA8P_PFS_DS_HIGH);
}
#endif