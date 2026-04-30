/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_i3c.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_I3C_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_I3C_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* I3C Register Offsets - Based on RA8P Hardware Manual and Zephyr definitions */
#define RA8P_I3C_CON_OFFSET              0x0000  /* Control Register */
#define RA8P_I3C_IBCC_OFFSET             0x0004  /* IB Bus Control Register */
#define RA8P_I3C_MSCON_OFFSET            0x0008  /* Master-Specific Control Register */
#define RA8P_I3C_MWSR_OFFSET             0x000C  /* Master Write Status Register */
#define RA8P_I3C_MRSR_OFFSET             0x0010  /* Master Read Status Register */
#define RA8P_I3C_TDER_OFFSET             0x0014  /* TD Early Report Register */
#define RA8P_I3C_CAP_OFFSET              0x0018  /* Capabilities Register */
#define RA8P_I3C_STATUS_OFFSET           0x001C  /* Status Register */
#define RA8P_I3C_IBSTATE_OFFSET          0x0020  /* IB Bus State Register */
#define RA8P_I3C_MERR_OFFSET             0x0024  /* Master Error Register */
#define RA8P_I3C_SERR_OFFSET             0x0028  /* Slave Error Register */
#define RA8P_I3C_MINT_OFFSET             0x002C  /* Master Interrupt Register */
#define RA8P_I3C_SINT_OFFSET             0x0030  /* Slave Interrupt Register */
#define RA8P_I3C_INTSET_OFFSET           0x0034  /* Interrupt Set Register */
#define RA8P_I3C_INTMASK_OFFSET          0x0038  /* Interrupt Mask Register */
#define RA8P_I3C_IBDAT_OFFSET            0x0040  /* IB Bus Data Register */
#define RA8P_I3C_IBADDR_OFFSET           0x0044  /* IB Bus Address Register */
#define RA8P_I3C_IBPIDS_OFFSET           0x0048  /* IB Bus PID for Static Address */
#define RA8P_I3C_IBDCT_OFFSET            0x004C  /* IB Bus Device Count Register */
#define RA8P_I3C_IBCR_OFFSET             0x0050  /* IB Bus Control Register */
#define RA8P_I3C_IBRANK_OFFSET           0x0054  /* IB Bus Rank Register */
#define RA8P_I3C_IBCTL_OFFSET            0x0058  /* IB Bus Control Register */
#define RA8P_I3C_IBCMD_OFFSET            0x005C  /* IB Bus Command Register */
#define RA8P_I3C_IBDATCTL_OFFSET         0x0060  /* IB Bus Data Control Register */
#define RA8P_I3C_IBDATST_OFFSET          0x0064  /* IB Bus Data Status Register */

/* CON - Control Register */
#define RA8P_I3C_CON_IBIEN               (1 << 0)   /* IB Interrupt Enable */
#define RA8P_I3C_CON_IBIDIS              (1 << 1)   /* IB Interrupt Disable */
#define RA8P_I3C_CON_IBIACK              (1 << 2)   /* IB Interrupt Acknowledge */
#define RA8P_I3C_CON_IBIWAKE             (1 << 3)   /* IB Interrupt Wake-up */
#define RA8P_I3C_CON_MSTEN               (1 << 4)   /* Master Enable */
#define RA8P_I3C_CON_MSTDIS              (1 << 5)   /* Master Disable */
#define RA8P_I3C_CON_SLVEN               (1 << 6)   /* Slave Enable */
#define RA8P_I3C_CON_SLVDIS              (1 << 7)   /* Slave Disable */
#define RA8P_I3C_CON_HJEN                (1 << 8)   /* Hot Join Enable */
#define RA8P_I3C_CON_HJDIS               (1 << 9)   /* Hot Join Disable */
#define RA8P_I3C_CON_HJACK               (1 << 10)  /* Hot Join Acknowledge */
#define RA8P_I3C_CON_MCS                 (1 << 11)  /* Master Clock Stretch */
#define RA8P_I3C_CON_SCS                 (1 << 12)  /* Slave Clock Stretch */

/* IBCC - IB Bus Control Register */
#define RA8P_I3C_IBCC_IBCC_MASK          (0x0F << 0) /* IB Clock Control Mask */
#define RA8P_I3C_IBCC_IBCC_SHIFT         0

/* MCON - Master Control Register */
#define RA8P_I3C_MCON_HOTJOIN            (1 << 0)   /* Hot Join Mode */
#define RA8P_I3C_MCON_IBIDLE             (1 << 1)   /* IB Idle */
#define RA8P_I3C_MCON_IBSTART            (1 << 2)   /* IB Start */
#define RA8P_I3C_MCON_IBSTOP             (1 << 3)   /* IB Stop */
#define RA8P_I3C_MCON_IBRESTART          (1 << 4)   /* IB Restart */
#define RA8P_I3C_MCON_IBREAD             (1 << 5)   /* IB Read */
#define RA8P_I3C_MCON_IBWRITE            (1 << 6)   /* IB Write */
#define RA8P_I3C_MCON_IBACK              (1 << 7)   /* IB Acknowledge */

/* STATUS - Status Register */
#define RA8P_I3C_STATUS_IBIBUSY          (1 << 0)   /* IB Bus Busy */
#define RA8P_I3C_STATUS_IBIWAIT          (1 << 1)   /* IB Interrupt Wait */
#define RA8P_I3C_STATUS_IBIWAKE          (1 << 2)   /* IB Interrupt Wake-up */
#define RA8P_I3C_STATUS_IBIRCV           (1 << 3)   /* IB Interrupt Received */
#define RA8P_I3C_STATUS_IBITX            (1 << 4)   /* IB Interrupt Transmit */
#define RA8P_I3C_STATUS_IBIRX            (1 << 5)   /* IB Interrupt Receive */

/* I3C Base Address */
#define RA8P_I3C0_BASE                   0x4035F000
#define RA8P_I3C_SIZE                    0x400

/* I3C Interrupt Numbers */
#define RA8P_IRQ_I3C0                    95

/* I3C Device Address Limits */
#define RA8P_I3C_MIN_DYNAMIC_ADDR        0x08
#define RA8P_I3C_MAX_DYNAMIC_ADDR        0x77
#define RA8P_I3C_STATIC_ADDR_MASK        0x7F

/* I3C Bus Speeds (kHz) */
#define RA8P_I3C_IBI_SPEED               100        /* IBI Speed: 100 kHz */
#define RA8P_I3C_HOTJOIN_SPEED           100        /* Hot Join Speed: 100 kHz */
#define RA8P_I3C_STD_SPEED               100        /* Standard Speed: 100 kHz */
#define RA8P_I3C_FM_SPEED               400        /* Fast Mode Speed: 400 kHz */
#define RA8P_I3C_FMP_SPEED              1000       /* Fast Mode Plus: 1 MHz */
#define RA8P_I3C_HS_SPEED               8000       /* High Speed: 8 MHz */
#define RA8P_I3C_UHS_SPEED              12500      /* Ultra High Speed: 12.5 MHz */

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* I3C device address structure */
struct ra8p_i3c_addr_s
{
  uint8_t addr;                       /* 7-bit I3C address */
  bool dynamic;                       /* True for dynamic address, false for static */
  uint8_t device_role;               /* Device role (master/slave) */
  bool ibi_enabled;                  /* IBI (In-Band Interrupt) enabled */
  bool hotjoin_enabled;              /* Hot join enabled */
};

/* I3C timing configuration structure */
struct ra8p_i3c_timing_s
{
  uint32_t standard_speed;           /* Standard speed in kHz */
  uint32_t fast_mode_speed;          /* Fast mode speed in kHz */
  uint32_t fast_mode_plus_speed;     /* Fast mode plus speed in kHz */
  uint32_t high_speed;               /* High speed in kHz */
  uint32_t t_hdsda;                  /* Data hold time after SDA assertion */
  uint32_t t_susta;                  /* Start condition hold time */
  uint32_t t_low;                    /* Clock low period */
  uint32_t t_high;                   /* Clock high period */
};

/* I3C transfer structure */
struct ra8p_i3c_transfer_s
{
  uint8_t addr;                       /* Target device address */
  uint8_t *buffer;                   /* Data buffer */
  uint32_t length;                   /* Data length */
  bool read;                         /* Read (true) or Write (false) */
  bool use_hs_mode;                  /* Use high-speed mode */
  bool with_stop;                    /* Send STOP after transfer */
  uint32_t timeout_ms;               /* Timeout in milliseconds */
};

/* I3C device configuration structure */
struct ra8p_i3c_config_s
{
  uint8_t channel;                   /* I3C channel (0-1) */
  struct ra8p_i3c_timing_s timing;   /* Timing configuration */
  uint32_t max_frequency;            /* Maximum supported frequency */
  bool hbi_enabled;                  /* HBI (Hardware Bandwidth Profile) enabled */
  bool dib_enabled;                  /* DIB (Dynamic Information Block) enabled */
  bool controller_role;              /* Master=true, Slave=false */
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_i3c_initialize
 *
 * Description:
 *   Initialize the I3C controller based on Zephyr i3c_renesas_ra.c implementation.
 *
 * Input Parameters:
 *   config - Pointer to I3C configuration structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_i3c_initialize(const struct ra8p_i3c_config_s *config);

/****************************************************************************
 * Name: ra8p_i3c_set_timing
 *
 * Description:
 *   Set I3C bus timing parameters based on Zephyr i3c_renesas_ra.c implementation.
 *
 * Input Parameters:
 *   channel - I3C channel (0-1)
 *   timing - Pointer to timing configuration structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_i3c_set_timing(uint8_t channel, const struct ra8p_i3c_timing_s *timing);

/****************************************************************************
 * Name: ra8p_i3c_set_dynamic_addr
 *
 * Description:
 *   Set I3C dynamic address based on Zephyr i3c_renesas_ra.c implementation.
 *
 * Input Parameters:
 *   channel - I3C channel (0-1)
 *   old_addr - Current address (0 for assignment)
 *   new_addr - New dynamic address
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_i3c_set_dynamic_addr(uint8_t channel, uint8_t old_addr, uint8_t new_addr);

/****************************************************************************
 * Name: ra8p_i3c_master_transfer
 *
 * Description:
 *   Perform an I3C master transfer based on Zephyr i3c_renesas_ra.c implementation.
 *
 * Input Parameters:
 *   channel - I3C channel (0-1)
 *   transfer - Pointer to transfer structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_i3c_master_transfer(uint8_t channel, struct ra8p_i3c_transfer_s *transfer);

/****************************************************************************
 * Name: ra8p_i3c_slave_transfer
 *
 * Description:
 *   Perform an I3C slave transfer based on Zephyr i3c_renesas_ra.c implementation.
 *
 * Input Parameters:
 *   channel - I3C channel (0-1)
 *   transfer - Pointer to transfer structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_i3c_slave_transfer(uint8_t channel, struct ra8p_i3c_transfer_s *transfer);

/****************************************************************************
 * Name: ra8p_i3c_enable_ibi
 *
 * Description:
 *   Enable IBI (In-Band Interrupt) for a device based on Zephyr i3c_renesas_ra.c implementation.
 *
 * Input Parameters:
 *   channel - I3C channel (0-1)
 *   addr - Device address
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_i3c_enable_ibi(uint8_t channel, uint8_t addr, bool enable);

/****************************************************************************
 * Name: ra8p_i3c_enable_hotjoin
 *
 * Description:
 *   Enable hot-join for a device based on Zephyr i3c_renesas_ra.c implementation.
 *
 * Input Parameters:
 *   channel - I3C channel (0-1)
 *   addr - Device address
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_i3c_enable_hotjoin(uint8_t channel, uint8_t addr, bool enable);

/****************************************************************************
 * Name: ra8p_i3c_assign_dynamic_addr
 *
 * Description:
 *   Assign a dynamic address to an I3C device based on Zephyr i3c_renesas_ra.c implementation.
 *
 * Input Parameters:
 *   channel - I3C channel (0-1)
 *   static_addr - Static address to assign from
 *
 * Returned Value:
 *   Dynamic address assigned, or negated errno on failure
 *
 ****************************************************************************/

int ra8p_i3c_assign_dynamic_addr(uint8_t channel, uint8_t static_addr);

/****************************************************************************
 * Name: ra8p_i3c_get_device_list
 *
 * Description:
 *   Get list of devices on I3C bus based on Zephyr i3c_renesas_ra.c implementation.
 *
 * Input Parameters:
 *   channel - I3C channel (0-1)
 *   device_list - Array to store device addresses
 *   max_devices - Maximum number of devices to store
 *
 * Returned Value:
 *   Number of devices found, or negated errno on failure
 *
 ****************************************************************************/

int ra8p_i3c_get_device_list(uint8_t channel, uint8_t *device_list, int max_devices);

/****************************************************************************
 * Name: ra8p_i3c_bus_reset
 *
 * Description:
 *   Reset the I3C bus based on Zephyr i3c_renesas_ra.c implementation.
 *
 * Input Parameters:
 *   channel - I3C channel (0-1)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_i3c_bus_reset(uint8_t channel);

/****************************************************************************
 * Name: ra8p_i3c_bus_scan
 *
 * Description:
 *   Scan I3C bus for devices based on Zephyr i3c_renesas_ra.c implementation.
 *
 * Input Parameters:
 *   channel - I3C channel (0-1)
 *   device_list - Array to store discovered device addresses
 *   max_devices - Maximum number of devices to scan
 *
 * Returned Value:
 *   Number of devices found, or negated errno on failure
 *
 ****************************************************************************/

int ra8p_i3c_bus_scan(uint8_t channel, uint8_t *device_list, int max_devices);

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_I3C_H */