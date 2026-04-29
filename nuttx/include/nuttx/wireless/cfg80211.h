/****************************************************************************
 * include/nuttx/wireless/cfg80211.h
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
 * cfg80211 - Configuration API for 802.11 Drivers
 *
 * This header defines the cfg80211 interface that WiFi drivers must implement
 * to provide a standardized configuration interface.
 * Based on Linux kernel cfg80211.h for compatibility.
 ****************************************************************************/

#ifndef __INCLUDE_NUTTX_WIRELESS_CFG80211_H
#define __INCLUDE_NUTTX_WIRELESS_CFG80211_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <nuttx/compiler.h>

#include <sys/socket.h>
#include <stdint.h>

#include <netpacket/netlink.h>
#include <nuttx/wireless/nl80211.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* WIPHY name maximum length */

#define WIPHY_NAME_MAXLEN                       64

/* BSSID length */

#define ETH_ALEN                                 6

/* Maximum number of SSIDs in a scan request */

#define IEEE80211_MAX_SSID_LEN                  32

/* Maximum number of channels */

#define IEEE80211_MAX_CHANNELS                  64

/* Maximum number of rates */

#define IEEE80211_MAX_RATES                     32

/* Maximum number of rates extended */

#define IEEE80211_MAX_RATES_EX                  64

/* Maximum key length */

#define IEEE80211_MAX_KEY_LEN                   32

/* Maximum IE length */

#define IEEE80211_MAX_IE_LEN                    512

/* TX queue depth */

#define IEEE80211_MAX_QUEUE                     16

/****************************************************************************
 * Data Types
 ****************************************************************************/

/* Forward declarations */

struct wiphy;
struct net_device;
struct wireless_dev;
struct cfg80211_scan_request;
struct cfg80211_ssid;
struct cfg80211_connect_params;
struct cfg80211_assoc_request;
struct cfg80211_deauth_request;
struct cfg80211_disassoc_request;
struct station_parameters;
struct station_info;
struct key_params;
struct cfg80211_ops;

/* struct ieee80211_channel - channel definition
 *
 * @band: band this channel belongs to
 * @center_freq: center frequency in MHz
 * @hw_value: hardware-specific value for this channel
 * @flags: channel flags (see below)
 * @ht_cap: HT capabilities (only valid if band is 2.4 GHz)
 * @vht_cap: VHT capabilities (only valid if band is 5 GHz)
 * @max_antenna_gain: maximum antenna gain in dBi
 * @max_power: maximum transmission power in dBm
 * @max_reg_power: maximum regulatory transmission power in dBm
 * @beacon_found: beacon was found on this channel
 * @orig_mpdu_min_spacing: minimum MPDU spacing for this channel
 * @orig_mpdu_aggr_max: maximum A-MPDU aggregation for this channel
 */

struct ieee80211_channel
{
  uint8_t band;                               /* IEEE80211_BAND_* */
  uint32_t center_freq;                        /* in MHz */
  uint16_t hw_value;
  uint32_t flags;                              /* channel flags */
  int8_t max_antenna_gain;                     /* in dBi */
  int8_t max_power;                            /* in dBm */
  int8_t max_reg_power;                        /* in dBm */
  uint8_t beacon_found;
  uint8_t orig_mpdu_min_spacing;
  uint16_t orig_mpdu_aggr_max;
};

/* Channel flags */

#define IEEE80211_CHAN_DISABLED                 (1 << 0)
#define IEEE80211_CHAN_NO_IR                    (1 << 1)
#define IEEE80211_CHAN_RADAR                    (1 << 2)
#define IEEE80211_CHAN_NO_HT40PLUS              (1 << 3)
#define IEEE80211_CHAN_NO_HT40MINUS             (1 << 4)
#define IEEE80211_CHAN_NO_80MHZ                 (1 << 5)
#define IEEE80211_CHAN_NO_160MHZ                (1 << 6)
#define IEEE80211_CHAN_INDOOR_ONLY              (1 << 7)
#define IEEE80211_CHAN_IR_CONCURRENT            (1 << 8)
#define IEEE80211_CHAN_NO_20MHZ                 (1 << 9)
#define IEEE80211_CHAN_NO_10MHZ                 (1 << 10)

/* Frequency bands */

enum ieee80211_band
{
  IEEE80211_BAND_2GHZ = 0,
  IEEE80211_BAND_5GHZ = 1,
  IEEE80211_BAND_60GHZ = 2,
  IEEE80211_BAND_6GHZ = 3,
  IEEE80211_NUM_BANDS
};

/* struct ieee80211_rate - bitrate definition
 *
 * @flags: bitrate flags
 * @bitrate: bitrate in units of 100 Kbps
 * @hw_value: hardware-specific value for this rate
 * @hw_value_short: hardware-specific value for this rate when using
 *     short preamble
 */

struct ieee80211_rate
{
  uint32_t flags;
  uint16_t bitrate;
  uint16_t hw_value;
  uint16_t hw_value_short;
};

/* Rate flags */

#define IEEE80211_RATE_SHORT_PREAMBLE           (1 << 0)
#define IEEE80211_RATE_ERP_G                   (1 << 1)
#define IEEE80211_RATE_SUPP_RATES_80211G       (1 << 2)
#define IEEE80211_RATE_OFDM_20MHZ             (1 << 3)
#define IEEE80211_RATE_OFDM_40MHZ             (1 << 4)

/* struct ieee80211_sta_ht_cap - HT capabilities
 *
 * @cap: capabilities
 * @ht_supported: HT supported
 * @ampdu_factor: maximum A-MPDU length factor
 * @ampdu_density: maximum A-MPDU density
 * @mcs: MCS information
 */

struct ieee80211_sta_ht_cap
{
  uint16_t cap;                               /* HT capabilities */
  uint8_t ht_supported;                        /* HT supported */
  uint8_t ampdu_factor;                        /* max A-MPDU length factor */
  uint8_t ampdu_density;                       /* max A-MPDU density */
  struct
  {
    uint8_t rx_mask[16];                       /* RX MCS mask */
    uint16_t rx_highest;                       /* highest RX rate */
    uint8_t tx_mask[16];                       /* TX MCS mask */
    uint8_t tx_params;                         /* TX params */
    uint8_t rx_highest_long;                   /* highest RX rate long GI */
    uint8_t tx_highest_long;                   /* highest TX rate long GI */
  } mcs;
};

/* struct ieee80211_sta_vht_cap - VHT capabilities
 *
 * @vht_supported: VHT supported
 * @cap: capabilities
 * @vht_mcs: VHT MCS information
 */

struct ieee80211_sta_vht_cap
{
  uint8_t vht_supported;                       /* VHT supported */
  uint32_t cap;                               /* VHT capabilities */
  struct
  {
    uint16_t rx_mcs_map;                      /* RX MCS map */
    uint16_t rx_highest;                      /* highest RX rate */
    uint16_t tx_mcs_map;                      /* TX MCS map */
    uint16_t tx_highest;                      /* highest TX rate */
  } vht_mcs;
};

/* struct ieee80211_supported_band - band information
 *
 * @band: the band this list represents
 * @n_channels: number of channels in this list
 * @channels: channel information
 * @n_bitrates: number of bitrates in this list
 * @bitrates: bitrate information
 * @ht_cap: HT capabilities
 * @vht_cap: VHT capabilities
 */

struct ieee80211_supported_band
{
  enum ieee80211_band band;
  uint8_t n_channels;
  FAR struct ieee80211_channel *channels;
  uint8_t n_bitrates;
  FAR struct ieee80211_rate *bitrates;
  struct ieee80211_sta_ht_cap ht_cap;
  struct ieee80211_sta_vht_cap vht_cap;
};

/* struct cfg80211_ssid - SSID information
 *
 * @ssid_len: length of SSID
 * @ssid: SSID bytes
 */

struct cfg80211_ssid
{
  uint8_t ssid_len;
  uint8_t ssid[IEEE80211_MAX_SSID_LEN];
};

/* struct cfg80211_scan_request - scan request
 *
 * @wiphy: wiphy pointer
 * @netdev: network device (may be NULL)
 * @ssids: SSIDs to scan for
 * @n_ssids: number of SSIDs
 * @channels: channels to scan
 * @n_channels: number of channels
 * @ie: additional IEs for probe request
 * @ie_len: length of ie
 * @duration: scan duration in ms
 * @duration_mandatory: duration is mandatory
 * @flags: scan flags
 */

struct cfg80211_scan_request
{
  struct wiphy *wiphy;
  struct net_device *netdev;
  struct cfg80211_ssid *ssids;
  uint8_t n_ssids;
  FAR struct ieee80211_channel **channels;
  uint8_t n_channels;
  FAR uint8_t *ie;
  size_t ie_len;
  uint16_t duration;
  uint8_t duration_mandatory;
  uint8_t flags;
  uint8_t n_iftype;
  FAR enum nl80211_iftype *iftypes;
};

/* struct cfg80211_bss - BSS information
 *
 * @wiphy: wiphy pointer
 * @netdev: network device
 * @bssid: BSSID
 * @ssid: SSID
 * @ssid_len: length of SSID
 * @channel: channel where this BSS was found
 * @beacon_interval: beacon interval advertised
 * @capability: capability information
 * @ies: IEs from probe response/beacon
 * @ies_len: length of ies
 * @beacon_ies: IEs from beacon
 * @beacon_ies_len: length of beacon ies
 * @signal: signal strength (dBm)
 * @beacon_signal: beacon signal strength (dBm)
 * @tsf: Timestamp
 * @priv: private area for driver use
 */

struct cfg80211_bss
{
  struct wiphy *wiphy;
  struct net_device *netdev;
  uint8_t bssid[ETH_ALEN];
  uint8_t ssid[IEEE80211_MAX_SSID_LEN];
  size_t ssid_len;
  struct ieee80211_channel *channel;
  uint16_t beacon_interval;
  uint16_t capability;
  FAR uint8_t *ies;
  size_t ies_len;
  FAR uint8_t *beacon_ies;
  size_t beacon_ies_len;
  int16_t signal;
  int16_t beacon_signal;
  uint64_t tsf;
  FAR void *priv;
};

/* struct station_parameters - station parameters
 *
 * @supported_rates: supported rates
 * @supported_rates_len: number of supported rates
 * @sta_flags_mask: station flags to update
 * @sta_flags_set: station flags to set
 * @listen_interval: listen interval
 * @aid: AID
 * @vlan: VLAN pointer
 * @plink_state: mesh plink state
 * @plink_action: mesh plink action
 * @ht_capa: HT capabilities
 * @vht_capa: VHT capabilities
 * @sta_modify_mask: mask of parameters to modify
 * @local_pm: local power save mode
 * @peer_pm: peer power save mode
 * @nonpeer_pm: non-peer power save mode
 */

struct station_parameters
{
  FAR uint8_t *supported_rates;
  size_t supported_rates_len;
  uint32_t sta_flags_mask;
  uint32_t sta_flags_set;
  uint16_t listen_interval;
  uint16_t aid;
  FAR struct net_device *vlan;
  uint8_t sta_flags_supp;
  uint8_t plink_state;
  uint8_t plink_action;
  FAR struct ieee80211_sta_ht_cap *ht_capa;
  FAR struct ieee80211_sta_vht_cap *vht_capa;
  uint32_t sta_modify_mask;
  uint32_t local_pm;
  uint32_t peer_pm;
  uint32_t nonpeer_pm;
  FAR uint8_t *ext_capab;
  size_t ext_capab_len;
  FAR uint8_t *supported_channels;
  size_t supported_channels_len;
  FAR uint8_t *supported_oper_classes;
  size_t supported_oper_classes_len;
};

/* struct station_info - station information
 *
 * @filled: which fields are filled
 * @connected_time: time since connection
 * @inactive_time: time since last activity
 * @rx_bytes: received bytes
 * @tx_bytes: transmitted bytes
 * @llid: mesh local link ID
 * @plid: mesh peer link ID
 * @plink_state: mesh plink state
 * @signal: signal strength (dBm)
 * @signal_avg: average signal strength (dBm)
 * @txrate: current bitrate for TX
 * @rxrate: current bitrate for RX
 * @rx_packets: received packets
 * @tx_packets: transmitted packets
 * @tx_retries: TX retries
 * @tx_failed: TX failed
 * @rx_dropped_misc: dropped RX packets
 * @bss_param: BSS parameters
 * @generation: generation number
 */

struct station_info
{
  uint32_t filled;
  uint32_t connected_time;
  uint32_t inactive_time;
  uint64_t rx_bytes;
  uint64_t tx_bytes;
  uint16_t llid;
  uint16_t plid;
  uint8_t plink_state;
  int8_t signal;
  int8_t signal_avg;
  struct
  {
    uint32_t legacy;
    uint16_t mcs;
    uint16_t nss;
    uint8_t bw;
    uint8_t short_gi;
  } txrate;
  struct
  {
    uint32_t legacy;
    uint16_t mcs;
    uint16_t nss;
    uint8_t bw;
    uint8_t short_gi;
  } rxrate;
  uint32_t rx_packets;
  uint32_t tx_packets;
  uint32_t tx_retries;
  uint32_t tx_failed;
  uint32_t rx_dropped_misc;
  struct
  {
    uint8_t dtim_period;
    uint16_t beacon_interval;
  } bss_param;
  int generation;
  FAR void *priv;
};

/* station_info filled flags */

#define CFG80211_STA_INFO_CONNECTED_TIME         (1 << 0)
#define CFG80211_STA_INFO_INACTIVE_TIME          (1 << 1)
#define CFG80211_STA_INFO_RX_BYTES               (1 << 2)
#define CFG80211_STA_INFO_TX_BYTES               (1 << 3)
#define CFG80211_STA_INFO_LLID                  (1 << 4)
#define CFG80211_STA_INFO_PLID                  (1 << 5)
#define CFG80211_STA_INFO_PLINK_STATE           (1 << 6)
#define CFG80211_STA_INFO_SIGNAL                (1 << 7)
#define CFG80211_STA_INFO_SIGNAL_AVG            (1 << 8)
#define CFG80211_STA_INFO_TX_BITRATE            (1 << 9)
#define CFG80211_STA_INFO_RX_BITRATE            (1 << 10)
#define CFG80211_STA_INFO_RX_PACKETS            (1 << 11)
#define CFG80211_STA_INFO_TX_PACKETS            (1 << 12)
#define CFG80211_STA_INFO_TX_RETRIES            (1 << 13)
#define CFG80211_STA_INFO_TX_FAILED             (1 << 14)
#define CFG80211_STA_INFO_RX_DROP_MISC          (1 << 15)
#define CFG80211_STA_INFO_BSS_PARAM             (1 << 16)
#define CFG80211_STA_INFO_STA_FLAGS             (1 << 17)
#define CFG80211_STA_INFO_LOCAL_PM              (1 << 18)
#define CFG80211_STA_INFO_PEER_PM              (1 << 19)
#define CFG80211_STA_INFO_NONPEER_PM            (1 << 20)

/* struct key_params - key parameters
 *
 * @key: key material
 * @key_len: length of key material
 * @cipher: cipher suite
 * @seq: sequence counter (for TKIP)
 * @seq_len: length of sequence counter
 */

struct key_params
{
  FAR uint8_t *key;
  size_t key_len;
  uint32_t cipher;
  FAR uint8_t *seq;
  size_t seq_len;
};

/* struct vif_params - virtual interface parameters
 *
 * @use_4addr: use 4-address frames
 * @macaddr: MAC address (may be NULL)
 */

struct vif_params
{
  int use_4addr;
  FAR uint8_t *macaddr;
};

/* struct cfg80211_connect_params - connection parameters
 *
 * @ssid: SSID
 * @bssid: BSSID (may be NULL)
 * @auth_type: authentication type
 * @ie: IEs for association request
 * @ie_len: length of ie
 * @privacy: privacy-enabled network
 * @bgscan_period: background scan period (0 = disabled)
 * @key: key material (for WEP)
 * @key_len: length of key
 * @key_idx: key index (for WEP)
 * @flags: connection flags
 */

struct cfg80211_connect_params
{
  struct cfg80211_ssid ssid;
  FAR uint8_t *bssid;
  uint32_t auth_type;
  FAR uint8_t *ie;
  size_t ie_len;
  uint8_t privacy;
  uint16_t bgscan_period;
  FAR uint8_t *key;
  size_t key_len;
  uint8_t key_idx;
  uint32_t flags;
  uint16_t reason_code;
  uint32_t cipher_group;
  uint32_t n_ciphers_pairwise;
  uint32_t *ciphers_pairwise;
  uint32_t n_akm_suites;
  uint32_t *akm_suites;
  uint8_t mfp;
};

/* Connection flags */

#define CFG80211_CONN_DISABLE_LEGACY              (1 << 0)
#define CFG80211_CONN_DISABLE_ROAMING             (1 << 1)
#define CFG80211_CONN_DO_ROAM                     (1 << 2)
#define CFG80211_CONN_CONNECT_FAILED               (1 << 3)
#define CFG80211_CONN_INFORM_4ADDR_MODE           (1 << 4)
#define CFG80211_CONN_DISABLE_HE                  (1 << 5)

/* struct cfg80211_assoc_request - association request
 *
 * @bss: BSS to associate with
 * @ie: IEs for association request
 * @ie_len: length of ie
 * @use_mfp: use MFP
 * @prev_bssid: previous BSSID for reassociation
 */

struct cfg80211_assoc_request
{
  FAR struct cfg80211_bss *bss;
  FAR uint8_t *ie;
  size_t ie_len;
  uint8_t use_mfp;
  FAR uint8_t *prev_bssid;
};

/* struct cfg80211_deauth_request - deauthentication request
 *
 * @bssid: BSSID
 * @reason_code: reason code
 * @local_state_change: local state change only
 */

struct cfg80211_deauth_request
{
  FAR uint8_t *bssid;
  uint16_t reason_code;
  uint8_t local_state_change;
};

/* struct cfg80211_disassoc_request - disassociation request
 *
 * @bssid: BSSID
 * @reason_code: reason code
 * @local_state_change: local state change only
 */

struct cfg80211_disassoc_request
{
  FAR uint8_t *bssid;
  uint16_t reason_code;
  uint8_t local_state_change;
};

/* struct cfg80211_pmksa - PMKSA cache entry
 *
 * @bssid: BSSID
 * @pmkid: PMKID
 * @pmk: PMK (may be NULL)
 * @pmk_len: length of PMK
 * @ssid: SSID (may be NULL)
 * @ssid_len: length of SSID
 * @cache_id: cache identifier
 */

struct cfg80211_pmksa
{
  FAR uint8_t *bssid;
  FAR uint8_t *pmkid;
  FAR uint8_t *pmk;
  size_t pmk_len;
  FAR uint8_t *ssid;
  size_t ssid_len;
  FAR uint8_t *cache_id;
};

/* struct cfg80211_beacon_settings - beacon settings
 *
 * @beacon: beacon head
 * @beacon_len: beacon head length
 * @beacon_tail: beacon tail
 * @beacon_tail_len: beacon tail length
 * @dtim_period: DTIM period
 * @dtim_count: DTIM count
 */

struct cfg80211_beacon_settings
{
  FAR uint8_t *beacon;
  size_t beacon_len;
  FAR uint8_t *beacon_tail;
  size_t beacon_tail_len;
  uint8_t dtim_period;
  uint8_t dtim_count;
};

/* struct cfg80211_ap_settings - AP settings
 *
 * @beacon: beacon settings
 * @ssid: SSID
 * @ssid_len: length of SSID
 * @privacy: privacy-enabled AP
 * @auth_type: authentication type
 * @inactivity_timeout: inactivity timeout
 * @he_capa: HE capabilities
 * @he_capa_len: length of HE capabilities
 */

struct cfg80211_ap_settings
{
  struct cfg80211_beacon_settings beacon;
  FAR uint8_t *ssid;
  size_t ssid_len;
  uint8_t privacy;
  uint32_t auth_type;
  int inactivity_timeout;
  FAR uint8_t *he_capa;
  size_t he_capa_len;
};

/* struct survey_info - survey information
 *
 * @channel: channel this survey is for
 * @filled: which fields are filled
 * @noise: noise level (dBm)
 * @channel_time: total channel time (in us)
 * @channel_time_busy: busy channel time (in us)
 * @channel_time_ext_busy: extension channel busy time (in us)
 * @channel_time_rx: RX channel time (in us)
 * @channel_time_tx: TX channel time (in us)
 */

struct survey_info
{
  FAR struct ieee80211_channel *channel;
  uint64_t filled;
  int8_t noise;
  uint64_t channel_time;
  uint64_t channel_time_busy;
  uint64_t channel_time_ext_busy;
  uint64_t channel_time_rx;
  uint64_t channel_time_tx;
};

/* survey_info filled flags */

#define CFG80211_SURVEY_INFO_FREQUENCY           (1 << 0)
#define CFG80211_SURVEY_INFO_NOISE               (1 << 1)
#define CFG80211_SURVEY_INFO_IN_USE              (1 << 2)
#define CFG80211_SURVEY_INFO_CHANNEL_TIME         (1 << 3)
#define CFG80211_SURVEY_INFO_CHANNEL_TIME_BUSY    (1 << 4)
#define CFG80211_SURVEY_INFO_CHANNEL_TIME_EXT_BUSY (1 << 5)
#define CFG80211_SURVEY_INFO_CHANNEL_TIME_RX      (1 << 6)
#define CFG80211_SURVEY_INFO_CHANNEL_TIME_TX      (1 << 7)

/* struct station_del_parameters - station delete parameters
 *
 * @mac: MAC address (may be NULL for all)
 * @subtype: subtype to send
 * @reason_code: reason code
 */

struct station_del_parameters
{
  FAR uint8_t *mac;
  uint16_t reason_code;
  uint8_t subtype;
};

/****************************************************************************
 * cfg80211 Operations
 *
 * These operations are implemented by WiFi drivers and registered with cfg80211.
 ****************************************************************************/

/* struct cfg80211_ops - cfg80211 operations
 *
 * This structure defines the operations that a WiFi driver must implement
 * to work with cfg80211.
 */

struct cfg80211_ops
{
  /* Virtual interface management */

  int (*add_virtual_intf)(FAR struct wiphy *wiphy, FAR const char *name,
                          enum nl80211_iftype type,
                          FAR struct vif_params *params);
  int (*del_virtual_intf)(FAR struct wiphy *wiphy,
                          FAR struct wireless_dev *wdev);
  int (*change_virtual_intf)(FAR struct wiphy *wiphy,
                             FAR struct wireless_dev *wdev,
                             enum nl80211_iftype type,
                             FAR struct vif_params *params);

  /* Key management */

  int (*add_key)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                 uint8_t key_index, bool pairwise,
                 FAR const uint8_t *mac_addr,
                 FAR struct key_params *params);
  int (*get_key)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                 uint8_t key_index, bool pairwise,
                 FAR const uint8_t *mac_addr, FAR void *cookie,
                 FAR void (*callback)(FAR void *cookie,
                                      FAR struct key_params *params));
  int (*del_key)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                 uint8_t key_index, bool pairwise,
                 FAR const uint8_t *mac_addr);
  int (*set_default_key)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                         uint8_t key_index, bool unicast, bool multicast);
  int (*set_default_mgmt_key)(FAR struct wiphy *wiphy,
                               FAR struct net_device *netdev,
                               uint8_t key_index);
  int (*set_rekey_data)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                        FAR struct cfg80211_gtk_rekey_data *data);

  /* Scanning */

  int (*scan)(FAR struct wiphy *wiphy,
              FAR struct cfg80211_scan_request *request);
  void (*abort_scan)(FAR struct wiphy *wiphy, FAR struct wireless_dev *wdev);

  /* Authentication/Association */

  int (*auth)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
              FAR struct cfg80211_auth_request *req);
  int (*assoc)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
               FAR struct cfg80211_assoc_request *req);
  int (*deauth)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                FAR struct cfg80211_deauth_request *req);
  int (*disassoc)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                  FAR struct cfg80211_disassoc_request *req);

  /* Connection */

  int (*connect)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                 FAR struct cfg80211_connect_params *sme);
  int (*disconnect)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                    uint16_t reason_code);
  int (*join_ibss)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                   FAR struct cfg80211_ibss_params *params);
  int (*leave_ibss)(FAR struct wiphy *wiphy, FAR struct net_device *netdev);

  /* AP mode */

  int (*start_ap)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                  FAR struct cfg80211_ap_settings *settings);
  int (*stop_ap)(FAR struct wiphy *wiphy, FAR struct net_device *netdev);
  int (*change_beacon)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                       FAR struct cfg80211_beacon_settings *info);
  int (*add_station)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                     FAR const uint8_t *mac,
                     FAR struct station_parameters *params);
  int (*del_station)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                     FAR struct station_del_parameters *params);
  int (*change_station)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                        FAR const uint8_t *mac,
                        FAR struct station_parameters *params);
  int (*get_station)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                      FAR const uint8_t *mac,
                      FAR struct station_info *sinfo);
  int (*dump_station)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                      int idx, FAR uint8_t *mac,
                      FAR struct station_info *sinfo);

  /* TX */

  int (*set_tx_power)(FAR struct wiphy *wiphy, FAR struct wireless_dev *wdev,
                      enum nl80211_tx_power_setting type, int mbm);
  int (*get_tx_power)(FAR struct wiphy *wiphy, FAR struct wireless_dev *wdev,
                      FAR int *dbm);
  int (*set_power_mgmt)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                        bool enabled, int timeout);

  /* WIPHY parameters */

  int (*set_wiphy_params)(FAR struct wiphy *wiphy, uint32_t changed);
  int (*set_antenna)(FAR struct wiphy *wiphy, uint32_t tx_ant,
                     uint32_t rx_ant);
  int (*get_antenna)(FAR struct wiphy *wiphy, FAR uint32_t *tx_ant,
                     FAR uint32_t *rx_ant);

  /* Channel */

  int (*set_channel)(FAR struct wiphy *wiphy,
                     FAR struct ieee80211_channel *chan,
                     enum nl80211_channel_width width,
                     uint8_t center_freq1, uint8_t center_freq2);

  /* PMKSA */

  int (*set_pmksa)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                    FAR struct cfg80211_pmksa *pmksa);
  int (*del_pmksa)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                    FAR struct cfg80211_pmksa *pmksa);
  int (*flush_pmksa)(FAR struct wiphy *wiphy, FAR struct net_device *netdev);

  /* Remain on channel */

  int (*remain_on_channel)(FAR struct wiphy *wiphy, FAR struct wireless_dev *wdev,
                           FAR struct ieee80211_channel *chan,
                           unsigned int duration, FAR uint64_t *cookie);
  int (*cancel_remain_on_channel)(FAR struct wiphy *wiphy,
                                   FAR struct wireless_dev *wdev,
                                   uint64_t cookie);

  /* mgmt TX */

  int (*mgmt_tx)(FAR struct wiphy *wiphy, FAR struct wireless_dev *wdev,
                 FAR struct cfg80211_mgmt_tx_params *params,
                 FAR uint64_t *cookie);
  int (*mgmt_tx_cancel_wait)(FAR struct wiphy *wiphy,
                             FAR struct wireless_dev *wdev, uint64_t cookie);

  /* Connection quality */

  int (*set_cqm_rssi_config)(FAR struct wiphy *wiphy,
                              FAR struct net_device *netdev,
                              int32_t rssi_thold, uint32_t rssi_hyst);
  int (*set_cqm_txe_config)(FAR struct wiphy *wiphy,
                             FAR struct net_device *netdev,
                             uint32_t rate, uint32_t pkts, uint32_t intvl);

  /* TDLS */

  int (*tdls_oper)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                    FAR const uint8_t *peer, enum nl80211_tdls_operation oper);
  int (*tdls_mgmt)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                    FAR const uint8_t *peer, uint8_t action_code,
                    uint8_t dialog_token, uint16_t status_code,
                    uint32_t peer_capability, bool initiator,
                    FAR const uint8_t *buf, size_t len);

  /* WoWLAN */

  int (*set_wowlan)(FAR struct wiphy *wiphy, FAR struct cfg80211_wowlan *wowlan);
  int (*get_wowlan)(FAR struct wiphy *wiphy, FAR struct cfg80211_wowlan *wowlan);

  /* Scheduled scan */

  int (*sched_scan_start)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                           FAR struct cfg80211_sched_scan_request *request);
  int (*sched_scan_stop)(FAR struct wiphy *wiphy, FAR struct net_device *netdev);

  /* Survey */

  int (*dump_survey)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                      int idx, FAR struct survey_info *info);

  /* External auth */

  int (*external_auth)(FAR struct wiphy *wiphy, FAR struct net_device *netdev,
                       FAR struct cfg80211_external_auth_params *params);

  /* Reserved for future use */

  FAR void *reserved[8];
};

/****************************************************************************
 * cfg80211 Data Structures
 ****************************************************************************/

/* struct wiphy - wireless hardware description
 *
 * This structure represents a wireless hardware device.
 */

struct wiphy
{
  /* Device information */

  char name[WIPHY_NAME_MAXLEN];
  uint8_t mac_addr[ETH_ALEN];

  /* Supported bands */

  FAR struct ieee80211_supported_band *bands[IEEE80211_NUM_BANDS];

  /* Interface types supported */

  uint32_t interface_modes;

  /* Maximum values */

  uint16_t max_scan_ssids;
  uint16_t max_sched_scan_ssids;
  uint16_t max_match_sets;
  uint16_t max_scan_ie_len;
  uint16_t max_sched_scan_ie_len;
  uint16_t max_remain_on_channel_duration;

  /* Signal type */

  int8_t signal_type;

  /* Cipher suites */

  uint8_t n_cipher_suites;
  FAR const uint32_t *cipher_suites;

  /* AKM suites */

  uint8_t n_akm_suites;
  FAR const uint32_t *akm_suites;

  /* Retry limits */

  uint8_t retry_short;
  uint8_t retry_long;
  uint16_t frag_threshold;
  uint16_t rts_threshold;

  /* Coverage class */

  uint8_t coverage_class;

  /* TX power levels */

  int8_t max_tx_power;
  int8_t min_tx_power;

  /* Antenna configuration */

  uint32_t available_antennas_tx;
  uint32_t available_antennas_rx;
  uint32_t tx_antenna_mask;
  uint32_t rx_antenna_mask;

  /* Feature flags */

  uint64_t features;

  /* Device capabilities */

  uint32_t flags;

  /* Number of probes */

  uint8_t max_num_pmkids;

  /* Supported extended capabilities */

  FAR uint8_t *extended_capabilities;
  size_t extended_capabilities_len;
  FAR uint8_t *extended_capabilities_mask;
  size_t extended_capabilities_mask_len;

  /* Private driver data */

  FAR void *priv;

  /* Operations */

  FAR struct cfg80211_ops *ops;
};

/* wiphy flags */

#define WIPHY_FLAG_NETNS_OK                     (1 << 0)
#define WIPHY_FLAG_PS_ON_BY_DEFAULT            (1 << 1)
#define WIPHY_FLAG_HAVE_AP_SME                 (1 << 2)
#define WIPHY_FLAG_REPORTS_OBSS                (1 << 3)
#define WIPHY_FLAG_4ADDR_AP                    (1 << 4)
#define WIPHY_FLAG_4ADDR_STATION               (1 << 5)
#define WIPHY_FLAG_CONTROL_PORT_PROTOCOL       (1 << 6)
#define WIPHY_FLAG_IBSS_RSN                    (1 << 7)
#define WIPHY_FLAG_MESH_AUTH                   (1 << 8)
#define WIPHY_FLAG_SUPPORTS_FW_ROAM            (1 << 9)
#define WIPHY_FLAG_AP_UAPSD                   (1 << 10)
#define WIPHY_FLAG_SUPPORTS_TDLS_BUFFER_STA    (1 << 11)
#define WIPHY_FLAG_TDLS_EXTERNAL_SETUP         (1 << 12)
#define WIPHY_FLAG_HAVE_AP_OFFCHANNEL          (1 << 13)
#define WIPHY_FLAG_STATION_DONOT_TRIGGER_ROAM  (1 << 14)
#define WIPHY_FLAG_SUPPORTS_5_10_MHZ           (1 << 15)
#define WIPHY_FLAG_HAS_CHANNEL_SWITCH          (1 << 16)
#define WIPHY_FLAG_P2P_GO_CTWindow            (1 << 17)
#define WIPHY_FLAG_P2P_GO_OPPPS               (1 << 18)
#define WIPHY_FLAG_SPLIT_BSS_DUMP             (1 << 19)
#define WIPHY_FLAG_DISABLE_HT                 (1 << 20)
#define WIPHY_FLAG_HT40_OBSS_SCAN             (1 << 21)
#define WIPHY_FLAG_TDLS_WIDER_BW             (1 << 22)
#define WIPHY_FLAG_SUPPORTS_SCHED_SCAN        (1 << 23)
#define WIPHY_FLAG_SUPPORTS_FW_ROAM_SKIPPED  (1 << 24)

/* signal type */

#define CFG80211_SIGNAL_TYPE_UNSPEC            0
#define CFG80211_SIGNAL_TYPE_MBM               1
#define CFG80211_SIGNAL_TYPE_DBM               2

/* struct wireless_dev - wireless device state
 *
 * This structure represents a virtual interface or monitor mode device.
 */

struct wireless_dev
{
  /* Reference to wiphy */

  struct wiphy *wiphy;

  /* Interface type */

  enum nl80211_iftype iftype;

  /* Device identifier */

  uint32_t identifier;

  /* Network device */

  struct net_device *netdev;

  /* List entry */

  FAR void *list;

  /* List of mgmt registrations */

  FAR void *mgmt_registrations;

  /* Cookie counter for mgmt TX */

  uint64_t cookie_counter;

  /* Current channel */

  FAR struct ieee80211_channel *current_channel;

  /* Connector */

  FAR uint8_t *conn_ie;
  size_t conn_ie_len;

  /* Private driver data */

  FAR void *priv;
};

/****************************************************************************
 * cfg80211 API Functions
 ****************************************************************************/

#ifdef __cplusplus
extern "C"
{
#endif

/* WIPHY functions */

FAR struct wiphy *wiphy_new(FAR struct cfg80211_ops *ops, size_t sizeof_priv);
void wiphy_free(FAR struct wiphy *wiphy);
int wiphy_register(FAR struct wiphy *wiphy);
void wiphy_unregister(FAR struct wiphy *wiphy);
FAR void *wiphy_priv(FAR struct wiphy *wiphy);
FAR struct wiphy *priv_to_wiphy(FAR void *priv);

/* BSS functions */

FAR struct cfg80211_bss *cfg80211_get_bss(FAR struct wiphy *wiphy,
                                          FAR struct ieee80211_channel *channel,
                                          FAR const uint8_t *bssid,
                                          FAR const uint8_t *ssid,
                                          size_t ssid_len);
void cfg80211_put_bss(FAR struct wiphy *wiphy, FAR struct cfg80211_bss *bss);
void cfg80211_inform_bss(FAR struct wiphy *wiphy,
                         FAR struct ieee80211_channel *channel,
                         FAR const uint8_t *bssid,
                         uint64_t tsf, uint16_t capability,
                         uint16_t beacon_interval,
                         FAR const uint8_t *ie, size_t ies_len,
                         int signal, gfp_t gfp);
void cfg80211_scan_done(FAR struct cfg80211_scan_request *request, bool aborted);

/* Connection functions */

void cfg80211_connect_result(FAR struct net_device *netdev,
                              FAR const uint8_t *bssid,
                              FAR const uint8_t *req_ie, size_t req_ie_len,
                              FAR const uint8_t *resp_ie, size_t resp_ie_len,
                              uint16_t status, gfp_t gfp);
void cfg80211_disconnected(FAR struct net_device *netdev, uint16_t reason,
                           FAR const uint8_t *ie, size_t ie_len,
                           bool locally_generated, gfp_t gfp);
void cfg80211_roamed(FAR struct net_device *netdev,
                     FAR const uint8_t *bssid,
                     FAR const uint8_t *req_ie, size_t req_ie_len,
                     FAR const uint8_t *resp_ie, size_t resp_ie_len,
                     gfp_t gfp);

/* Station functions */

void cfg80211_new_sta(FAR struct net_device *netdev, FAR const uint8_t *mac_addr,
                      FAR struct station_info *sinfo, gfp_t gfp);
void cfg80211_del_sta(FAR struct net_device *netdev, FAR const uint8_t *mac_addr,
                      gfp_t gfp);
void cfg80211_sta_info(FAR struct net_device *netdev, FAR const uint8_t *mac,
                       uint16_t type, FAR const uint8_t *data, size_t len);

/* Channel functions */

void cfg80211_chandef_create(FAR struct wiphy *wiphy,
                              FAR struct cfg80211_chan_def *chandef,
                              FAR struct ieee80211_channel *channel,
                              enum nl80211_channel_width width);
bool cfg80211_chandef_valid(FAR const struct cfg80211_chan_def *chandef);
void cfg80211_set_wiphy_chan_default(FAR struct wiphy *wiphy,
                                      FAR struct ieee80211_channel *channel);

/* Regulatory functions */

void regulatory_hint(FAR const char *alpha2);
void regulatory_hint_11d(FAR const uint8_t *alpha2, FAR uint8_t *country_ie,
                        size_t country_ie_len);
void wiphy_apply_custom_regulatory(FAR struct wiphy *wiphy,
                                   FAR const struct ieee80211_regdomain *regd);

/* Remain on channel functions */

void cfg80211_ready_on_channel(FAR struct wireless_dev *wdev, uint64_t cookie,
                               FAR struct ieee80211_channel *channel,
                               unsigned int duration, gfp_t gfp);
void cfg80211_remain_on_channel_expired(FAR struct wireless_dev *wdev,
                                        uint64_t cookie,
                                        FAR struct ieee80211_channel *channel,
                                        gfp_t gfp);

/* MGMT TX functions */

void cfg80211_mgmt_tx_status(FAR struct wireless_dev *wdev, uint64_t cookie,
                              FAR const uint8_t *frame, size_t len,
                              bool ack, gfp_t gfp);

/* Station info update */

void cfg80211_sched_scan_stopped(FAR struct net_device *netdev, gfp_t gfp);

/* External auth */

void cfg80211_external_auth_result(FAR struct net_device *netdev,
                                   FAR struct cfg80211_external_auth_params *params,
                                   gfp_t gfp);

#ifdef __cplusplus
}
#endif

/****************************************************************************
 * Additional Structures for cfg80211
 ****************************************************************************/

/* struct cfg80211_chan_def - channel definition
 *
 * @chan: channel
 * @width: channel width
 * @center_freq1: center frequency 1
 * @center_freq2: center frequency 2 (for 80+80 MHz)
 */

struct cfg80211_chan_def
{
  FAR struct ieee80211_channel *chan;
  enum nl80211_channel_width width;
  uint16_t center_freq1;
  uint16_t center_freq2;
};

/* struct cfg80211_gtk_rekey_data - GTK rekey data
 *
 * @kek: key encryption key
 * @kek_len: length of KEK
 * @kck: key confirmation key
 * @kck_len: length of KCK
 * @replay_ctr: replay counter
 */

struct cfg80211_gtk_rekey_data
{
  uint8_t kek[NL80211_KEK_LEN];
  size_t kek_len;
  uint8_t kck[NL80211_KCK_LEN];
  size_t kck_len;
  uint8_t replay_ctr[NL80211_REPLAY_CTR_LEN];
};

/* GTK rekey constants */

#define NL80211_KEK_LEN                         16
#define NL80211_KCK_LEN                         16
#define NL80211_REPLAY_CTR_LEN                  8

/* struct cfg80211_auth_request - authentication request
 *
 * @bss: BSS to authenticate with
 * @auth_type: authentication type
 * @ie: IEs for authentication request
 * @ie_len: length of ie
 * @key_len: length of key
 * @key_idx: key index
 * @auth_data: authentication data (for SAE)
 * @auth_data_len: length of authentication data
 */

struct cfg80211_auth_request
{
  FAR struct cfg80211_bss *bss;
  uint32_t auth_type;
  FAR uint8_t *ie;
  size_t ie_len;
  FAR uint8_t *key;
  size_t key_len;
  uint8_t key_idx;
  FAR uint8_t *auth_data;
  size_t auth_data_len;
};

/* struct cfg80211_ibss_params - IBSS parameters
 *
 * @ssid: SSID
 * @ssid_len: length of SSID
 * @bssid: BSSID (may be NULL for random)
 * @channel: channel to use
 * @channel_width: channel width
 * @center_freq1: center frequency 1
 * @center_freq2: center frequency 2
 * @beacon_interval: beacon interval
 * @privacy: privacy-enabled network
 * @basic_rates: basic rates
 * @basic_rates_len: number of basic rates
 * @mcast_rate: multicast rate
 * @ht_capa: HT capabilities
 * @ht_capa_mask: HT capabilities mask
 * @wep_keys: WEP keys
 * @wep_key_idx: default WEP key index
 */

struct cfg80211_ibss_params
{
  struct cfg80211_ssid ssid;
  FAR uint8_t *bssid;
  FAR struct ieee80211_channel *channel;
  enum nl80211_channel_width channel_width;
  uint16_t center_freq1;
  uint16_t center_freq2;
  uint16_t beacon_interval;
  uint8_t privacy;
  FAR uint32_t *basic_rates;
  size_t basic_rates_len;
  int mcast_rate[IEEE80211_NUM_BANDS];
  FAR struct ieee80211_sta_ht_cap *ht_capa;
  FAR struct ieee80211_sta_ht_cap *ht_capa_mask;
  FAR struct key_params *wep_keys;
  uint8_t wep_key_idx;
};

/* struct cfg80211_mgmt_tx_params - MGMT TX parameters
 *
 * @chan: channel to transmit on
 * @offchan: transmit off channel
 * @wait: duration to wait for response
 * @buf: frame to transmit
 * @len: length of frame
 * @no_cck: don't send CCK frame
 * @dont_wait_for_ack: don't wait for ACK
 */

struct cfg80211_mgmt_tx_params
{
  FAR struct ieee80211_channel *chan;
  unsigned int wait;
  bool offchan;
  bool no_cck;
  bool dont_wait_for_ack;
  FAR uint8_t *buf;
  size_t len;
};

/* struct cfg80211_wowlan - WoWLAN configuration
 *
 * @any: wake up on any event
 * @disconnect: wake up on disconnect
 * @magic_pkt: wake up on magic packet
 * @patterns: wake up on pattern match
 * @n_patterns: number of patterns
 */

struct cfg80211_wowlan
{
  bool any;
  bool disconnect;
  bool magic_pkt;
  bool gtk_rekey_failure;
  bool eap_identity_req;
  bool four_way_handshake;
  bool rfkill_release;
  FAR struct cfg80211_pkt_pattern *patterns;
  int n_patterns;
  FAR struct cfg80211_wowlan_triggers *additional;
};

/* struct cfg80211_pkt_pattern - packet pattern
 *
 * @mask: pattern mask
 * @pattern: pattern bytes
 * @pattern_len: length of pattern
 * @pkt_offset: packet offset to start matching
 */

struct cfg80211_pkt_pattern
{
  FAR uint8_t *mask;
  FAR uint8_t *pattern;
  size_t pattern_len;
  size_t pkt_offset;
};

/* struct cfg80211_wowlan_triggers - WoWLAN triggers
 *
 * @n_patterns: number of patterns
 * @patterns: patterns
 * @tcp: TCP configuration
 * @tcp_syn: TCP SYN configuration
 */

struct cfg80211_wowlan_triggers
{
  int n_patterns;
  FAR struct cfg80211_pkt_pattern *patterns;
  FAR struct cfg80211_wowlan_tcp *tcp;
  FAR struct cfg80211_wowlan_tcp_syn *tcp_syn;
};

/* struct cfg80211_wowlan_tcp - WoWLAN TCP configuration
 *
 * @src: source IP address
 * @dst: destination IP address
 * @tos: Type of Service
 * @ip_proto: IP protocol
 * @src_port: source port
 * @dst_port: destination port
 * @payload: payload
 * @payload_len: payload length
 */

struct cfg80211_wowlan_tcp
{
  uint32_t src;
  uint32_t dst;
  uint8_t tos;
  uint8_t ip_proto;
  uint16_t src_port;
  uint16_t dst_port;
  FAR uint8_t *payload;
  uint16_t payload_len;
};

/* struct cfg80211_wowlan_tcp_syn - WoWLAN TCP SYN configuration
 *
 * @payload_seq: payload sequence number
 * @payload_len: payload length
 */

struct cfg80211_wowlan_tcp_syn
{
  uint32_t payload_seq;
  uint16_t payload_len;
};

/* struct cfg80211_sched_scan_request - scheduled scan request
 *
 * @wiphy: wiphy pointer
 * @netdev: network device
 * @scan_plans: scan plans
 * @n_scan_plans: number of scan plans
 * @flags: scan flags
 * @match_sets: match sets
 * @n_match_sets: number of match sets
 * @ie: IEs for probe request
 * @ie_len: length of ie
 * @channels: channels to scan
 * @n_channels: number of channels
 * @min_rssi_thold: minimum RSSI threshold
 * @max_rssi_thold: maximum RSSI threshold
 */

struct cfg80211_sched_scan_request
{
  struct wiphy *wiphy;
  struct net_device *netdev;
  FAR struct cfg80211_sched_scan_plan *scan_plans;
  size_t n_scan_plans;
  uint32_t flags;
  FAR struct cfg80211_match_set *match_sets;
  size_t n_match_sets;
  FAR uint8_t *ie;
  size_t ie_len;
  FAR struct ieee80211_channel **channels;
  size_t n_channels;
  int32_t min_rssi_thold;
  int32_t max_rssi_thold;
};

/* struct cfg80211_sched_scan_plan - scheduled scan plan
 *
 * @interval: scan interval
 * @iterations: number of iterations
 */

struct cfg80211_sched_scan_plan
{
  uint32_t interval;
  uint32_t iterations;
};

/* struct cfg80211_match_set - match set
 *
 * @ssid: SSID
 * @rssi_thold: RSSI threshold
 */

struct cfg80211_match_set
{
  struct cfg80211_ssid ssid;
  int32_t rssi_thold;
};

/* struct cfg80211_external_auth_params - external auth params
 *
 * @action: action to perform
 * @bssid: BSSID
 * @ssid: SSID
 * @key_mgmt: key management type
 * @status: status code
 * @pmkid: PMKID
 */

struct cfg80211_external_auth_params
{
  enum nl80211_external_auth_action action;
  uint8_t bssid[ETH_ALEN];
  struct cfg80211_ssid ssid;
  uint32_t key_mgmt;
  uint16_t status;
  uint8_t pmkid[16];
};

#endif /* __INCLUDE_NUTTX_WIRELESS_CFG80211_H */
