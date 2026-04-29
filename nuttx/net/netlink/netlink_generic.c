/****************************************************************************
 * net/netlink/netlink_generic.c
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

#include "netlink/netlink.h"

#ifdef CONFIG_NETLINK_GENERIC

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Maximum number of registered Generic Netlink families */

#ifndef CONFIG_NETLINK_GENERIC_MAX_FAMILIES
#  define CONFIG_NETLINK_GENERIC_MAX_FAMILIES 16
#endif

/* Maximum number of multicast groups per family */

#ifndef CONFIG_NETLINK_GENERIC_MAX_MCAST_GROUPS
#  define CONFIG_NETLINK_GENERIC_MAX_MCAST_GROUPS 8
#endif

/* Maximum family name length */

#define GENL_FAMILY_NAME_MAXLEN         16

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* Generic Netlink multicast group */

struct genl_mcast_group_s
{
  char name[GENL_FAMILY_NAME_MAXLEN];   /* Group name */
  uint32_t id;                          /* Group ID (dynamically assigned) */
};

/* Generic Netlink operation */

struct genl_ops_s
{
  uint8_t cmd;                          /* Command identifier */
  uint8_t flags;                        /* Operation flags */
  int (*handler)(NETLINK_HANDLE handle,
                 FAR const struct nlmsghdr *nlh,
                 FAR const struct genlmsghdr *gnlh,
                 FAR struct nlattr **attrs,
                 FAR struct netlink_ext_ack *extack);
};

/* Generic Netlink family */

struct genl_family_s
{
  char name[GENL_FAMILY_NAME_MAXLEN];   /* Family name */
  uint16_t id;                          /* Family ID (dynamically assigned) */
  uint8_t version;                      /* Interface version */
  uint8_t hdrsize;                      /* Header size (excluding genlmsghdr) */
  uint16_t maxattr;                     /* Maximum attribute type */
  FAR const struct nla_policy *policy;  /* Attribute validation policy */
  FAR struct genl_ops_s *ops;           /* Operations array */
  uint8_t n_ops;                        /* Number of operations */
  struct genl_mcast_group_s mcast_groups[CONFIG_NETLINK_GENERIC_MAX_MCAST_GROUPS];
  uint8_t n_mcast_groups;               /* Number of multicast groups */
  bool registered;                      /* Family is registered */
};

/* CTRL_CMD_GETFAMILY response */

struct ctrl_getfamily_response_s
{
  struct nlmsghdr hdr;
  struct genlmsghdr genlh;
  struct nlattr attr_family_id;
  uint16_t family_id;
  struct nlattr attr_family_name;
  char family_name[GENL_FAMILY_NAME_MAXLEN];
  struct nlattr attr_version;
  uint8_t version;
  struct nlattr attr_hdrsize;
  uint8_t hdrsize;
  struct nlattr attr_maxattr;
  uint16_t maxattr;
};

/****************************************************************************
 * Private Data
 ****************************************************************************/

/* Registered Generic Netlink families */

static struct genl_family_s g_genl_families[CONFIG_NETLINK_GENERIC_MAX_FAMILIES];

/* Next dynamic family ID (starts after reserved IDs) */

static uint16_t g_genl_next_id = GENL_ID_CTRL + 1;

/* Next dynamic multicast group ID */

static uint32_t g_genl_next_mcast_id = 1;

/* Mutex for family registration */

static mutex_t g_genl_lock = NXMUTEX_INITIALIZER;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: genl_find_family_by_name
 *
 * Description:
 *   Find a registered family by name.
 *
 ****************************************************************************/

static FAR struct genl_family_s *genl_find_family_by_name(FAR const char *name)
{
  int i;

  for (i = 0; i < CONFIG_NETLINK_GENERIC_MAX_FAMILIES; i++)
    {
      if (g_genl_families[i].registered &&
          strcmp(g_genl_families[i].name, name) == 0)
        {
          return &g_genl_families[i];
        }
    }

  return NULL;
}

/****************************************************************************
 * Name: genl_find_family_by_id
 *
 * Description:
 *   Find a registered family by ID.
 *
 ****************************************************************************/

static FAR struct genl_family_s *genl_find_family_by_id(uint16_t id)
{
  int i;

  for (i = 0; i < CONFIG_NETLINK_GENERIC_MAX_FAMILIES; i++)
    {
      if (g_genl_families[i].registered &&
          g_genl_families[i].id == id)
        {
          return &g_genl_families[i];
        }
    }

  return NULL;
}

/****************************************************************************
 * Name: genl_find_op
 *
 * Description:
 *   Find an operation in a family by command.
 *
 ****************************************************************************/

static FAR struct genl_ops_s *genl_find_op(FAR struct genl_family_s *family,
                                           uint8_t cmd)
{
  int i;

  for (i = 0; i < family->n_ops; i++)
    {
      if (family->ops[i].cmd == cmd)
        {
          return &family->ops[i];
        }
    }

  return NULL;
}

/****************************************************************************
 * Name: genl_ctrl_getfamily
 *
 * Description:
 *   Handle CTRL_CMD_GETFAMILY command.
 *
 ****************************************************************************/

static int genl_ctrl_getfamily(NETLINK_HANDLE handle,
                               FAR const struct nlmsghdr *nlh,
                               FAR struct nlattr **attrs,
                               FAR struct netlink_ext_ack *extack)
{
  FAR struct genl_family_s *family = NULL;
  FAR struct ctrl_getfamily_response_s *resp;
  FAR const char *family_name;
  size_t resp_len;
  size_t name_len;

  /* Check for family name attribute */

  if (attrs[CTRL_ATTR_FAMILY_NAME] != NULL)
    {
      family_name = (FAR const char *)nla_data(attrs[CTRL_ATTR_FAMILY_NAME]);
      family = genl_find_family_by_name(family_name);
    }
  else if (attrs[CTRL_ATTR_FAMILY_ID] != NULL)
    {
      uint16_t family_id = *(FAR uint16_t *)nla_data(attrs[CTRL_ATTR_FAMILY_ID]);
      family = genl_find_family_by_id(family_id);
    }
  else
    {
      /* Need either family name or ID */

      nl_set_err_msg_attr(extack, NULL, "missing family name or id");
      return -EINVAL;
    }

  if (family == NULL)
    {
      return -ENOENT;
    }

  /* Build response */

  name_len = strlen(family->name) + 1;
  resp_len = sizeof(struct ctrl_getfamily_response_s);

  resp = kmm_zalloc(resp_len);
  if (resp == NULL)
    {
      return -ENOMEM;
    }

  /* Fill response header */

  resp->hdr.nlmsg_len   = resp_len;
  resp->hdr.nlmsg_type  = family->id;
  resp->hdr.nlmsg_flags = 0;
  resp->hdr.nlmsg_seq   = nlh->nlmsg_seq;
  resp->hdr.nlmsg_pid   = nlh->nlmsg_pid;

  /* Fill Generic Netlink header */

  resp->genlh.cmd     = CTRL_CMD_NEWFAMILY;
  resp->genlh.version = 0x01;
  resp->genlh.reserved = 0;

  /* Fill attributes */

  resp->attr_family_id.nla_len = sizeof(struct nlattr) + sizeof(uint16_t);
  resp->attr_family_id.nla_type = CTRL_ATTR_FAMILY_ID;
  resp->family_id = family->id;

  resp->attr_family_name.nla_len = sizeof(struct nlattr) + name_len;
  resp->attr_family_name.nla_type = CTRL_ATTR_FAMILY_NAME;
  strncpy(resp->family_name, family->name, GENL_FAMILY_NAME_MAXLEN - 1);

  resp->attr_version.nla_len = sizeof(struct nlattr) + sizeof(uint8_t);
  resp->attr_version.nla_type = CTRL_ATTR_VERSION;
  resp->version = family->version;

  resp->attr_hdrsize.nla_len = sizeof(struct nlattr) + sizeof(uint8_t);
  resp->attr_hdrsize.nla_type = CTRL_ATTR_HDRSIZE;
  resp->hdrsize = family->hdrsize;

  resp->attr_maxattr.nla_len = sizeof(struct nlattr) + sizeof(uint16_t);
  resp->attr_maxattr.nla_type = CTRL_ATTR_MAXATTR;
  resp->maxattr = family->maxattr;

  /* Add response to the queue */

  netlink_add_response(handle, (FAR struct netlink_response_s *)resp);

  return OK;
}

/****************************************************************************
 * Name: genl_ctrl_handler
 *
 * Description:
 *   Handle Generic Netlink controller (nlctrl) commands.
 *
 ****************************************************************************/

static int genl_ctrl_handler(NETLINK_HANDLE handle,
                             FAR const struct nlmsghdr *nlh,
                             FAR const struct genlmsghdr *gnlh,
                             FAR struct nlattr **attrs,
                             FAR struct netlink_ext_ack *extack)
{
  switch (gnlh->cmd)
    {
      case CTRL_CMD_GETFAMILY:
        return genl_ctrl_getfamily(handle, nlh, attrs, extack);

      default:
        nl_set_err_msg_attr(extack, NULL, "unsupported controller command");
        return -EOPNOTSUPP;
    }
}

/* Controller family operations */

static struct genl_ops_s g_genl_ctrl_ops[] =
{
  {
    .cmd = CTRL_CMD_GETFAMILY,
    .flags = 0,
    .handler = genl_ctrl_handler,
  },
};

/****************************************************************************
 * Name: genl_dispatch_cmd
 *
 * Description:
 *   Dispatch a Generic Netlink command to the appropriate handler.
 *
 ****************************************************************************/

static int genl_dispatch_cmd(NETLINK_HANDLE handle,
                             FAR const struct nlmsghdr *nlh,
                             FAR struct genlmsghdr *gnlh,
                             FAR struct netlink_ext_ack *extack)
{
  FAR struct genl_family_s *family;
  FAR struct genl_ops_s *op;
  FAR struct nlattr *attrs[CTRL_ATTR_MAX + 1];
  int ret;

  /* Find the family by message type */

  family = genl_find_family_by_id(nlh->nlmsg_type);
  if (family == NULL)
    {
      nl_set_err_msg_attr(extack, NULL, "unknown family");
      return -ENOENT;
    }

  /* Find the operation */

  op = genl_find_op(family, gnlh->cmd);
  if (op == NULL)
    {
      nl_set_err_msg_attr(extack, NULL, "unknown command");
      return -EOPNOTSUPP;
    }

  /* Parse attributes if policy is provided */

  if (family->maxattr > 0 && family->policy != NULL)
    {
      ret = nlmsg_parse(nlh, GENL_HDRLEN, attrs, family->maxattr,
                        family->policy, extack);
      if (ret < 0)
        {
          return ret;
        }
    }
  else
    {
      memset(attrs, 0, sizeof(attrs));
    }

  /* Call the operation handler */

  return op->handler(handle, nlh, gnlh, attrs, extack);
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: netlink_generic_sendto
 *
 * Description:
 *   Perform the sendto() operation for the NETLINK_GENERIC protocol.
 *
 ****************************************************************************/

ssize_t netlink_generic_sendto(NETLINK_HANDLE handle,
                               FAR const struct nlmsghdr *nlmsg,
                               size_t len, int flags,
                               FAR const struct sockaddr_nl *to,
                               socklen_t tolen)
{
  FAR struct genlmsghdr *gnlh;
  struct netlink_ext_ack extack;
  int ret;

  DEBUGASSERT(nlmsg != NULL && nlmsg->nlmsg_len >= sizeof(struct nlmsghdr));

  /* Verify message has Generic Netlink header */

  if (nlmsg->nlmsg_len < NLMSG_HDRLEN + GENL_HDRLEN)
    {
      nerr("ERROR: Message too short for Generic Netlink\n");
      return -EINVAL;
    }

  /* Get Generic Netlink header */

  gnlh = (FAR struct genlmsghdr *)nlmsg_data((FAR struct nlmsghdr *)nlmsg);

  /* Initialize extended ack */

  memset(&extack, 0, sizeof(extack));

  /* Dispatch the command */

  nxmutex_lock(&g_genl_lock);
  ret = genl_dispatch_cmd(handle, nlmsg, gnlh, &extack);
  nxmutex_unlock(&g_genl_lock);

  if (ret < 0)
    {
      nerr("ERROR: Generic Netlink command failed: %d\n", ret);
      return ret;
    }

  return len;
}

/****************************************************************************
 * Name: genl_register_family
 *
 * Description:
 *   Register a Generic Netlink family.
 *
 * Input Parameters:
 *   family   - Pointer to family structure to register.
 *              The 'name' and 'ops' fields must be filled.
 *   version  - Interface version.
 *   hdrsize  - Header size (excluding genlmsghdr).
 *   maxattr  - Maximum attribute type.
 *   policy   - Attribute validation policy (may be NULL).
 *
 * Returned Value:
 *   Family ID on success, negated errno on failure.
 *
 ****************************************************************************/

int genl_register_family(FAR struct genl_family_s *family,
                         FAR const char *name,
                         uint8_t version, uint8_t hdrsize,
                         uint16_t maxattr,
                         FAR const struct nla_policy *policy)
{
  int i;
  int ret = -ENOMEM;

  nxmutex_lock(&g_genl_lock);

  /* Check if family already registered */

  if (genl_find_family_by_name(name) != NULL)
    {
      nxmutex_unlock(&g_genl_lock);
      return -EEXIST;
    }

  /* Find a free slot */

  for (i = 0; i < CONFIG_NETLINK_GENERIC_MAX_FAMILIES; i++)
    {
      if (!g_genl_families[i].registered)
        {
          break;
        }
    }

  if (i >= CONFIG_NETLINK_GENERIC_MAX_FAMILIES)
    {
      nxmutex_unlock(&g_genl_lock);
      return -ENOMEM;
    }

  /* Allocate family ID */

  if (g_genl_next_id >= 0xffff)
    {
      nxmutex_unlock(&g_genl_lock);
      return -ENOSPC;
    }

  /* Initialize family */

  strncpy(g_genl_families[i].name, name, GENL_FAMILY_NAME_MAXLEN - 1);
  g_genl_families[i].name[GENL_FAMILY_NAME_MAXLEN - 1] = '\0';
  g_genl_families[i].id = g_genl_next_id++;
  g_genl_families[i].version = version;
  g_genl_families[i].hdrsize = hdrsize;
  g_genl_families[i].maxattr = maxattr;
  g_genl_families[i].policy = policy;
  g_genl_families[i].registered = true;

  ret = g_genl_families[i].id;

  nxmutex_unlock(&g_genl_lock);

  return ret;
}

/****************************************************************************
 * Name: genl_unregister_family
 *
 * Description:
 *   Unregister a Generic Netlink family.
 *
 * Input Parameters:
 *   family_id - Family ID returned by genl_register_family.
 *
 * Returned Value:
 *   OK on success, negated errno on failure.
 *
 ****************************************************************************/

int genl_unregister_family(int family_id)
{
  FAR struct genl_family_s *family;
  int ret = OK;

  nxmutex_lock(&g_genl_lock);

  family = genl_find_family_by_id(family_id);
  if (family == NULL)
    {
      nxmutex_unlock(&g_genl_lock);
      return -ENOENT;
    }

  family->registered = false;
  family->id = 0;
  family->n_ops = 0;
  family->n_mcast_groups = 0;

  nxmutex_unlock(&g_genl_lock);

  return ret;
}

/****************************************************************************
 * Name: genl_register_ops
 *
 * Description:
 *   Register operations for a Generic Netlink family.
 *
 * Input Parameters:
 *   family_id - Family ID returned by genl_register_family.
 *   ops       - Array of operations to register.
 *   n_ops     - Number of operations.
 *
 * Returned Value:
 *   OK on success, negated errno on failure.
 *
 ****************************************************************************/

int genl_register_ops(int family_id, FAR struct genl_ops_s *ops, uint8_t n_ops)
{
  FAR struct genl_family_s *family;
  int ret = OK;

  nxmutex_lock(&g_genl_lock);

  family = genl_find_family_by_id(family_id);
  if (family == NULL)
    {
      nxmutex_unlock(&g_genl_lock);
      return -ENOENT;
    }

  family->ops = ops;
  family->n_ops = n_ops;

  nxmutex_unlock(&g_genl_lock);

  return ret;
}

/****************************************************************************
 * Name: genl_register_mcast_group
 *
 * Description:
 *   Register a multicast group for a Generic Netlink family.
 *
 * Input Parameters:
 *   family_id - Family ID returned by genl_register_family.
 *   name      - Multicast group name.
 *
 * Returned Value:
 *   Group ID on success, negated errno on failure.
 *
 ****************************************************************************/

int genl_register_mcast_group(int family_id, FAR const char *name)
{
  FAR struct genl_family_s *family;
  int ret;

  nxmutex_lock(&g_genl_lock);

  family = genl_find_family_by_id(family_id);
  if (family == NULL)
    {
      nxmutex_unlock(&g_genl_lock);
      return -ENOENT;
    }

  if (family->n_mcast_groups >= CONFIG_NETLINK_GENERIC_MAX_MCAST_GROUPS)
    {
      nxmutex_unlock(&g_genl_lock);
      return -ENOSPC;
    }

  /* Allocate group ID */

  if (g_genl_next_mcast_id >= 0xffffffff)
    {
      nxmutex_unlock(&g_genl_lock);
      return -ENOSPC;
    }

  strncpy(family->mcast_groups[family->n_mcast_groups].name, name,
          GENL_FAMILY_NAME_MAXLEN - 1);
  family->mcast_groups[family->n_mcast_groups].name[GENL_FAMILY_NAME_MAXLEN - 1] = '\0';
  family->mcast_groups[family->n_mcast_groups].id = g_genl_next_mcast_id++;

  ret = family->mcast_groups[family->n_mcast_groups].id;
  family->n_mcast_groups++;

  nxmutex_unlock(&g_genl_lock);

  return ret;
}

/****************************************************************************
 * Name: genl_multicast
 *
 * Description:
 *   Send a multicast message to all listeners of a multicast group.
 *
 * Input Parameters:
 *   family_id - Family ID.
 *   group_id  - Multicast group ID.
 *   msg       - Message to send (nlmsghdr followed by genlmsghdr and payload).
 *
 * Returned Value:
 *   OK on success, negated errno on failure.
 *
 ****************************************************************************/

int genl_multicast(int family_id, int group_id, FAR struct nlmsghdr *msg)
{
  /* Use netlink_add_broadcast to send to all interested listeners */

  netlink_add_broadcast(group_id, (FAR struct netlink_response_s *)msg);

  return OK;
}

/****************************************************************************
 * Name: netlink_generic_initialize
 *
 * Description:
 *   Initialize Generic Netlink subsystem.
 *
 ****************************************************************************/

void netlink_generic_initialize(void)
{
  int family_id;

  /* Register the controller family */

  family_id = genl_register_family(NULL, "nlctrl", 0x01, 0,
                                   CTRL_ATTR_MAX, NULL);
  if (family_id < 0)
    {
      nerr("ERROR: Failed to register controller family: %d\n", family_id);
      return;
    }

  /* Register controller operations */

  genl_register_ops(family_id, g_genl_ctrl_ops,
                    sizeof(g_genl_ctrl_ops) / sizeof(g_genl_ctrl_ops[0]));
}

#endif /* CONFIG_NETLINK_GENERIC */
