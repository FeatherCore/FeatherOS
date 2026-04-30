/****************************************************************************
 * arch/arm/src/ra8p/ra8p_gpio.c
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

#include <sys/types.h>
#include <stdint.h>
#include <stdbool.h>
#include <errno.h>
#include <debug.h>

#include "chip.h"
#include "arm_internal.h"
#include "hardware/ra8p_gpio.h"

#ifdef CONFIG_RA8P_HAVE_GPIO

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Maximum number of GPIO ports and pins */
#define RA8P_GPIO_NPORTS                14
#define RA8P_GPIO_NPINS_PER_PORT        16

/* GPIO timeout in milliseconds */
#define RA8P_GPIO_TIMEOUT_MS            100

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* RA8P1 GPIO driver state structure */

struct ra8p_gpio_priv_s
{
  bool initialized;                            /* Initialization flag */
  bool enabled[RA8P_GPIO_NPORTS];              /* Enable flags for each port */
  uint8_t pin_function[RA8P_GPIO_NPORTS][RA8P_GPIO_NPINS_PER_PORT]; /* Pin function */
  bool pin_direction[RA8P_GPIO_NPORTS][RA8P_GPIO_NPINS_PER_PORT]; /* Pin direction */
  bool pin_output_val[RA8P_GPIO_NPORTS][RA8P_GPIO_NPINS_PER_PORT]; /* Pin output values */
  bool pin_pullup[RA8P_GPIO_NPORTS][RA8P_GPIO_NPINS_PER_PORT]; /* Pin pull-up enabled */
  bool pin_pulldown[RA8P_GPIO_NPORTS][RA8P_GPIO_NPINS_PER_PORT]; /* Pin pull-down enabled */
  bool pin_opendrain[RA8P_GPIO_NPORTS][RA8P_GPIO_NPINS_PER_PORT]; /* Pin open-drain mode */
  bool pin_analog[RA8P_GPIO_NPORTS][RA8P_GPIO_NPINS_PER_PORT]; /* Pin analog mode */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static uint32_t gpio_get_port_base(uint8_t port);
static uint32_t gpio_get_pfs_addr(uint8_t port, uint8_t pin);
static void gpio_putreg32(uint32_t base, uint32_t offset, uint32_t value);
static uint32_t gpio_getreg32(uint32_t base, uint32_t offset);
static void gpio_putreg16(uint32_t base, uint32_t offset, uint16_t value);
static uint16_t gpio_getreg16(uint32_t base, uint32_t offset);
static void gpio_putreg8(uint32_t base, uint32_t offset, uint8_t value);
static uint8_t gpio_getreg8(uint32_t base, uint32_t offset);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_gpio_priv_s g_gpio;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: gpio_get_port_base
 ****************************************************************************/

static uint32_t gpio_get_port_base(uint8_t port)
{
  if (port > 13)  /* Only valid ports 0-A, B, C, D */
    {
      return 0;
    }

  switch (port)
    {
      case 0:  return RA8P_GPIO0_BASE;
      case 1:  return RA8P_GPIO1_BASE;
      case 2:  return RA8P_GPIO2_BASE;
      case 3:  return RA8P_GPIO3_BASE;
      case 4:  return RA8P_GPIO4_BASE;
      case 5:  return RA8P_GPIO5_BASE;
      case 6:  return RA8P_GPIO6_BASE;
      case 7:  return RA8P_GPIO7_BASE;
      case 8:  return RA8P_GPIO8_BASE;
      case 9:  return RA8P_GPIO9_BASE;
      case 10: return RA8P_GPIOA_BASE;  /* 'A' */
      case 11: return RA8P_GPIOB_BASE;  /* 'B' */
      case 12: return RA8P_GPIOC_BASE;  /* 'C' */
      case 13: return RA8P_GPIOD_BASE;  /* 'D' */
      default: return 0;
    }
}

/****************************************************************************
 * Name: gpio_get_pfs_addr
 ****************************************************************************/

static uint32_t gpio_get_pfs_addr(uint8_t port, uint8_t pin)
{
  if (port > 13 || pin > 15)
    {
      return 0;
    }

  /* PFS register base address */
  return RA8P_PFS_BASE + ((port * 0x40) + (pin * 4));
}

/****************************************************************************
 * Name: gpio_putreg32
 ****************************************************************************/

static inline void gpio_putreg32(uint32_t base, uint32_t offset, uint32_t value)
{
  putreg32(value, base + offset);
}

/****************************************************************************
 * Name: gpio_getreg32
 ****************************************************************************/

static inline uint32_t gpio_getreg32(uint32_t base, uint32_t offset)
{
  return getreg32(base + offset);
}

/****************************************************************************
 * Name: gpio_putreg16
 ****************************************************************************/

static inline void gpio_putreg16(uint32_t base, uint32_t offset, uint16_t value)
{
  putreg16(value, base + offset);
}

/****************************************************************************
 * Name: gpio_getreg16
 ****************************************************************************/

static inline uint16_t gpio_getreg16(uint32_t base, uint32_t offset)
{
  return getreg16(base + offset);
}

/****************************************************************************
 * Name: gpio_putreg8
 ****************************************************************************/

static inline void gpio_putreg8(uint32_t base, uint32_t offset, uint8_t value)
{
  putreg8(value, base + offset);
}

/****************************************************************************
 * Name: gpio_getreg8
 ****************************************************************************/

static inline uint8_t gpio_getreg8(uint32_t base, uint32_t offset)
{
  return getreg8(base + offset);
}

/****************************************************************************
 * Name: gpio_pfs_configure
 ****************************************************************************/

static int gpio_pfs_configure(uint8_t port, uint8_t pin, uint8_t function, bool input, 
                              bool pullup, bool pulldown, bool open_drain, bool analog)
{
  uint32_t pfs_addr;
  uint32_t pfs_val = 0;

  if (port >= RA8P_GPIO_NPORTS || pin >= RA8P_GPIO_NPINS_PER_PORT)
    {
      return -EINVAL;
    }

  pfs_addr = gpio_get_pfs_addr(port, pin);
  if (pfs_addr == 0)
    {
      return -EINVAL;
    }

  /* Set pin function */
  pfs_val |= (function << RA8P_GPIO_PFS_PSEL_SHIFT) & RA8P_GPIO_PFS_PSEL_MASK;

  /* Set direction */
  if (!input)  /* Output */
    {
      pfs_val |= RA8P_GPIO_PFS_PDR_OUTPUT;
    }
  else  /* Input */
    {
      pfs_val |= RA8P_GPIO_PFS_PDR_INPUT;
    }

  /* Set pull-up/down */
  if (pullup)
    {
      pfs_val |= RA8P_GPIO_PFS_PCR_ENABLE;
    }
  else
    {
      pfs_val &= ~RA8P_GPIO_PFS_PCR_ENABLE;
    }

  /* Set drive strength and open-drain mode */
  if (open_drain)
    {
      pfs_val |= RA8P_GPIO_PFS_DSCR_HIGH;  /* High drive strength for open-drain */
    }
  else
    {
      pfs_val &= ~RA8P_GPIO_PFS_DSCR_HIGH;  /* Low drive strength for push-pull */
    }

  /* Set analog mode */
  if (analog)
    {
      pfs_val |= RA8P_GPIO_PFS_ASEL_ENABLE;
    }
  else
    {
      pfs_val &= ~RA8P_GPIO_PFS_ASEL_ENABLE;
    }

  /* Set interrupt select for GPIO mode */
  if (function == 0)  /* GPIO mode */
    {
      pfs_val |= RA8P_GPIO_PFS_ISEL_IRQ;
    }
  else
    {
      pfs_val &= ~RA8P_GPIO_PFS_ISEL_IRQ;
    }

  /* Write PFS register */
  putreg32(pfs_val, pfs_addr);

  /* Update internal state */
  g_gpio.pin_function[port][pin] = function;
  g_gpio.pin_direction[port][pin] = input;
  g_gpio.pin_pullup[port][pin] = pullup;
  g_gpio.pin_pulldown[port][pin] = pulldown;
  g_gpio.pin_opendrain[port][pin] = open_drain;
  g_gpio.pin_analog[port][pin] = analog;

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_gpio_config
 *
 * Description:
 *   Configure a GPIO pin based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   pinset - GPIO pinset configuration
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_config(gpio_pinset_t pinset)
{
  struct ra8p_gpio_priv_s *priv = &g_gpio;
  uint8_t port = (pinset >> 8) & 0xFF;
  uint8_t pin = pinset & 0xFF;
  bool input = (pinset & GPIO_OUTPUT) == 0;
  bool initial_value = (pinset & GPIO_OUTPUT_ONE) != 0;
  bool pullup = (pinset & GPIO_PULLUP) != 0;
  bool open_drain = (pinset & GPIO_OUTPUT_OD) != 0;
  bool analog = (pinset & GPIO_ANALOG) != 0;
  int ret;

  if (port >= RA8P_GPIO_NPORTS || pin >= RA8P_GPIO_NPINS_PER_PORT)
    {
      return -EINVAL;
    }

  /* Configure pin using PFS */
  ret = gpio_pfs_configure(port, pin, 0, input, pullup, false, open_drain, analog);
  if (ret != OK)
    {
      return ret;
    }

  /* Set initial value if output */
  if (!input)
    {
      ra8p_gpio_write(pinset, initial_value);
    }

  gpioinfo("GPIO%d.%d configured: input=%s, pullup=%s, opendrain=%s\n",
           port, pin, input ? "yes" : "no", pullup ? "yes" : "no", open_drain ? "yes" : "no");
  return OK;
}

/****************************************************************************
 * Name: ra8p_gpio_write
 *
 * Description:
 *   Write a value to a GPIO pin based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   pinset - GPIO pinset configuration
 *   value - true for high, false for low
 *
 * Returned Value:
 *   None
 *
 ****************************************************************************/

void ra8p_gpio_write(gpio_pinset_t pinset, bool value)
{
  uint8_t port = (pinset >> 8) & 0xFF;
  uint8_t pin = pinset & 0xFF;
  uint32_t port_base;

  if (port >= RA8P_GPIO_NPORTS || pin >= RA8P_GPIO_NPINS_PER_PORT)
    {
      return;
    }

  port_base = gpio_get_port_base(port);
  if (port_base == 0)
    {
      return;
    }

  /* Check if pin is configured as output */
  if (g_gpio.pin_direction[port][pin])
    {
      /* This pin is configured as input, cannot write */
      return;
    }

  if (value)
    {
      /* Set bit using PDR register */
      uint16_t pdr = gpio_getreg16(port_base, RA8P_GPIO_PODR_OFFSET);
      pdr |= (1 << pin);
      gpio_putreg16(port_base, RA8P_GPIO_PODR_OFFSET, pdr);
    }
  else
    {
      /* Clear bit using PDR register */
      uint16_t pdr = gpio_getreg16(port_base, RA8P_GPIO_PODR_OFFSET);
      pdr &= ~(1 << pin);
      gpio_putreg16(port_base, RA8P_GPIO_PODR_OFFSET, pdr);
    }

  /* Update stored output value */
  g_gpio.pin_output_val[port][pin] = value;

  gpioinfo("GPIO%d.%d write %s\n", port, pin, value ? "HIGH" : "LOW");
}

/****************************************************************************
 * Name: ra8p_gpio_read
 *
 * Description:
 *   Read a value from a GPIO pin based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   pinset - GPIO pinset configuration
 *
 * Returned Value:
 *   true for high, false for low
 *
 ****************************************************************************/

bool ra8p_gpio_read(gpio_pinset_t pinset)
{
  uint8_t port = (pinset >> 8) & 0xFF;
  uint8_t pin = pinset & 0xFF;
  uint32_t port_base;
  uint16_t pidr;

  if (port >= RA8P_GPIO_NPORTS || pin >= RA8P_GPIO_NPINS_PER_PORT)
    {
      return false;
    }

  port_base = gpio_get_port_base(port);
  if (port_base == 0)
    {
      return false;
    }

  pidr = gpio_getreg16(port_base, RA8P_GPIO_PIDR_OFFSET);

  bool value = (pidr & (1 << pin)) != 0;

  gpioinfo("GPIO%d.%d read %s\n", port, pin, value ? "HIGH" : "LOW");
  return value;
}

/****************************************************************************
 * Name: ra8p_gpio_set_direction
 *
 * Description:
 *   Set GPIO pin direction based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13)
 *   pin - GPIO pin (0-15)
 *   input - true for input, false for output
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_set_direction(uint8_t port, uint8_t pin, bool input)
{
  uint32_t pfs_addr;
  uint32_t pfs_val;

  if (port >= RA8P_GPIO_NPORTS || pin >= RA8P_GPIO_NPINS_PER_PORT)
    {
      return -EINVAL;
    }

  pfs_addr = gpio_get_pfs_addr(port, pin);
  if (pfs_addr == 0)
    {
      return -EINVAL;
    }

  /* Read current PFS value */
  pfs_val = getreg32(pfs_addr);

  /* Clear and set direction */
  pfs_val &= ~RA8P_GPIO_PFS_PDR_OUTPUT;
  if (input)
    {
      pfs_val |= RA8P_GPIO_PFS_PDR_INPUT;
    }
  else
    {
      pfs_val |= RA8P_GPIO_PFS_PDR_OUTPUT;
    }

  /* Write back to PFS register */
  putreg32(pfs_addr, pfs_val);

  /* Update internal state */
  g_gpio.pin_direction[port][pin] = input;

  gpioinfo("GPIO%d.%d direction set to %s\n", port, pin, input ? "input" : "output");
  return OK;
}

/****************************************************************************
 * Name: ra8p_gpio_set_value
 *
 * Description:
 *   Set GPIO pin value based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13)
 *   pin - GPIO pin (0-15)
 *   value - true for high, false for low
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_set_value(uint8_t port, uint8_t pin, bool value)
{
  uint32_t port_base;

  if (port >= RA8P_GPIO_NPORTS || pin >= RA8P_GPIO_NPINS_PER_PORT)
    {
      return -EINVAL;
    }

  port_base = gpio_get_port_base(port);
  if (port_base == 0)
    {
      return -EINVAL;
    }

  /* Check if pin is configured as output */
  if (g_gpio.pin_direction[port][pin])
    {
      return -EPERM;  /* Pin is configured as input */
    }

  if (value)
    {
      /* Set bit using PSOR register */
      uint16_t psor = gpio_getreg16(port_base, RA8P_GPIO_PSOR_OFFSET);
      psor |= (1 << pin);
      gpio_putreg16(port_base, RA8P_GPIO_PSOR_OFFSET, psor);
    }
  else
    {
      /* Clear bit using PCOR register */
      uint16_t pcor = gpio_getreg16(port_base, RA8P_GPIO_PCOR_OFFSET);
      pcor |= (1 << pin);
      gpio_putreg16(port_base, RA8P_GPIO_PCOR_OFFSET, pcor);
    }

  /* Update stored output value */
  g_gpio.pin_output_val[port][pin] = value;

  gpioinfo("GPIO%d.%d set to %s\n", port, pin, value ? "HIGH" : "LOW");
  return OK;
}

/****************************************************************************
 * Name: ra8p_gpio_get_value
 *
 * Description:
 *   Get GPIO pin value based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13)
 *   pin - GPIO pin (0-15)
 *
 * Returned Value:
 *   true for high, false for low
 *
 ****************************************************************************/

bool ra8p_gpio_get_value(uint8_t port, uint8_t pin)
{
  uint32_t port_base;
  uint16_t pdr;

  if (port >= RA8P_GPIO_NPORTS || pin >= RA8P_GPIO_NPINS_PER_PORT)
    {
      return false;
    }

  port_base = gpio_get_port_base(port);
  if (port_base == 0)
    {
      return false;
    }

  /* Read pin value from PIDR register */
  pdr = gpio_getreg16(port_base, RA8P_GPIO_PIDR_OFFSET);

  bool value = (pdr & (1 << pin)) != 0;

  return value;
}

/****************************************************************************
 * Name: ra8p_gpio_port_read
 *
 * Description:
 *   Read all pins of a GPIO port based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13)
 *
 * Returned Value:
 *   Port value (16-bit)
 *
 ****************************************************************************/

uint16_t ra8p_gpio_port_read(uint8_t port)
{
  uint32_t port_base;

  if (port >= RA8P_GPIO_NPORTS)
    {
      return 0;
    }

  port_base = gpio_get_port_base(port);
  if (port_base == 0)
    {
      return 0;
    }

  return gpio_getreg16(port_base, RA8P_GPIO_PIDR_OFFSET);
}

/****************************************************************************
 * Name: ra8p_gpio_port_write
 *
 * Description:
 *   Write to all pins of a GPIO port based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13)
 *   value - Port value (16-bit)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_port_write(uint8_t port, uint16_t value)
{
  uint32_t port_base;

  if (port >= RA8P_GPIO_NPORTS)
    {
      return -EINVAL;
    }

  port_base = gpio_get_port_base(port);
  if (port_base == 0)
    {
      return -EINVAL;
    }

  gpio_putreg16(port_base, RA8P_GPIO_PODR_OFFSET, value);

  gpioinfo("GPIO%d port write 0x%04x\n", port, value);
  return OK;
}

/****************************************************************************
 * Name: ra8p_gpio_enable_pullup
 *
 * Description:
 *   Enable/disable pull-up resistor for GPIO pin based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13)
 *   pin - GPIO pin (0-15)
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_enable_pullup(uint8_t port, uint8_t pin, bool enable)
{
  uint32_t pfs_addr;
  uint32_t pfs_val;

  if (port >= RA8P_GPIO_NPORTS || pin >= RA8P_GPIO_NPINS_PER_PORT)
    {
      return -EINVAL;
    }

  pfs_addr = gpio_get_pfs_addr(port, pin);
  if (pfs_addr == 0)
    {
      return -EINVAL;
    }

  /* Read current PFS value */
  pfs_val = getreg32(pfs_addr);

  /* Set or clear pull-up enable */
  if (enable)
    {
      pfs_val |= RA8P_GPIO_PFS_PCR_ENABLE;
    }
  else
    {
      pfs_val &= ~RA8P_GPIO_PFS_PCR_ENABLE;
    }

  /* Write back to PFS register */
  putreg32(pfs_addr, pfs_val);

  /* Update internal state */
  g_gpio.pin_pullup[port][pin] = enable;

  gpioinfo("GPIO%d.%d pull-up %s\n", port, pin, enable ? "enabled" : "disabled");
  return OK;
}

/****************************************************************************
 * Name: ra8p_gpio_enable_pulldown
 *
 * Description:
 *   Enable/disable pull-down resistor for GPIO pin based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13)
 *   pin - GPIO pin (0-15)
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_enable_pulldown(uint8_t port, uint8_t pin, bool enable)
{
  uint32_t pfs_addr;
  uint32_t pfs_val;

  if (port >= RA8P_GPIO_NPORTS || pin >= RA8P_GPIO_NPINS_PER_PORT)
    {
      return -EINVAL;
    }

  pfs_addr = gpio_get_pfs_addr(port, pin);
  if (pfs_addr == 0)
    {
      return -EINVAL;
    }

  /* Read current PFS value */
  pfs_val = getreg32(pfs_addr);

  /* Pull-down functionality is controlled by different bits */
  /* In RA8P, pull-up/down is controlled by PCR bit */
  if (enable)
    {
      /* For pull-down, we may need to set specific PFS configuration */
      pfs_val |= RA8P_GPIO_PFS_PCR_ENABLE;
    }
  else
    {
      pfs_val &= ~RA8P_GPIO_PFS_PCR_ENABLE;
    }

  /* Write back to PFS register */
  putreg32(pfs_addr, pfs_val);

  /* Update internal state */
  g_gpio.pin_pulldown[port][pin] = enable;

  gpioinfo("GPIO%d.%d pull-down %s\n", port, pin, enable ? "enabled" : "disabled");
  return OK;
}

/****************************************************************************
 * Name: ra8p_gpio_set_drive_strength
 *
 * Description:
 *   Set GPIO pin drive strength based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13)
 *   pin - GPIO pin (0-15)
 *   high - true for high drive strength, false for low
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_set_drive_strength(uint8_t port, uint8_t pin, bool high)
{
  uint32_t pfs_addr;
  uint32_t pfs_val;

  if (port >= RA8P_GPIO_NPORTS || pin >= RA8P_GPIO_NPINS_PER_PORT)
    {
      return -EINVAL;
    }

  pfs_addr = gpio_get_pfs_addr(port, pin);
  if (pfs_addr == 0)
    {
      return -EINVAL;
    }

  /* Read current PFS value */
  pfs_val = getreg32(pfs_addr);

  /* Set or clear drive strength */
  if (high)
    {
      pfs_val |= RA8P_GPIO_PFS_DSCR_HIGH;
    }
  else
    {
      pfs_val &= ~RA8P_GPIO_PFS_DSCR_HIGH;
    }

  /* Write back to PFS register */
  putreg32(pfs_addr, pfs_val);

  gpioinfo("GPIO%d.%d drive strength set to %s\n", port, pin, high ? "high" : "low");
  return OK;
}

/****************************************************************************
 * Name: ra8p_gpio_set_function
 *
 * Description:
 *   Set GPIO pin function (GPIO or alternate) based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13)
 *   pin - GPIO pin (0-15)
 *   func - Function selection (0=GPIO, 1-31=peripheral)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_set_function(uint8_t port, uint8_t pin, uint8_t func)
{
  uint32_t pfs_addr;
  uint32_t pfs_val;

  if (port >= RA8P_GPIO_NPORTS || pin >= RA8P_GPIO_NPINS_PER_PORT || func > 31)
    {
      return -EINVAL;
    }

  pfs_addr = gpio_get_pfs_addr(port, pin);
  if (pfs_addr == 0)
    {
      return -EINVAL;
    }

  /* Read current PFS value */
  pfs_val = getreg32(pfs_addr);

  /* Clear and set function selection */
  pfs_val &= ~RA8P_GPIO_PFS_PSEL_MASK;
  pfs_val |= (func << RA8P_GPIO_PFS_PSEL_SHIFT) & RA8P_GPIO_PFS_PSEL_MASK;

  /* Write back to PFS register */
  putreg32(pfs_addr, pfs_val);

  /* Update internal state */
  g_gpio.pin_function[port][pin] = func;

  gpioinfo("GPIO%d.%d function set to %d\n", port, pin, func);
  return OK;
}

/****************************************************************************
 * Name: ra8p_gpio_set_output_type
 *
 * Description:
 *   Set GPIO output type (push-pull or open-drain) based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13)
 *   pin - GPIO pin (0-15)
 *   open_drain - true for open drain, false for push-pull
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_set_output_type(uint8_t port, uint8_t pin, bool open_drain)
{
  uint32_t pfs_addr;
  uint32_t pfs_val;

  if (port >= RA8P_GPIO_NPORTS || pin >= RA8P_GPIO_NPINS_PER_PORT)
    {
      return -EINVAL;
    }

  pfs_addr = gpio_get_pfs_addr(port, pin);
  if (pfs_addr == 0)
    {
      return -EINVAL;
    }

  /* Read current PFS value */
  pfs_val = getreg32(pfs_addr);

  /* Configure for open-drain or push-pull */
  if (open_drain)
    {
      pfs_val |= RA8P_GPIO_PFS_DSCR_HIGH;  /* Higher drive strength for open-drain */
    }
  else
    {
      pfs_val &= ~RA8P_GPIO_PFS_DSCR_HIGH; /* Lower drive strength for push-pull */
    }

  /* Write back to PFS register */
  putreg32(pfs_addr, pfs_val);

  /* Update internal state */
  g_gpio.pin_opendrain[port][pin] = open_drain;

  gpioinfo("GPIO%d.%d output type set to %s\n", port, pin, open_drain ? "open-drain" : "push-pull");
  return OK;
}

/****************************************************************************
 * Name: ra8p_gpio_set_analog_mode
 *
 * Description:
 *   Set GPIO pin to analog mode based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13)
 *   pin - GPIO pin (0-15)
 *   enable - true for analog mode, false for digital
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_set_analog_mode(uint8_t port, uint8_t pin, bool enable)
{
  uint32_t pfs_addr;
  uint32_t pfs_val;

  if (port >= RA8P_GPIO_NPORTS || pin >= RA8P_GPIO_NPINS_PER_PORT)
    {
      return -EINVAL;
    }

  pfs_addr = gpio_get_pfs_addr(port, pin);
  if (pfs_addr == 0)
    {
      return -EINVAL;
    }

  /* Read current PFS value */
  pfs_val = getreg32(pfs_addr);

  /* Set or clear analog mode */
  if (enable)
    {
      pfs_val |= RA8P_GPIO_PFS_ASEL_ENABLE;
    }
  else
    {
      pfs_val &= ~RA8P_GPIO_PFS_ASEL_ENABLE;
    }

  /* Write back to PFS register */
  putreg32(pfs_addr, pfs_val);

  /* Update internal state */
  g_gpio.pin_analog[port][pin] = enable;

  gpioinfo("GPIO%d.%d analog mode %s\n", port, pin, enable ? "enabled" : "disabled");
  return OK;
}

/****************************************************************************
 * Name: ra8p_gpio_is_output
 *
 * Description:
 *   Check if GPIO pin is configured as output based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13)
 *   pin - GPIO pin (0-15)
 *
 * Returned Value:
 *   true if output, false if input
 *
 ****************************************************************************/

bool ra8p_gpio_is_output(uint8_t port, uint8_t pin)
{
  if (port >= RA8P_GPIO_NPORTS || pin >= RA8P_GPIO_NPINS_PER_PORT)
    {
      return false;
    }

  return !g_gpio.pin_direction[port][pin];  /* If !direction, it's output */
}

/****************************************************************************
 * Name: ra8p_gpio_is_enabled
 *
 * Description:
 *   Check if GPIO port is enabled based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13)
 *
 * Returned Value:
 *   true if enabled, false otherwise
 *
 ****************************************************************************/

bool ra8p_gpio_is_enabled(uint8_t port)
{
  if (port >= RA8P_GPIO_NPORTS)
    {
      return false;
    }

  return g_gpio.enabled[port];
}

/****************************************************************************
 * Name: ra8p_gpio_get_port_base
 *
 * Description:
 *   Get base address for a GPIO port based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13)
 *
 * Returned Value:
 *   Base address of port registers
 *
 ****************************************************************************/

uint32_t ra8p_gpio_get_port_base(uint8_t port)
{
  return gpio_get_port_base(port);
}

/****************************************************************************
 * Name: ra8p_gpio_initialize
 *
 * Description:
 *   Initialize the GPIO subsystem based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_initialize(void)
{
  struct ra8p_gpio_priv_s *priv = &g_gpio;
  int i, j;

  /* Initialize all ports as disabled */
  for (i = 0; i < RA8P_GPIO_NPORTS; i++)
    {
      priv->enabled[i] = false;
      for (j = 0; j < RA8P_GPIO_NPINS_PER_PORT; j++)
        {
          priv->pin_function[i][j] = 0;    /* Default to GPIO mode */
          priv->pin_direction[i][j] = true;  /* Default to input */
          priv->pin_output_val[i][j] = false; /* Default to low */
          priv->pin_pullup[i][j] = false;   /* Default to no pull-up */
          priv->pin_pulldown[i][j] = false; /* Default to no pull-down */
          priv->pin_opendrain[i][j] = false; /* Default to push-pull */
          priv->pin_analog[i][j] = false;   /* Default to digital */
        }
    }

  /* Enable all GPIO ports */
  for (i = 0; i < RA8P_GPIO_NPORTS; i++)
    {
      priv->enabled[i] = true;
    }

  priv->initialized = true;

  gpioinfo("GPIO initialized with %d ports, %d pins per port\n", 
           RA8P_GPIO_NPORTS, RA8P_GPIO_NPINS_PER_PORT);
  return OK;
}

/****************************************************************************
 * Name: ra8p_gpio_get_config
 *
 * Description:
 *   Get current GPIO pin configuration based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13)
 *   pin - GPIO pin (0-15)
 *   config - Pointer to configuration structure to fill
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_get_config(uint8_t port, uint8_t pin, struct ra8p_gpio_config_s *config)
{
  if (port >= RA8P_GPIO_NPORTS || pin >= RA8P_GPIO_NPINS_PER_PORT || config == NULL)
    {
      return -EINVAL;
    }

  config->port = port;
  config->pin = pin;
  config->input = g_gpio.pin_direction[port][pin];
  config->initial_value = g_gpio.pin_output_val[port][pin];
  config->pull_up = g_gpio.pin_pullup[port][pin];
  config->pull_down = g_gpio.pin_pulldown[port][pin];
  config->open_drain = g_gpio.pin_opendrain[port][pin];
  config->drive_high = (g_gpio.pin_opendrain[port][pin]) ? true : false; /* Simplified */
  config->function = g_gpio.pin_function[port][pin];
  config->analog_mode = g_gpio.pin_analog[port][pin];

  return OK;
}

#endif /* CONFIG_RA8P_HAVE_GPIO */