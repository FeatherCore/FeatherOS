/****************************************************************************
 * arch/arm/src/ra8p/ra8p_usbphy.c
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
#include "hardware/ra8p_usbphy.h"
#include "hardware/ra8p_memorymap.h"

#ifdef CONFIG_RA8P_USBPHY

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define RA8P_USBPHY_BASE                RA8P_USBPHY_BASE

/* USBPHY timeout in milliseconds */
#define RA8P_USBPHY_TIMEOUT_MS          1000

/* USBPHY refresh sequences */
#define RA8P_USBPHY_REFRESH_SEQ1        0xAC
#define RA8P_USBPHY_REFRESH_SEQ2        0x53

/* Default USBPHY frequency (48 MHz) */
#define RA8P_USBPHY_DEFAULT_FREQ        48000000

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* RA8P1 USBPHY driver state structure */

struct ra8p_usbphy_priv_s
{
  uint32_t base;                              /* Base address of USBPHY registers */
  bool initialized;                           /* Initialization flag */
  bool enabled;                               /* Enable flag */
  bool pll_enabled;                           /* PLL status */
  bool pll_locked;                           /* PLL lock status */
  uint8_t mode;                              /* Current mode (FS/HS) */
  bool dplus_pullup;                         /* D+ pull-up enabled */
  bool dminus_pulldown;                      /* D- pull-down enabled */
  uint32_t frequency;                        /* Current frequency */
  uint32_t pclk;                             /* PCLK frequency for calculations */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int usbphy_wait_pll_locked(struct ra8p_usbphy_priv_s *priv);
static void usbphy_putreg32(struct ra8p_usbphy_priv_s *priv, uint32_t offset, uint32_t value);
static uint32_t usbphy_getreg32(struct ra8p_usbphy_priv_s *priv, uint32_t offset);
static void usbphy_putreg16(struct ra8p_usbphy_priv_s *priv, uint32_t offset, uint16_t value);
static uint16_t usbphy_getreg16(struct ra8p_usbphy_priv_s *priv, uint32_t offset);
static void usbphy_putreg8(struct ra8p_usbphy_priv_s *priv, uint32_t offset, uint8_t value);
static uint8_t usbphy_getreg8(struct ra8p_usbphy_priv_s *priv, uint32_t offset);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_usbphy_priv_s g_usbphy;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: usbphy_putreg32
 ****************************************************************************/

static inline void usbphy_putreg32(struct ra8p_usbphy_priv_s *priv, uint32_t offset, uint32_t value)
{
  putreg32(value, priv->base + offset);
}

/****************************************************************************
 * Name: usbphy_getreg32
 ****************************************************************************/

static inline uint32_t usbphy_getreg32(struct ra8p_usbphy_priv_s *priv, uint32_t offset)
{
  return getreg32(priv->base + offset);
}

/****************************************************************************
 * Name: usbphy_putreg16
 ****************************************************************************/

static inline void usbphy_putreg16(struct ra8p_usbphy_priv_s *priv, uint32_t offset, uint16_t value)
{
  putreg16(value, priv->base + offset);
}

/****************************************************************************
 * Name: usbphy_getreg16
 ****************************************************************************/

static inline uint16_t usbphy_getreg16(struct ra8p_usbphy_priv_s *priv, uint32_t offset)
{
  return getreg16(priv->base + offset);
}

/****************************************************************************
 * Name: usbphy_putreg8
 ****************************************************************************/

static inline void usbphy_putreg8(struct ra8p_usbphy_priv_s *priv, uint32_t offset, uint8_t value)
{
  putreg8(value, priv->base + offset);
}

/****************************************************************************
 * Name: usbphy_getreg8
 ****************************************************************************/

static inline uint8_t usbphy_getreg8(struct ra8p_usbphy_priv_s *priv, uint32_t offset)
{
  return getreg8(priv->base + offset);
}

/****************************************************************************
 * Name: usbphy_wait_pll_locked
 ****************************************************************************/

static int usbphy_wait_pll_locked(struct ra8p_usbphy_priv_s *priv)
{
  volatile int timeout = RA8P_USBPHY_TIMEOUT_MS * 1000;  /* microseconds */

  /* Wait for PLL to lock */
  while (!(usbphy_getreg8(priv, RA8P_USBPHY_PLLSTAT_OFFSET) & RA8P_USBPHY_PLLSTAT_PLL_LOCK) && 
         timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  priv->pll_locked = true;
  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_usbphy_initialize
 *
 * Description:
 *   Initialize the USBPHY controller based on Nuttx style.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_initialize(void)
{
  struct ra8p_usbphy_priv_s *priv = &g_usbphy;
  uint8_t regval;
  int ret;

  priv->base = RA8P_USBPHY_BASE;
  priv->initialized = false;
  priv->enabled = false;
  priv->pll_enabled = false;
  priv->pll_locked = false;
  priv->mode = RA8P_USBPHY_MODE_FS;  /* Default to full speed */
  priv->dplus_pullup = false;
  priv->dminus_pulldown = true;
  priv->frequency = RA8P_USBPHY_DEFAULT_FREQ;
  priv->pclk = 62500000;  /* Use appropriate PCLKB frequency */

  /* Reset USBPHY */
  regval = usbphy_getreg8(priv, RA8P_USBPHY_PHYCONF_OFFSET);
  regval |= RA8P_USBPHY_PHYCONF_PLLRESET;  /* Set PLL reset */
  regval |= RA8P_USBPHY_PHYCONF_USBPWDN;   /* Power down */
  usbphy_putreg8(priv, RA8P_USBPHY_PHYCONF_OFFSET, regval);

  /* Configure PHY timing settings */
  regval = RA8P_USBPHY_PHYTIM0_USBSIGMD;  /* Enable USB signal mode */
  regval &= ~RA8P_USBPHY_PHYTIM0_USBSUSPEND;  /* Clear USB suspend */
  usbphy_putreg8(priv, RA8P_USBPHY_PHYTIM0_OFFSET, regval);

  /* Configure pad control */
  regval = RA8P_USBPHY_PHYPADCTL_HSEDM;   /* Enable Hi-Speed DM */
  regval |= RA8P_USBPHY_PHYPADCTL_HSEDPP;  /* Enable Hi-Speed D+ */
  regval |= RA8P_USBPHY_PHYPADCTL_DMRPD;  /* Enable D- Resistor Pull-down */
  regval |= RA8P_USBPHY_PHYPADCTL_DPRPU;  /* Enable D+ Resistor Pull-up */
  regval |= RA8P_USBPHY_PHYPADCTL_DVS;    /* Enable Data Validation Sequence */
  usbphy_putreg8(priv, RA8P_USBPHY_PHYPADCTL_OFFSET, regval);

  /* Configure PLL control */
  regval = usbphy_getreg8(priv, RA8P_USBPHY_PLLCTRL_OFFSET);
  regval |= RA8P_USBPHY_PLLCTRL_PLL_EN;      /* Enable PLL */
  regval &= ~RA8P_USBPHY_PLLCTRL_PLL_RESET; /* Clear PLL reset */
  regval |= RA8P_USBPHY_PLLCTRL_PLL_CLKSEL; /* Select PLL clock */
  usbphy_putreg8(priv, RA8P_USBPHY_PLLCTRL_OFFSET, regval);

  /* Wait for PLL to lock */
  ret = usbphy_wait_pll_locked(priv);
  if (ret != OK)
    {
      return ret;
    }

  /* Configure additional PHY settings */
  regval = usbphy_getreg8(priv, RA8P_USBPHY_PHYADDITION_OFFSET);
  regval |= 0x01;  /* Enable additional PHY functions */
  usbphy_putreg8(priv, RA8P_USBPHY_PHYADDITION_OFFSET, regval);

  /* Configure PHY PLL control */
  regval = usbphy_getreg8(priv, RA8P_USBPHY_PHYPLLCTRL_OFFSET);
  regval |= 0x0F;  /* Set PLL control bits */
  usbphy_putreg8(priv, RA8P_USBPHY_PHYPLLCTRL_OFFSET, regval);

  /* Configure external oscillator if needed */
  regval = usbphy_getreg8(priv, RA8P_USBPHY_PHYCONF_OFFSET);
  regval &= ~RA8P_USBPHY_PHYCONF_PLLRESET;  /* Clear PLL reset */
  regval &= ~RA8P_USBPHY_PHYCONF_USBPWDN;   /* Clear power down */
  regval |= RA8P_USBPHY_PHYCONF_PLLWAIT;    /* Enable PLL wait */
  usbphy_putreg8(priv, RA8P_USBPHY_PHYCONF_OFFSET, regval);

  /* Enable pull-up resistor for D+ */
  regval = usbphy_getreg8(priv, RA8P_USBPHY_PHYPADCTL_OFFSET);
  regval |= RA8P_USBPHY_PHYPADCTL_DPRPU;  /* Enable D+ pull-up */
  usbphy_putreg8(priv, RA8P_USBPHY_PHYPADCTL_OFFSET, regval);

  priv->initialized = true;
  priv->pll_enabled = true;
  priv->enabled = true;
  priv->dplus_pullup = true;

  usbphyinfo("USBPHY initialized at 0x%08x\n", priv->base);
  return OK;
}

/****************************************************************************
 * Name: ra8p_usbphy_enable
 *
 * Description:
 *   Enable the USBPHY controller based on Nuttx style.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_enable(void)
{
  struct ra8p_usbphy_priv_s *priv = &g_usbphy;
  uint8_t regval;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  /* Exit power-down mode */
  regval = usbphy_getreg8(priv, RA8P_USBPHY_PHYCONF_OFFSET);
  regval &= ~RA8P_USBPHY_PHYCONF_USBPWDN;  /* Clear power down */
  usbphy_putreg8(priv, RA8P_USBPHY_PHYCONF_OFFSET, regval);

  /* Wait for PHY to be ready */
  volatile int timeout = RA8P_USBPHY_TIMEOUT_MS * 1000;
  while ((usbphy_getreg8(priv, RA8P_USBPHY_PHYSTAT_OFFSET) & 0x01) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  /* Enable PLL if not already enabled */
  if (!priv->pll_enabled)
    {
      regval = usbphy_getreg8(priv, RA8P_USBPHY_PLLCTRL_OFFSET);
      regval |= RA8P_USBPHY_PLLCTRL_PLL_EN;
      regval &= ~RA8P_USBPHY_PLLCTRL_PLL_RESET;
      usbphy_putreg8(priv, RA8P_USBPHY_PLLCTRL_OFFSET, regval);

      int ret = usbphy_wait_pll_locked(priv);
      if (ret != OK)
        {
          return ret;
        }

      priv->pll_enabled = true;
    }

  /* Enable D+ pull-up if requested */
  if (priv->dplus_pullup)
    {
      regval = usbphy_getreg8(priv, RA8P_USBPHY_PHYPADCTL_OFFSET);
      regval |= RA8P_USBPHY_PHYPADCTL_DPRPU;  /* Enable D+ pull-up */
      usbphy_putreg8(priv, RA8P_USBPHY_PHYPADCTL_OFFSET, regval);
    }

  priv->enabled = true;

  usbphyinfo("USBPHY enabled\n");
  return OK;
}

/****************************************************************************
 * Name: ra8p_usbphy_disable
 *
 * Description:
 *   Disable the USBPHY controller based on Nuttx style.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_disable(void)
{
  struct ra8p_usbphy_priv_s *priv = &g_usbphy;
  uint8_t regval;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  /* Disable D+ pull-up */
  regval = usbphy_getreg8(priv, RA8P_USBPHY_PHYPADCTL_OFFSET);
  regval &= ~RA8P_USBPHY_PHYPADCTL_DPRPU;  /* Disable D+ pull-up */
  usbphy_putreg8(priv, RA8P_USBPHY_PHYPADCTL_OFFSET, regval);

  /* Put PHY in power-down mode */
  regval = usbphy_getreg8(priv, RA8P_USBPHY_PHYCONF_OFFSET);
  regval |= RA8P_USBPHY_PHYCONF_USBPWDN;   /* Set power down */
  regval |= RA8P_USBPHY_PHYCONF_PLLRESET; /* Set PLL reset */
  usbphy_putreg8(priv, RA8P_USBPHY_PHYCONF_OFFSET, regval);

  /* Disable PLL */
  regval = usbphy_getreg8(priv, RA8P_USBPHY_PLLCTRL_OFFSET);
  regval &= ~RA8P_USBPHY_PLLCTRL_PLL_EN;
  regval |= RA8P_USBPHY_PLLCTRL_PLL_RESET;
  usbphy_putreg8(priv, RA8P_USBPHY_PLLCTRL_OFFSET, regval);

  priv->enabled = false;
  priv->pll_enabled = false;
  priv->pll_locked = false;

  usbphyinfo("USBPHY disabled\n");
  return OK;
}

/****************************************************************************
 * Name: ra8p_usbphy_set_mode
 *
 * Description:
 *   Set USBPHY mode (FS/HS) based on Nuttx style.
 *
 * Input Parameters:
 *   mode - PHY mode (0=FS, 1=HS)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_set_mode(uint8_t mode)
{
  struct ra8p_usbphy_priv_s *priv = &g_usbphy;
  uint8_t padctl;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  if (mode > RA8P_USBPHY_MODE_HSIC)
    {
      return -EINVAL;
    }

  padctl = usbphy_getreg8(priv, RA8P_USBPHY_PHYPADCTL_OFFSET);

  if (mode == RA8P_USBPHY_MODE_HS)
    {
      /* Configure for high-speed mode */
      padctl |= RA8P_USBPHY_PHYPADCTL_HSEDM;   /* Enable Hi-Speed DM */
      padctl |= RA8P_USBPHY_PHYPADCTL_HSEDPP;  /* Enable Hi-Speed D+ */
      usbphyinfo("USBPHY set to high-speed mode\n");
    }
  else if (mode == RA8P_USBPHY_MODE_HSIC)
    {
      /* Configure for high-speed IC mode */
      padctl |= RA8P_USBPHY_PHYPADCTL_HSEDM;   /* Enable Hi-Speed DM */
      padctl |= RA8P_USBPHY_PHYPADCTL_HSEDPP;  /* Enable Hi-Speed D+ */
      padctl &= ~RA8P_USBPHY_PHYCONF_XCVRSELDPS;  /* Disable D+ pull-up */
      usbphyinfo("USBPHY set to high-speed IC mode\n");
    }
  else
    {
      /* Configure for full-speed mode */
      padctl &= ~RA8P_USBPHY_PHYPADCTL_HSEDM;  /* Disable Hi-Speed DM */
      padctl &= ~RA8P_USBPHY_PHYPADCTL_HSEDPP; /* Disable Hi-Speed D+ */
      padctl |= RA8P_USBPHY_PHYCONF_XCVRSELDPS; /* Enable D+ pull-up */
      usbphyinfo("USBPHY set to full-speed mode\n");
    }

  usbphy_putreg8(priv, RA8P_USBPHY_PHYPADCTL_OFFSET, padctl);

  priv->mode = mode;

  return OK;
}

/****************************************************************************
 * Name: ra8p_usbphy_set_pullup
 *
 * Description:
 *   Set USB D+ pull-up resistor based on Nuttx style.
 *
 * Input Parameters:
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_set_pullup(bool enable)
{
  struct ra8p_usbphy_priv_s *priv = &g_usbphy;
  uint8_t regval;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  regval = usbphy_getreg8(priv, RA8P_USBPHY_PHYCONF_OFFSET);

  if (enable)
    {
      regval |= RA8P_USBPHY_PHYCONF_XCVRSELDPS;  /* Enable D+ pull-up */
      priv->dplus_pullup = true;
    }
  else
    {
      regval &= ~RA8P_USBPHY_PHYCONF_XCVRSELDPS; /* Disable D+ pull-up */
      priv->dplus_pullup = false;
    }

  usbphy_putreg8(priv, RA8P_USBPHY_PHYCONF_OFFSET, regval);

  usbphyinfo("USB D+ pull-up %s\n", enable ? "enabled" : "disabled");
  return OK;
}

/****************************************************************************
 * Name: ra8p_usbphy_set_pulldown
 *
 * Description:
 *   Set USB D- pull-down resistor based on Nuttx style.
 *
 * Input Parameters:
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_set_pulldown(bool enable)
{
  struct ra8p_usbphy_priv_s *priv = &g_usbphy;
  uint8_t regval;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  regval = usbphy_getreg8(priv, RA8P_USBPHY_PHYTIM0_OFFSET);

  if (enable)
    {
      regval |= RA8P_USBPHY_PHYTIM0_USBDMPD;  /* Enable D- pull-down */
    }
  else
    {
      regval &= ~RA8P_USBPHY_PHYTIM0_USBDMPD; /* Disable D- pull-down */
    }

  usbphy_putreg8(priv, RA8P_USBPHY_PHYTIM0_OFFSET, regval);

  usbphyinfo("USB D- pull-down %s\n", enable ? "enabled" : "disabled");
  return OK;
}

/****************************************************************************
 * Name: ra8p_usbphy_pll_enable
 *
 * Description:
 *   Enable USBPHY PLL based on Nuttx style.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_pll_enable(void)
{
  struct ra8p_usbphy_priv_s *priv = &g_usbphy;
  uint8_t regval;
  int ret;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  /* Enable PLL */
  regval = usbphy_getreg8(priv, RA8P_USBPHY_PLLCTRL_OFFSET);
  regval |= RA8P_USBPHY_PLLCTRL_PLL_EN;
  regval &= ~RA8P_USBPHY_PLLCTRL_PLL_RESET;
  usbphy_putreg8(priv, RA8P_USBPHY_PLLCTRL_OFFSET, regval);

  /* Wait for PLL to lock */
  ret = usbphy_wait_pll_locked(priv);
  if (ret != OK)
    {
      return ret;
    }

  priv->pll_enabled = true;

  usbphyinfo("USBPHY PLL enabled\n");
  return OK;
}

/****************************************************************************
 * Name: ra8p_usbphy_pll_disable
 *
 * Description:
 *   Disable USBPHY PLL based on Nuttx style.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_pll_disable(void)
{
  struct ra8p_usbphy_priv_s *priv = &g_usbphy;
  uint8_t regval;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  /* Disable PLL */
  regval = usbphy_getreg8(priv, RA8P_USBPHY_PLLCTRL_OFFSET);
  regval &= ~RA8P_USBPHY_PLLCTRL_PLL_EN;
  regval |= RA8P_USBPHY_PLLCTRL_PLL_RESET;
  usbphy_putreg8(priv, RA8P_USBPHY_PLLCTRL_OFFSET, regval);

  priv->pll_enabled = false;
  priv->pll_locked = false;

  usbphyinfo("USBPHY PLL disabled\n");
  return OK;
}

/****************************************************************************
 * Name: ra8p_usbphy_is_pll_locked
 *
 * Description:
 *   Check if USBPHY PLL is locked based on Nuttx style.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if PLL is locked, false otherwise
 *
 ****************************************************************************/

bool ra8p_usbphy_is_pll_locked(void)
{
  struct ra8p_usbphy_priv_s *priv = &g_usbphy;
  uint8_t regval;

  if (!priv->initialized)
    {
      return false;
    }

  regval = usbphy_getreg8(priv, RA8P_USBPHY_PLLSTAT_OFFSET);
  return (regval & RA8P_USBPHY_PLLSTAT_PLL_LOCK) != 0;
}

/****************************************************************************
 * Name: ra8p_usbphy_power_down
 *
 * Description:
 *   Put USBPHY in power-down mode based on Nuttx style.
 *
 * Input Parameters:
 *   enable - true to enter power-down, false to exit
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_power_down(bool enable)
{
  struct ra8p_usbphy_priv_s *priv = &g_usbphy;
  uint8_t regval;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  regval = usbphy_getreg8(priv, RA8P_USBPHY_PHYCONF_OFFSET);

  if (enable)
    {
      regval |= RA8P_USBPHY_PHYCONF_USBPWDN;   /* Set power down */
    }
  else
    {
      regval &= ~RA8P_USBPHY_PHYCONF_USBPWDN;  /* Clear power down */
    }

  usbphy_putreg8(priv, RA8P_USBPHY_PHYCONF_OFFSET, regval);

  usbphyinfo("USBPHY power-down %s\n", enable ? "enabled" : "disabled");
  return OK;
}

/****************************************************************************
 * Name: ra8p_usbphy_get_status
 *
 * Description:
 *   Get USBPHY status flags based on Nuttx style.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Status flags
 *
 ****************************************************************************/

uint8_t ra8p_usbphy_get_status(void)
{
  struct ra8p_usbphy_priv_s *priv = &g_usbphy;

  if (!priv->initialized)
    {
      return 0;
    }

  return usbphy_getreg8(priv, RA8P_USBPHY_PHYSTAT_OFFSET);
}

/****************************************************************************
 * Name: ra8p_usbphy_get_error_flags
 *
 * Description:
 *   Get USBPHY error flags based on Nuttx style.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Error flags
 *
 ****************************************************************************/

uint32_t ra8p_usbphy_get_error_flags(void)
{
  struct ra8p_usbphy_priv_s *priv = &g_usbphy;

  if (!priv->initialized)
    {
      return 0;
    }

  return usbphy_getreg8(priv, RA8P_USBPHY_PHYERR_OFFSET);
}

/****************************************************************************
 * Name: ra8p_usbphy_clear_error
 *
 * Description:
 *   Clear USBPHY error flags based on Nuttx style.
 *
 * Input Parameters:
 *   error_flags - Error flags to clear
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_clear_error(uint32_t error_flags)
{
  struct ra8p_usbphy_priv_s *priv = &g_usbphy;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  usbphy_putreg8(priv, RA8P_USBPHY_PHYERRCLR_OFFSET, (uint8_t)error_flags);

  usbphyinfo("USBPHY error flags cleared: 0x%02x\n", (uint8_t)error_flags);
  return OK;
}

/****************************************************************************
 * Name: ra8p_usbphy_is_initialized
 *
 * Description:
 *   Check if USBPHY is initialized based on Nuttx style.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if initialized, false otherwise
 *
 ****************************************************************************/

bool ra8p_usbphy_is_initialized(void)
{
  return g_usbphy.initialized;
}

/****************************************************************************
 * Name: ra8p_usbphy_is_enabled
 *
 * Description:
 *   Check if USBPHY is enabled based on Nuttx style.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if enabled, false otherwise
 *
 ****************************************************************************/

bool ra8p_usbphy_is_enabled(void)
{
  return g_usbphy.enabled;
}

/****************************************************************************
 * Name: ra8p_usbphy_reset
 *
 * Description:
 *   Reset USBPHY controller based on Nuttx style.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_reset(void)
{
  struct ra8p_usbphy_priv_s *priv = &g_usbphy;
  uint8_t regval;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  /* Reset PHY by setting PLL reset and power down */
  regval = usbphy_getreg8(priv, RA8P_USBPHY_PHYCONF_OFFSET);
  regval |= RA8P_USBPHY_PHYCONF_PLLRESET;  /* Set PLL reset */
  regval |= RA8P_USBPHY_PHYCONF_USBPWDN;   /* Power down */
  usbphy_putreg8(priv, RA8P_USBPHY_PHYCONF_OFFSET, regval);

  /* Wait briefly */
  up_udelay(100);

  /* Clear reset and power down */
  regval &= ~RA8P_USBPHY_PHYCONF_PLLRESET;  /* Clear PLL reset */
  regval &= ~RA8P_USBPHY_PHYCONF_USBPWDN;   /* Clear power down */
  usbphy_putreg8(priv, RA8P_USBPHY_PHYCONF_OFFSET, regval);

  usbphyinfo("USBPHY reset complete\n");
  return OK;
}

/****************************************************************************
 * Name: ra8p_usbphy_set_frequency
 *
 * Description:
 *   Set USBPHY frequency based on Nuttx style.
 *
 * Input Parameters:
 *   frequency - Desired frequency in Hz
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_set_frequency(uint32_t frequency)
{
  struct ra8p_usbphy_priv_s *priv = &g_usbphy;
  
  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  if (frequency == 0)
    {
      return -EINVAL;
    }

  /* In the RA8P1 implementation, USBPHY frequency is controlled through 
   * the CGC (Clock Generation Circuit) and the USBFS/USBHS clock settings.
   * The USBPHY itself is usually fixed to 48MHz or 24MHz depending on configuration.
   */
   
  priv->frequency = frequency;
  usbphyinfo("USBPHY frequency set to %u Hz\n", frequency);
  return OK;
}

/****************************************************************************
 * Name: ra8p_usbphy_get_frequency
 *
 * Description:
 *   Get current USBPHY frequency based on Nuttx style.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Current frequency in Hz
 *
 ****************************************************************************/

uint32_t ra8p_usbphy_get_frequency(void)
{
  struct ra8p_usbphy_priv_s *priv = &g_usbphy;

  if (!priv->initialized)
    {
      return 0;
    }

  return priv->frequency;
}

/****************************************************************************
 * Name: ra8p_usbphy_set_high_speed
 *
 * Description:
 *   Set high-speed mode for USBPHY based on Nuttx style.
 *
 * Input Parameters:
 *   enable - true to enable high-speed mode, false for full-speed
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_set_high_speed(bool enable)
{
  return ra8p_usbphy_set_mode(enable ? RA8P_USBPHY_MODE_HS : RA8P_USBPHY_MODE_FS);
}

/****************************************************************************
 * Name: ra8p_usbphy_is_vbus_detected
 *
 * Description:
 *   Check if VBUS is detected based on Nuttx style.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if VBUS detected, false otherwise
 *
 ****************************************************************************/

bool ra8p_usbphy_is_vbus_detected(void)
{
  struct ra8p_usbphy_priv_s *priv = &g_usbphy;
  uint8_t regval;

  if (!priv->initialized)
    {
      return false;
    }

  regval = usbphy_getreg8(priv, RA8P_USBPHY_PHYSTAT_OFFSET);
  return (regval & RA8P_USBPHY_PHYSTAT_VBUSSTS) != 0;
}

/****************************************************************************
 * Name: ra8p_usbphy_is_overcurrent
 *
 * Description:
 *   Check if overcurrent condition occurred based on Nuttx style.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if overcurrent, false otherwise
 *
 ****************************************************************************/

bool ra8p_usbphy_is_overcurrent(void)
{
  struct ra8p_usbphy_priv_s *priv = &g_usbphy;
  uint8_t regval;

  if (!priv->initialized)
    {
      return false;
    }

  regval = usbphy_getreg8(priv, RA8P_USBPHY_PHYSTAT_OFFSET);
  return (regval & RA8P_USBPHY_PHYSTAT_OVRCURSTS) != 0;
}

#endif /* CONFIG_RA8P_USBPHY */