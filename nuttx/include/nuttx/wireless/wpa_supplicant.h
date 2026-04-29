/****************************************************************************
 * include/nuttx/wireless/wpa_supplicant.h
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

#ifndef __INCLUDE_NUTTX_WIRELESS_WPA_SUPPLICANT_H
#define __INCLUDE_NUTTX_WIRELESS_WPA_SUPPLICANT_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

#include <stdint.h>
#include <stdbool.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define WPA_SUPPLICANT_SSID_MAX_LEN     32
#define WPA_SUPPLICANT_PASSPHRASE_MAX_LEN 64
#define WPA_SUPPLICANT_BSSID_LEN        6

/* WPA supplicant event types */

#define WPA_EVENT_CONNECTED            1
#define WPA_EVENT_DISCONNECTED         2
#define WPA_EVENT_SCAN_RESULTS         3
#define WPA_EVENT_ASSOC_REJECT         4
#define WPA_EVENT_AUTH_REJECT          5

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* WPA supplicant configuration */

struct wpa_supplicant_config_s
{
  char ssid[WPA_SUPPLICANT_SSID_MAX_LEN];
  size_t ssid_len;
  char passphrase[WPA_SUPPLICANT_PASSPHRASE_MAX_LEN];
  size_t passphrase_len;
  uint8_t bssid[WPA_SUPPLICANT_BSSID_LEN];
  bool bssid_set;
  uint32_t auth_alg;              /* Authentication algorithm */
  uint32_t key_mgmt;              /* Key management */
  uint32_t pair_wise;             /* Pairwise cipher */
  uint32_t group_wise;            /* Group cipher */
};

/* WPA supplicant event */

struct wpa_supplicant_event_s
{
  uint32_t event_type;
  uint8_t bssid[WPA_SUPPLICANT_BSSID_LEN];
  char ssid[WPA_SUPPLICANT_SSID_MAX_LEN];
  size_t ssid_len;
  int reason_code;
  bool locally_generated;
};

/* WPA supplicant event callback */

typedef CODE int (*wpa_event_callback_t)(FAR struct wpa_supplicant_event_s *event,
                                        FAR void *arg);

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

#ifdef __cplusplus
extern "C"
{
#endif

/* WPA supplicant initialization */

int wpa_supplicant_initialize(void);
int wpa_supplicant_uninitialize(void);

/* Network configuration */

int wpa_supplicant_add_network(void);
int wpa_supplicant_set_network(int network_id,
                              FAR struct wpa_supplicant_config_s *config);
int wpa_supplicant_enable_network(int network_id);
int wpa_supplicant_disable_network(int network_id);
int wpa_supplicant_remove_network(int network_id);

/* Connection management */

int wpa_supplicant_connect(int network_id);
int wpa_supplicant_disconnect(void);

/* Scanning */

int wpa_supplicant_scan(void);
int wpa_supplicant_get_scan_results(void);

/* Event handling */

int wpa_supplicant_register_event_callback(wpa_event_callback_t callback,
                                          FAR void *arg);
int wpa_supplicant_unregister_event_callback(wpa_event_callback_t callback);

/* Status */

int wpa_supplicant_get_status(char *buf, size_t buf_len);

#ifdef __cplusplus
}
#endif

#endif /* __INCLUDE_NUTTX_WIRELESS_WPA_SUPPLICANT_H */