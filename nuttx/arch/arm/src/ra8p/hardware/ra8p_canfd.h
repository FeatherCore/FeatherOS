/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_canfd.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_CANFD_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_CANFD_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* CANFD Register Offsets - Based on RA8P Hardware Manual and Zephyr definitions */
#define RA8P_CANFD_CFDGCFG_OFFSET         0x0000  /* Global Configuration Register */
#define RA8P_CANFD_CFDGCTR_OFFSET         0x0004  /* Global Control Register */
#define RA8P_CANFD_CFDGSTS_OFFSET         0x0008  /* Global Status Register */
#define RA8P_CANFD_CFDGREF_OFFSET         0x000C  /* Global Reference Register */
#define RA8P_CANFD_CFDGERR_OFFSET         0x0010  /* Global Error Register */
#define RA8P_CANFD_CFDGMSTS_OFFSET        0x0014  /* Global Message Status Register */
#define RA8P_CANFD_CFDGAFLCFG0_OFFSET     0x0024  /* Global Acceptance Filter Config 0 */
#define RA8P_CANFD_CFDGAFLCFG1_OFFSET     0x0028  /* Global Acceptance Filter Config 1 */
#define RA8P_CANFD_CFDF0CFG_OFFSET        0x0040  /* Common FIFO Config 0 */
#define RA8P_CANFD_CFDF1CFG_OFFSET        0x0044  /* Common FIFO Config 1 */
#define RA8P_CANFD_CFDF2CFG_OFFSET        0x0048  /* Common FIFO Config 2 */
#define RA8P_CANFD_CFDF3_OFFSET           0x004C  /* Common FIFO Config 3 */
#define RA8P_CANFD_CFDRFCC0_OFFSET        0x0080  /* RX FIFO Config Control 0 */
#define RA8P_CANFD_CFDRFCC1_OFFSET        0x0084  /* RX FIFO Config Control 1 */
#define RA8P_CANFD_CFD0CTR_OFFSET         0x0100  /* Channel 0 Control Register */
#define RA8P_CANFD_CFD0STS_OFFSET         0x0104  /* Channel 0 Status Register */
#define RA8P_CANFD_CFD0ER_OFFSET          0x0108  /* Channel 0 Error Register */
#define RA8P_CANFD_CFD0REC_OFFSET         0x010C  /* Channel 0 Receive Error Counter */
#define RA8P_CANFD_CFD0TEC_OFFSET         0x0110  /* Channel 0 Transmit Error Counter */
#define RA8P_CANFD_CFD0COMSTS_OFFSET      0x0114  /* Channel 0 Communication Status */
#define RA8P_CANFD_CFD0CTRMK_OFFSET       0x0118  /* Channel 0 Command Mask */
#define RA8P_CANFD_CFD0CMDTR_OFFSET       0x011C  /* Channel 0 Command Trigger */
#define RA8P_CANFD_CFD0GBCTRL_OFFSET      0x0120  /* Channel 0 Global Buffer Control */
#define RA8P_CANFD_CFD0GBSTS_OFFSET       0x0124  /* Channel 0 Global Buffer Status */
#define RA8P_CANFD_CFD0GIC_OFFSET         0x0128  /* Channel 0 Global Interrupt Control */
#define RA8P_CANFD_CFD0INTSTS_OFFSET      0x012C  /* Channel 0 Interrupt Status */
#define RA8P_CANFD_CFD0NBTR_OFFSET        0x0130  /* Channel 0 Nominal Bit Timing */
#define RA8P_CANFD_CFD0DBTR_OFFSET        0x0134  /* Channel 0 Data Bit Timing */
#define RA8P_CANFD_CFD0TSC_OFFSET         0x0138  /* Channel 0 Timestamp Counter */
#define RA8P_CANFD_CFD0RMNCFG_OFFSET      0x013C  /* Channel 0 Reception Multiple Node Config */
#define RA8P_CANFD_CFD1CTR_OFFSET         0x0200  /* Channel 1 Control Register */
#define RA8P_CANFD_CFD0TF_BASE            0x4000 /* TX Message Buffer Base */
#define RA8P_CANFD_CFD0RM_BASE            0x5000 /* RX Message Buffer Base */
#define RA8P_CANFD_CFD0THL_BASE           0x7000 /* TX History List Base */
#define RA8P_CANFD_CFDGAFL_BASE           0x8000 /* Global Acceptance Filter List Base */
#define RA8P_CANFD_CFD0MBSIZE_OFFSET      0x0A000 /* Channel 0 Message Buffer Size */
#define RA8P_CANFD_CFD0GFERR_OFFSET       0x0A004 /* Channel 0 Global Frame Error */
#define RA8P_CANFD_CFDRMNB_OFFSET         0x0A008 /* RX Message Buffer Number */
#define RA8P_CANFD_CFDRFSTS0_OFFSET       0x0A00C /* RX FIFO Status 0 */
#define RA8P_CANFD_CFDRFSTS1_OFFSET       0x0A010 /* RX FIFO Status 1 */
#define RA8P_CANFD_CFDRFC0_OFFSET         0x0A014 /* RX FIFO Read Counter 0 */
#define RA8P_CANFD_CFDRFC1_OFFSET         0x0A018 /* RX FIFO Read Counter 1 */

/* CFDGCFG - Global Configuration Register */
#define RA8P_CANFD_CFDGCFG_TPRI           (1 << 0)   /* Transmission Priority */
#define RA8P_CANFD_CFDGCFG_DCE            (1 << 1)   /* DLC Check Enable */
#define RA8P_CANFD_CFDGCFG_DCS            (1 << 2)   /* DLC Check Select */
#define RA8P_CANFD_CFDGCFG_MME            (1 << 3)   /* Message Lost Enable */
#define RA8P_CANFD_CFDGCFG_IME            (1 << 4)   /* Illegal Message Enable */
#define RA8P_CANFD_CFDGCFG_THLE           (1 << 5)   /* TX History List Enable */
#define RA8P_CANFD_CFDGCFG_CMPOE          (1 << 6)   /* CANFD Message Payload Overflow Enable */

/* CFDGCTR - Global Control Register */
#define RA8P_CANFD_CFDGCTR_GRST           (1 << 0)   /* Global Reset */
#define RA8P_CANFD_CFDGCTR_GSLPR          (1 << 1)   /* Global Sleep Request */
#define RA8P_CANFD_CFDGCTR_GWU            (1 << 2)   /* Global Wake-up */
#define RA8P_CANFD_CFDGCTR_GMDC_MASK      (0x03 << 3) /* Global Mode Control */
#define RA8P_CANFD_CFDGCTR_GMDC_SHIFT     3
#define RA8P_CANFD_CFDGCTR_DEIE           (1 << 8)   /* DLC Error Interrupt Enable */
#define RA8P_CANFD_CFDGCTR_MEIE           (1 << 9)   /* Message Lost Error IE */
#define RA8P_CANFD_CFDGCTR_THLEIE         (1 << 10)  /* TX History List Entry Lost IE */
#define RA8P_CANFD_CFDGCTR_CMPOFIE        (1 << 11)  /* CANFD Message Payload Overflow Flag IE */

/* CFDGSTS - Global Status Register */
#define RA8P_CANFD_CFDGSTS_GRSTSTS        (1 << 0)   /* Global Reset Status */
#define RA8P_CANFD_CFDGSTS_GSLPSTS        (1 << 1)   /* Global Sleep Status */
#define RA8P_CANFD_CFDGSTS_GRWUSTS        (1 << 2)   /* Global Wake-up Status */

/* CFD0CTR - Channel 0 Control Register */
#define RA8P_CANFD_CFD0CTR_CHMNT          (1 << 0)   /* Channel Halt Mode Entry Request */
#define RA8P_CANFD_CFD0CTR_CHSLP          (1 << 1)   /* Channel Sleep Request */
#define RA8P_CANFD_CFD0CTR_CHWU           (1 << 2)   /* Channel Wake-up */
#define RA8P_CANFD_CFD0CTR_CHMDC_MASK     (0x03 << 3) /* Channel Mode Control */
#define RA8P_CANFD_CFD0CTR_CHMDC_SHIFT    3
#define RA8P_CANFD_CFD0CTR_CHCTR          (1 << 7)   /* Channel Control Enable */
#define RA8P_CANFD_CFD0CTR_FDEN           (1 << 8)   /* CANFD Enable */
#define RA8P_CANFD_CFD0CTR_BOM_MASK       (0x03 << 9) /* Bus-Off Mode */
#define RA8P_CANFD_CFD0CTR_BOM_SHIFT      9
#define RA8P_CANFD_CFD0CTR_ERM            (1 << 11)  /* Error Mode */
#define RA8P_CANFD_CFD0CTR_MLM            (1 << 12)  /* Bit Rate Mode */
#define RA8P_CANFD_CFD0CTR_ESIM           (1 << 13)  /* Enhanced Silent Mode */
#define RA8P_CANFD_CFD0CTR_ISOCC          (1 << 14)  /* ISO CANFD CC Mode */
#define RA8P_CANFD_CFD0CTR_EDPM           (1 << 15)  /* External Loopback Mode */

/* CFD0STS - Channel 0 Status Register */
#define RA8P_CANFD_CFD0STS_ERR            (1 << 0)   /* Error Occurred */
#define RA8P_CANFD_CFD0STS_CHALT          (1 << 1)   /* Channel Halt Mode Status */
#define RA8P_CANFD_CFD0STS_CHSLP          (1 << 2)   /* Channel Sleep Status */
#define RA8P_CANFD_CFD0STS_CHWUP          (1 << 3)   /* Channel Wake-up Status */
#define RA8P_CANFD_CFD0STS_COMSTS_MASK    (0x07 << 4) /* Communication Status */
#define RA8P_CANFD_CFD0STS_COMSTS_SHIFT   4

/* CFD0NBTR - Channel 0 Nominal Bit Timing Register */
#define RA8P_CANFD_CFD0NBTR_NSJW_MASK     (0x0F << 28) /* Nominal SJW Mask */
#define RA8P_CANFD_CFD0NBTR_NSJW_SHIFT    28
#define RA8P_CANFD_CFD0NBTR_NTSEG2_MASK   (0x0F << 24) /* Nominal TSEG2 Mask */
#define RA8P_CANFD_CFD0NBTR_NTSEG2_SHIFT  24
#define RA8P_CANFD_CFD0NBTR_NTSEG1_MASK   (0x0F << 20) /* Nominal TSEG1 Mask */
#define RA8P_CANFD_CFD0NBTR_NTSEG1_SHIFT  20
#define RA8P_CANFD_CFD0NBTR_NBRP_MASK     (0x0F << 0)  /* Nominal BRP Mask */
#define RA8P_CANFD_CFD0NBTR_NBRP_SHIFT    0

/* CFD0DBTR - Channel 0 Data Bit Timing Register */
#define RA8P_CANFD_CFD0DBTR_DSJW_MASK     (0x0F << 28) /* Data SJW Mask */
#define RA8P_CANFD_CFD0DBTR_DSJW_SHIFT    28
#define RA8P_CANFD_CFD0DBTR_DTSEG2_MASK   (0x0F << 24) /* Data TSEG2 Mask */
#define RA8P_CANFD_CFD0DBTR_DTSEG2_SHIFT  24
#define RA8P_CANFD_CFD0DBTR_DTSEG1_MASK   (0x0F << 20) /* Data TSEG1 Mask */
#define RA8P_CANFD_CFD0DBTR_DTSEG1_SHIFT  20
#define RA8P_CANFD_CFD0DBTR_DBRP_MASK     (0x0F << 0)  /* Data BRP Mask */
#define RA8P_CANFD_CFD0DBTR_DBRP_SHIFT    0

/* Message Buffer Register Offsets */
#define RA8P_CANFD_MB_ID_OFFSET           0x00
#define RA8P_CANFD_MB_DLC_OFFSET          0x04
#define RA8P_CANFD_MB_DATA_OFFSET         0x08
#define RA8P_CANFD_MB_TS_OFFSET           0x18

/* CANFD Message ID bits */
#define RA8P_CANFD_IDE                    (1 << 29)  /* Extended ID */
#define RA8P_CANFD_RTR                    (1 << 28)  /* Remote Transmission Request */
#define RA8P_CANFD_EDL                    (1 << 21)  /* Extended Data Length (CANFD) */
#define RA8P_CANFD_BRS                    (1 << 20)  /* Bit Rate Switch */
#define RA8P_CANFD_ESI                    (1 << 19)  /* Error State Indicator */

/* CANFD DLC to Data Length Mapping */
#define RA8P_CANFD_DLC_0                  0
#define RA8P_CANFD_DLC_1                  1
#define RA8P_CANFD_DLC_2                  2
#define RA8P_CANFD_DLC_3                  3
#define RA8P_CANFD_DLC_4                  4
#define RA8P_CANFD_DLC_5                  5
#define RA8P_CANFD_DLC_6                  6
#define RA8P_CANFD_DLC_7                  7
#define RA8P_CANFD_DLC_8                  8
#define RA8P_CANFD_DLC_12                 9
#define RA8P_CANFD_DLC_16                 10
#define RA8P_CANFD_DLC_20                 11
#define RA8P_CANFD_DLC_24                 12
#define RA8P_CANFD_DLC_32                 13
#define RA8P_CANFD_DLC_48                 14
#define RA8P_CANFD_DLC_64                 15

/* Maximum number of acceptance filters */
#define RA8P_CANFD_MAX_FILTERS            160

/* Maximum number of TX/RX buffers */
#define RA8P_CANFD_TX_BUFFERS             32
#define RA8P_CANFD_RX_BUFFERS             64

/* Base addresses */
#define RA8P_CANFD0_BASE                  0x40380000
#define RA8P_CANFD1_BASE                  0x40390000

/* Message RAM base address and size */
#define RA8P_CANFD_MSGRAM_BASE            0x48000000
#define RA8P_CANFD_MSGRAM_SIZE            0x4000

/* Interrupt numbers */
#define RA8P_IRQ_CANFD0                   94
#define RA8P_IRQ_CANFD1                   95

/* CANFD timeout in milliseconds */
#define RA8P_CANFD_TIMEOUT_MS             1000

/* Maximum payload size for CANFD */
#define RA8P_CANFD_MAX_PAYLOAD            64

/* CANFD error interrupt flags */
#define RA8P_CANFD_ERROR_OVERFLOW         (1 << 0)
#define RA8P_CANFD_ERROR_UNDERFLOW        (1 << 1)
#define RA8P_CANFD_ERROR_MESSAGE_LOST     (1 << 2)
#define RA8P_CANFD_ERROR_OVERLOAD         (1 << 3)

/* CANFD communication statuses */
#define RA8P_CANFD_COMSTS_ERR_ACTIVE      0x00
#define RA8P_CANFD_COMSTS_ERR_PASSIVE     0x01
#define RA8P_CANFD_COMSTS_BUS_OFF         0x02
#define RA8P_CANFD_COMSTS_SLEEP           0x03
#define RA8P_CANFD_COMSTS_HALT            0x04
#define RA8P_CANFD_COMSTS_RESET           0x05

/* CANFD bus-off modes */
#define RA8P_CANFD_BOM_ISO                0x00  /* ISO CANFD CC */
#define RA8P_CANFD_BOM_ISO_AUTO           0x01  /* ISO CANFD CC with auto recovery */
#define RA8P_CANFD_BOM_NON_ISO            0x02  /* Non-ISO CANFD CC */
#define RA8P_CANFD_BOM_NON_ISO_AUTO       0x03  /* Non-ISO CANFD CC with auto recovery */

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* CANFD timing configuration structure */
struct ra8p_canfd_timing_s
{
  uint32_t bitrate;                /* Nominal bitrate */
  uint32_t data_bitrate;           /* Data bitrate for CANFD (0 for classic CAN) */
  uint32_t sample_point;           /* Sample point in 0.1% (e.g., 875 = 87.5%) */
  uint8_t sjw;                    /* Synchronization Jump Width */
  uint8_t tseg1;                  /* Time Segment 1 */
  uint8_t tseg2;                  /* Time Segment 2 */
  uint8_t brp;                    /* Baud Rate Prescaler */
  uint8_t prescaler;              /* Clock prescaler */
};

/* CANFD frame structure */
struct ra8p_canfd_frame_s
{
  uint32_t id;                    /* CAN ID (11-bit or 29-bit) */
  uint8_t dlc;                    /* Data Length Code (0-15) */
  uint8_t data[RA8P_CANFD_MAX_PAYLOAD]; /* Data payload (up to 64 bytes for CANFD) */
  bool extended;                  /* Extended ID flag */
  bool rtr;                       /* Remote Transmission Request flag */
  bool fd;                        /* CANFD format flag */
  bool brs;                       /* Bit Rate Switch flag */
  bool esi;                       /* Error State Indicator flag */
  bool timestamp;                 /* Include timestamp flag */
  uint32_t ts_value;              /* Timestamp value if enabled */
};

/* CANFD filter structure */
struct ra8p_canfd_filter_s
{
  uint32_t id;                    /* Filter ID */
  uint32_t mask;                  /* Filter mask */
  bool extended;                  /* Extended ID matching */
  uint8_t channel;                /* CANFD channel (0 or 1) */
  bool enabled;                   /* Filter enabled */
};

/* CANFD error counter structure */
struct ra8p_canfd_error_count_s
{
  uint8_t tx_errors;              /* Transmit error counter */
  uint8_t rx_errors;              /* Receive error counter */
  uint8_t tx_warning_limit;       /* TX warning limit */
  uint8_t rx_warning_limit;       /* RX warning limit */
};

/* CANFD device configuration */
struct ra8p_canfd_config_s
{
  uint8_t channel;                /* CANFD channel (0 or 1) */
  uint32_t max_frequency;         /* Maximum clock frequency */
  uint32_t bitrate;               /* Nominal bitrate */
  uint32_t data_bitrate;          /* Data bitrate for CANFD */
  bool canfd_mode;                /* Enable CANFD mode */
  uint8_t num_filters;            /* Number of filters */
  struct ra8p_canfd_filter_s filters[RA8P_CANFD_MAX_FILTERS]; /* Acceptance filters */
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_canfd_initialize
 *
 * Description:
 *   Initialize the CANFD controller based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   bitrate - CAN bitrate in Hz
 *   data_bitrate - CANFD data bitrate in Hz (0 for classic CAN mode)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_initialize(uint8_t channel, uint32_t bitrate, uint32_t data_bitrate);

/****************************************************************************
 * Name: ra8p_canfd_start
 *
 * Description:
 *   Start CANFD channel operation based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_start(uint8_t channel);

/****************************************************************************
 * Name: ra8p_canfd_stop
 *
 * Description:
 *   Stop CANFD channel operation based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_stop(uint8_t channel);

/****************************************************************************
 * Name: ra8p_canfd_send
 *
 * Description:
 *   Send a CAN/CANFD frame based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   frame - Pointer to CANFD frame structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_send(uint8_t channel, const struct ra8p_canfd_frame_s *frame);

/****************************************************************************
 * Name: ra8p_canfd_receive
 *
 * Description:
 *   Receive a CAN/CANFD frame based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   frame - Pointer to CANFD frame structure to fill
 *   timeout - Timeout in ms (0 for non-blocking)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_receive(uint8_t channel, struct ra8p_canfd_frame_s *frame, 
                      unsigned int timeout);

/****************************************************************************
 * Name: ra8p_canfd_add_filter
 *
 * Description:
 *   Add a CAN acceptance filter based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   filter - Pointer to filter configuration
 *
 * Returned Value:
 *   Filter index on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_add_filter(uint8_t channel, const struct ra8p_canfd_filter_s *filter);

/****************************************************************************
 * Name: ra8p_canfd_remove_filter
 *
 * Description:
 *   Remove a CAN acceptance filter based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   index - Filter index to remove
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_remove_filter(uint8_t channel, int index);

/****************************************************************************
 * Name: ra8p_canfd_set_bitrate
 *
 * Description:
 *   Set CANFD bitrate based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   bitrate - Nominal bitrate in Hz
 *   data_bitrate - Data bitrate in Hz (0 for classic CAN)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_set_bitrate(uint8_t channel, uint32_t bitrate, uint32_t data_bitrate);

/****************************************************************************
 * Name: ra8p_canfd_set_mode
 *
 * Description:
 *   Set CANFD operating mode (classic CAN vs CANFD) based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   fdmode - true for CANFD mode, false for classic CAN mode
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_set_mode(uint8_t channel, bool fdmode);

/****************************************************************************
 * Name: ra8p_canfd_is_enabled
 *
 * Description:
 *   Check if CANFD channel is enabled based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   true if enabled, false otherwise
 *
 ****************************************************************************/

bool ra8p_canfd_is_enabled(uint8_t channel);

/****************************************************************************
 * Name: ra8p_canfd_is_error
 *
 * Description:
 *   Check if CANFD has error flags set based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   true if error occurred, false otherwise
 *
 ****************************************************************************/

bool ra8p_canfd_is_error(uint8_t channel);

/****************************************************************************
 * Name: ra8p_canfd_clear_error
 *
 * Description:
 *   Clear CANFD error flags based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_clear_error(uint8_t channel);

/****************************************************************************
 * Name: ra8p_canfd_get_status
 *
 * Description:
 *   Get CANFD status flags based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   Status flags
 *
 ****************************************************************************/

uint32_t ra8p_canfd_get_status(uint8_t channel);

/****************************************************************************
 * Name: ra8p_canfd_get_error_count
 *
 * Description:
 *   Get CANFD error count (TX/RX) based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   count - Pointer to error count structure to fill
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_get_error_count(uint8_t channel, struct ra8p_canfd_error_count_s *count);

/****************************************************************************
 * Name: ra8p_canfd_reset_counters
 *
 * Description:
 *   Reset CANFD error counters based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_reset_counters(uint8_t channel);

/****************************************************************************
 * Name: ra8p_canfd_is_canfd_mode
 *
 * Description:
 *   Check if CANFD is operating in CANFD mode based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   true if in CANFD mode, false if in classic CAN mode
 *
 ****************************************************************************/

bool ra8p_canfd_is_canfd_mode(uint8_t channel);

/****************************************************************************
 * Name: ra8p_canfd_set_autoretry
 *
 * Description:
 *   Enable/disable CANFD auto-retry mechanism based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   enable - true to enable auto-retry, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_set_autoretry(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_canfd_get_bitrate
 *
 * Description:
 *   Get current CANFD bitrate based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   nominal - Pointer to store nominal bitrate
 *   data - Pointer to store data bitrate
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_get_bitrate(uint8_t channel, uint32_t *nominal, uint32_t *data);

/****************************************************************************
 * Name: ra8p_canfd_get_bus_state
 *
 * Description:
 *   Get CANFD bus state based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   Bus state (0=error-active, 1=error-passive, 2=bus-off, 3=sleep, 4=halt, 5=reset)
 *
 ****************************************************************************/

uint8_t ra8p_canfd_get_bus_state(uint8_t channel);

/****************************************************************************
 * Name: ra8p_canfd_clear_status
 *
 * Description:
 *   Clear CANFD status flags based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   flags - Flags to clear
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_clear_status(uint8_t channel, uint32_t flags);

/****************************************************************************
 * Name: ra8p_canfd_enable_interrupts
 *
 * Description:
 *   Enable CANFD interrupts based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_enable_interrupts(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_canfd_set_bus_off_recovery
 *
 * Description:
 *   Set CANFD bus-off recovery mode based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   recovery - true for automatic recovery, false for manual
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_set_bus_off_recovery(uint8_t channel, bool recovery);

/****************************************************************************
 * Name: ra8p_canfd_set_error_detection
 *
 * Description:
 *   Set CANFD error detection mode based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   enable - true to enable error detection, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_set_error_detection(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_canfd_get_error_flags
 *
 * Description:
 *   Get CANFD error flags based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   Error flags
 *
 ****************************************************************************/

uint32_t ra8p_canfd_get_error_flags(uint8_t channel);

/****************************************************************************
 * Name: ra8p_canfd_clear_error_flags
 *
 * Description:
 *   Clear CANFD error flags based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   flags - Error flags to clear
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_clear_error_flags(uint8_t channel, uint32_t flags);

/****************************************************************************
 * Name: ra8p_canfd_is_bus_off
 *
 * Description:
 *   Check if CANFD is in bus-off state based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   true if in bus-off state, false otherwise
 *
 ****************************************************************************/

bool ra8p_canfd_is_bus_off(uint8_t channel);

/****************************************************************************
 * Name: ra8p_canfd_recover_from_bus_off
 *
 * Description:
 *   Recover CANFD from bus-off state based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_recover_from_bus_off(uint8_t channel);

/****************************************************************************
 * Name: ra8p_canfd_set_loopback
 *
 * Description:
 *   Set CANFD loopback mode based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   enable - true for loopback mode, false for normal mode
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_set_loopback(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_canfd_enable_silent_mode
 *
 * Description:
 *   Enable/disable CANFD silent mode based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   enable - true for silent mode, false for normal mode
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_enable_silent_mode(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_canfd_is_silent_mode
 *
 * Description:
 *   Check if CANFD is in silent mode based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   true if in silent mode, false otherwise
 *
 ****************************************************************************/

bool ra8p_canfd_is_silent_mode(uint8_t channel);

/****************************************************************************
 * Name: ra8p_canfd_set_dlc_check
 *
 * Description:
 *   Set CANFD DLC check mode based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   enable - true to enable DLC check, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_set_dlc_check(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_canfd_is_dlc_check_enabled
 *
 * Description:
 *   Check if CANFD DLC check is enabled based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   true if DLC check enabled, false otherwise
 *
 ****************************************************************************/

bool ra8p_canfd_is_dlc_check_enabled(uint8_t channel);

/****************************************************************************
 * Name: ra8p_canfd_get_message_count
 *
 * Description:
 *   Get the number of received messages in FIFO based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   Number of received messages
 *
 ****************************************************************************/

uint8_t ra8p_canfd_get_message_count(uint8_t channel);

/****************************************************************************
 * Name: ra8p_canfd_get_tx_error_count
 *
 * Description:
 *   Get transmit error count based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   TX error count
 *
 ****************************************************************************/

uint8_t ra8p_canfd_get_tx_error_count(uint8_t channel);

/****************************************************************************
 * Name: ra8p_canfd_get_rx_error_count
 *
 * Description:
 *   Get receive error count based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   RX error count
 *
 ****************************************************************************/

uint8_t ra8p_canfd_get_rx_error_count(uint8_t channel);

/****************************************************************************
 * Name: ra8p_canfd_set_error_warning_limits
 *
 * Description:
 *   Set CANFD error warning limits based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   tx_limit - TX warning limit (0-255)
 *   rx_limit - RX warning limit (0-255)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_set_error_warning_limits(uint8_t channel, uint8_t tx_limit, uint8_t rx_limit);

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_CANFD_H */