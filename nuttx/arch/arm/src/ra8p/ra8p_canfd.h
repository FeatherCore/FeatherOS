/****************************************************************************
 * arch/arm/src/ra8p/ra8p_canfd.h
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

#ifndef __ARCH_ARM_SRC_RA8P_RA8P_CANFD_H
#define __ARCH_ARM_SRC_RA8P_RA8P_CANFD_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

#include <stdint.h>
#include <stdbool.h>
#include <time.h>

#include "hardware/ra8p_canfd.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Maximum number of CANFD filters */
#define RA8P_CANFD_MAX_FILTERS           160

/* CANFD message RAM base and size */
#define RA8P_CANFD_MSG_RAM_BASE          0x48000000
#define RA8P_CANFD_MSG_RAM_SIZE          0x4000

/* Default timeouts */
#define RA8P_CANFD_TIMEOUT_MS            1000

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* CANFD timing configuration structure */
struct ra8p_canfd_timing_s
{
  uint32_t bitrate;                /* Nominal bitrate */
  uint32_t data_bitrate;           /* Data bitrate for CANFD (0 for classic CAN) */
  uint8_t sample_point;            /* Sample point in 0.1% (e.g., 875 = 87.5%) */
  uint8_t sjw;                    /* Synchronization Jump Width */
  uint8_t tseg1;                  /* Time Segment 1 */
  uint8_t tseg2;                  /* Time Segment 2 */
  uint8_t prescaler;              /* Baud Rate Prescaler */
};

/* CANFD frame structure based on Nuttx can.h definitions */
struct ra8p_canfd_frame_s
{
  union
  {
    struct
    {
      uint32_t ch_id;             /* CAN ID (11-bit or 29-bit) */
      uint8_t ch_dlc;             /* Data Length Code (0-15) */
      uint8_t ch_rtr : 1;         /* Remote Transmission Request */
      uint8_t ch_ide : 1;         /* Extended ID flag */
      uint8_t ch_fd  : 1;         /* CANFD frame flag */
      uint8_t ch_brs : 1;         /* Bit Rate Switch */
      uint8_t ch_esi : 1;         /* Error State Indicator */
      uint8_t        : 3;         /* Reserved */
    } ch;
    uint64_t align1;
  } cm_hdr;                       /* Header portion */
  union
  {
    uint8_t cm_data[64];         /* CANFD data payload (max 64 bytes) */
    uint64_t align2[8];
  };
  uint8_t cm_nbyte;              /* Number of valid bytes */
};

/* CANFD filter structure */
struct ra8p_canfd_filter_s
{
  uint32_t cf_id;                /* Filter ID */
  uint32_t cf_mask;              /* Filter mask */
  bool cf_extid;                 /* Extended ID matching */
  uint8_t cf_channel;            /* CANFD channel (0 or 1) */
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
 *   frame - CANFD frame to send
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
 *   frame - Buffer to store received frame
 *   timeout - Timeout in milliseconds (0 for non-blocking)
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
 *   filter - Filter configuration
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

#endif /* __ARCH_ARM_SRC_RA8P_RA8P_CANFD_H */