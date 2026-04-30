/****************************************************************************
 * drivers/wireless/esp32/esp_cfg80211.h
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

/****************************************************************************
 * ESP32 cfg80211 Driver for NuttX
 *
 * This driver provides cfg80211 interface for ESP32 WiFi chips
 * communicating via SDIO or SPI. Based on Espressif's esp-hosted-ng
 * architecture.
 ****************************************************************************/

#ifndef __DRIVERS_WIRELESS_ESP32_ESP_CFG80211_H
#define __DRIVERS_WIRELESS_ESP32_ESP_CFG80211_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <nuttx/compiler.h>

#include <stdint.h>
#include <stdbool.h>

#include <nuttx/wireless/cfg80211.h>
#include <nuttx/wireless/nl80211.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* MAC address length */

#define ESP_MAC_ADDR_LEN                   6

/* Maximum SSID length */

#define ESP_MAX_SSID_LEN                   32

/* Maximum key length */

#define ESP_MAX_KEY_LEN                    32

/* Maximum sequence length */

#define ESP_MAX_SEQ_LEN                    10

/* Maximum IE length */

#define ESP_MAX_IE_LEN                     512

/* Maximum multicast address count */

#define ESP_MAX_MCAST_ADDR_COUNT           8

/* Maximum number of interfaces */

#define ESP_MAX_INTERFACE                  2

/* Interface types */

#define ESP_STA_IF                         0
#define ESP_AP_IF                          1

/* Network interface numbers */

#define ESP_STA_NW_IF                      0
#define ESP_AP_NW_IF                       1

/* Interface types for communication */

#define ESP_IF_TYPE_SDIO                   1
#define ESP_IF_TYPE_SPI                    2

/* Link states */

#define ESP_LINK_DOWN                      0
#define ESP_LINK_UP                        1

/* Maximum TX power in dBm */

#define ESP_MAX_TX_POWER_DBM               20
#define ESP_MIN_TX_POWER_DBM               8

/* Payload header size */

#define ESP_PAYLOAD_HEADER_SIZE            12

/* Maximum alignment padding */

#define ESP_MAX_ALIGN_PADDING              4

/* Command timeout in milliseconds */

#define ESP_CMD_TIMEOUT_MS                 5000

/* Number of command nodes */

#define ESP_NUM_OF_CMD_NODES               20

/* Size of command node buffer */

#define ESP_SIZE_OF_CMD_NODE               2048

/* ESP Capabilities flags */

#define ESP_WLAN_SDIO_SUPPORT              (1 << 0)
#define ESP_BT_UART_SUPPORT                (1 << 1)
#define ESP_BT_SDIO_SUPPORT                (1 << 2)
#define ESP_BLE_ONLY_SUPPORT               (1 << 3)
#define ESP_BR_EDR_ONLY_SUPPORT            (1 << 4)
#define ESP_WLAN_SPI_SUPPORT               (1 << 5)
#define ESP_BT_SPI_SUPPORT                 (1 << 6)
#define ESP_CHECKSUM_ENABLED               (1 << 7)

/****************************************************************************
 * Enumerations
 ****************************************************************************/

/* ESP interface type enumeration */

enum esp_interface_type
{
  ESP_STA_IF_TYPE,
  ESP_AP_IF_TYPE,
  ESP_HCI_IF_TYPE,
  ESP_INTERNAL_IF_TYPE,
  ESP_TEST_IF_TYPE,
  ESP_MAX_IF_TYPE
};

/* ESP packet type enumeration */

enum esp_packet_type
{
  ESP_PACKET_TYPE_DATA,
  ESP_PACKET_TYPE_COMMAND_REQUEST,
  ESP_PACKET_TYPE_COMMAND_RESPONSE,
  ESP_PACKET_TYPE_EVENT,
  ESP_PACKET_TYPE_EAPOL
};

/* ESP command codes - matching esp-hosted protocol */

enum esp_command_code
{
  ESP_CMD_INIT_INTERFACE = 1,
  ESP_CMD_SET_MAC = 2,
  ESP_CMD_GET_MAC = 3,
  ESP_CMD_SCAN_REQUEST = 4,
  ESP_CMD_STA_CONNECT = 5,
  ESP_CMD_DISCONNECT = 6,
  ESP_CMD_DEINIT_INTERFACE = 7,
  ESP_CMD_ADD_KEY = 8,
  ESP_CMD_DEL_KEY = 9,
  ESP_CMD_SET_DEFAULT_KEY = 10,
  ESP_CMD_STA_AUTH = 11,
  ESP_CMD_STA_ASSOC = 12,
  ESP_CMD_SET_IP_ADDR = 13,
  ESP_CMD_SET_MCAST_MAC_ADDR = 14,
  ESP_CMD_GET_TXPOWER = 15,
  ESP_CMD_SET_TXPOWER = 16,
  ESP_CMD_GET_REG_DOMAIN = 17,
  ESP_CMD_SET_REG_DOMAIN = 18,
  ESP_CMD_SET_MODE = 22,
  ESP_CMD_SET_IE = 23,
  ESP_CMD_AP_CONFIG = 24,
  ESP_CMD_MGMT_TX = 25,
  ESP_CMD_AP_STATION = 26,
  ESP_CMD_STA_RSSI = 27,
  ESP_CMD_MAX
};

/* ESP event codes */

enum esp_event_code
{
  ESP_EVENT_SCAN_RESULT = 1,
  ESP_EVENT_STA_CONNECT,
  ESP_EVENT_STA_DISCONNECT,
  ESP_EVENT_AUTH_RX,
  ESP_EVENT_ASSOC_RX,
  ESP_EVENT_AP_MGMT_RX
};

/* ESP command response status */

enum esp_cmd_response_status
{
  ESP_CMD_RESPONSE_PENDING,
  ESP_CMD_RESPONSE_FAIL,
  ESP_CMD_RESPONSE_SUCCESS,
  ESP_CMD_RESPONSE_BUSY,
  ESP_CMD_RESPONSE_UNSUPPORTED,
  ESP_CMD_RESPONSE_INVALID
};

/* ESP IE types */

enum esp_ie_type
{
  ESP_IE_BEACON,
  ESP_IE_PROBE_RESP,
  ESP_IE_ASSOC_RESP,
  ESP_IE_RSN,
  ESP_IE_BEACON_PROBE_HEAD,
  ESP_IE_BEACON_PROBE_TAIL
};

/* ESP chipset types */

enum esp_chipset_type
{
  ESP_CHIPSET_UNRECOGNIZED = 0xff,
  ESP_CHIPSET_ESP32 = 0x0,
  ESP_CHIPSET_ESP32S2 = 0x2,
  ESP_CHIPSET_ESP32C3 = 0x5,
  ESP_CHIPSET_ESP32S3 = 0x9,
  ESP_CHIPSET_ESP32C2 = 0x0C,
  ESP_CHIPSET_ESP32C6 = 0x0D,
  ESP_CHIPSET_ESP32C61 = 0x14,
  ESP_CHIPSET_ESP32C5 = 0x17
};

/* ESP WiFi mode */

enum esp_wifi_mode
{
  ESP_WIFI_MODE_STA = 0,
  ESP_WIFI_MODE_AP = 1,
  ESP_WIFI_MODE_APSTA = 2
};

/****************************************************************************
 * Structures
 ****************************************************************************/

/* ESP payload header - matches esp-hosted protocol */

struct esp_payload_header
{
  uint8_t if_type : 4;              /* Interface type */
  uint8_t if_num : 4;               /* Interface number */
  uint8_t flags;                    /* Flags */
  uint8_t packet_type;              /* Packet type */
  uint8_t reserved1;
  uint16_t len;                     /* Payload length */
  uint16_t offset;                  /* Offset to payload */
  uint16_t checksum;                /* Checksum */
  uint8_t reserved2;
  uint8_t reserved3;
};

/* ESP command header */

struct esp_command_header
{
  uint8_t cmd_code;                 /* Command code */
  uint8_t cmd_status;               /* Command status */
  uint16_t len;                     /* Payload length */
  uint16_t seq_num;                 /* Sequence number */
  uint8_t reserved1;
  uint8_t reserved2;
};

/* ESP event header */

struct esp_event_header
{
  uint8_t event_code;               /* Event code */
  uint8_t status;                   /* Status */
  uint16_t len;                     /* Payload length */
};

/* ESP scan request command */

struct esp_scan_request_cmd
{
  struct esp_command_header header;
  uint8_t bssid[ESP_MAC_ADDR_LEN];
  uint16_t duration;
  char ssid[ESP_MAX_SSID_LEN + 1];
  uint8_t channel;
  uint8_t pad[2];
};

/* ESP scan result event */

struct esp_scan_result_event
{
  struct esp_event_header header;
  uint8_t bssid[ESP_MAC_ADDR_LEN];
  uint8_t frame_type;
  uint8_t channel;
  uint32_t rssi;
  uint64_t tsf;
  uint16_t frame_len;
  uint8_t pad[2];
  uint8_t frame[];
};

/* ESP connect command */

struct esp_connect_cmd
{
  struct esp_command_header header;
  uint8_t bssid[ESP_MAC_ADDR_LEN];
  uint16_t assoc_flags;
  char ssid[ESP_MAX_SSID_LEN + 1];
  uint8_t channel;
  uint8_t is_auth_open;
  uint8_t assoc_ie_len;
  uint8_t assoc_ie[];
};

/* ESP disconnect command */

struct esp_disconnect_cmd
{
  struct esp_command_header header;
  uint16_t reason_code;
  uint8_t mac[ESP_MAC_ADDR_LEN];
};

/* ESP disconnect event */

struct esp_disconnect_event
{
  struct esp_event_header header;
  uint8_t bssid[ESP_MAC_ADDR_LEN];
  char ssid[ESP_MAX_SSID_LEN + 1];
  uint8_t reason;
};

/* ESP WiFi security key */

struct esp_wifi_sec_key
{
  uint32_t algo;                    /* Algorithm */
  uint32_t index;                   /* Key index */
  uint8_t data[ESP_MAX_KEY_LEN];    /* Key data */
  uint32_t len;                     /* Key length */
  uint8_t mac_addr[ESP_MAC_ADDR_LEN];
  uint8_t seq[ESP_MAX_SEQ_LEN];     /* Sequence */
  uint32_t seq_len;
  uint8_t del;                      /* Delete flag */
  uint8_t set_cur;                  /* Set current flag */
  uint8_t pad[2];
};

/* ESP key operation command */

struct esp_key_cmd
{
  struct esp_command_header header;
  struct esp_wifi_sec_key key;
};

/* ESP AP configuration */

struct esp_ap_config
{
  uint8_t ssid[ESP_MAX_SSID_LEN];
  uint8_t ssid_len;
  uint8_t channel;
  uint8_t authmode;
  uint8_t ssid_hidden;
  uint8_t max_connection;
  uint8_t pairwise_cipher;
  uint8_t pmf_cfg;
  uint8_t sae_pwe_h2e;
  uint16_t beacon_interval;
  uint16_t inactivity_timeout;
  uint8_t privacy;
};

/* ESP AP config command */

struct esp_ap_config_cmd
{
  struct esp_command_header header;
  struct esp_ap_config ap_config;
};

/* ESP MAC address config command */

struct esp_mac_addr_cmd
{
  struct esp_command_header header;
  uint8_t mac_addr[ESP_MAC_ADDR_LEN];
  uint8_t pad[2];
};

/* ESP set/get value command */

struct esp_set_get_cmd
{
  struct esp_command_header header;
  uint32_t value;
};

/* ESP set mode command */

struct esp_set_mode_cmd
{
  struct esp_command_header header;
  uint16_t mode;
  uint8_t pad[2];
};

/* ESP set IE command */

struct esp_set_ie_cmd
{
  struct esp_command_header header;
  uint8_t ie_type;
  uint8_t pad;
  uint16_t ie_len;
  uint8_t ie[];
};

/* ESP management TX command */

struct esp_mgmt_tx_cmd
{
  struct esp_command_header header;
  uint8_t channel;
  uint8_t offchan;
  uint32_t wait;
  uint8_t no_cck;
  uint8_t dont_wait_for_ack;
  uint32_t len;
  uint8_t buf[];
};

/* ESP AP station command */

struct esp_ap_sta_cmd
{
  struct esp_command_header header;
  uint8_t mac[ESP_MAC_ADDR_LEN];
  uint16_t cmd;                     /* ADD_STA, CHANGE_STA, DEL_STA */
  uint32_t sta_flags_mask;
  uint32_t sta_flags_set;
  uint32_t sta_modify_mask;
  int32_t listen_interval;
  uint16_t aid;
  uint8_t ext_capab[6];
  uint8_t supported_rates[12];
  uint8_t ht_caps[28];
  uint8_t vht_caps[14];
  uint8_t pad1[2];
};

/* ESP adapter structure - main driver context */

struct esp_adapter
{
  FAR struct wiphy *wiphy;          /* cfg80211 wiphy */
  uint8_t if_type;                  /* Interface type (SDIO/SPI) */
  uint32_t capabilities;            /* Capabilities */
  int chipset;                      /* Chipset type */
  
  /* Interface context (SDIO or SPI) */
  FAR void *if_context;
  FAR struct esp_if_ops *if_ops;
  
  /* WiFi device instances */
  FAR struct esp_wifi_device *priv[ESP_MAX_INTERFACE];
  
  /* Command handling */
  FAR struct esp_cmd_node *cmd_pool;
  sq_queue_t cmd_free_queue;
  sq_queue_t cmd_pending_queue;
  FAR struct esp_cmd_node *cur_cmd;
  uint16_t cmd_seq_num;
  sem_t cmd_lock;
  sem_t cmd_resp_sem;
  uint8_t cmd_resp_status;
  
  /* Event handling */
  sq_queue_t event_queue;
  sem_t event_lock;
  
  /* Work queue for async operations */
  struct work_s cmd_work;
  struct work_s event_work;
  
  /* State flags */
  volatile uint8_t state_flags;
};

/* ESP device structure */

struct esp_device
{
  FAR struct wiphy *wiphy;
  FAR struct esp_adapter *adapter;
};

/* ESP WiFi device structure - per-interface context */

struct esp_wifi_device
{
  struct wireless_dev wdev;         /* cfg80211 wireless_dev */
  FAR struct net_device *netdev;    /* Network device */
  FAR struct esp_device *esp_dev;
  FAR struct esp_adapter *adapter;
  
  /* Device info */
  uint8_t mac_address[ESP_MAC_ADDR_LEN];
  uint8_t if_type;                  /* ESP_STA_IF or ESP_AP_IF */
  uint8_t if_num;
  
  /* Connection state */
  uint8_t link_state;               /* ESP_LINK_UP/DOWN */
  uint32_t ssid_len;
  uint8_t ssid[ESP_MAX_SSID_LEN];
  FAR struct cfg80211_bss *bss;
  
  /* Scan state */
  FAR struct cfg80211_scan_request *scan_request;
  uint8_t scan_in_progress;
  uint8_t waiting_for_scan_done;
  
  /* Statistics */
  uint64_t rx_bytes;
  uint64_t tx_bytes;
  uint32_t rx_packets;
  uint32_t tx_packets;
  uint32_t tx_dropped;
  
  /* Power settings */
  uint8_t tx_pwr_type;
  uint8_t tx_pwr;
  uint32_t rssi;
  
  /* Regulatory */
  char country_code[4];
  
  /* Control flags */
  volatile uint8_t stop_data;
  volatile uint8_t port_open;
  bool local_disconnect_req;
};

/* ESP command node - for command queue */

struct esp_cmd_node
{
  sq_entry_t node;
  uint8_t cmd_code;
  uint8_t *cmd_buf;
  size_t cmd_len;
  uint8_t *resp_buf;
  size_t resp_len;
  bool in_cmd_queue;
};

/* ESP interface operations - must be implemented by SDIO/SPI layer */

struct esp_if_ops
{
  /* Initialize interface */
  int (*init)(FAR struct esp_adapter *adapter);
  
  /* Deinitialize interface */
  void (*deinit)(FAR struct esp_adapter *adapter);
  
  /* Send data/command */
  int (*send)(FAR struct esp_adapter *adapter, FAR const uint8_t *data,
              size_t len);
  
  /* Receive data/event */
  int (*receive)(FAR struct esp_adapter *adapter, FAR uint8_t *data,
                 size_t max_len);
  
  /* Check if data available */
  bool (*data_available)(FAR struct esp_adapter *adapter);
  
  /* Reset ESP */
  int (*reset)(FAR struct esp_adapter *adapter);
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

#ifdef __cplusplus
extern "C"
{
#endif

/* Driver initialization */

int esp_cfg80211_init(FAR struct esp_if_ops *ops, int if_type);
int esp_cfg80211_deinit(void);

/* Interface management */

int esp_add_interface(FAR struct esp_adapter *adapter,
                      FAR const char *name,
                      enum nl80211_iftype type);
int esp_remove_interface(FAR struct esp_adapter *adapter, uint8_t if_num);

/* Command handling */

int esp_send_command(FAR struct esp_wifi_device *priv,
                     uint8_t cmd_code, FAR const uint8_t *data, size_t len);
int esp_process_command_response(FAR struct esp_adapter *adapter,
                                 FAR const uint8_t *data, size_t len);
int esp_process_event(FAR struct esp_adapter *adapter,
                      FAR const uint8_t *data, size_t len);

/* Specific commands */

int esp_cmd_init_interface(FAR struct esp_wifi_device *priv);
int esp_cmd_deinit_interface(FAR struct esp_wifi_device *priv);
int esp_cmd_get_mac(FAR struct esp_wifi_device *priv);
int esp_cmd_set_mac(FAR struct esp_wifi_device *priv, FAR const uint8_t *mac);
int esp_cmd_scan_request(FAR struct esp_wifi_device *priv,
                         FAR struct cfg80211_scan_request *request);
int esp_cmd_connect(FAR struct esp_wifi_device *priv,
                    FAR struct cfg80211_connect_params *params);
int esp_cmd_disconnect(FAR struct esp_wifi_device *priv, uint16_t reason,
                       FAR const uint8_t *mac);
int esp_cmd_add_key(FAR struct esp_wifi_device *priv, uint8_t key_index,
                    bool pairwise, FAR const uint8_t *mac_addr,
                    FAR struct key_params *params);
int esp_cmd_del_key(FAR struct esp_wifi_device *priv, uint8_t key_index,
                    bool pairwise, FAR const uint8_t *mac_addr);
int esp_cmd_set_default_key(FAR struct esp_wifi_device *priv,
                            uint8_t key_index);
int esp_cmd_set_mode(FAR struct esp_wifi_device *priv, uint8_t mode);
int esp_cmd_set_ie(FAR struct esp_wifi_device *priv,
                   enum esp_ie_type type, FAR const uint8_t *ie, size_t len);
int esp_cmd_start_ap(FAR struct esp_wifi_device *priv,
                     FAR struct cfg80211_ap_settings *settings);
int esp_cmd_stop_ap(FAR struct esp_wifi_device *priv);
int esp_cmd_mgmt_tx(FAR struct esp_wifi_device *priv,
                    FAR struct cfg80211_mgmt_tx_params *params);
int esp_cmd_add_station(FAR struct esp_wifi_device *priv,
                        FAR const uint8_t *mac,
                        FAR struct station_parameters *params);
int esp_cmd_del_station(FAR struct esp_wifi_device *priv,
                        FAR const uint8_t *mac, uint16_t reason);
int esp_cmd_get_rssi(FAR struct esp_wifi_device *priv);
int esp_cmd_set_tx_power(FAR struct esp_wifi_device *priv, int power);
int esp_cmd_get_tx_power(FAR struct esp_wifi_device *priv);
int esp_cmd_set_reg_domain(FAR struct esp_wifi_device *priv);

/* Event notifications */

void esp_notify_scan_done(FAR struct esp_wifi_device *priv, bool aborted);
void esp_notify_connect_result(FAR struct esp_wifi_device *priv,
                                FAR const uint8_t *bssid, int status);
void esp_notify_disconnect(FAR struct esp_wifi_device *priv, uint16_t reason,
                           bool locally_generated);

/* Utility functions */

uint16_t esp_compute_checksum(FAR const uint8_t *buf, uint16_t len);
int esp_mbm_to_power(int mbm);
int esp_power_to_dbm(int power);

/* Get adapter */

FAR struct esp_adapter *esp_get_adapter(void);

/* Network device functions */

int esp32_netdev_register(struct esp_wifi_device *priv);
int esp32_netdev_unregister(struct esp_wifi_device *priv);
int esp32_netdev_receive(struct esp_adapter *adapter);
void esp_port_open(struct esp_wifi_device *priv);
void esp_port_close(struct esp_wifi_device *priv);
void esp_tx_pause(struct esp_wifi_device *priv);
void esp_tx_resume(struct esp_wifi_device *priv);

/* Boot and initialization functions */

int esp32_boot_init(struct esp_adapter *adapter);
int esp32_check_capabilities(struct esp_adapter *adapter);
bool esp_checksum_enabled(struct esp_adapter *adapter);
const char *esp_get_chipset_name(struct esp_adapter *adapter);

#ifdef __cplusplus
}
#endif

#endif /* __DRIVERS_WIRELESS_ESP32_ESP_CFG80211_H */