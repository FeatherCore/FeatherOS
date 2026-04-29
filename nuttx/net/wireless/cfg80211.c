/****************************************************************************
 * net/wireless/cfg80211.c
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
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

#include <sys/types.h>
#include <string.h>
#include <assert.h>
#include <errno.h>
#include <debug.h>

#include <net/if.h>
#include <net/ethernet.h>

#include <nuttx/kmalloc.h>
#include <nuttx/queue.h>
#include <nuttx/wqueue.h>
#include <nuttx/net/net.h>
#include <nuttx/net/netlink.h>
#include <nuttx/wireless/nl80211.h>
#include <nuttx/wireless/cfg80211.h>

#ifdef CONFIG_CFG80211

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Maximum number of registered wiphy devices */

#ifndef CONFIG_CFG80211_MAX_WIPHY
#  define CONFIG_CFG80211_MAX_WIPHY 4
#endif

/* Maximum number of BSS entries in cache */

#ifndef CONFIG_CFG80211_MAX_BSS
#  define CONFIG_CFG80211_MAX_BSS 64
#endif

/* Maximum number of scan requests */

#ifndef CONFIG_CFG80211_MAX_SCAN_REQUESTS
#  define CONFIG_CFG80211_MAX_SCAN_REQUESTS 4
#endif

/* BSS expiration time (in seconds) */

#define CFG80211_BSS_EXPIRE_TIME                120

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* Internal representation of cfg80211 wiphy */

struct cfg80211_intern_wiphy
{
  FAR struct wiphy *wiphy;
  uint32_t wiphy_idx;
  bool registered;
  sq_queue_t bss_list;        /* List of known BSS */
  sq_queue_t scan_requests;   /* Pending scan requests */
  uint32_t cookie_counter;
};

/* Internal representation of cfg80211 BSS */

struct cfg80211_intern_bss
{
  sq_entry_t node;
  struct cfg80211_bss bss;
  time_t last_seen;           /* Time when this BSS was last seen */
  uint32_t refcount;          /* Reference count */
};

/* Internal representation of scan request */

struct cfg80211_intern_scan_req
{
  sq_entry_t node;
  struct cfg80211_scan_request request;
  time_t start_time;
};

/****************************************************************************
 * Private Data
 ****************************************************************************/

/* Registered wiphy devices */

static struct cfg80211_intern_wiphy g_cfg80211_wiphys[CONFIG_CFG80211_MAX_WIPHY];

/* Mutex for cfg80211 operations */

static mutex_t g_cfg80211_lock = NXMUTEX_INITIALIZER;

/* Work queue for delayed operations */

static struct work_s g_cfg80211_work;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: cfg80211_find_wiphy_by_ptr
 *
 * Description:
 *   Find a registered wiphy by pointer.
 *
 ****************************************************************************/

static FAR struct cfg80211_intern_wiphy *
cfg80211_find_wiphy_by_ptr(FAR struct wiphy *wiphy)
{
  int i;

  for (i = 0; i < CONFIG_CFG80211_MAX_WIPHY; i++)
    {
      if (g_cfg80211_wiphys[i].registered &&
          g_cfg80211_wiphys[i].wiphy == wiphy)
        {
          return &g_cfg80211_wiphys[i];
        }
    }

  return NULL;
}

/****************************************************************************
 * Name: cfg80211_find_wiphy_by_idx
 *
 * Description:
 *   Find a registered wiphy by index.
 *
 ****************************************************************************/

static FAR struct cfg80211_intern_wiphy *
cfg80211_find_wiphy_by_idx(uint32_t wiphy_idx)
{
  int i;

  for (i = 0; i < CONFIG_CFG80211_MAX_WIPHY; i++)
    {
      if (g_cfg80211_wiphys[i].registered &&
          g_cfg80211_wiphys[i].wiphy_idx == wiphy_idx)
        {
          return &g_cfg80211_wiphys[i];
        }
    }

  return NULL;
}

/****************************************************************************
 * Name: cfg80211_intern_bss_find
 *
 * Description:
 *   Find a BSS in the internal cache.
 *
 ****************************************************************************/

static FAR struct cfg80211_intern_bss *
cfg80211_intern_bss_find(FAR struct cfg80211_intern_wiphy *intern,
                         FAR const uint8_t *bssid,
                         FAR const uint8_t *ssid, size_t ssid_len)
{
  FAR struct cfg80211_intern_bss *bss;

  sq_for_every_entry(&intern->bss_list, bss, struct cfg80211_intern_bss, node)
    {
      if (memcmp(bss->bss.bssid, bssid, ETH_ALEN) == 0 &&
          (ssid == NULL || (bss->bss.ssid_len == ssid_len &&
                           memcmp(bss->bss.ssid, ssid, ssid_len) == 0)))
        {
          return bss;
        }
    }

  return NULL;
}

/****************************************************************************
 * Name: cfg80211_intern_bss_add
 *
 * Description:
 *   Add a BSS to the internal cache.
 *
 ****************************************************************************/

static FAR struct cfg80211_intern_bss *
cfg80211_intern_bss_add(FAR struct cfg80211_intern_wiphy *intern,
                        FAR struct cfg80211_bss *new_bss)
{
  FAR struct cfg80211_intern_bss *bss;
  FAR struct cfg80211_intern_bss *oldest = NULL;
  time_t now;
  time_t oldest_time = 0;
  int count = 0;

  /* Check if BSS already exists */

  bss = cfg80211_intern_bss_find(intern, new_bss->bssid, new_bss->ssid,
                                  new_bss->ssid_len);
  if (bss != NULL)
    {
      /* Update existing entry */

      memcpy(&bss->bss, new_bss, sizeof(struct cfg80211_bss));
      bss->last_seen = time(NULL);
      bss->refcount++;
      return bss;
    }

  /* Find an empty slot or evict oldest entry */

  now = time(NULL);

  sq_for_every_entry(&intern->bss_list, bss, struct cfg80211_intern_bss, node)
    {
      count++;

      /* Check if this entry has expired */

      if (now - bss->last_seen > CFG80211_BSS_EXPIRE_TIME)
        {
          /* Remove expired entry */

          sq_rem(&bss->node, &intern->bss_list);
          kmm_free(bss);
          count--;
          continue;
        }

      /* Track oldest entry for eviction */

      if (oldest == NULL || bss->last_seen < oldest_time)
        {
          oldest = bss;
          oldest_time = bss->last_seen;
        }
    }

  /* If we've reached max BSS count, evict oldest */

  if (count >= CONFIG_CFG80211_MAX_BSS && oldest != NULL)
    {
      sq_rem(&oldest->node, &intern->bss_list);
      kmm_free(oldest);
    }

  /* Allocate new BSS entry */

  bss = kmm_zalloc(sizeof(struct cfg80211_intern_bss));
  if (bss == NULL)
    {
      return NULL;
    }

  memcpy(&bss->bss, new_bss, sizeof(struct cfg80211_bss));
  bss->last_seen = now;
  bss->refcount = 1;

  sq_addlast(&bss->node, &intern->bss_list);

  return bss;
}

/****************************************************************************
 * Name: cfg80211_intern_bss_ref
 *
 * Description:
 *   Increase reference count of a BSS entry.
 *
 ****************************************************************************/

static void cfg80211_intern_bss_ref(FAR struct cfg80211_intern_bss *bss)
{
  if (bss != NULL)
    {
      bss->refcount++;
    }
}

/****************************************************************************
 * Name: cfg80211_intern_bss_unref
 *
 * Description:
 *   Decrease reference count of a BSS entry.
 *
 ****************************************************************************/

static void cfg80211_intern_bss_unref(FAR struct cfg80211_intern_bss *bss)
{
  if (bss != NULL)
    {
      bss->refcount--;
      if (bss->refcount == 0)
        {
          /* Should only be freed when removed from list */
        }
    }
}

/****************************************************************************
 * Name: cfg80211_validate_wiphy
 *
 * Description:
 *   Validate wiphy parameters.
 *
 ****************************************************************************/

static int cfg80211_validate_wiphy(FAR struct wiphy *wiphy)
{
  if (wiphy == NULL || wiphy->ops == NULL)
    {
      return -EINVAL;
    }

  /* Validate required operations */

  if (wiphy->max_scan_ssids == 0)
    {
      wiphy->max_scan_ssids = 4;
    }

  if (wiphy->max_sched_scan_ssids == 0)
    {
      wiphy->max_sched_scan_ssids = 16;
    }

  if (wiphy->max_match_sets == 0)
    {
      wiphy->max_match_sets = 16;
    }

  return OK;
}

/****************************************************************************
 * Name: cfg80211_work_handler
 *
 * Description:
 *   Background work handler for delayed operations.
 *
 ****************************************************************************/

static void cfg80211_work_handler(FAR void *arg)
{
  /* Placeholder for future work */
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: wiphy_new
 *
 * Description:
 *   Allocate and initialize a new wiphy.
 *
 ****************************************************************************/

FAR struct wiphy *wiphy_new(FAR struct cfg80211_ops *ops, size_t sizeof_priv)
{
  FAR struct wiphy *wiphy;

  wiphy = kmm_zalloc(sizeof(struct wiphy) + sizeof_priv);
  if (wiphy == NULL)
    {
      return NULL;
    }

  wiphy->ops = ops;

  /* Set default values */

  wiphy->max_scan_ssids = 4;
  wiphy->max_sched_scan_ssids = 16;
  wiphy->max_match_sets = 16;
  wiphy->max_scan_ie_len = IEEE80211_MAX_IE_LEN;
  wiphy->max_sched_scan_ie_len = IEEE80211_MAX_IE_LEN;
  wiphy->signal_type = CFG80211_SIGNAL_TYPE_MBM;

  return wiphy;
}

/****************************************************************************
 * Name: wiphy_free
 *
 * Description:
 *   Free a wiphy structure.
 *
 ****************************************************************************/

void wiphy_free(FAR struct wiphy *wiphy)
{
  if (wiphy != NULL)
    {
      kmm_free(wiphy);
    }
}

/****************************************************************************
 * Name: wiphy_register
 *
 * Description:
 *   Register a wiphy with cfg80211.
 *
 ****************************************************************************/

int wiphy_register(FAR struct wiphy *wiphy)
{
  FAR struct cfg80211_intern_wiphy *intern;
  int i;
  int ret = -ENOMEM;

  if (wiphy == NULL)
    {
      return -EINVAL;
    }

  ret = cfg80211_validate_wiphy(wiphy);
  if (ret < 0)
    {
      return ret;
    }

  nxmutex_lock(&g_cfg80211_lock);

  /* Find a free slot */

  for (i = 0; i < CONFIG_CFG80211_MAX_WIPHY; i++)
    {
      if (!g_cfg80211_wiphys[i].registered)
        {
          break;
        }
    }

  if (i >= CONFIG_CFG80211_MAX_WIPHY)
    {
      nxmutex_unlock(&g_cfg80211_lock);
      return -ENOMEM;
    }

  intern = &g_cfg80211_wiphys[i];
  intern->wiphy = wiphy;
  intern->wiphy_idx = i;
  intern->registered = true;

  /* Initialize BSS and scan request lists */

  sq_init(&intern->bss_list);
  sq_init(&intern->scan_requests);

  /* Assign a default name if none provided */

  if (wiphy->name[0] == '\0')
    {
      snprintf(wiphy->name, WIPHY_NAME_MAXLEN, "phy%d", i);
    }

  ninfo("Registered wiphy %s (idx=%d)\n", wiphy->name, i);

  nxmutex_unlock(&g_cfg80211_lock);

  /* Register with nl80211 */

#ifdef CONFIG_NL80211
  ret = nl80211_register_wiphy(wiphy);
  if (ret < 0)
    {
      nxmutex_lock(&g_cfg80211_lock);
      intern->registered = false;
      nxmutex_unlock(&g_cfg80211_lock);
      return ret;
    }
#endif

  return OK;
}

/****************************************************************************
 * Name: wiphy_unregister
 *
 * Description:
 *   Unregister a wiphy from cfg80211.
 *
 ****************************************************************************/

void wiphy_unregister(FAR struct wiphy *wiphy)
{
  FAR struct cfg80211_intern_wiphy *intern;
  FAR struct cfg80211_intern_bss *bss;
  FAR struct cfg80211_intern_scan_req *scan_req;

  if (wiphy == NULL)
    {
      return;
    }

  intern = cfg80211_find_wiphy_by_ptr(wiphy);
  if (intern == NULL)
    {
      return;
    }

  nxmutex_lock(&g_cfg80211_lock);

  /* Clear scan requests */

  while ((scan_req = (FAR struct cfg80211_intern_scan_req *)
          sq_peek(&intern->scan_requests)) != NULL)
    {
      sq_rem(&scan_req->node, &intern->scan_requests);
      kmm_free(scan_req);
    }

  /* Clear BSS list */

  while ((bss = (FAR struct cfg80211_intern_bss *)
          sq_peek(&intern->bss_list)) != NULL)
    {
      sq_rem(&bss->node, &intern->bss_list);
      kmm_free(bss);
    }

  intern->registered = false;
  intern->wiphy = NULL;

  nxmutex_unlock(&g_cfg80211_lock);

  /* Unregister from nl80211 */

#ifdef CONFIG_NL80211
  nl80211_unregister_wiphy(wiphy);
#endif
}

/****************************************************************************
 * Name: wiphy_priv
 *
 * Description:
 *   Get private data pointer from wiphy.
 *
 ****************************************************************************/

FAR void *wiphy_priv(FAR struct wiphy *wiphy)
{
  if (wiphy == NULL)
    {
      return NULL;
    }

  return (FAR void *)((FAR char *)wiphy + sizeof(struct wiphy));
}

/****************************************************************************
 * Name: priv_to_wiphy
 *
 * Description:
 *   Get wiphy from private data pointer.
 *
 ****************************************************************************/

FAR struct wiphy *priv_to_wiphy(FAR void *priv)
{
  if (priv == NULL)
    {
      return NULL;
    }

  return (FAR struct wiphy *)((FAR char *)priv - sizeof(struct wiphy));
}

/****************************************************************************
 * Name: cfg80211_get_bss
 *
 * Description:
 *   Get BSS information by BSSID and SSID.
 *
 ****************************************************************************/

FAR struct cfg80211_bss *cfg80211_get_bss(FAR struct wiphy *wiphy,
                                          FAR struct ieee80211_channel *channel,
                                          FAR const uint8_t *bssid,
                                          FAR const uint8_t *ssid,
                                          size_t ssid_len)
{
  FAR struct cfg80211_intern_wiphy *intern;
  FAR struct cfg80211_intern_bss *intern_bss;
  FAR struct cfg80211_bss *result = NULL;

  if (wiphy == NULL || bssid == NULL)
    {
      return NULL;
    }

  intern = cfg80211_find_wiphy_by_ptr(wiphy);
  if (intern == NULL)
    {
      return NULL;
    }

  nxmutex_lock(&g_cfg80211_lock);

  intern_bss = cfg80211_intern_bss_find(intern, bssid, ssid, ssid_len);
  if (intern_bss != NULL)
    {
      cfg80211_intern_bss_ref(intern_bss);
      result = &intern_bss->bss;
    }

  nxmutex_unlock(&g_cfg80211_lock);

  return result;
}

/****************************************************************************
 * Name: cfg80211_put_bss
 *
 * Description:
 *   Release BSS information obtained from cfg80211_get_bss.
 *
 ****************************************************************************/

void cfg80211_put_bss(FAR struct wiphy *wiphy, FAR struct cfg80211_bss *bss)
{
  FAR struct cfg80211_intern_wiphy *intern;
  FAR struct cfg80211_intern_bss *intern_bss;

  if (wiphy == NULL || bss == NULL)
    {
      return;
    }

  intern = cfg80211_find_wiphy_by_ptr(wiphy);
  if (intern == NULL)
    {
      return;
    }

  nxmutex_lock(&g_cfg80211_lock);

  /* Find the internal BSS representation */

  sq_for_every_entry(&intern->bss_list, intern_bss, struct cfg80211_intern_bss, node)
    {
      if (&intern_bss->bss == bss)
        {
          cfg80211_intern_bss_unref(intern_bss);
          break;
        }
    }

  nxmutex_unlock(&g_cfg80211_lock);
}

/****************************************************************************
 * Name: cfg80211_inform_bss
 *
 * Description:
 *   Inform cfg80211 about a discovered BSS.
 *
 ****************************************************************************/

void cfg80211_inform_bss(FAR struct wiphy *wiphy,
                         FAR struct ieee80211_channel *channel,
                         FAR const uint8_t *bssid,
                         uint64_t tsf, uint16_t capability,
                         uint16_t beacon_interval,
                         FAR const uint8_t *ie, size_t ies_len,
                         int signal, gfp_t gfp)
{
  FAR struct cfg80211_intern_wiphy *intern;
  struct cfg80211_bss new_bss;
  FAR struct cfg80211_intern_bss *intern_bss;

  if (wiphy == NULL || bssid == NULL)
    {
      return;
    }

  intern = cfg80211_find_wiphy_by_ptr(wiphy);
  if (intern == NULL)
    {
      return;
    }

  /* Prepare new BSS entry */

  memset(&new_bss, 0, sizeof(new_bss));
  new_bss.wiphy = wiphy;
  new_bss.netdev = NULL; /* May be set by caller */
  memcpy(new_bss.bssid, bssid, ETH_ALEN);
  
  /* Extract SSID from IE if available */
  if (ie != NULL && ies_len > 0)
    {
      FAR const uint8_t *pos = ie;
      FAR const uint8_t *end = ie + ies_len;
      
      while (pos + 2 < end)
        {
          if (pos[0] == 0 && pos[1] > 0) /* SSID element ID is 0 */
            {
              size_t ssid_len = pos[1];
              if (ssid_len > IEEE80211_MAX_SSID_LEN)
                {
                  ssid_len = IEEE80211_MAX_SSID_LEN;
                }
              
              memcpy(new_bss.ssid, pos + 2, ssid_len);
              new_bss.ssid_len = ssid_len;
              break;
            }
          
          pos += 2 + pos[1];
        }
    }

  new_bss.channel = channel;
  new_bss.beacon_interval = beacon_interval;
  new_bss.capability = capability;
  new_bss.ies = (FAR uint8_t *)ie;
  new_bss.ies_len = ies_len;
  new_bss.signal = signal;
  new_bss.tsf = tsf;

  nxmutex_lock(&g_cfg80211_lock);

  intern_bss = cfg80211_intern_bss_add(intern, &new_bss);
  if (intern_bss != NULL)
    {
      ninfo("Added/updated BSS %02x:%02x:%02x:%02x:%02x:%02x (SSID: %.*s)\n",
            bssid[0], bssid[1], bssid[2], bssid[3], bssid[4], bssid[5],
            (int)new_bss.ssid_len, new_bss.ssid);
    }

  nxmutex_unlock(&g_cfg80211_lock);
}

/****************************************************************************
 * Name: cfg80211_scan_done
 *
 * Description:
 *   Notify cfg80211 that a scan has completed.
 *
 ****************************************************************************/

void cfg80211_scan_done(FAR struct cfg80211_scan_request *request, bool aborted)
{
  /* Send scan completion notification to userspace via nl80211 */
#ifdef CONFIG_NL80211
  if (request != NULL && request->netdev != NULL)
    {
      FAR const char *ifname = request->netdev->name; /* Assuming netdev has name field */
      nl80211_notify_scan_done(ifname, aborted);
    }
#endif

  ninfo("Scan %scompleted\n", aborted ? "aborted " : "");
}

/****************************************************************************
 * Name: cfg80211_connect_result
 *
 * Description:
 *   Notify cfg80211 of a successful connection.
 *
 ****************************************************************************/

void cfg80211_connect_result(FAR struct net_device *netdev,
                              FAR const uint8_t *bssid,
                              FAR const uint8_t *req_ie, size_t req_ie_len,
                              FAR const uint8_t *resp_ie, size_t resp_ie_len,
                              uint16_t status, gfp_t gfp)
{
  /* Send connection result notification to userspace via nl80211 */
#ifdef CONFIG_NL80211
  if (netdev != NULL)
    {
      FAR const char *ifname = netdev->name; /* Assuming netdev has name field */
      nl80211_notify_connect_result(ifname, (FAR uint8_t *)bssid, status);
    }
#endif

  ninfo("Connection %s to BSS %02x:%02x:%02x:%02x:%02x:%02x\n",
        status == 0 ? "success" : "failed",
        bssid[0], bssid[1], bssid[2], bssid[3], bssid[4], bssid[5]);
}

/****************************************************************************
 * Name: cfg80211_disconnected
 *
 * Description:
 *   Notify cfg80211 of a disconnection.
 *
 ****************************************************************************/

void cfg80211_disconnected(FAR struct net_device *netdev, uint16_t reason,
                           FAR const uint8_t *ie, size_t ie_len,
                           bool locally_generated, gfp_t gfp)
{
  /* Send disconnection notification to userspace via nl80211 */
#ifdef CONFIG_NL80211
  if (netdev != NULL)
    {
      FAR const char *ifname = netdev->name; /* Assuming netdev has name field */
      nl80211_notify_disconnect(ifname, reason, locally_generated);
    }
#endif

  ninfo("Disconnected from network, reason: %d\n", reason);
}

/****************************************************************************
 * Name: cfg80211_new_sta
 *
 * Description:
 *   Notify cfg80211 of a new station joining.
 *
 ****************************************************************************/

void cfg80211_new_sta(FAR struct net_device *netdev, FAR const uint8_t *mac_addr,
                      FAR struct station_info *sinfo, gfp_t gfp)
{
  ninfo("New station joined: %02x:%02x:%02x:%02x:%02x:%02x\n",
        mac_addr[0], mac_addr[1], mac_addr[2], mac_addr[3], mac_addr[4], mac_addr[5]);
}

/****************************************************************************
 * Name: cfg80211_del_sta
 *
 * Description:
 *   Notify cfg80211 of a station leaving.
 *
 ****************************************************************************/

void cfg80211_del_sta(FAR struct net_device *netdev, FAR const uint8_t *mac_addr,
                      gfp_t gfp)
{
  ninfo("Station left: %02x:%02x:%02x:%02x:%02x:%02x\n",
        mac_addr[0], mac_addr[1], mac_addr[2], mac_addr[3], mac_addr[4], mac_addr[5]);
}

/****************************************************************************
 * Name: regulatory_hint
 *
 * Description:
 *   Provide a regulatory hint to cfg80211.
 *
 ****************************************************************************/

void regulatory_hint(FAR const char *alpha2)
{
  ninfo("Regulatory hint: %s\n", alpha2 ? alpha2 : "NULL");
}

/****************************************************************************
 * Name: wiphy_apply_custom_regulatory
 *
 * Description:
 *   Apply a custom regulatory domain to a wiphy.
 *
 ****************************************************************************/

void wiphy_apply_custom_regulatory(FAR struct wiphy *wiphy,
                                   FAR const struct ieee80211_regdomain *regd)
{
  ninfo("Applying custom regulatory domain to wiphy\n");
}

/****************************************************************************
 * Name: cfg80211_work_start
 *
 * Description:
 *   Start the cfg80211 work queue.
 *
 ****************************************************************************/

void cfg80211_initialize(void)
{
  /* Initialize mutex */
  nxmutex_init(&g_cfg80211_lock);

  /* Initialize work queue for delayed operations */
  work_queue(LPWORK, &g_cfg80211_work, cfg80211_work_handler, NULL, 0);

  ninfo("cfg80211 initialized\n");
}

/****************************************************************************
 * Name: cfg80211_cleanup
 *
 * Description:
 *   Cleanup cfg80211 resources.
 *
 ****************************************************************************/

void cfg80211_cleanup(void)
{
  int i;

  /* Unregister all wiphy devices */

  for (i = 0; i < CONFIG_CFG80211_MAX_WIPHY; i++)
    {
      if (g_cfg80211_wiphys[i].registered)
        {
          wiphy_unregister(g_cfg80211_wiphys[i].wiphy);
        }
    }

  /* Destroy mutex */
  nxmutex_destroy(&g_cfg80211_lock);

  ninfo("cfg80211 cleaned up\n");
}

#endif /* CONFIG_CFG80211 */