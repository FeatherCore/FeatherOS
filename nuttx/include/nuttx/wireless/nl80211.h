/****************************************************************************
 * include/nuttx/wireless/nl80211.h
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
 * nl80211 - 802.11 Netlink interface
 *
 * This header defines the nl80211 interface for wireless configuration.
 * Based on Linux kernel nl80211.h for compatibility.
 ****************************************************************************/

#ifndef __INCLUDE_NUTTX_WIRELESS_NL80211_H
#define __INCLUDE_NUTTX_WIRELESS_NL80211_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <nuttx/compiler.h>

#include <sys/socket.h>
#include <stdint.h>

#include <netpacket/netlink.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* nl80211 family ID
 * This is assigned by the Generic Netlink controller.
 * The actual value is dynamically assigned at runtime.
 */

#define NL80211_GENL_NAME                 "nl80211"
#define NL80211_GENL_VERSION              1

/* nl80211 multicast groups */

#define NL80211_MULTICAST_GROUP_WDEV      "wdev"
#define NL80211_MULTICAST_GROUP_MESH      "mesh"
#define NL80211_MULTICAST_GROUP_AUDIT     "audit"
#define NL80211_MULTICAST_GROUP_TESTMODE  "testmode"

#define NL80211_MCGRP_TESTMODE            0
#define NL80211_MCGRP_WDEV                1
#define NL80211_MCGRP_MESH                2
#define NL80211_MCGRP_AUDIT               3

/****************************************************************************
 * nl80211 Commands
 * These commands are used in struct genlmsghdr's cmd field.
 ****************************************************************************/

enum nl80211_commands
{
  NL80211_CMD_UNSPEC,

  /* Device configuration */

  NL80211_CMD_GET_WIPHY,
  NL80211_CMD_SET_WIPHY,
  NL80211_CMD_NEW_WIPHY,
  NL80211_CMD_DEL_WIPHY,

  NL80211_CMD_GET_INTERFACE,
  NL80211_CMD_SET_INTERFACE,
  NL80211_CMD_NEW_INTERFACE,
  NL80211_CMD_DEL_INTERFACE,

  NL80211_CMD_GET_KEY,
  NL80211_CMD_SET_KEY,
  NL80211_CMD_NEW_KEY,
  NL80211_CMD_DEL_KEY,

  NL80211_CMD_GET_BEACON,
  NL80211_CMD_SET_BEACON,
  NL80211_CMD_START_AP,
  NL80211_CMD_STOP_AP,

  NL80211_CMD_GET_STATION,
  NL80211_CMD_SET_STATION,
  NL80211_CMD_NEW_STATION,
  NL80211_CMD_DEL_STATION,

  NL80211_CMD_GET_MPATH,
  NL80211_CMD_SET_MPATH,
  NL80211_CMD_NEW_MPATH,
  NL80211_CMD_DEL_MPATH,

  NL80211_CMD_SET_BSS,
  NL80211_CMD_SET_REG,
  NL80211_CMD_REQ_SET_REG,

  NL80211_CMD_GET_MESH_CONFIG,
  NL80211_CMD_SET_MESH_CONFIG,

  NL80211_CMD_SET_MGMT_EXTRA_IE,
  NL80211_CMD_GET_REG,

  /* Scanning */

  NL80211_CMD_GET_SCAN,
  NL80211_CMD_TRIGGER_SCAN,
  NL80211_CMD_NEW_SCAN_RESULTS,
  NL80211_CMD_SCAN_ABORTED,

  /* Authentication/Association */

  NL80211_CMD_REG_CHANGE,
  NL80211_CMD_AUTHENTICATE,
  NL80211_CMD_ASSOCIATE,
  NL80211_CMD_DEAUTHENTICATE,
  NL80211_CMD_DISASSOCIATE,

  /* Security */

  NL80211_CMD_MICHAEL_MIC_FAILURE,

  /* IBSS */

  NL80211_CMD_REG_BEACON_HINT,
  NL80211_CMD_JOIN_IBSS,
  NL80211_CMD_LEAVE_IBSS,

  /* Test mode */

  NL80211_CMD_TESTMODE,

  /* Connection */

  NL80211_CMD_CONNECT,
  NL80211_CMD_ROAM,
  NL80211_CMD_DISCONNECT,

  /* WIPHY */

  NL80211_CMD_SET_WIPHY_NETNS,
  NL80211_CMD_GET_SURVEY,
  NL80211_CMD_NEW_SURVEY_RESULTS,

  /* PMKSA */

  NL80211_CMD_SET_PMKSA,
  NL80211_CMD_DEL_PMKSA,
  NL80211_CMD_FLUSH_PMKSA,

  /* Remain on channel */

  NL80211_CMD_REMAIN_ON_CHANNEL,
  NL80211_CMD_CANCEL_REMAIN_ON_CHANNEL,

  /* Rate control */

  NL80211_CMD_SET_TX_BITRATE_MASK,

  /* Frame registration */

  NL80211_CMD_REGISTER_FRAME,
  NL80211_CMD_FRAME,
  NL80211_CMD_FRAME_TX_STATUS,

  /* Power management */

  NL80211_CMD_SET_POWER_SAVE,
  NL80211_CMD_GET_POWER_SAVE,

  /* Connection quality */

  NL80211_CMD_SET_CQM,
  NL80211_CMD_NOTIFY_CQM,

  /* Channel */

  NL80211_CMD_SET_CHANNEL,

  /* WDS */

  NL80211_CMD_SET_WDS_PEER,

  /* Frame waiting */

  NL80211_CMD_FRAME_WAIT_CANCEL,

  /* Mesh */

  NL80211_CMD_JOIN_MESH,
  NL80211_CMD_LEAVE_MESH,

  /* Unexpected frame */

  NL80211_CMD_UNEXPECTED_FRAME,

  /* WoWLAN */

  NL80211_CMD_GET_WOWLAN,
  NL80211_CMD_SET_WOWLAN,

  /* Scheduled scan */

  NL80211_CMD_START_SCHED_SCAN,
  NL80211_CMD_STOP_SCHED_SCAN,
  NL80211_CMD_SCHED_SCAN_RESULTS,
  NL80211_CMD_SCHED_SCAN_STOPPED,

  /* Rekey offload */

  NL80211_CMD_SET_REKEY_OFFLOAD,

  /* PMKSA candidates */

  NL80211_CMD_PMKSA_CANDIDATE,

  /* TDLS */

  NL80211_CMD_TDLS_OPER,
  NL80211_CMD_TDLS_MGMT,
  NL80211_CMD_UNEXPECTED_4ADDR_FRAME,

  /* Probe client */

  NL80211_CMD_PROBE_CLIENT,

  /* Beacon handling */

  NL80211_CMD_REGISTER_BEACONS,
  NL80211_CMD_SET_NOACK_MAP,

  /* Channel switch */

  NL80211_CMD_CH_SWITCH_NOTIFY,
  NL80211_CMD_CH_SWITCH_STARTED_NOTIFY,

  /* Connection quality */

  NL80211_CMD_CQM_RSSI_NOTIFY,

  /* External auth */

  NL80211_CMD_EXTERNAL_AUTH,

  /* Stations */

  NL80211_CMD_STA_OPMODE_CHANGED,

  /* Control port */

  NL80211_CMD_CONTROL_PORT_FRAME,
  NL80211_CMD_CONTROL_PORT_FRAME_TX_STATUS,

  /* Management frame registration */

  NL80211_CMD_MANAGE_RFENCE,

  /* NAN */

  NL80211_CMD_NAN_FUNC_MATCH,
  NL80211_CMD_NAN_FUNC_CHANGE,

  /* Multicast to unicast */

  NL80211_CMD_SET_MULTICAST_TO_UNICAST,

  /* TXQ stats */

  NL80211_CMD_GET_TXQ_STATS,

  /* AC parameters */

  NL80211_CMD_SET_MAC_ACL,

  /* Radar detection */

  NL80211_CMD_RADAR_DETECT,

  /* Get power save */

  NL80211_CMD_GET_POWER_SAVE,

  /* FTM responder */

  NL80211_CMD_FTM_RESPOND,

  /* FTM session */

  NL80211_CMD_FTM_SESSION_START,
  NL80211_CMD_FTM_SESSION_END,
  NL80211_CMD_FTM_SESSION_COMPLETE,
  NL80211_CMD_FTM_CFGRESP,

  /* Statistics */

  NL80211_CMD_GET_STATION_INFO,
  NL80211_CMD_SET_BEACON_GENERATION,
  NL80211_CMD_SET_TID_CONFIG,
  NL80211_CMD_SET_PUNCTURE,
  NL80211_CMD_SET_SAR_SPECS,

  /* Unknown commands */

  NL80211_CMD_MAX
};

/****************************************************************************
 * nl80211 Attributes
 * These are used in struct nlattr's nla_type field.
 ****************************************************************************/

enum nl80211_attrs
{
  NL80211_ATTR_UNSPEC,

  /* MAC address */

  NL80211_ATTR_MAC,
  NL80211_ATTR_MAC_MASK,

  /* Key data */

  NL80211_ATTR_KEY_DATA,
  NL80211_ATTR_KEY_IDX,
  NL80211_ATTR_KEY_CIPHER,
  NL80211_ATTR_KEY_SEQ,
  NL80211_ATTR_KEY_TYPE,
  NL80211_ATTR_KEY_DEFAULT,
  NL80211_ATTR_KEY_SCOPING,

  /* Beacon */

  NL80211_ATTR_BEACON_INTERVAL,
  NL80211_ATTR_DTIM_PERIOD,
  NL80211_ATTR_BEACON_HEAD,
  NL80211_ATTR_BEACON_TAIL,

  /* Station */

  NL80211_ATTR_STA_AID,
  NL80211_ATTR_STA_FLAGS,
  NL80211_ATTR_STA_LISTEN_INTERVAL,
  NL80211_ATTR_STA_SUPPORTED_RATES,
  NL80211_ATTR_STA_VLAN,
  NL80211_ATTR_STA_INFO,
  NL80211_ATTR_WIPHY_BANDS,
  NL80211_ATTR_STA_PLINK_ACTION,
  NL80211_ATTR_STA_PLINK_STATE,

  /* WIPHY */

  NL80211_ATTR_WIPHY,
  NL80211_ATTR_WIPHY_NAME,
  NL80211_ATTR_WIPHY_TX_POWER_LEVEL,
  NL80211_ATTR_WIPHY_ANTENNA_TX,
  NL80211_ATTR_WIPHY_ANTENNA_RX,
  NL80211_ATTR_WIPHY_ANTENNA_AVAIL_TX,
  NL80211_ATTR_WIPHY_ANTENNA_AVAIL_RX,
  NL80211_ATTR_WIPHY_ANTENNA_GAIN,
  NL80211_ATTR_WIPHY_RTS_THRESHOLD,
  NL80211_ATTR_WIPHY_FRAG_THRESHOLD,
  NL80211_ATTR_WIPHY_RETRY_SHORT,
  NL80211_ATTR_WIPHY_RETRY_LONG,
  NL80211_ATTR_WIPHY_COVERAGE_CLASS,
  NL80211_ATTR_WIPHY_FREQ,
  NL80211_ATTR_WIPHY_CHANNEL_TYPE,
  NL80211_ATTR_WIPHY_SEC_RETRY_SHORT,
  NL80211_ATTR_WIPHY_SEC_RETRY_LONG,

  /* Interface */

  NL80211_ATTR_IFINDEX,
  NL80211_ATTR_IFNAME,
  NL80211_ATTR_IFTYPE,
  NL80211_ATTR_MAC_HINT,
  NL80211_ATTR_WIPHY_FREQ_HINT,
  NL80211_ATTR_MAC_CHANGED,
  NL80211_ATTR_WIPHY_CHANGED,

  /* Mesh */

  NL80211_ATTR_MPATH_NEXT_HOP,
  NL80211_ATTR_MPATH_INFO,
  NL80211_ATTR_MNTR_FLAGS,
  NL80211_ATTR_MESH_ID,
  NL80211_ATTR_MESH_CONFIG,

  /* BSS */

  NL80211_ATTR_BSS_CTS_PROT,
  NL80211_ATTR_BSS_SHORT_PREAMBLE,
  NL80211_ATTR_BSS_SHORT_SLOT_TIME,
  NL80211_ATTR_HT_CAPABILITY,
  NL80211_ATTR_BSS_BASIC_RATES,
  NL80211_ATTR_SUPPORTED_IFTYPES,
  NL80211_ATTR_REG_ALPHA2,
  NL80211_ATTR_REG_RULES,

  /* Scan */

  NL80211_ATTR_SCAN_FREQUENCIES,
  NL80211_ATTR_SCAN_SSIDS,
  NL80211_ATTR_GENERATION,
  NL80211_ATTR_BSS,

  /* Authentication */

  NL80211_ATTR_AUTH_TYPE,
  NL80211_ATTR_REASON_CODE,
  NL80211_ATTR_MAX_SCAN_IE_LEN,
  NL80211_ATTR_SCAN_GENERATION,
  NL80211_ATTR_MAX_MATCH_SETS,

  /* Security */

  NL80211_ATTR_CIPHER_SUITES,
  NL80211_ATTR_FREQ_BEFORE,
  NL80211_ATTR_FREQ_AFTER,
  NL80211_ATTR_CIPHER_SUITES_PAIRWISE,
  NL80211_ATTR_CIPHER_SUITE_GROUP,
  NL80211_ATTR_WPA_VERSIONS,
  NL80211_ATTR_AKM_SUITES,
  NL80211_ATTR_PMKID,
  NL80211_ATTR_PMK,
  NL80211_ATTR_PMKR0_NAME,
  NL80211_ATTR_PMKR0,
  NL80211_ATTR_PMKR1_NAME,
  NL80211_ATTR_PMKR1,

  /* Connection */

  NL80211_ATTR_DURATION,
  NL80211_ATTR_COOKIE,
  NL80211_ATTR_WIPHY_COVERAGE_CLASS,
  NL80211_ATTR_TX_RATES,
  NL80211_ATTR_FRAME,
  NL80211_ATTR_FRAME_MATCH,
  NL80211_ATTR_ACK,
  NL80211_ATTR_PS_STATE,
  NL80211_ATTR_CQM,
  NL80211_ATTR_LOCAL_STATE_CHANGE,
  NL80211_ATTR_AP_ISOLATE,
  NL80211_ATTR_WIPHY_TX_POWER_SETTING,
  NL80211_ATTR_TX_NO_CCK_RATE,

  /* SSID */

  NL80211_ATTR_SSID,
  NL80211_ATTR_SCAN_FLAGS,
  NL80211_ATTR_CONNECTION_MODE,
  NL80211_ATTR_CONTROL_PORT,
  NL80211_ATTR_CONTROL_PORT_ETHERTYPE,
  NL80211_ATTR_CONTROL_PORT_NO_ENCRYPT,
  NL80211_ATTR_CONTROL_PORT_OVER_NL80211,
  NL80211_ATTR_CONTROL_PORT_NO_PREAUTH,
  NL80211_ATTR_CONTROL_PORT_REQUIRE_GTK_2ND,
  NL80211_ATTR_CONTROL_PORT_ENC_KEY_IDX,
  NL80211_ATTR_CONTROL_PORT_REQUIRE_4ADDR,

  /* Status */

  NL80211_ATTR_STATUS_CODE,
  NL80211_ATTR_CIPHER,
  NL80211_ATTR_MGMT_PROTOCOL,
  NL80211_ATTR_MAX_NUM_PMKIDS,
  NL80211_ATTR_PREV_BSSID,
  NL80211_ATTR_KEY,
  NL80211_ATTR_KEYS,
  NL80211_ATTR_FEATURE_FLAGS,

  /* Survey */

  NL80211_ATTR_MAX_NUM_SCAN_SSIDS,
  NL80211_ATTR_MAX_SCAN_IE_LEN,
  NL80211_ATTR_MAX_SCHED_SCAN_IE_LEN,
  NL80211_ATTR_MAX_MATCH_SETS,
  NL80211_ATTR_MAX_REKEY_DATA,

  /* Regulatory */

  NL80211_ATTR_REG_INITIATOR,
  NL80211_ATTR_REG_TYPE,
  NL80211_ATTR_SUPPORTED_COMMANDS,

  /* Additional IE */

  NL80211_ATTR_IE,
  NL80211_ATTR_IE_RSN,
  NL80211_ATTR_IE_HT_CAP,
  NL80211_ATTR_IE_HT_OPERATION,
  NL80211_ATTR_IE_VHT_CAP,
  NL80211_ATTR_IE_VHT_OPERATION,

  /* Mesh ID */

  NL80211_ATTR_MESH_VENDOR_SPECIFIC_IEs,

  /* Vendor specific */

  NL80211_ATTR_VENDOR_ID,
  NL80211_ATTR_VENDOR_SUBCMD,
  NL80211_ATTR_VENDOR_DATA,
  NL80211_ATTR_VENDOR_EVENTS,

  /* TDLS */

  NL80211_ATTR_TDLS_ACTION,
  NL80211_ATTR_TDLS_DIALOG_TOKEN,
  NL80211_ATTR_TDLS_OPERATION_CODE,
  NL80211_ATTR_TDLS_SUPPORT,
  NL80211_ATTR_TDLS_EXTERNAL_SETUP,

  /* WoWLAN */

  NL80211_ATTR_WOWLAN_TRIGGERS,
  NL80211_ATTR_WOWLAN_TRIGGERS_SUPPORTED,
  NL80211_ATTR_WOWLAN_VTRIGGERS,

  /* Scheduled scan */

  NL80211_ATTR_SCHED_SCAN_INTERVAL,
  NL80211_ATTR_SCHED_SCAN_MATCH,
  NL80211_ATTR_SCHED_SCAN_MATCH_SET,
  NL80211_ATTR_SCHED_SCAN_MAX_DELAY,

  /* Probe req */

  NL80211_ATTR_PROBE_RESP_OFFLOAD,

  /* Scan width */

  NL80211_ATTR_SCAN_WIDTH,
  NL80211_ATTR_CH_SWITCH_COUNT,
  NL80211_ATTR_CH_SWITCH_BANDWIDTH,
  NL80211_ATTR_WIPHY_SELF_MANAGED_REG,
  NL80211_ATTR_EXT_CAPA,
  NL80211_ATTR_EXT_CAPA_MASK,
  NL80211_ATTR_STA_CAPABILITY,
  NL80211_ATTR_STA_EXT_CAPABILITY,
  NL80211_ATTR_PROTOCOL_FEATURES,
  NL80211_ATTR_SPLIT_WIPHY_DUMP,
  NL80211_ATTR_DISABLE_VHT,
  NL80211_ATTR_AMPDU_FACTOR,
  NL80211_ATTR_AMPDU_DENSITY,
  NL80211_ATTR_MAX_AMSDU_SUBFRAMES,

  /* Mesh peering */

  NL80211_ATTR_STA_PLINK_STATE,

  /* TDLS peer STA */

  NL80211_ATTR_TDLS_PEER_CAPABILITY,

  /* Non-inheritance */

  NL80211_ATTR_TSID,
  NL80211_ATTR_USER_PRIO,
  NL80211_ATTR_ADMITTED_TIME,

  /* FTM */

  NL80211_ATTR_FTM_RESPONDER,
  NL80211_ATTR_FTM_SESSION,
  NL80211_ATTR_FTM_CFGRESP,
  NL80211_ATTR_FTM_CFGRESP_SP,
  NL80211_ATTR_FTM_CFGRESP_ASAP,
  NL80211_ATTR_FTM_CFGRESP_BSSID,
  NL80211_ATTR_FTM_CFGRESP_LCI,
  NL80211_ATTR_FTM_CFGRESP_CIVICLOC,

  /* Unknown */

  NL80211_ATTR_MAX
};

/* Number of attributes */

#define NL80211_ATTR_MAX                  (NL80211_ATTR_MAX - 1)

/****************************************************************************
 * nl80211 Interface Types
 ****************************************************************************/

enum nl80211_iftype
{
  NL80211_IFTYPE_UNSPECIFIED,
  NL80211_IFTYPE_ADHOC,
  NL80211_IFTYPE_STATION,
  NL80211_IFTYPE_AP,
  NL80211_IFTYPE_AP_VLAN,
  NL80211_IFTYPE_WDS,
  NL80211_IFTYPE_MONITOR,
  NL80211_IFTYPE_MESH_POINT,
  NL80211_IFTYPE_P2P_CLIENT,
  NL80211_IFTYPE_P2P_GO,
  NL80211_IFTYPE_P2P_DEVICE,
  NL80211_IFTYPE_NAN,
  NUM_NL80211_IFTYPES,
  NL80211_IFTYPE_MAX = NUM_NL80211_IFTYPES - 1
};

/****************************************************************************
 * nl80211 Authentication Types
 ****************************************************************************/

enum nl80211_auth_type
{
  NL80211_AUTHTYPE_OPEN_SYSTEM,
  NL80211_AUTHTYPE_SHARED_KEY,
  NL80211_AUTHTYPE_FT,
  NL80211_AUTHTYPE_NETWORK_EAP,
  NL80211_AUTHTYPE_SAE,
  NL80211_AUTHTYPE_FILS_SK,
  NL80211_AUTHTYPE_FILS_SK_PFS,
  NL80211_AUTHTYPE_FILS_PK,
  __NL80211_AUTHTYPE_NUM,
  NL80211_AUTHTYPE_MAX = __NL80211_AUTHTYPE_NUM - 1,
  NL80211_AUTHTYPE_AUTOMATIC
};

/****************************************************************************
 * nl80211 Connection Status
 ****************************************************************************/

enum nl80211_conn_status
{
  NL80211_CONN_STATUS_MAX,
  NL80211_CONN_STATUS_INVALID = 0xffff
};

/****************************************************************************
 * nl80211 Cipher Suites
 ****************************************************************************/

#define NL80211_CIPHER_SUITE_WEP40        0x000FAC01
#define NL80211_CIPHER_SUITE_WEP104       0x000FAC05
#define NL80211_CIPHER_SUITE_TKIP         0x000FAC02
#define NL80211_CIPHER_SUITE_CCMP         0x000FAC04
#define NL80211_CIPHER_SUITE_AES_CMAC     0x000FAC06
#define NL80211_CIPHER_SUITE_GCMP         0x000FAC08
#define NL80211_CIPHER_SUITE_GCMP_256     0x000FAC09
#define NL80211_CIPHER_SUITE_CCMP_256     0x000FAC0A
#define NL80211_CIPHER_SUITE_BIP_GMAC_128 0x000FAC0B
#define NL80211_CIPHER_SUITE_BIP_GMAC_256 0x000FAC0C
#define NL80211_CIPHER_SUITE_BIP_CMAC_256 0x000FAC0D

/****************************************************************************
 * nl80211 AKM Suites
 ****************************************************************************/

#define NL80211_AKM_SUITE_8021X           0x000FAC01
#define NL80211_AKM_SUITE_PSK              0x000FAC02
#define NL80211_AKM_SUITE_SAE              0x000FAC08
#define NL80211_AKM_SUITE_8021X_SHA256     0x000FAC05
#define NL80211_AKM_SUITE_PSK_SHA256       0x000FAC06
#define NL80211_AKM_SUITE_TDLS             0x000FAC09
#define NL80211_AKM_SUITE_SAE_SHA256       0x000FAC0A
#define NL80211_AKM_SUITE_FILS_SHA256      0x000FAC0D
#define NL80211_AKM_SUITE_FILS_SHA384      0x000FAC0E
#define NL80211_AKM_SUITE_OWE              0x000FAC12

/****************************************************************************
 * nl80211 TX Power Setting Types
 ****************************************************************************/

enum nl80211_tx_power_setting
{
  NL80211_TX_POWER_AUTOMATIC,
  NL80211_TX_POWER_FIXED,
  NL80211_TX_POWER_LIMITED,
  NL80211_TX_POWER_VHT_CHANGED,
  NL80211_TX_POWER_FIXED_VHT,
  NL80211_TX_POWER_LIMITED_VHT
};

/****************************************************************************
 * nl80211 Channel Width
 ****************************************************************************/

enum nl80211_channel_width
{
  NL80211_CHAN_WIDTH_20_NOHT,
  NL80211_CHAN_WIDTH_20,
  NL80211_CHAN_WIDTH_40,
  NL80211_CHAN_WIDTH_80,
  NL80211_CHAN_WIDTH_80P80,
  NL80211_CHAN_WIDTH_160,
  NL80211_CHAN_WIDTH_5,
  NL80211_CHAN_WIDTH_10,
  NL80211_CHAN_WIDTH_1,
  NL80211_CHAN_WIDTH_2,
  NL80211_CHAN_WIDTH_4,
  NL80211_CHAN_WIDTH_8,
  NL80211_CHAN_WIDTH_16,
  NL80211_CHAN_WIDTH_NOHT,
  NL80211_CHAN_WIDTH_20HT,
  NL80211_CHAN_WIDTH_20U,
  NL80211_CHAN_WIDTH_40MINUS,
  NL80211_CHAN_WIDTH_40PLUS,
  NL80211_CHAN_WIDTH_80MINUS,
  NL80211_CHAN_WIDTH_80PLUS,
  NL80211_CHAN_WIDTH_160MINUS,
  NL80211_CHAN_WIDTH_160PLUS,
  NL80211_CHAN_WIDTH_80P80MINUS,
  NL80211_CHAN_WIDTH_80P80PLUS,
  NL80211_CHAN_WIDTH_160MINUS_80PLUS,
  NL80211_CHAN_WIDTH_160PLUS_80MINUS,
  NL80211_CHAN_WIDTH_INVALID
};

/****************************************************************************
 * nl80211 Mesh Power Level
 ****************************************************************************/

enum nl80211_mesh_power_level
{
  NL80211_MESH_POWER_UNKNOWN,
  NL80211_MESH_POWER_MIN,
  NL80211_MESH_POWER_MAX,
  NL80211_MESH_POWER_ACTIVE,
  NL80211_MESH_POWER_LEVELED
};

/****************************************************************************
 * nl80211 TDLS Operations
 ****************************************************************************/

enum nl80211_tdls_operation
{
  NL80211_TDLS_DISCOVERY_REQ,
  NL80211_TDLS_DISCOVERY_RESP,
  NL80211_TDLS_SETUP,
  NL80211_TDLS_TEARDOWN,
  NL80211_TDLS_ENABLE_LINK,
  NL80211_TDLS_DISABLE_LINK,
  NL80211_TDLS_ENABLE,
  NL80211_TDLS_DISABLE
};

/****************************************************************************
 * nl80211 Mesh Plink Actions
 ****************************************************************************/

enum nl80211_plink_action
{
  NL80211_PLINK_ACTION_NO_ACTION,
  NL80211_PLINK_ACTION_OPEN,
  NL80211_PLINK_ACTION_BLOCK,
  NL80211_PLINK_ACTION_AUTH,
  NL80211_PLINK_ACTION_ESTAB,
  NL80211_PLINK_ACTION_CONFIRM,
  NL80211_PLINK_ACTION_HOLDING,
  NL80211_PLINK_ACTION_DECLINE,
  NL80211_PLINK_ACTION_CANCEL,
  _NL80211_PLINK_ACTION_MAX
};

/****************************************************************************
 * nl80211 Mesh Plink States
 ****************************************************************************/

enum nl80211_plink_state
{
  NL80211_PLID_LISTEN,
  NL80211_PLID_OPEN_SENT,
  NL80211_PLID_OPEN_RECV,
  NL80211_PLID_CONFIRM_SENT,
  NL80211_PLID_ESTAB,
  NL80211_PLID_HOLDING_SENT,
  NL80211_PLID_HOLDING_RECV,
  NL80211_PLID_HOLDING,
  _NL80211_PLINK_STATE_MAX
};

/****************************************************************************
 * nl80211 Scan Flags
 ****************************************************************************/

#define NL80211_SCAN_FLAG_LOW_PRIORITY            (1 << 0)
#define NL80211_SCAN_FLAG_FLUSH                   (1 << 1)
#define NL80211_SCAN_FLAG_AP                      (1 << 2)
#define NL80211_SCAN_FLAG_RANDOM_SN               (1 << 3)
#define NL80211_SCAN_FLAG_FILS_MAX_CHANNEL_TIME   (1 << 4)
#define NL80211_SCAN_FLAG_ACCEPT_BCAST_PROBE_RESP (1 << 5)
#define NL80211_SCAN_FLAG_OCE_PROBE_REQ_DEFERRAL_SUPPRESSION (1 << 6)
#define NL80211_SCAN_FLAG_OCE_PROBE_RESP_DEFERRAL_SUPPRESSION (1 << 7)
#define NL80211_SCAN_FLAG_LOW_SPAN                (1 << 8)
#define NL80211_SCAN_FLAG_LOW_POWER               (1 << 9)
#define NL80211_SCAN_FLAG_FAST_SCAN               (1 << 10)

/****************************************************************************
 * nl80211 Station Flags
 ****************************************************************************/

#define NL80211_STA_FLAG_AUTHORIZED               (1 << 0)
#define NL80211_STA_FLAG_SHORT_PREAMBLE           (1 << 1)
#define NL80211_STA_FLAG_WME                      (1 << 2)
#define NL80211_STA_FLAG_MFP                      (1 << 3)
#define NL80211_STA_FLAG_AUTHENTICATED            (1 << 4)
#define NL80211_STA_FLAG_TDLS_PEER               (1 << 5)
#define NL80211_STA_FLAG_ASSOCIATED               (1 << 6)
#define NL80211_STA_FLAG_VLAN                     (1 << 7)
#define NL80211_STA_FLAG_NONZERO_PMKR0_NAME      (1 << 8)

/****************************************************************************
 * nl80211 Station Info
 ****************************************************************************/

enum nl80211_sta_info
{
  NL80211_STA_INFO_UNSPEC,
  NL80211_STA_INFO_CONNECTED_TIME,
  NL80211_STA_INFO_STAFLAGS,
  NL80211_STA_INFO_BEACON_LOSS,
  NL80211_STA_INFO_T_OFFSET,
  NL80211_STA_INFO_LOCAL_PM,
  NL80211_STA_INFO_PEER_PM,
  NL80211_STA_INFO_NONPEER_PM,
  NL80211_STA_INFO_RX_BYTES,
  NL80211_STA_INFO_TX_BYTES,
  NL80211_STA_INFO_RX_BYTES64,
  NL80211_STA_INFO_TX_BYTES64,
  NL80211_STA_INFO_RX_PACKETS,
  NL80211_STA_INFO_TX_PACKETS,
  NL80211_STA_INFO_RX_DROP_MISC,
  NL80211_STA_INFO_TX_FAILED,
  NL80211_STA_INFO_BEACON_AVG,
  NL80211_STA_INFO_RX_RATE,
  NL80211_STA_INFO_TX_RATE,
  NL80211_STA_INFO_SIGNAL,
  NL80211_STA_INFO_SIGNAL_AVG,
  NL80211_STA_INFO_T_OFFSET,
  NL80211_STA_INFO_TX_BITRATE,
  NL80211_STA_INFO_TX_BITRATE_AVG,
  NL80211_STA_INFO_RX_BITRATE,
  NL80211_STA_INFO_RX_BITRATE_AVG,
  NL80211_STA_INFO_BSS_PARAM,
  NL80211_STA_INFO_CHAIN_SIGNAL,
  NL80211_STA_INFO_CHAIN_SIGNAL_AVG,
  NL80211_STA_INFO_TX_DURATION,
  NL80211_STA_INFO_AIRTIME_WEIGHT,
  NL80211_STA_INFO_AIRTIME,
  NL80211_STA_INFO_MAX
};

#define NL80211_STA_INFO_MAX                      (NL80211_STA_INFO_MAX - 1)

/****************************************************************************
 * nl80211 Survey Info
 ****************************************************************************/

enum nl80211_survey_info
{
  NL80211_SURVEY_INFO_UNSPEC,
  NL80211_SURVEY_INFO_FREQUENCY,
  NL80211_SURVEY_INFO_NOISE,
  NL80211_SURVEY_INFO_IN_USE,
  NL80211_SURVEY_INFO_TIME,
  NL80211_SURVEY_INFO_TIME_BUSY,
  NL80211_SURVEY_INFO_TIME_EXT_BUSY,
  NL80211_SURVEY_INFO_TIME_RX,
  NL80211_SURVEY_INFO_TIME_TX,
  NL80211_SURVEY_INFO_TIME_SCAN,
  NL80211_SURVEY_INFO_TIME_BSS_RX,
  NL80211_SURVEY_INFO_MAX
};

#define NL80211_SURVEY_INFO_MAX                   (NL80211_SURVEY_INFO_MAX - 1)

/****************************************************************************
 * nl80211 BSS Parameters
 ****************************************************************************/

enum nl80211_bss
{
  NL80211_BSS_UNSPEC,
  NL80211_BSS_BSSID,
  NL80211_BSS_FREQUENCY,
  NL80211_BSS_TSF,
  NL80211_BSS_BEACON_INTERVAL,
  NL80211_BSS_CAPABILITY,
  NL80211_BSS_INFORMATION_ELEMENTS,
  NL80211_BSS_SIGNAL_MBM,
  NL80211_BSS_SIGNAL_UNSPEC,
  NL80211_BSS_STATUS,
  NL80211_BSS_SEEN_MS_AGO,
  NL80211_BSS_BEACON_IES,
  NL80211_BSS_CHAIN_SIGNAL,
  NL80211_BSS_CHAIN_SIGNAL_AVG,
  NL80211_BSS_MODE,
  NL80211_BSS_MAX
};

#define NL80211_BSS_MAX                          (NL80211_BSS_MAX - 1)

/****************************************************************************
 * nl80211 BSS Status
 ****************************************************************************/

enum nl80211_bss_status
{
  NL80211_BSS_STATUS_AUTHENTICATED,
  NL80211_BSS_STATUS_ASSOCIATED,
  NL80211_BSS_STATUS_IBSS_JOINED,
  NL80211_BSS_STATUS_FOLLOW_AP,
  NL80211_BSS_STATUS_FOLLOW_STA,
  NL80211_BSS_STATUS_MAX
};

/****************************************************************************
 * nl80211 Key Attributes
 ****************************************************************************/

enum nl80211_key_attributes
{
  NL80211_KEY_UNSPEC,
  NL80211_KEY_DATA,
  NL80211_KEY_IDX,
  NL80211_KEY_CIPHER,
  NL80211_KEY_DEFAULT,
  NL80211_KEY_SEQ,
  NL80211_KEY_TYPE,
  NL80211_KEY_DEFAULT_MGMT,
  NL80211_KEY_DEFAULT_TYPE,
  NL80211_KEY_MAX
};

#define NL80211_KEY_MAX                          (NL80211_KEY_MAX - 1)

/****************************************************************************
 * nl80211 Key Types
 ****************************************************************************/

enum nl80211_key_type
{
  NL80211_KEYTYPE_GROUP,
  NL80211_KEYTYPE_PAIRWISE,
  NL80211_KEYTYPE_PEERKEY,
  NL80211_KEYTYPE_NO_WEP,
  NL80211_KEYTYPE_DEFAULT,
  NL80211_KEYTYPE_MAX
};

/****************************************************************************
 * nl80211 Power Save State
 ****************************************************************************/

enum nl80211_ps_state
{
  NL80211_PS_ENABLED,
  NL80211_PS_DISABLED
};

/****************************************************************************
 * nl80211 TDLS External Setup
 ****************************************************************************/

enum nl80211_tdls_external_setup
{
  NL80211_TDLS_UNKNOWN,
  NL80211_TDLS_EXTERNAL_SETUP_BEGIN,
  NL80211_TDLS_EXTERNAL_SETUP_END
};

/****************************************************************************
 * nl80211 Regdom Initiators
 ****************************************************************************/

enum nl80211_reg_initiator
{
  NL80211_REGDOM_SET_BY_CORE,
  NL80211_REGDOM_SET_BY_USER,
  NL80211_REGDOM_SET_BY_DRIVER,
  NL80211_REGDOM_SET_BY_COUNTRY_IE,
  NL80211_REGDOM_SET_BY_USER_LINKSAFE,
  NL80211_REGDOM_SET_BY_USER_SIGMETRIC,
  NL80211_REGDOM_SET_BY_USER_GNSS,
  NL80211_REGDOM_SET_BY_USER_CNTRACKER,
  NL80211_REGDOM_MAX
};

/****************************************************************************
 * nl80211 Regdom Types
 ****************************************************************************/

enum nl80211_regdom_type
{
  NL80211_REGDOM_TYPE_CORE,
  NL80211_REGDOM_TYPE_STRICT_REG,
  NL80211_REGDOM_TYPE_INTERSECT,
  NL80211_REGDOM_TYPE_CUSTOM,
  NL80211_REGDOM_TYPE_DEFAULT,
  NL80211_REGDOM_TYPE_MAX
};

/****************************************************************************
 * nl80211 User Regdom Hint Types
 ****************************************************************************/

enum nl80211_user_reg_hint_type
{
  NL80211_USER_REG_HINT_USER,
  NL80211_USER_REG_HINT_CNTRACKER,
  NL80211_USER_REG_HINT_GNSS,
  NL80211_USER_REG_HINT_DEFAULT
};

/****************************************************************************
 * nl80211 Mow Mode Flags
 ****************************************************************************/

#define NL80211_MESHCONF_MAX_PAIRING_TIMEOUT    60000

#define NL80211_MESHCONF_MAX_RETRIES           16

#define NL80211_MESHCONF_RETRY_TIMEOUT         10000

#define NL80211_MESHCONF_CONFIRM_TIMEOUT       10000

#define NL80211_MESHCONF_HOLDING_TIMEOUT       10000

#define NL80211_MESHCONF_MAX_AWAKE_WINDOW      5000

#define NL80211_MESHCONF_AWAKE_WINDOW_BASIC_RATES   0

#define NL80211_MESHCONF_BEACON_INTERVAL         1

#define NL80211_MESHCONF_DTIM_PERIOD             2

#define NL80211_MESHCONF_HT_OPMODE               3

#define NL80211_MESHCONF_GATE_ANNOUNCEMENTS       4

#define NL80211_MESHCONF_FORWARDING               5

#define NL80211_MESHCONF_RSSI_THRESHOLD           6

#define NL80211_MESHCONF_SYNC_OFFSET_MAX_NEIGHBOR 7

#define NL80211_MESHCONF_HT_SM_POWER_MODE         8

#define NL80211_MESHCONF_TTL                     9

#define NL80211_MESHCONF_AUTO_OPEN_PLINKS        10

#define NL80211_MESHCONF_HWMP_MAX_PREQ_RETRIES   11

#define NL80211_MESHCONF_PATH_REFRESH_TIME       12

#define NL80211_MESHCONF_MIN_DISCOVERY_TIMEOUT   13

#define NL80211_MESHCONF_HWMP_ACTIVE_PATH_TIMEOUT 14

#define NL80211_MESHCONF_HWMP_PREQ_MIN_INTERVAL  15

#define NL80211_MESHCONF_HWMP_PERR_MIN_INTERVAL  16

#define NL80211_MESHCONF_HWMP_NET_DIAM_TRVS_TIME 17

#define NL80211_MESHCONF_HWMP_ROOTMODE           18

#define NL80211_MESHCONF_HWMP_RANN_INTERVAL      19

#define NL80211_MESHCONF_GATE_MODE               20

#define NL80211_MESHCONF_PLINK_TIMEOUT           21

#define NL80211_MESHCONF_CONNECTED_TO_GATE       22

#define NL80211_MESHCONF_NOLEAF_PEER             23

#define NL80211_MESHCONF_MAX                      NL80211_MESHCONF_CONNECTED_TO_GATE

/****************************************************************************
 * nl80211 Reason Codes
 * 802.11 Reason codes (IEEE Std 802.11-2016, 9.4.1.7, Table 9-45)
 ****************************************************************************/

enum nl80211_reason_code
{
  NL80211_REASON_UNSPECIFIED                     = 1,
  NL80211_REASON_PREV_AUTH_NOT_VALID             = 2,
  NL80211_REASON_DEAUTH_LEAVING                 = 3,
  NL80211_REASON_DISASSOC_DUE_TO_INACTIVITY     = 4,
  NL80211_REASON_DISASSOC_AP_BUSY               = 5,
  NL80211_REASON_CLASS2_FRAME_FROM_NONAUTH_STA  = 6,
  NL80211_REASON_CLASS3_FRAME_FROM_NONASSOC_STA = 7,
  NL80211_REASON_DISASSOC_STA_HAS_LEFT          = 8,
  NL80211_REASON_ASSOC_REQ_NOT_AUTH             = 9,
  NL80211_REASON_POWER_CAPABILITY_NOT_VALID     = 10,
  NL80211_REASON_SUPPORTED_CHANNELS_NOT_VALID   = 11,
  NL80211_REASON_BSS_TRANSITION_DISASSOC        = 12,
  NL80211_REASON_INVALID_IE                     = 13,
  NL80211_REASON_MIC_FAILURE                    = 14,
  NL80211_REASON_4WAY_HANDSHAKE_TIMEOUT         = 15,
  NL80211_REASON_GROUP_KEY_UPDATE_TIMEOUT       = 16,
  NL80211_REASON_IE_IN_4WAY_DIFFERS            = 17,
  NL80211_REASON_MULTICAST_CIPHER_NOT_VALID    = 18,
  NL80211_REASON_UNICAST_CIPHER_NOT_VALID     = 19,
  NL80211_REASON_AKMP_NOT_VALID                = 20,
  NL80211_REASON_UNSUPPORTED_RSNE_VER          = 21,
  NL80211_REASON_RSN_CAP_MISMATCH              = 22,
  NL80211_REASON_CIPHER_REJ_POLICY             = 23,
  NL80211_REASON_TS_NOT_ACCEPTED               = 24,
  NL80211_REASON_DIRECT_LINK_NOT_ALLOWED       = 25,
  NL80211_REASON_STA_NOT_PRESENT               = 26,
  NL80211_REASON_STA_NOT_QOS_STA               = 27,
  NL80211_REASON_ASSOC_BLOCKED_BY_RANGE         = 28,
  NL80211_REASON_ASSOC_NOT_AUTH_STA            = 29,
  NL80211_REASON_POWER_CAPABILITY_MISMATCH     = 30,
  NL80211_REASON_SUPPORTED_CHANNELS_MISMATCH   = 31,
  NL80211_REASON_ASSOC_REQ_EXCEEDS_LIMIT      = 32,
  NL80211_REASON_STA_NOT_VHT_STA               = 33,
  NL80211_REASON_MDIE_NOT_VALID                = 34,
  NL80211_REASON_FAST_BSS_TRANSITION_NOT_SUPPORTED = 35,
  NL80211_REASON_PEER_STA_NOT_SUPPORTED        = 36,
  NL80211_REASON_REFUSED_EXTERNAL_REASON       = 37,
  NL80211_REASON_REFUSED_AP_OUT_OF_RANGE       = 38,
  NL80211_REASON_REFUSED_AP_BUSY               = 39,
  NL80211_REASON_GROUPCIPHER_NOT_VALID         = 40,
  NL80211_REASON_PAIRWISE_CIPHER_NOT_VALID     = 41,
  NL80211_REASON_AKMP_NOT_VALID               = 42,
  NL80211_REASON_UNSUPPORTED_RSNNE_VERSION    = 43,
  NL80211_REASON_RSN_CAP_MISMATCH             = 44,
  NL80211_REASON_CIPHER_REJ_POLICY             = 45,
  NL80211_REASON_DISASSOC_PEER_STA_TIMEOUT    = 46,
  NL80211_REASON_INVALID_PMKID                = 49,
  NL80211_REASON_MESH_PEER_MAX                = 60,
  NL80211_REASON_MESH_CONFIRM_TIMEOUT         = 61,
  NL80211_REASON_MESH_MAX_PEERS               = 62,
  NL80211_REASON_MESH_CANCELLED               = 63,
  NL80211_REASON_MESH_LEAVING                = 64,
  NL80211_REASON_MESH_INVALID_ACTION          = 65,
  NL80211_REASON_MESH_INVALID_GTK             = 66,
  NL80211_REASON_MESH_INCONSISTENT_PARAM      = 67,
  NL80211_REASON_MESH_INVALID_SECURITY_CAP    = 68,
  NL80211_REASON_MESH_PATHERR_NOROUTE         = 69,
  NL80211_REASON_MESH_PATH_DEST_UNREACHABLE  = 70,
  NL80211_REASON_MAC_EXISTS_IN_MBSS          = 71,
  NL80211_REASON_MESH_CHANNEL_SWITCH_REGULATORY = 72,
  NL80211_REASON_MESH_CHANNEL_SWITCH_UNSPEC   = 73,
};

/****************************************************************************
 * nl80211 Status Codes
 * 802.11 Status codes (IEEE Std 802.11-2016, 9.4.1.9, Table 9-46)
 ****************************************************************************/

enum nl80211_status_code
{
  NL80211_STATUS_SUCCESS                       = 0,
  NL80211_STATUS_UNSPECIFIED_FAILURE          = 1,
  NL80211_STATUS_TDLS_WAKEUP_ALTERNATE        = 2,
  NL80211_STATUS_TDLS_WAKEUP_UNSPECIFIED      = 3,
  NL80211_STATUS_SAE_HASH_TO_ELEMENT         = 4,
  NL80211_STATUS_SAE_PK                      = 5,
  NL80211_STATUS_UNSUPPORTED_FREQ_BAND       = 12,
  NL80211_STATUS_INVALID_PMKID                = 53,
  NL80211_STATUS_INVALID_IE                   = 54,
  NL80211_STATUS_MESH_CONFIGURATION_UNSUPPORTED = 55,
  NL80211_STATUS_MESH_TOPOLOGY_UNSUPPORTED   = 56,
  NL80211_STATUS_MESH_MAX_PEERS_EXCEEDED     = 57,
  NL80211_STATUS_MESH_NO_SELF_CANDIDATE      = 58,
  NL80211_STATUS_MESH_CONFIRM_TIMEOUT        = 59,
  NL80211_STATUS_MESH_INVALID_GTK             = 60,
  NL80211_STATUS_MESH_INVALID_SECURITY_CAP   = 61,
  NL80211_STATUS_MESH_PATH_NOROUTE           = 62,
  NL80211_STATUS_MESH_PATH_DEST_UNREACHABLE  = 63,
  NL80211_STATUS_MESH_MAC_EXISTS_IN_MBSS     = 64,
  NL80211_STATUS_MESH_CHANNEL_SWITCH         = 65,
  NL80211_STATUS_MESH_CHANNEL_SWITCH_REGULATORY = 66,
  NL80211_STATUS_MESH_CHANNEL_SWITCH_UNSPEC  = 67,
};

/****************************************************************************
 * nl80211 Band Capabilities
 ****************************************************************************/

#define NL80211_BAND_ATTR_FREQS    1
#define NL80211_BAND_ATTR_RATES    2
#define NL80211_BAND_ATTR_HT_MCS_SET     3
#define NL80211_BAND_ATTR_HT_CAPA         4
#define NL80211_BAND_ATTR_HT_AMPDU_FACTOR  5
#define NL80211_BAND_ATTR_HT_AMPDU_DENSITY 6
#define NL80211_BAND_ATTR_VHT_MCS_SET     7
#define NL80211_BAND_ATTR_VHT_CAPA         8
#define NL80211_BAND_ATTR_IFTYPE_DATA      9
#define NL80211_BAND_ATTR_MAX              NL80211_BAND_ATTR_IFTYPE_DATA

/****************************************************************************
 * nl80211 Feature Flags
 ****************************************************************************/

#define NL80211_FEATURE_SAE                     (1 << 0)
#define NL80211_FEATURE_FILS_SK_OFFLOAD         (1 << 1)
#define NL80211_FEATURE_LOW_PRIORITY_SCAN        (1 << 2)
#define NL80211_FEATURE_SCAN_FLUSH               (1 << 3)
#define NL80211_FEATURE_AP_STA_CONCURRENCY       (1 << 4)
#define NL80211_FEATURE_TDLS_CHANNEL_SWITCH      (1 << 5)
#define NL80211_FEATURE_SCAN_RANDOM_MAC_ADDR      (1 << 6)
#define NL80211_FEATURE_SCHED_SCAN_RANDOM_MAC_ADDR (1 << 7)
#define NL80211_FEATURE_ND_RANDOM_MAC_ADDR       (1 << 8)
#define NL80211_FEATURE_MAC_ON_SUSPEND           (1 << 9)
#define NL80211_FEATURE_P2P_GO_CTWIN             (1 << 10)
#define NL80211_FEATURE_P2P_GO_OPPPS             (1 << 11)
#define NL80211_FEATURE_ADVERTISE_CHAN_LIMITS    (1 << 12)
#define NL80211_FEATURE_FULL_AP_CLIENT_STATE     (1 << 13)
#define NL80211_FEATURE_DS_PARAM_SET_IE_IN_PROBES (1 << 14)
#define NL80211_FEATURE_WFA_TPC_IE_IN_PROBES     (1 << 15)
#define NL80211_FEATURE_QUIET                    (1 << 16)
#define NL80211_FEATURE_TX_POWER_INSERTION        (1 << 17)
#define NL80211_FEATURE_ACKTO_ESTIMATION         (1 << 18)
#define NL80211_FEATURE_STATIC_SMPS              (1 << 19)
#define NL80211_FEATURE_DYNAMIC_SMPS             (1 << 20)
#define NL80211_FEATURE_WIRELESS_AGGREGATION      (1 << 21)
#define NL80211_FEATURE_VHT_IBSS                 (1 << 22)
#define NL80211_FEATURE_TDLS_OFFCHANNEL          (1 << 23)
#define NL80211_FEATURE_SCAN_MIN_PREQ_CONTENT    (1 << 24)
#define NL80211_FEATURE_ICTOE_MGMT_OFFCHAN       (1 << 25)
#define NL80211_FEATURE_PROTECTED_TDLS           (1 << 26)
#define NL80211_FEATURE_MLO                      (1 << 27)
#define NL80211_FEATURE_MLO_STA_MODE             (1 << 28)
#define NL80211_FEATURE_MULTICAST_REGISTRATIONS  (1 << 29)
#define NL80211_FEATURE_SCAN_RSSI_ANOMALY_THRESHOLD (1 << 30)

/****************************************************************************
 * Public Type Definitions
 ****************************************************************************/

/* nl80211 wiphy band data - simplified for embedded systems */

struct nl80211_band_info
{
  uint32_t band;                              /* Frequency band */
  uint8_t n_channels;                         /* Number of channels */
  uint8_t n_mcs_sets;                         /* Number of MCS sets */
  uint8_t ht_supported;                       /* HT supported */
  uint8_t vht_supported;                      /* VHT supported */
  /* Channel data follows */
};

/* nl80211 channel info */

struct nl80211_channel_info
{
  uint32_t freq;                              /* Frequency in MHz */
  uint8_t channel_width;                      /* Channel width (enum nl80211_channel_width) */
  uint8_t center_freq1;                       /* Primary channel center frequency */
  uint8_t center_freq2;                       /* Secondary channel center frequency */
  uint8_t tx_power;                           /* Transmit power in dBm */
  uint8_t band;                               /* Frequency band */
};

/* nl80211 station parameters */

struct nl80211_sta_params
{
  uint8_t sta_flags_mask;
  uint8_t sta_flags_set;
  uint8_t sta_supported_rates;
  uint8_t sta_listen_interval;
  uint8_t sta_ht_capa;
  uint8_t sta_vht_capa;
  uint8_t sta_ext_capab;
  uint8_t sta_plink_state;
  uint8_t sta_plink_action;
  uint8_t tdls_peer;
};

/* nl80211 connection parameters */

struct nl80211_connect_params
{
  FAR uint8_t *ssid;
  size_t ssid_len;
  FAR uint8_t *bssid;
  uint8_t auth_type;
  uint32_t cipher_suite;
  uint32_t akm_suite;
  FAR uint8_t *ie;
  size_t ie_len;
  uint8_t privacy;
  uint8_t bgscan_period;
  uint16_t reason_code;
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

#ifdef __cplusplus
extern "C"
{
#endif

/* nl80211 family registration functions */

int nl80211_register(FAR void);
int nl80211_unregister(FAR void);

/* nl80211 wiphy registration */

struct wiphy;
struct wireless_dev;

int nl80211_register_wiphy(FAR struct wiphy *wiphy);
int nl80211_unregister_wiphy(FAR struct wiphy *wiphy);
int nl80211_register_interface(FAR struct wireless_dev *wdev,
                               FAR const char *ifname, uint32_t ifindex);
int nl80211_unregister_interface(uint32_t ifindex);

/* nl80211 event notification */

void nl80211_notify_iface_added(FAR const char *ifname, uint32_t wiphy_idx);
void nl80211_notify_iface_removed(FAR const char *ifname, uint32_t wiphy_idx);
void nl80211_notify_scan_triggered(FAR const char *ifname, uint32_t wiphy_idx);
void nl80211_notify_scan_done(FAR const char *ifname, bool aborted);
void nl80211_notify_connect_result(FAR const char *ifname, FAR uint8_t *bssid,
                                   int status);
void nl80211_notify_disconnect(FAR const char *ifname, uint16_t reason,
                               bool local_state_change);
void nl80211_notify_michael_mic_failure(FAR const char *ifname, FAR uint8_t *addr,
                                        int key_type, int key_idx);

#ifdef __cplusplus
}
#endif

#endif /* __INCLUDE_NUTTX_WIRELESS_NL80211_H */
