/****************************************************************************
 * net/wireless/nl80211.c
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

#include <netpacket/netlink.h>

#include <nuttx/kmalloc.h>
#include <nuttx/net/net.h>
#include <nuttx/net/netlink.h>
#include <nuttx/wireless/nl80211.h>
#include <nuttx/wireless/cfg80211.h>

#include "netlink/netlink.h"

#ifdef CONFIG_NL80211

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Maximum number of wiphy devices */

#ifndef CONFIG_NL80211_MAX_WIPHY
#  define CONFIG_NL80211_MAX_WIPHY 4
#endif

/* Maximum number of wireless interfaces */

#ifndef CONFIG_NL80211_MAX_INTERFACES
#  define CONFIG_NL80211_MAX_INTERFACES 8
#endif

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* nl80211 registered wiphy */

struct nl80211_registered_wiphy
{
  FAR struct wiphy *wiphy;
  uint32_t wiphy_idx;
  bool registered;
};

/* nl80211 registered interface */

struct nl80211_registered_iface
{
  FAR struct wireless_dev *wdev;
  uint32_t ifindex;
  char ifname[IFNAMSIZ];
  bool registered;
};

/* nl80211 context for command handling */

struct nl80211_context
{
  NETLINK_HANDLE handle;
  FAR struct nlmsghdr *nlh;
  FAR struct genlmsghdr *gnlh;
  FAR struct nlattr **attrs;
  FAR struct netlink_ext_ack *extack;
  FAR struct nl80211_registered_wiphy *wiphy;
  FAR struct nl80211_registered_iface *iface;
};

/****************************************************************************
 * Private Data
 ****************************************************************************/

/* Registered wiphy devices */

static struct nl80211_registered_wiphy g_nl80211_wiphys[CONFIG_NL80211_MAX_WIPHY];

/* Registered interfaces */

static struct nl80211_registered_iface g_nl80211_ifaces[CONFIG_NL80211_MAX_INTERFACES];

/* Next wiphy index */

static uint32_t g_nl80211_next_wiphy_idx = 0;

/* nl80211 family ID (assigned by Generic Netlink) */

static int g_nl80211_family_id = -1;

/* Mutex for registration */

static mutex_t g_nl80211_lock = NXMUTEX_INITIALIZER;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: nl80211_find_wiphy_by_idx
 *
 * Description:
 *   Find a registered wiphy by index.
 *
 ****************************************************************************/

static FAR struct nl80211_registered_wiphy *
nl80211_find_wiphy_by_idx(uint32_t wiphy_idx)
{
  int i;

  for (i = 0; i < CONFIG_NL80211_MAX_WIPHY; i++)
    {
      if (g_nl80211_wiphys[i].registered &&
          g_nl80211_wiphys[i].wiphy_idx == wiphy_idx)
        {
          return &g_nl80211_wiphys[i];
        }
    }

  return NULL;
}

/****************************************************************************
 * Name: nl80211_find_iface_by_ifindex
 *
 * Description:
 *   Find a registered interface by ifindex.
 *
 ****************************************************************************/

static FAR struct nl80211_registered_iface *
nl80211_find_iface_by_ifindex(uint32_t ifindex)
{
  int i;

  for (i = 0; i < CONFIG_NL80211_MAX_INTERFACES; i++)
    {
      if (g_nl80211_ifaces[i].registered &&
          g_nl80211_ifaces[i].ifindex == ifindex)
        {
          return &g_nl80211_ifaces[i];
        }
    }

  return NULL;
}

/****************************************************************************
 * Name: nl80211_build_response
 *
 * Description:
 *   Build a nl80211 response message.
 *
 ****************************************************************************/

static FAR struct netlink_response_s *
nl80211_build_response(uint16_t cmd, uint32_t pid, uint32_t seq,
                       size_t payload_size)
{
  FAR struct netlink_response_s *resp;
  size_t total_size;

  total_size = sizeof(struct nlmsghdr) + GENL_HDRLEN + payload_size;

  resp = kmm_zalloc(sizeof(sq_entry_t) + total_size);
  if (resp == NULL)
    {
      return NULL;
    }

  resp->msg.nlmsg_len = total_size;
  resp->msg.nlmsg_type = g_nl80211_family_id;
  resp->msg.nlmsg_flags = 0;
  resp->msg.nlmsg_seq = seq;
  resp->msg.nlmsg_pid = pid;

  /* Fill Generic Netlink header */

  FAR struct genlmsghdr *genlh = (FAR struct genlmsghdr *)nlmsg_data(&resp->msg);
  genlh->cmd = cmd;
  genlh->version = NL80211_GENL_VERSION;
  genlh->reserved = 0;

  return resp;
}

/****************************************************************************
 * Name: nl80211_add_attr
 *
 * Description:
 *   Add a netlink attribute to a message.
 *
 ****************************************************************************/

static size_t nl80211_add_attr(FAR struct nlmsghdr *nlh, size_t offset,
                               uint16_t attr_type, FAR const void *data,
                               size_t data_len)
{
  FAR struct nlattr *attr;
  size_t attr_len;

  attr_len = NLA_HDRLEN + data_len;
  attr = (FAR struct nlattr *)((FAR char *)nlh + offset);

  attr->nla_len = attr_len;
  attr->nla_type = attr_type;

  if (data != NULL && data_len > 0)
    {
      memcpy((FAR char *)attr + NLA_HDRLEN, data, data_len);
    }

  return NLA_ALIGN(attr_len);
}

/****************************************************************************
 * Name: nl80211_cmd_get_wiphy
 *
 * Description:
 *   Handle NL80211_CMD_GET_WIPHY command.
 *
 ****************************************************************************/

static int nl80211_cmd_get_wiphy(FAR struct nl80211_context *ctx)
{
  FAR struct nl80211_registered_wiphy *rwiphy;
  FAR struct wiphy *wiphy;
  FAR struct netlink_response_s *resp;
  size_t offset;
  uint32_t wiphy_idx;
  int i;

  /* Check if specific wiphy is requested */

  if (ctx->attrs[NL80211_ATTR_WIPHY] != NULL)
    {
      wiphy_idx = *(FAR uint32_t *)nla_data(ctx->attrs[NL80211_ATTR_WIPHY]);
      rwiphy = nl80211_find_wiphy_by_idx(wiphy_idx);
      if (rwiphy == NULL)
        {
          return -ENOENT;
        }

      wiphy = rwiphy->wiphy;
    }
  else
    {
      /* Dump all wiphy devices */

      for (i = 0; i < CONFIG_NL80211_MAX_WIPHY; i++)
        {
          if (!g_nl80211_wiphys[i].registered)
            {
              continue;
            }

          wiphy = g_nl80211_wiphys[i].wiphy;

          /* Build response for each wiphy */

          resp = nl80211_build_response(NL80211_CMD_NEW_WIPHY,
                                        ctx->nlh->nlmsg_pid,
                                        ctx->nlh->nlmsg_seq,
                                        256);
          if (resp == NULL)
            {
              return -ENOMEM;
            }

          offset = NLMSG_HDRLEN + GENL_HDRLEN;

          /* Add wiphy index */

          wiphy_idx = g_nl80211_wiphys[i].wiphy_idx;
          offset += nl80211_add_attr(&resp->msg, offset, NL80211_ATTR_WIPHY,
                                      &wiphy_idx, sizeof(uint32_t));

          /* Add wiphy name */

          offset += nl80211_add_attr(&resp->msg, offset, NL80211_ATTR_WIPHY_NAME,
                                      wiphy->name, strlen(wiphy->name) + 1);

          netlink_add_response(ctx->handle, resp);
        }

      return OK;
    }

  /* Build single wiphy response */

  resp = nl80211_build_response(NL80211_CMD_NEW_WIPHY,
                                ctx->nlh->nlmsg_pid,
                                ctx->nlh->nlmsg_seq,
                                256);
  if (resp == NULL)
    {
      return -ENOMEM;
    }

  offset = NLMSG_HDRLEN + GENL_HDRLEN;

  /* Add wiphy index */

  offset += nl80211_add_attr(&resp->msg, offset, NL80211_ATTR_WIPHY,
                              &rwiphy->wiphy_idx, sizeof(uint32_t));

  /* Add wiphy name */

  offset += nl80211_add_attr(&resp->msg, offset, NL80211_ATTR_WIPHY_NAME,
                              wiphy->name, strlen(wiphy->name) + 1);

  /* Add supported interface modes */

  offset += nl80211_add_attr(&resp->msg, offset, NL80211_ATTR_SUPPORTED_IFTYPES,
                              &wiphy->interface_modes, sizeof(uint32_t));

  /* Add max scan SSIDs */

  offset += nl80211_add_attr(&resp->msg, offset, NL80211_ATTR_MAX_NUM_SCAN_SSIDS,
                              &wiphy->max_scan_ssids, sizeof(uint16_t));

  /* Update message length */

  resp->msg.nlmsg_len = NLMSG_HDRLEN + GENL_HDRLEN + offset;

  netlink_add_response(ctx->handle, resp);

  return OK;
}

/****************************************************************************
 * Name: nl80211_cmd_get_interface
 *
 * Description:
 *   Handle NL80211_CMD_GET_INTERFACE command.
 *
 ****************************************************************************/

static int nl80211_cmd_get_interface(FAR struct nl80211_context *ctx)
{
  FAR struct nl80211_registered_iface *riface;
  FAR struct wireless_dev *wdev;
  FAR struct netlink_response_s *resp;
  size_t offset;
  uint32_t ifindex;
  int i;

  /* Check if specific interface is requested */

  if (ctx->attrs[NL80211_ATTR_IFINDEX] != NULL)
    {
      ifindex = *(FAR uint32_t *)nla_data(ctx->attrs[NL80211_ATTR_IFINDEX]);
      riface = nl80211_find_iface_by_ifindex(ifindex);
      if (riface == NULL)
        {
          return -ENOENT;
        }

      wdev = riface->wdev;
    }
  else
    {
      /* Dump all interfaces */

      for (i = 0; i < CONFIG_NL80211_MAX_INTERFACES; i++)
        {
          if (!g_nl80211_ifaces[i].registered)
            {
              continue;
            }

          wdev = g_nl80211_ifaces[i].wdev;

          resp = nl80211_build_response(NL80211_CMD_NEW_INTERFACE,
                                        ctx->nlh->nlmsg_pid,
                                        ctx->nlh->nlmsg_seq,
                                        128);
          if (resp == NULL)
            {
              return -ENOMEM;
            }

          offset = NLMSG_HDRLEN + GENL_HDRLEN;

          /* Add ifindex */

          offset += nl80211_add_attr(&resp->msg, offset, NL80211_ATTR_IFINDEX,
                                      &g_nl80211_ifaces[i].ifindex,
                                      sizeof(uint32_t));

          /* Add ifname */

          offset += nl80211_add_attr(&resp->msg, offset, NL80211_ATTR_IFNAME,
                                      g_nl80211_ifaces[i].ifname,
                                      strlen(g_nl80211_ifaces[i].ifname) + 1);

          /* Add iftype */

          offset += nl80211_add_attr(&resp->msg, offset, NL80211_ATTR_IFTYPE,
                                      &wdev->iftype, sizeof(uint32_t));

          netlink_add_response(ctx->handle, resp);
        }

      return OK;
    }

  /* Build single interface response */

  resp = nl80211_build_response(NL80211_CMD_NEW_INTERFACE,
                                ctx->nlh->nlmsg_pid,
                                ctx->nlh->nlmsg_seq,
                                128);
  if (resp == NULL)
    {
      return -ENOMEM;
    }

  offset = NLMSG_HDRLEN + GENL_HDRLEN;

  /* Add ifindex */

  offset += nl80211_add_attr(&resp->msg, offset, NL80211_ATTR_IFINDEX,
                              &riface->ifindex, sizeof(uint32_t));

  /* Add ifname */

  offset += nl80211_add_attr(&resp->msg, offset, NL80211_ATTR_IFNAME,
                              riface->ifname, strlen(riface->ifname) + 1);

  /* Add iftype */

  offset += nl80211_add_attr(&resp->msg, offset, NL80211_ATTR_IFTYPE,
                              &wdev->iftype, sizeof(uint32_t));

  resp->msg.nlmsg_len = NLMSG_HDRLEN + GENL_HDRLEN + offset;

  netlink_add_response(ctx->handle, resp);

  return OK;
}

/****************************************************************************
 * Name: nl80211_cmd_trigger_scan
 *
 * Description:
 *   Handle NL80211_CMD_TRIGGER_SCAN command.
 *
 ****************************************************************************/

static int nl80211_cmd_trigger_scan(FAR struct nl80211_context *ctx)
{
  FAR struct nl80211_registered_iface *riface;
  FAR struct wireless_dev *wdev;
  FAR struct wiphy *wiphy;
  FAR struct cfg80211_scan_request *scan_req;
  uint32_t ifindex;
  int ret;

  /* Get interface */

  if (ctx->attrs[NL80211_ATTR_IFINDEX] == NULL)
    {
      nl_set_err_msg_attr(ctx->extack, NULL, "missing ifindex");
      return -EINVAL;
    }

  ifindex = *(FAR uint32_t *)nla_data(ctx->attrs[NL80211_ATTR_IFINDEX]);
  riface = nl80211_find_iface_by_ifindex(ifindex);
  if (riface == NULL)
    {
      return -ENOENT;
    }

  wdev = riface->wdev;
  wiphy = wdev->wiphy;

  /* Allocate scan request */

  scan_req = kmm_zalloc(sizeof(struct cfg80211_scan_request));
  if (scan_req == NULL)
    {
      return -ENOMEM;
    }

  scan_req->wiphy = wiphy;
  scan_req->netdev = wdev->netdev;

  /* Parse SSIDs */

  if (ctx->attrs[NL80211_ATTR_SCAN_SSIDS] != NULL)
    {
      /* Parse nested SSID attributes */
      /* TODO: Implement nested attribute parsing */
    }

  /* Call driver scan function */

  if (wiphy->ops != NULL && wiphy->ops->scan != NULL)
    {
      ret = wiphy->ops->scan(wiphy, scan_req);
      if (ret < 0)
        {
          kmm_free(scan_req);
          return ret;
        }
    }
  else
    {
      kmm_free(scan_req);
      return -EOPNOTSUPP;
    }

  return OK;
}

/****************************************************************************
 * Name: nl80211_cmd_connect
 *
 * Description:
 *   Handle NL80211_CMD_CONNECT command.
 *
 ****************************************************************************/

static int nl80211_cmd_connect(FAR struct nl80211_context *ctx)
{
  FAR struct nl80211_registered_iface *riface;
  FAR struct wireless_dev *wdev;
  FAR struct wiphy *wiphy;
  FAR struct cfg80211_connect_params *conn_params;
  uint32_t ifindex;
  int ret;

  /* Get interface */

  if (ctx->attrs[NL80211_ATTR_IFINDEX] == NULL)
    {
      nl_set_err_msg_attr(ctx->extack, NULL, "missing ifindex");
      return -EINVAL;
    }

  ifindex = *(FAR uint32_t *)nla_data(ctx->attrs[NL80211_ATTR_IFINDEX]);
  riface = nl80211_find_iface_by_ifindex(ifindex);
  if (riface == NULL)
    {
      return -ENOENT;
    }

  wdev = riface->wdev;
  wiphy = wdev->wiphy;

  /* Allocate connect params */

  conn_params = kmm_zalloc(sizeof(struct cfg80211_connect_params));
  if (conn_params == NULL)
    {
      return -ENOMEM;
    }

  /* Parse SSID */

  if (ctx->attrs[NL80211_ATTR_SSID] != NULL)
    {
      FAR struct nlattr *ssid_attr = ctx->attrs[NL80211_ATTR_SSID];
      conn_params->ssid.ssid_len = nla_len(ssid_attr);
      if (conn_params->ssid.ssid_len > IEEE80211_MAX_SSID_LEN)
        {
          conn_params->ssid.ssid_len = IEEE80211_MAX_SSID_LEN;
        }

      memcpy(conn_params->ssid.ssid, nla_data(ssid_attr),
             conn_params->ssid.ssid_len);
    }

  /* Parse BSSID */

  if (ctx->attrs[NL80211_ATTR_MAC] != NULL)
    {
      conn_params->bssid = nla_data(ctx->attrs[NL80211_ATTR_MAC]);
    }

  /* Parse auth type */

  if (ctx->attrs[NL80211_ATTR_AUTH_TYPE] != NULL)
    {
      conn_params->auth_type = *(FAR uint32_t *)nla_data(
                                  ctx->attrs[NL80211_ATTR_AUTH_TYPE]);
    }

  /* Call driver connect function */

  if (wiphy->ops != NULL && wiphy->ops->connect != NULL)
    {
      ret = wiphy->ops->connect(wiphy, wdev->netdev, conn_params);
      if (ret < 0)
        {
          kmm_free(conn_params);
          return ret;
        }
    }
  else
    {
      kmm_free(conn_params);
      return -EOPNOTSUPP;
    }

  kmm_free(conn_params);
  return OK;
}

/****************************************************************************
 * Name: nl80211_cmd_disconnect
 *
 * Description:
 *   Handle NL80211_CMD_DISCONNECT command.
 *
 ****************************************************************************/

static int nl80211_cmd_disconnect(FAR struct nl80211_context *ctx)
{
  FAR struct nl80211_registered_iface *riface;
  FAR struct wireless_dev *wdev;
  FAR struct wiphy *wiphy;
  uint32_t ifindex;
  uint16_t reason_code = NL80211_REASON_UNSPECIFIED;
  int ret;

  /* Get interface */

  if (ctx->attrs[NL80211_ATTR_IFINDEX] == NULL)
    {
      nl_set_err_msg_attr(ctx->extack, NULL, "missing ifindex");
      return -EINVAL;
    }

  ifindex = *(FAR uint32_t *)nla_data(ctx->attrs[NL80211_ATTR_IFINDEX]);
  riface = nl80211_find_iface_by_ifindex(ifindex);
  if (riface == NULL)
    {
      return -ENOENT;
    }

  wdev = riface->wdev;
  wiphy = wdev->wiphy;

  /* Get reason code */

  if (ctx->attrs[NL80211_ATTR_REASON_CODE] != NULL)
    {
      reason_code = *(FAR uint16_t *)nla_data(
                      ctx->attrs[NL80211_ATTR_REASON_CODE]);
    }

  /* Call driver disconnect function */

  if (wiphy->ops != NULL && wiphy->ops->disconnect != NULL)
    {
      ret = wiphy->ops->disconnect(wiphy, wdev->netdev, reason_code);
      if (ret < 0)
        {
          return ret;
        }
    }
  else
    {
      return -EOPNOTSUPP;
    }

  return OK;
}

/****************************************************************************
 * Name: nl80211_handler
 *
 * Description:
 *   Main nl80211 command handler.
 *
 ****************************************************************************/

static int nl80211_handler(NETLINK_HANDLE handle,
                           FAR const struct nlmsghdr *nlh,
                           FAR const struct genlmsghdr *gnlh,
                           FAR struct nlattr **attrs,
                           FAR struct netlink_ext_ack *extack)
{
  struct nl80211_context ctx;
  int ret;

  memset(&ctx, 0, sizeof(ctx));
  ctx.handle = handle;
  ctx.nlh = (FAR struct nlmsghdr *)nlh;
  ctx.gnlh = (FAR struct genlmsghdr *)gnlh;
  ctx.attrs = attrs;
  ctx.extack = extack;

  switch (gnlh->cmd)
    {
      case NL80211_CMD_GET_WIPHY:
        ret = nl80211_cmd_get_wiphy(&ctx);
        break;

      case NL80211_CMD_GET_INTERFACE:
        ret = nl80211_cmd_get_interface(&ctx);
        break;

      case NL80211_CMD_TRIGGER_SCAN:
        ret = nl80211_cmd_trigger_scan(&ctx);
        break;

      case NL80211_CMD_CONNECT:
        ret = nl80211_cmd_connect(&ctx);
        break;

      case NL80211_CMD_DISCONNECT:
        ret = nl80211_cmd_disconnect(&ctx);
        break;

      default:
        nl_set_err_msg_attr(extack, NULL, "unsupported nl80211 command");
        ret = -EOPNOTSUPP;
        break;
    }

  return ret;
}

/* nl80211 operations array */

static struct genl_ops_s g_nl80211_ops[] =
{
  {
    .cmd = NL80211_CMD_GET_WIPHY,
    .flags = 0,
    .handler = nl80211_handler,
  },
  {
    .cmd = NL80211_CMD_GET_INTERFACE,
    .flags = 0,
    .handler = nl80211_handler,
  },
  {
    .cmd = NL80211_CMD_TRIGGER_SCAN,
    .flags = 0,
    .handler = nl80211_handler,
  },
  {
    .cmd = NL80211_CMD_CONNECT,
    .flags = 0,
    .handler = nl80211_handler,
  },
  {
    .cmd = NL80211_CMD_DISCONNECT,
    .flags = 0,
    .handler = nl80211_handler,
  },
};

/* nl80211 attribute policy */

static const struct nla_policy g_nl80211_policy[NL80211_ATTR_MAX + 1] =
{
  [NL80211_ATTR_WIPHY]        = { .type = NLA_U32 },
  [NL80211_ATTR_WIPHY_NAME]   = { .type = NLA_STRING, .len = WIPHY_NAME_MAXLEN },
  [NL80211_ATTR_IFINDEX]      = { .type = NLA_U32 },
  [NL80211_ATTR_IFNAME]       = { .type = NLA_STRING, .len = IFNAMSIZ },
  [NL80211_ATTR_IFTYPE]       = { .type = NLA_U32 },
  [NL80211_ATTR_MAC]          = { .type = NLA_BINARY, .len = ETH_ALEN },
  [NL80211_ATTR_SSID]         = { .type = NLA_BINARY, .len = IEEE80211_MAX_SSID_LEN },
  [NL80211_ATTR_AUTH_TYPE]    = { .type = NLA_U32 },
  [NL80211_ATTR_REASON_CODE]  = { .type = NLA_U16 },
};

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: nl80211_register
 *
 * Description:
 *   Register nl80211 family with Generic Netlink.
 *
 ****************************************************************************/

int nl80211_register(void)
{
  int ret;

  /* Register nl80211 family */

  ret = genl_register_family(NULL, NL80211_GENL_NAME,
                             NL80211_GENL_VERSION, 0,
                             NL80211_ATTR_MAX, g_nl80211_policy);
  if (ret < 0)
    {
      nerr("ERROR: Failed to register nl80211 family: %d\n", ret);
      return ret;
    }

  g_nl80211_family_id = ret;

  /* Register operations */

  ret = genl_register_ops(g_nl80211_family_id, g_nl80211_ops,
                          sizeof(g_nl80211_ops) / sizeof(g_nl80211_ops[0]));
  if (ret < 0)
    {
      nerr("ERROR: Failed to register nl80211 ops: %d\n", ret);
      genl_unregister_family(g_nl80211_family_id);
      g_nl80211_family_id = -1;
      return ret;
    }

  /* Register multicast groups */

  genl_register_mcast_group(g_nl80211_family_id, NL80211_MULTICAST_GROUP_WDEV);

  ninfo("nl80211 registered with family ID %d\n", g_nl80211_family_id);

  return OK;
}

/****************************************************************************
 * Name: nl80211_unregister
 *
 * Description:
 *   Unregister nl80211 family.
 *
 ****************************************************************************/

int nl80211_unregister(void)
{
  if (g_nl80211_family_id >= 0)
    {
      genl_unregister_family(g_nl80211_family_id);
      g_nl80211_family_id = -1;
    }

  return OK;
}

/****************************************************************************
 * Name: nl80211_register_wiphy
 *
 * Description:
 *   Register a wiphy device with nl80211.
 *
 ****************************************************************************/

int nl80211_register_wiphy(FAR struct wiphy *wiphy)
{
  int i;
  int ret = -ENOMEM;

  nxmutex_lock(&g_nl80211_lock);

  /* Find a free slot */

  for (i = 0; i < CONFIG_NL80211_MAX_WIPHY; i++)
    {
      if (!g_nl80211_wiphys[i].registered)
        {
          break;
        }
    }

  if (i >= CONFIG_NL80211_MAX_WIPHY)
    {
      nxmutex_unlock(&g_nl80211_lock);
      return -ENOMEM;
    }

  g_nl80211_wiphys[i].wiphy = wiphy;
  g_nl80211_wiphys[i].wiphy_idx = g_nl80211_next_wiphy_idx++;
  g_nl80211_wiphys[i].registered = true;

  ret = g_nl80211_wiphys[i].wiphy_idx;

  nxmutex_unlock(&g_nl80211_lock);

  return ret;
}

/****************************************************************************
 * Name: nl80211_unregister_wiphy
 *
 * Description:
 *   Unregister a wiphy device.
 *
 ****************************************************************************/

int nl80211_unregister_wiphy(FAR struct wiphy *wiphy)
{
  int i;

  nxmutex_lock(&g_nl80211_lock);

  for (i = 0; i < CONFIG_NL80211_MAX_WIPHY; i++)
    {
      if (g_nl80211_wiphys[i].registered &&
          g_nl80211_wiphys[i].wiphy == wiphy)
        {
          g_nl80211_wiphys[i].registered = false;
          g_nl80211_wiphys[i].wiphy = NULL;
          break;
        }
    }

  nxmutex_unlock(&g_nl80211_lock);

  return OK;
}

/****************************************************************************
 * Name: nl80211_register_interface
 *
 * Description:
 *   Register a wireless interface with nl80211.
 *
 ****************************************************************************/

int nl80211_register_interface(FAR struct wireless_dev *wdev,
                               FAR const char *ifname, uint32_t ifindex)
{
  int i;
  int ret = -ENOMEM;

  nxmutex_lock(&g_nl80211_lock);

  /* Find a free slot */

  for (i = 0; i < CONFIG_NL80211_MAX_INTERFACES; i++)
    {
      if (!g_nl80211_ifaces[i].registered)
        {
          break;
        }
    }

  if (i >= CONFIG_NL80211_MAX_INTERFACES)
    {
      nxmutex_unlock(&g_nl80211_lock);
      return -ENOMEM;
    }

  g_nl80211_ifaces[i].wdev = wdev;
  g_nl80211_ifaces[i].ifindex = ifindex;
  strncpy(g_nl80211_ifaces[i].ifname, ifname, IFNAMSIZ - 1);
  g_nl80211_ifaces[i].ifname[IFNAMSIZ - 1] = '\0';
  g_nl80211_ifaces[i].registered = true;

  ret = ifindex;

  nxmutex_unlock(&g_nl80211_lock);

  return ret;
}

/****************************************************************************
 * Name: nl80211_unregister_interface
 *
 * Description:
 *   Unregister a wireless interface.
 *
 ****************************************************************************/

int nl80211_unregister_interface(uint32_t ifindex)
{
  int i;

  nxmutex_lock(&g_nl80211_lock);

  for (i = 0; i < CONFIG_NL80211_MAX_INTERFACES; i++)
    {
      if (g_nl80211_ifaces[i].registered &&
          g_nl80211_ifaces[i].ifindex == ifindex)
        {
          g_nl80211_ifaces[i].registered = false;
          g_nl80211_ifaces[i].wdev = NULL;
          break;
        }
    }

  nxmutex_unlock(&g_nl80211_lock);

  return OK;
}

/****************************************************************************
 * Name: nl80211_notify_scan_done
 *
 * Description:
 *   Notify nl80211 that a scan has completed.
 *
 ****************************************************************************/

void nl80211_notify_scan_done(FAR const char *ifname, bool aborted)
{
  /* TODO: Send NL80211_CMD_NEW_SCAN_RESULTS or NL80211_CMD_SCAN_ABORTED
   * via multicast
   */
}

/****************************************************************************
 * Name: nl80211_notify_connect_result
 *
 * Description:
 *   Notify nl80211 of a connection result.
 *
 ****************************************************************************/

void nl80211_notify_connect_result(FAR const char *ifname, FAR uint8_t *bssid,
                                   int status)
{
  /* TODO: Send connection result notification via multicast
   * NL80211_CMD_CONNECT for success, NL80211_CMD_DISCONNECT for failure
   */
}

/****************************************************************************
 * Name: nl80211_notify_disconnect
 *
 * Description:
 *   Notify nl80211 of a disconnection.
 *
 ****************************************************************************/

void nl80211_notify_disconnect(FAR const char *ifname, uint16_t reason,
                               bool local_state_change)
{
  /* TODO: Send NL80211_CMD_DISCONNECT notification via multicast
   */
}

#endif /* CONFIG_NL80211 */
