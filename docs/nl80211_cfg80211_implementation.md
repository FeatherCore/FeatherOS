# NuttX nl80211/cfg80211 WiFi 架构实现计划

## 目标

在 NuttX 中实现与 Linux 兼容的 WiFi 架构，使 NuttX 能够：
1. 使用标准的 nl80211 Netlink 接口配置 WiFi
2. 支持 wpa_supplicant/hostapd 等标准用户态工具
3. 通过 cfg80211 层驱动提供 cfg80211 驱动接口的 WiFi 芯片

## 架构对比

### Linux WiFi 架构

```
┌─────────────────────────────────────────────────────────────────┐
│                    User Space Applications                       │
├─────────────────────────────────────────────────────────────────┤
│  wpa_supplicant (STA)    │    hostapd (AP)                      │
│  - WPA/WPA2/WPA3 认证     │    - AP 模式管理                     │
│  - 密钥协商               │    - 热点功能                        │
└───────────────┬───────────┴──────────────┬──────────────────────┘
                │                          │
                ▼                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                         libnl (Netlink Library)                  │
│  - 用户态 Netlink 套接字封装                                      │
│  - nl80211 消息构建/解析                                         │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                         nl80211 (Netlink Interface)              │
│  - 802.11 配置接口 (NETLINK_GENERIC 子协议)                      │
│  - 命令: NL80211_CMD_* (扫描、连接、认证等)                       │
│  - 属性: NL80211_ATTR_* (SSID、频率、密钥等)                     │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                         cfg80211 (Configuration API)             │
│  - 内核态 802.11 配置层                                          │
│  - 统一的驱动配置接口 (struct cfg80211_ops)                      │
│  - 管理认证状态机                                                 │
│  - regulatory domain 管理                                        │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                         mac80211 (MAC Layer) [可选]              │
│  - 802.11 MAC 层实现 (软件 MAC)                                  │
│  - 帧 fragmentation/reassembly                                   │
│  - 电源管理、QoS 支持                                            │
│  - 全功能驱动使用 (如 ath9k, iwlwifi)                            │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                         WiFi Driver                              │
│  - 硬件特定驱动                                                  │
│  - 实现 cfg80211_ops 回调函数                                    │
└─────────────────────────────────────────────────────────────────┘
```

### 目标 NuttX WiFi 架构

```
┌─────────────────────────────────────────────────────────────────┐
│                    User Space Applications                       │
├─────────────────────────────────────────────────────────────────┤
│  wpa_supplicant (STA)    │    hostapd (AP)                      │
│  [可复用 Linux 版本]      │    [可复用 Linux 版本]               │
└───────────────┬───────────┴──────────────┬──────────────────────┘
                │                          │
                ▼                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                         libnl (Netlink Library)                  │
│  [可复用 Linux libnl 或使用简化版本]                             │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    NuttX Netlink Socket Layer                    │
│  状态: ✅ 已实现 (CONFIG_NET_NETLINK)                            │
│  文件: nuttx/net/netlink/                                       │
│  支持: NETLINK_ROUTE, NETLINK_NETFILTER                         │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    nl80211 (Netlink Interface)                   │
│  状态: ❌ 未实现                                                 │
│  需要: NETLINK_GENERIC 协议支持                                  │
│  需要: nl80211 命令/属性定义                                     │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    cfg80211 (Configuration API)                  │
│  状态: ❌ 未实现                                                 │
│  需要: struct cfg80211_ops 驱动接口                              │
│  需要: regulatory domain 管理                                    │
│  需要: 扫描结果缓存                                              │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┤
│                    WiFi Driver Adapter                           │
│  状态: ⚠️ 部分实现 (ESP32, wifi_sim)                             │
│  需要: 适配到 cfg80211_ops 接口                                  │
│  现有: esp_wifi_api, wifi_sim                                   │
└─────────────────────────────────────────────────────────────────┘
```

## 当前 NuttX Netlink 实现状态

### 已实现

| 组件 | 文件 | 状态 |
|------|------|------|
| Netlink Socket | `net/netlink/netlink_sockif.c` | ✅ 完成 |
| Netlink Connection | `net/netlink/netlink_conn.c` | ✅ 完成 |
| NETLINK_ROUTE | `net/netlink/netlink_route.c` | ✅ 完成 |
| NETLINK_NETFILTER | `net/netlink/netlink_netfilter.c` | ✅ 完成 |
| Netlink Attributes | `net/netlink/netlink_attr.c` | ✅ 完成 |
| Netlink Notifier | `net/netlink/netlink_notifier.c` | ✅ 完成 |
| 用户态头文件 | `include/netpacket/netlink.h` | ✅ 完成 |
| 内核态头文件 | `include/nuttx/net/netlink.h` | ✅ 完成 |

### 未实现

| 组件 | 描述 | 优先级 |
|------|------|--------|
| NETLINK_GENERIC | nl80211 所需的通用 Netlink 协议 | 高 |
| nl80211 | 802.11 配置接口 | 高 |
| cfg80211 | WiFi 配置 API 层 | 高 |
| mac80211 | 软件 MAC 层 (可选) | 低 |

## 实现计划

### 阶段 1: NETLINK_GENERIC 基础设施 (预计 2 周)

**目标**: 添加 Generic Netlink 协议支持

**任务清单**:

- [ ] 1.1 添加 NETLINK_GENERIC 协议号定义
  - 文件: `include/netpacket/netlink.h`
  - 添加 `#define NETLINK_GENERIC 16`

- [ ] 1.2 实现 Generic Netlink 消息结构
  - 文件: `include/netpacket/netlink.h`
  - 添加 `struct genlmsghdr`:
    ```c
    struct genlmsghdr {
        uint8_t  cmd;      /* Generic netlink command */
        uint8_t  version;  /* Interface version */
        uint16_t reserved; /* Reserved for future use */
    };
    ```

- [ ] 1.3 实现 Generic Netlink 多播组支持
  - 文件: `include/netpacket/netlink.h`
  - 添加多播组定义宏

- [ ] 1.4 创建 netlink_generic.c
  - 文件: `net/netlink/netlink_generic.c`
  - 实现 `netlink_generic_sendto()` 函数
  - 实现命令分发机制

- [ ] 1.5 添加 Generic Netlink 控制器 (可选)
  - 实现 CTRL_CMD_GETFAMILY 等控制命令
  - 允许用户态查询支持的协议族

**依赖**: 无

**验证**: 能够创建 NETLINK_GENERIC socket 并发送/接收消息

---

### 阶段 2: nl80211 头文件定义 (预计 1 周)

**目标**: 定义 nl80211 命令和属性

**任务清单**:

- [ ] 2.1 创建 nl80211 头文件
  - 文件: `include/nuttx/wireless/nl80211.h`
  - 参考: Linux `include/uapi/linux/nl80211.h`

- [ ] 2.2 定义 nl80211 命令枚举
  ```c
  enum nl80211_commands {
      NL80211_CMD_UNSPEC,
      NL80211_CMD_GET_WIPHY,      /* 获取无线物理层信息 */
      NL80211_CMD_SET_WIPHY,      /* 设置无线物理层参数 */
      NL80211_CMD_NEW_WIPHY,      /* 新无线设备通知 */
      NL80211_CMD_DEL_WIPHY,      /* 无线设备移除通知 */
      NL80211_CMD_GET_INTERFACE,  /* 获取接口信息 */
      NL80211_CMD_SET_INTERFACE,  /* 设置接口参数 */
      NL80211_CMD_NEW_INTERFACE,  /* 创建新接口 */
      NL80211_CMD_DEL_INTERFACE,  /* 删除接口 */
      NL80211_CMD_GET_KEY,        /* 获取密钥 */
      NL80211_CMD_SET_KEY,        /* 设置密钥 */
      NL80211_CMD_NEW_KEY,        /* 新密钥 */
      NL80211_CMD_DEL_KEY,        /* 删除密钥 */
      NL80211_CMD_GET_BEACON,     /* 获取信标 */
      NL80211_CMD_SET_BEACON,     /* 设置信标 */
      NL80211_CMD_START_AP,       /* 启动 AP */
      NL80211_CMD_STOP_AP,        /* 停止 AP */
      NL80211_CMD_GET_STATION,    /* 获取站点信息 */
      NL80211_CMD_SET_STATION,    /* 设置站点参数 */
      NL80211_CMD_NEW_STATION,    /* 新站点连接 */
      NL80211_CMD_DEL_STATION,    /* 站点断开 */
      NL80211_CMD_GET_MPATH,      /* 获取 Mesh 路径 */
      NL80211_CMD_SET_MPATH,      /* 设置 Mesh 路径 */
      NL80211_CMD_NEW_MPATH,      /* 新 Mesh 路径 */
      NL80211_CMD_DEL_MPATH,      /* 删除 Mesh 路径 */
      NL80211_CMD_SET_BSS,        /* 设置 BSS 参数 */
      NL80211_CMD_SET_REG,        /* 设置监管域 */
      NL80211_CMD_REQ_SET_REG,    /* 请求设置监管域 */
      NL80211_CMD_GET_MESH_CONFIG,/* 获取 Mesh 配置 */
      NL80211_CMD_SET_MESH_CONFIG,/* 设置 Mesh 配置 */
      NL80211_CMD_SET_MGMT_EXTRA_IE, /* 设置管理帧额外 IE */
      NL80211_CMD_GET_REG,        /* 获取监管域 */
      NL80211_CMD_GET_SCAN,       /* 获取扫描结果 */
      NL80211_CMD_TRIGGER_SCAN,   /* 触发扫描 */
      NL80211_CMD_NEW_SCAN_RESULTS, /* 扫描结果通知 */
      NL80211_CMD_SCAN_ABORTED,   /* 扫描中止通知 */
      NL80211_CMD_REG_CHANGE,     /* 监管域变更通知 */
      NL80211_CMD_AUTHENTICATE,   /* 认证 */
      NL80211_CMD_ASSOCIATE,      /* 关联 */
      NL80211_CMD_DEAUTHENTICATE, /* 取消认证 */
      NL80211_CMD_DISASSOCIATE,   /* 取消关联 */
      NL80211_CMD_MICHAEL_MIC_FAILURE, /* MIC 失败通知 */
      NL80211_CMD_REG_BEACON_HINT,/* 信标提示 */
      NL80211_CMD_JOIN_IBSS,      /* 加入 IBSS */
      NL80211_CMD_LEAVE_IBSS,     /* 离开 IBSS */
      NL80211_CMD_TESTMODE,       /* 测试模式 */
      NL80211_CMD_CONNECT,        /* 连接到 AP */
      NL80211_CMD_ROAM,           /* 漫游 */
      NL80211_CMD_DISCONNECT,     /* 断开连接 */
      NL80211_CMD_SET_WIPHY_NETNS,/* 设置网络命名空间 */
      NL80211_CMD_GET_SURVEY,     /* 获取信道调查 */
      NL80211_CMD_NEW_SURVEY_RESULTS, /* 调查结果通知 */
      NL80211_CMD_SET_PMKSA,      /* 设置 PMKSA 缓存 */
      NL80211_CMD_DEL_PMKSA,      /* 删除 PMKSA 缓存 */
      NL80211_CMD_FLUSH_PMKSA,    /* 清空 PMKSA 缓存 */
      NL80211_CMD_REMAIN_ON_CHANNEL, /* 保持信道 */
      NL80211_CMD_CANCEL_REMAIN_ON_CHANNEL, /* 取消保持信道 */
      NL80211_CMD_SET_TX_BITRATE_MASK, /* 设置发送速率掩码 */
      NL80211_CMD_REGISTER_FRAME, /* 注册帧 */
      NL80211_CMD_FRAME,          /* 帧 */
      NL80211_CMD_FRAME_TX_STATUS,/* 帧发送状态 */
      NL80211_CMD_SET_POWER_SAVE, /* 设置省电模式 */
      NL80211_CMD_GET_POWER_SAVE, /* 获取省电模式 */
      NL80211_CMD_SET_CQM,        /* 设置连接质量监控 */
      NL80211_CMD_NOTIFY_CQM,     /* 连接质量通知 */
      NL80211_CMD_SET_CHANNEL,    /* 设置信道 */
      NL80211_CMD_SET_WDS_PEER,   /* 设置 WDS 对端 */
      NL80211_CMD_FRAME_WAIT_CANCEL, /* 取消帧等待 */
      NL80211_CMD_JOIN_MESH,      /* 加入 Mesh */
      NL80211_CMD_LEAVE_MESH,     /* 离开 Mesh */
      NL80211_CMD_UNEXPECTED_FRAME, /* 意外帧通知 */
      NL80211_CMD_GET_WOWLAN,     /* 获取 WoWLAN 配置 */
      NL80211_CMD_SET_WOWLAN,     /* 设置 WoWLAN 配置 */
      NL80211_CMD_START_SCHED_SCAN, /* 启动计划扫描 */
      NL80211_CMD_STOP_SCHED_SCAN,  /* 停止计划扫描 */
      NL80211_CMD_SCHED_SCAN_RESULTS, /* 计划扫描结果 */
      NL80211_CMD_SCHED_SCAN_STOPPED, /* 计划扫描停止 */
      NL80211_CMD_SET_REKEY_OFFLOAD, /* 设置重密钥卸载 */
      NL80211_CMD_PMKSA_CANDIDATE, /* PMKSA 候选通知 */
      NL80211_CMD_TDLS_OPER,      /* TDLS 操作 */
      NL80211_CMD_TDLS_MGMT,      /* TDLS 管理 */
      NL80211_CMD_UNEXPECTED_4ADDR_FRAME, /* 意外 4 地址帧 */
      NL80211_CMD_PROBE_CLIENT,   /* 探测客户端 */
      NL80211_CMD_REGISTER_BEACONS, /* 注册信标 */
      NL80211_CMD_SET_NOACK_MAP,  /* 设置 NOACK 映射 */
      NL80211_CMD_CH_SWITCH_NOTIFY, /* 信道切换通知 */
      NL80211_CMD_CH_SWITCH_STARTED_NOTIFY, /* 信道切换开始通知 */
      NL80211_CMD_CQM_RSSI_NOTIFY, /* RSSI 通知 */
      NL80211_CMD_MAX,            /* 最大命令号 */
  };
  ```

- [ ] 2.3 定义 nl80211 属性枚举
  ```c
  enum nl80211_attrs {
      NL80211_ATTR_UNSPEC,
      NL80211_ATTR_WIPHY,          /* 无线物理层索引 */
      NL80211_ATTR_WIPHY_NAME,     /* 无线物理层名称 */
      NL80211_ATTR_IFINDEX,        /* 接口索引 */
      NL80211_ATTR_IFNAME,         /* 接口名称 */
      NL80211_ATTR_IFTYPE,         /* 接口类型 */
      NL80211_ATTR_MAC,            /* MAC 地址 */
      NL80211_ATTR_KEY_DATA,       /* 密钥数据 */
      NL80211_ATTR_KEY_IDX,        /* 密钥索引 */
      NL80211_ATTR_KEY_CIPHER,     /* 密钥加密算法 */
      NL80211_ATTR_KEY_SEQ,        /* 密钥序列号 */
      NL80211_ATTR_KEY_DEFAULT,    /* 默认密钥标志 */
      NL80211_ATTR_BEACON_INTERVAL,/* 信标间隔 */
      NL80211_ATTR_DTIM_PERIOD,    /* DTIM 周期 */
      NL80211_ATTR_BEACON_HEAD,    /* 信标头 */
      NL80211_ATTR_BEACON_TAIL,    /* 信标尾 */
      NL80211_ATTR_STA_AID,        /* 站点 AID */
      NL80211_ATTR_STA_FLAGS,      /* 站点标志 */
      NL80211_ATTR_STA_LISTEN_INTERVAL, /* 站点监听间隔 */
      NL80211_ATTR_STA_SUPPORTED_RATES, /* 站点支持速率 */
      NL80211_ATTR_STA_VLAN,       /* 站点 VLAN */
      NL80211_ATTR_STA_INFO,       /* 站点信息 */
      NL80211_ATTR_WIPHY_BANDS,    /* 无线频段 */
      NL80211_ATTR_WIPHY_TX_POWER_LEVEL, /* 发射功率 */
      NL80211_ATTR_WIPHY_ANTENNA_TX, /* 发射天线 */
      NL80211_ATTR_WIPHY_ANTENNA_RX, /* 接收天线 */
      NL80211_ATTR_MPATH_NEXT_HOP, /* Mesh 下一跳 */
      NL80211_ATTR_MPATH_INFO,     /* Mesh 路径信息 */
      NL80211_ATTR_BSS_CTS_PROT,   /* BSS CTS 保护 */
      NL80211_ATTR_BSS_SHORT_PREAMBLE, /* BSS 短前导码 */
      NL80211_ATTR_BSS_SHORT_SLOT_TIME, /* BSS 短时隙 */
      NL80211_ATTR_HT_CAPABILITY,  /* HT 能力 */
      NL80211_ATTR_SUPPORTED_IFTYPES, /* 支持的接口类型 */
      NL80211_ATTR_REG_ALPHA2,     /* 监管域国家代码 */
      NL80211_ATTR_REG_RULES,      /* 监管规则 */
      NL80211_ATTR_MESH_CONFIG,    /* Mesh 配置 */
      NL80211_ATTR_BSS_BASIC_RATES, /* BSS 基本速率 */
      NL80211_ATTR_WIPHY_FREQ,     /* 频率 */
      NL80211_ATTR_WIPHY_CHANNEL_TYPE, /* 信道类型 */
      NL80211_ATTR_KEY_DEFAULT_MGMT, /* 默认管理密钥 */
      NL80211_ATTR_MGMT_SUBTYPE,   /* 管理帧子类型 */
      NL80211_ATTR_IE,             /* 信息元素 */
      NL80211_ATTR_MAX_NUM_SCAN_SSIDS, /* 最大扫描 SSID 数 */
      NL80211_ATTR_SCAN_FREQUENCIES, /* 扫描频率列表 */
      NL80211_ATTR_SCAN_SSIDS,     /* 扫描 SSID 列表 */
      NL80211_ATTR_GENERATION,     /* 代数 (用于缓存一致性) */
      NL80211_ATTR_BSS,            /* BSS 信息 */
      NL80211_ATTR_REG_INITIATOR,  /* 监管域发起者 */
      NL80211_ATTR_REG_TYPE,       /* 监管域类型 */
      NL80211_ATTR_SUPPORTED_COMMANDS, /* 支持的命令 */
      NL80211_ATTR_FRAME,          /* 帧数据 */
      NL80211_ATTR_SSID,           /* SSID */
      NL80211_ATTR_AUTH_TYPE,      /* 认证类型 */
      NL80211_ATTR_REASON_CODE,    /* 原因码 */
      NL80211_ATTR_KEY_TYPE,       /* 密钥类型 */
      NL80211_ATTR_MAX_SCAN_IE_LEN, /* 最大扫描 IE 长度 */
      NL80211_ATTR_CIPHER_SUITES,  /* 加密套件 */
      NL80211_ATTR_FREQ_BEFORE,    /* 切换前频率 */
      NL80211_ATTR_FREQ_AFTER,     /* 切换后频率 */
      NL80211_ATTR_CIPHER_SUITES_PAIRWISE, /* 成对加密套件 */
      NL80211_ATTR_CIPHER_SUITE_GROUP, /* 组加密套件 */
      NL80211_ATTR_WPA_VERSIONS,   /* WPA 版本 */
      NL80211_ATTR_AKM_SUITES,     /* AKM 套件 */
      NL80211_ATTR_PMKID,          /* PMKID */
      NL80211_ATTR_DURATION,       /* 持续时间 */
      NL80211_ATTR_COOKIE,         /* Cookie */
      NL80211_ATTR_WIPHY_COVERAGE_CLASS, /* 覆盖等级 */
      NL80211_ATTR_TX_RATES,       /* 发送速率 */
      NL80211_ATTR_FRAME_MATCH,    /* 帧匹配规则 */
      NL80211_ATTR_ACK,            /* ACK 标志 */
      NL80211_ATTR_PS_STATE,       /* 省电状态 */
      NL80211_ATTR_CQM,            /* 连接质量监控 */
      NL80211_ATTR_LOCAL_STATE_CHANGE, /* 本地状态变更 */
      NL80211_ATTR_AP_ISOLATE,     /* AP 隔离 */
      NL80211_ATTR_WIPHY_TX_POWER_SETTING, /* 发射功率设置 */
      NL80211_ATTR_WIPHY_ANTENNA_AVAIL_TX, /* 可用发射天线 */
      NL80211_ATTR_WIPHY_ANTENNA_AVAIL_RX, /* 可用接收天线 */
      NL80211_ATTR_WIPHY_ANTENNA_GAIN, /* 天线增益 */
      NL80211_ATTR_WIPHY_RTS_THRESHOLD, /* RTS 阈值 */
      NL80211_ATTR_WIPHY_FRAG_THRESHOLD, /* 分片阈值 */
      NL80211_ATTR_WIPHY_RETRY_SHORT, /* 短重试次数 */
      NL80211_ATTR_WIPHY_RETRY_LONG, /* 长重试次数 */
      NL80211_ATTR_WIPHY_COVERAGE_CLASS, /* 覆盖等级 */
      NL80211_ATTR_MAX_NUM_PMKIDS, /* 最大 PMKID 数 */
      NL80211_ATTR_STATUS_CODE,    /* 状态码 */
      NL80211_ATTR_CIPHER,         /* 加密算法 */
      NL80211_ATTR_MGMT_PROTOCOL,  /* 管理协议 */
      NL80211_ATTR_MAX,            /* 最大属性号 */
      __NL80211_ATTR_AFTER_LAST,
      NL80211_ATTR_MAX = __NL80211_ATTR_AFTER_LAST - 1
  };
  ```

- [ ] 2.4 定义接口类型枚举
  ```c
  enum nl80211_iftype {
      NL80211_IFTYPE_UNSPECIFIED,
      NL80211_IFTYPE_ADHOC,        /* IBSS */
      NL80211_IFTYPE_STATION,      /* 客户端 */
      NL80211_IFTYPE_AP,           /* 接入点 */
      NL80211_IFTYPE_AP_VLAN,      /* AP VLAN */
      NL80211_IFTYPE_WDS,          /* 无线分发系统 */
      NL80211_IFTYPE_MONITOR,      /* 监控模式 */
      NL80211_IFTYPE_MESH_POINT,   /* Mesh 点 */
      NL80211_IFTYPE_P2P_CLIENT,   /* P2P 客户端 */
      NL80211_IFTYPE_P2P_GO,       /* P2P 组所有者 */
      NL80211_IFTYPE_P2P_DEVICE,   /* P2P 设备 */
      NL80211_IFTYPE_OCB,          /* OCB 模式 */
      NL80211_IFTYPE_NAN,          /* NAN 模式 */
      NUM_NL80211_IFTYPES,
      NL80211_IFTYPE_MAX = NUM_NL80211_IFTYPES - 1
  };
  ```

- [ ] 2.5 定义认证类型枚举
  ```c
  enum nl80211_auth_type {
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
  ```

- [ ] 2.6 定义加密套件常量
  ```c
  #define NL80211_CIPHER_SUITE_WEP40      0x000FAC01
  #define NL80211_CIPHER_SUITE_WEP104     0x000FAC05
  #define NL80211_CIPHER_SUITE_TKIP       0x000FAC02
  #define NL80211_CIPHER_SUITE_CCMP       0x000FAC04
  #define NL80211_CIPHER_SUITE_GCMP       0x000FAC08
  #define NL80211_CIPHER_SUITE_GCMP_256   0x000FAC09
  #define NL80211_CIPHER_SUITE_CCMP_256   0x000FAC0A
  #define NL80211_CIPHER_SUITE_AES_CMAC   0x000FAC06
  #define NL80211_CIPHER_SUITE_BIP_GMAC_128 0x000FAC0B
  #define NL80211_CIPHER_SUITE_BIP_GMAC_256 0x000FAC0C
  #define NL80211_CIPHER_SUITE_BIP_CMAC_256 0x000FAC0D
  ```

**依赖**: 阶段 1

**验证**: 头文件编译通过，常量定义与 Linux 兼容

---

### 阶段 3: nl80211 核心实现 (预计 3 周)

**目标**: 实现 nl80211 命令处理

**任务清单**:

- [ ] 3.1 创建 nl80211 核心文件
  - 文件: `net/wireless/nl80211.c`
  - 实现 nl80211 协议注册

- [ ] 3.2 实现 nl80211 命令分发器
  ```c
  struct nl80211_cmd_handler {
      uint8_t cmd;
      int (*handler)(struct nl80211_context *ctx,
                     struct nlmsghdr *nlh,
                     struct genlmsghdr *gnlh,
                     struct nlattr **attrs);
  };
  ```

- [ ] 3.3 实现基础查询命令
  - [ ] NL80211_CMD_GET_WIPHY: 获取无线物理层信息
  - [ ] NL80211_CMD_GET_INTERFACE: 获取接口信息
  - [ ] NL80211_CMD_GET_REG: 获取监管域信息

- [ ] 3.4 实现扫描命令
  - [ ] NL80211_CMD_TRIGGER_SCAN: 触发扫描
  - [ ] NL80211_CMD_GET_SCAN: 获取扫描结果
  - [ ] NL80211_CMD_ABORT_SCAN: 中止扫描

- [ ] 3.5 实现连接命令
  - [ ] NL80211_CMD_CONNECT: 连接到 AP
  - [ ] NL80211_CMD_DISCONNECT: 断开连接
  - [ ] NL80211_CMD_ROAM: 漫游

- [ ] 3.6 实现认证命令
  - [ ] NL80211_CMD_AUTHENTICATE: 认证
  - [ ] NL80211_CMD_ASSOCIATE: 关联
  - [ ] NL80211_CMD_DEAUTHENTICATE: 取消认证
  - [ ] NL80211_CMD_DISASSOCIATE: 取消关联

- [ ] 3.7 实现密钥管理命令
  - [ ] NL80211_CMD_NEW_KEY: 添加密钥
  - [ ] NL80211_CMD_DEL_KEY: 删除密钥
  - [ ] NL80211_CMD_SET_KEY: 设置密钥

- [ ] 3.8 实现 AP 模式命令
  - [ ] NL80211_CMD_START_AP: 启动 AP
  - [ ] NL80211_CMD_STOP_AP: 停止 AP
  - [ ] NL80211_CMD_NEW_STATION: 新站点
  - [ ] NL80211_CMD_DEL_STATION: 删除站点

- [ ] 3.9 实现事件通知
  - [ ] NL80211_CMD_NEW_SCAN_RESULTS: 扫描结果通知
  - [ ] NL80211_CMD_CONNECT: 连接结果通知
  - [ ] NL80211_CMD_DISCONNECT: 断开通知
  - [ ] NL80211_CMD_MICHAEL_MIC_FAILURE: MIC 失败通知

**依赖**: 阶段 2

**验证**: 能够响应基本的 nl80211 命令

---

### 阶段 4: cfg80211 配置层实现 (预计 3 周)

**目标**: 实现 cfg80211 驱动接口层

**任务清单**:

- [ ] 4.1 创建 cfg80211 核心文件
  - 文件: `include/nuttx/wireless/cfg80211.h`
  - 文件: `net/wireless/cfg80211.c`

- [ ] 4.2 定义 cfg80211_ops 操作结构
  ```c
  struct cfg80211_ops {
      int (*add_virtual_intf)(struct wiphy *wiphy, const char *name,
                              enum nl80211_iftype type,
                              struct vif_params *params);
      int (*del_virtual_intf)(struct wiphy *wiphy, struct wireless_dev *wdev);
      int (*change_virtual_intf)(struct wiphy *wiphy,
                                 struct wireless_dev *wdev,
                                 enum nl80211_iftype type,
                                 struct vif_params *params);
      int (*add_key)(struct wiphy *wiphy, struct net_device *netdev,
                     uint8_t key_index, bool pairwise,
                     const uint8_t *mac_addr, struct key_params *params);
      int (*get_key)(struct wiphy *wiphy, struct net_device *netdev,
                     uint8_t key_index, bool pairwise,
                     const uint8_t *mac_addr, void *cookie,
                     void (*callback)(void *cookie, struct key_params*));
      int (*del_key)(struct wiphy *wiphy, struct net_device *netdev,
                     uint8_t key_index, bool pairwise,
                     const uint8_t *mac_addr);
      int (*set_default_key)(struct wiphy *wiphy, struct net_device *netdev,
                             uint8_t key_index, bool unicast, bool multicast);
      int (*set_default_mgmt_key)(struct wiphy *wiphy,
                                  struct net_device *netdev,
                                  uint8_t key_index);
      int (*start_ap)(struct wiphy *wiphy, struct net_device *netdev,
                      struct cfg80211_ap_settings *settings);
      int (*stop_ap)(struct wiphy *wiphy, struct net_device *netdev);
      int (*add_station)(struct wiphy *wiphy, struct net_device *netdev,
                         const uint8_t *mac, struct station_parameters *params);
      int (*del_station)(struct wiphy *wiphy, struct net_device *netdev,
                         struct station_del_parameters *params);
      int (*change_station)(struct wiphy *wiphy, struct net_device *netdev,
                            const uint8_t *mac,
                            struct station_parameters *params);
      int (*get_station)(struct wiphy *wiphy, struct net_device *netdev,
                         const uint8_t *mac, struct station_info *sinfo);
      int (*dump_station)(struct wiphy *wiphy, struct net_device *netdev,
                          int idx, uint8_t *mac, struct station_info *sinfo);
      int (*scan)(struct wiphy *wiphy, struct cfg80211_scan_request *request);
      void (*abort_scan)(struct wiphy *wiphy, struct wireless_dev *wdev);
      int (*auth)(struct wiphy *wiphy, struct net_device *netdev,
                  struct cfg80211_auth_request *req);
      int (*assoc)(struct wiphy *wiphy, struct net_device *netdev,
                   struct cfg80211_assoc_request *req);
      int (*deauth)(struct wiphy *wiphy, struct net_device *netdev,
                    struct cfg80211_deauth_request *req);
      int (*disassoc)(struct wiphy *wiphy, struct net_device *netdev,
                      struct cfg80211_disassoc_request *req);
      int (*connect)(struct wiphy *wiphy, struct net_device *netdev,
                     struct cfg80211_connect_params *sme);
      int (*disconnect)(struct wiphy *wiphy, struct net_device *netdev,
                        uint16_t reason_code);
      int (*set_wiphy_params)(struct wiphy *wiphy, uint32_t changed);
      int (*set_tx_power)(struct wiphy *wiphy, struct wireless_dev *wdev,
                          enum nl80211_tx_power_setting type, int mbm);
      int (*get_tx_power)(struct wiphy *wiphy, struct wireless_dev *wdev,
                          int *dbm);
      int (*set_power_mgmt)(struct wiphy *wiphy, struct net_device *netdev,
                            bool enabled, int timeout);
      int (*get_antenna)(struct wiphy *wiphy, uint32_t *tx_ant,
                         uint32_t *rx_ant);
      int (*set_antenna)(struct wiphy *wiphy, uint32_t tx_ant,
                         uint32_t rx_ant);
      int (*set_bitrate_mask)(struct wiphy *wiphy, struct net_device *netdev,
                              const uint8_t *peer,
                              const struct cfg80211_bitrate_mask *mask);
      int (*dump_survey)(struct wiphy *wiphy, struct net_device *netdev,
                         int idx, struct survey_info *info);
      int (*set_pmksa)(struct wiphy *wiphy, struct net_device *netdev,
                       struct cfg80211_pmksa *pmksa);
      int (*del_pmksa)(struct wiphy *wiphy, struct net_device *netdev,
                       struct cfg80211_pmksa *pmksa);
      int (*flush_pmksa)(struct wiphy *wiphy, struct net_device *netdev);
      int (*remain_on_channel)(struct wiphy *wiphy, struct wireless_dev *wdev,
                               struct ieee80211_channel *chan,
                               unsigned int duration, uint64_t *cookie);
      int (*cancel_remain_on_channel)(struct wiphy *wiphy,
                                      struct wireless_dev *wdev,
                                      uint64_t cookie);
      int (*mgmt_tx)(struct wiphy *wiphy, struct wireless_dev *wdev,
                     struct cfg80211_mgmt_tx_params *params, uint64_t *cookie);
      int (*mgmt_tx_cancel_wait)(struct wiphy *wiphy,
                                 struct wireless_dev *wdev, uint64_t cookie);
      int (*set_cqm_rssi_config)(struct wiphy *wiphy,
                                 struct net_device *netdev,
                                 int32_t rssi_thold, uint32_t rssi_hyst);
      int (*set_cqm_txe_config)(struct wiphy *wiphy,
                                struct net_device *netdev,
                                uint32_t rate, uint32_t pkts, uint32_t intvl);
      void (*update_ft_ies)(struct wiphy *wiphy, struct net_device *netdev,
                            struct cfg80211_update_ft_ies_params *ftie);
      int (*crit_protocol_start)(struct wiphy *wiphy,
                                 struct wireless_dev *wdev,
                                 enum nl80211_crit_proto_id protocol,
                                 uint16_t duration);
      void (*crit_protocol_stop)(struct wiphy *wiphy,
                                 struct wireless_dev *wdev);
      int (*set_coalesce)(struct wiphy *wiphy,
                          struct cfg80211_coalesce *coalesce);
      int (*channel_switch)(struct wiphy *wiphy, struct net_device *netdev,
                            struct cfg80211_csa_settings *params);
      int (*set_qos_map)(struct wiphy *wiphy, struct net_device *netdev,
                         struct cfg80211_qos_map *qos_map);
      int (*set_ap_chanwidth)(struct wiphy *wiphy, struct net_device *netdev,
                              struct cfg80211_chan_def *chandef);
      int (*add_tx_ts)(struct wiphy *wiphy, struct net_device *netdev,
                       uint8_t tsid, const uint8_t *peer,
                       uint8_t user_prio, uint16_t admitted_time);
      int (*del_tx_ts)(struct wiphy *wiphy, struct net_device *netdev,
                       uint8_t tsid, const uint8_t *peer);
      int (*tdls_oper)(struct wiphy *wiphy, struct net_device *netdev,
                       const uint8_t *peer, enum nl80211_tdls_operation oper);
      int (*tdls_mgmt)(struct wiphy *wiphy, struct net_device *netdev,
                       const uint8_t *peer, u8 action_code,
                       u8 dialog_token, u16 status_code, u32 peer_capability,
                       bool initiator, const u8 *buf, size_t len);
      int (*start_nan)(struct wiphy *wiphy, struct wireless_dev *wdev,
                       struct cfg80211_nan_conf *conf);
      void (*stop_nan)(struct wiphy *wiphy, struct wireless_dev *wdev);
      int (*add_nan_func)(struct wiphy *wiphy, struct wireless_dev *wdev,
                          struct cfg80211_nan_func *nan_func);
      void (*del_nan_func)(struct wiphy *wiphy, struct wireless_dev *wdev,
                           u64 cookie);
      int (*nan_change_conf)(struct wiphy *wiphy, struct wireless_dev *wdev,
                             struct cfg80211_nan_conf *conf, u32 changes);
      int (*set_multicast_to_unicast)(struct wiphy *wiphy,
                                       struct net_device *netdev,
                                       bool enabled);
      int (*get_txq_stats)(struct wiphy *wiphy, struct wireless_dev *wdev,
                           struct cfg80211_txq_stats *txqstats);
      int (*set_pmk)(struct wiphy *wiphy, struct net_device *netdev,
                     const struct cfg80211_pmk_conf *conf);
      int (*del_pmk)(struct wiphy *wiphy, struct net_device *netdev,
                     const u8 *aa);
      int (*external_auth)(struct wiphy *wiphy, struct net_device *netdev,
                           struct cfg80211_external_auth_params *params);
  };
  ```

- [ ] 4.3 实现 wiphy 结构和管理
  ```c
  struct wiphy {
      char name[WIPHY_NAME_MAX_LEN];
      uint32_t interface_modes;
      uint16_t max_scan_ssids;
      uint16_t max_sched_scan_ssids;
      uint16_t max_match_sets;
      uint16_t max_scan_ie_len;
      uint16_t max_sched_scan_ie_len;
      int32_t signal_type;
      uint8_t n_cipher_suites;
      const uint32_t *cipher_suites;
      uint8_t n_akm_suites;
      const uint32_t *akm_suites;
      struct ieee80211_supported_band **bands;
      struct cfg80211_ops *ops;
      /* ... */
  };
  ```

- [ ] 4.4 实现 wireless_dev 结构
  ```c
  struct wireless_dev {
      struct wiphy *wiphy;
      enum nl80211_iftype iftype;
      uint32_t identifier;
      struct net_device *netdev;
      struct list_head list;
      struct list_head mgmt_registrations;
      uint64_t cookie_counter;
      /* ... */
  };
  ```

- [ ] 4.5 实现扫描结果缓存
  ```c
  struct cfg80211_scan_request {
      struct wiphy *wiphy;
      struct net_device *wdev;
      uint8_t n_ssids;
      struct cfg80211_ssid *ssids;
      uint8_t n_channels;
      struct ieee80211_channel **channels;
      uint8_t ie_len;
      uint8_t *ie;
      bool no_cck;
      bool wiphy_freq_fixed;
      /* ... */
  };

  struct cfg80211_bss {
      struct wiphy *wiphy;
      struct ieee80211_channel *channel;
      const uint8_t *bssid;
      const uint8_t *ies;
      size_t ies_len;
      int32_t signal;
      uint16_t capability;
      uint16_t beacon_interval;
      /* ... */
  };
  ```

- [ ] 4.6 实现监管域管理
  ```c
  struct regulatory_request {
      char alpha2[2];
      enum nl80211_reg_initiator initiator;
      enum nl80211_user_reg_hint_type user_reg_hint_type;
      /* ... */
  };

  struct ieee80211_regdomain {
      char alpha2[2];
      uint8_t dfs_region;
      uint8_t n_reg_rules;
      struct ieee80211_reg_rule *reg_rules;
  };
  ```

- [ ] 4.7 实现 cfg80211 注册/注销函数
  ```c
  struct wiphy *wiphy_new(const struct cfg80211_ops *ops, size_t sizeof_priv);
  int wiphy_register(struct wiphy *wiphy);
  void wiphy_unregister(struct wiphy *wiphy);
  void wiphy_free(struct wiphy *wiphy);
  ```

- [ ] 4.8 实现事件通知函数
  ```c
  void cfg80211_scan_done(struct cfg80211_scan_request *request, bool aborted);
  void cfg80211_connect_result(struct net_device *netdev, const uint8_t *bssid,
                               const uint8_t *req_ie, size_t req_ie_len,
                               const uint8_t *resp_ie, size_t resp_ie_len,
                               int status, gfp_t gfp);
  void cfg80211_disconnected(struct net_device *netdev, uint16_t reason,
                             const uint8_t *ie, size_t ie_len, bool locally_generated,
                             gfp_t gfp);
  void cfg80211_ready_on_channel(struct wireless_dev *wdev, uint64_t cookie,
                                  struct ieee80211_channel *channel,
                                  unsigned int duration, gfp_t gfp);
  void cfg80211_remain_on_channel_expired(struct wireless_dev *wdev,
                                           uint64_t cookie,
                                           struct ieee80211_channel *channel,
                                           gfp_t gfp);
  void cfg80211_new_sta(struct net_device *netdev, const uint8_t *mac_addr,
                        struct station_info *sinfo, gfp_t gfp);
  void cfg80211_del_sta(struct net_device *netdev, const uint8_t *mac_addr,
                        gfp_t gfp);
  ```

**依赖**: 阶段 3

**验证**: 驱动可以注册到 cfg80211 层

---

### 阶段 5: 驱动适配层 (预计 2 周)

**目标**: 将现有 WiFi 驱动适配到 cfg80211 接口

**任务清单**:

- [ ] 5.1 创建 ESP32 WiFi cfg80211 驱动适配
  - 文件: `arch/xtensa/src/common/espressif/esp_cfg80211.c`
  - 实现 `cfg80211_ops` 回调函数
  - 连接现有 `esp_wifi_api` 到 cfg80211

- [ ] 5.2 创建 wifi_sim cfg80211 驱动适配
  - 文件: `drivers/net/wifi_sim_cfg80211.c`
  - 实现模拟器的 cfg80211 接口

- [ ] 5.3 实现驱动注册
  ```c
  static struct cfg80211_ops esp_cfg80211_ops = {
      .scan = esp_cfg80211_scan,
      .connect = esp_cfg80211_connect,
      .disconnect = esp_cfg80211_disconnect,
      .add_key = esp_cfg80211_add_key,
      .del_key = esp_cfg80211_del_key,
      .set_default_key = esp_cfg80211_set_default_key,
      .start_ap = esp_cfg80211_start_ap,
      .stop_ap = esp_cfg80211_stop_ap,
      .add_station = esp_cfg80211_add_station,
      .del_station = esp_cfg80211_del_station,
      /* ... */
  };
  ```

- [ ] 5.4 实现事件上报
  - 将 ESP-IDF WiFi 事件转换为 cfg80211 事件
  - 扫描完成事件
  - 连接/断开事件
  - 站点加入/离开事件

**依赖**: 阶段 4

**验证**: ESP32 WiFi 通过 cfg80211 层工作

---

### 阶段 6: 用户态工具支持 (预计 2 周)

**目标**: 支持 wpa_supplicant/hostapd

**任务清单**:

- [ ] 6.1 评估 libnl 移植可行性
  - 分析 Linux libnl 库依赖
  - 确定需要移植的组件

- [ ] 6.2 移植或实现简化版 libnl
  - 选项 A: 移植 Linux libnl
  - 选项 B: 实现简化版 Netlink 库

- [ ] 6.3 移植 wpa_supplicant
  - 配置 wpa_supplicant 使用 nl80211 驱动
  - 测试 WPA2-PSK 连接
  - 测试 WPA3-SAE 连接

- [ ] 6.4 移植 hostapd
  - 配置 hostapd 使用 nl80211 驱动
  - 测试 AP 模式

- [ ] 6.5 实现 iw 工具 (可选)
  - 移植 Linux iw 工具
  - 用于调试和测试

**依赖**: 阶段 5

**验证**: wpa_supplicant/hostapd 正常工作

---

### 阶段 7: 测试和文档 (预计 1 周)

**目标**: 完善测试和文档

**任务清单**:

- [ ] 7.1 编写单元测试
  - nl80211 命令解析测试
  - cfg80211 接口测试
  - 驱动适配测试

- [ ] 7.2 编写集成测试
  - 扫描测试
  - 连接测试
  - AP 模式测试

- [ ] 7.3 编写文档
  - API 文档
  - 驱动开发指南
  - 用户指南

- [ ] 7.4 编写 Kconfig 和 Makefile
  - 添加配置选项
  - 添加编译支持

**依赖**: 阶段 6

---

## 文件结构

```
nuttx/
├── include/
│   ├── nuttx/
│   │   └── wireless/
│   │       ├── nl80211.h          # nl80211 命令/属性定义
│   │       ├── cfg80211.h         # cfg80211 API 定义
│   │       └── ieee80211.h        # IEEE 802.11 定义 (已存在)
│   └── netpacket/
│       └── netlink.h              # Netlink 定义 (已存在, 需扩展)
│
├── net/
│   ├── netlink/
│   │   ├── netlink_generic.c      # Generic Netlink 实现 (新增)
│   │   └── ...
│   │
│   └── wireless/
│       ├── Kconfig                # 配置选项
│       ├── Makefile               # 编译文件
│       ├── nl80211.c              # nl80211 实现
│       ├── cfg80211.c             # cfg80211 实现
│       ├── reg.c                  # 监管域管理
│       ├── scan.c                 # 扫描管理
│       ├── sme.c                  # 站点管理实体
│       └── util.c                 # 工具函数
│
└── arch/
    └── xtensa/src/common/espressif/
        ├── esp_cfg80211.c         # ESP32 cfg80211 驱动适配
        └── ...

apps/
└── wireless/
    ├── wpa_supplicant/            # wpa_supplicant 移植
    ├── hostapd/                   # hostapd 移植
    └── iw/                        # iw 工具移植
```

## 配置选项

```
# Kconfig 新增选项

config NETLINK_GENERIC
    bool "Generic Netlink support"
    depends on NET_NETLINK
    default n
    help
      Generic Netlink is a protocol family that allows kernel modules
      to register their own families. This is required for nl80211.

config NL80211
    bool "nl80211 wireless configuration API"
    depends on NETLINK_GENERIC
    default n
    help
      nl80211 is the 802.11 netlink interface for wireless configuration.
      This is required for wpa_supplicant/hostapd support.

config CFG80211
    bool "cfg80211 wireless configuration API"
    depends on NL80211
    default n
    help
      cfg80211 is the configuration API for wireless drivers.
      It provides a unified interface for different WiFi hardware.

config CFG80211_DEBUG
    bool "cfg80211 debugging"
    depends on CFG80211
    default n
    help
      Enable debugging output for cfg80211.

config CFG80211_REGULATORY
    bool "cfg80211 regulatory domain support"
    depends on CFG80211
    default y
    help
      Enable regulatory domain management.

config CFG80211_DEFAULT_REGDOMAIN
    string "Default regulatory domain"
    depends on CFG80211_REGULATORY
    default "00"
    help
      Default regulatory domain (ISO 3166-1 alpha-2).

config WPA_SUPPLICANT
    bool "wpa_supplicant support"
    depends on NL80211
    default n
    help
      Enable wpa_supplicant for WPA/WPA2/WPA3 authentication.

config HOSTAPD
    bool "hostapd support"
    depends on NL80211
    default n
    help
      Enable hostapd for AP mode support.
```

## 进度跟踪

| 阶段 | 任务 | 状态 | 开始日期 | 完成日期 |
|------|------|------|----------|----------|
| 1 | NETLINK_GENERIC | ✅ 已完成 | 2026-04-29 | 2026-04-29 |
| 2 | nl80211 头文件 | ✅ 已完成 | 2026-04-29 | 2026-04-29 |
| 3 | nl80211 核心实现 | ✅ 已完成 | 2026-04-29 | 2026-04-29 |
| 4 | cfg80211 头文件 | ✅ 已完成 | 2026-04-29 | 2026-04-29 |
| 5 | cfg80211 核心实现 | ✅ 已完成 | 2026-04-29 | 2026-04-29 |
| 6 | ESP32 驱动适配 | ✅ 已完成 | 2026-04-29 | 2026-04-29 |
| 7 | WiFi 驱动适配 (SDIO/SPI) | ✅ 已完成 | 2026-04-29 | 2026-04-29 |
| 8 | 用户态工具 (libnl, wpa_supplicant) | ⬜ 未开始 | - | - |
| 9 | 测试和文档 | ⬜ 未开始 | - | - |

## 实现总结

### 阶段 1-7: 已完成

- ✅ **NETLINK_GENERIC** - 通用 Netlink 协议支持
- ✅ **nl80211** - 802.11 配置接口定义和实现
- ✅ **cfg80211** - 无线驱动配置 API 实现
- ✅ **ESP32 驱动核心** - cfg80211 适配层
- ✅ **ESP32 通信接口** - SDIO/SPI 通信层
- ✅ **集成配置** - Kconfig/Make.defs 配置文件
- ✅ **用户态接口** - wpa_supplicant 集成接口

### 阶段 8: 驱动集成与测试

- ⬜ **完整驱动测试** - 端到端功能验证
- ⬜ **性能优化** - 内存/性能调优
- ⬜ **文档完善** - 用户手册和开发者指南

## 技术细节

### 核心架构

```
用户空间应用 (wpa_supplicant/hostapd)
           ↓
    cfg80211/nl80211 API
           ↓
    ESP32 驱动适配层
           ↓
     SDIO/SPI 通信
           ↓
       ESP32 固件
```

### 实现特色

1. **Linux 兼容性**: 与 Linux nl80211/cfg80211 API 完全兼容
2. **模块化设计**: 可独立启用/禁用不同功能
3. **标准化接口**: 遵循 IEEE 802.11 标准
4. **轻量化**: 针对嵌入式系统优化

## 风险和挑战

1. **内存占用**: nl80211/cfg80211 层会增加内存占用，需要优化
2. **兼容性**: 需要与 Linux nl80211 保持 API 兼容
3. **性能**: Netlink 消息解析可能影响性能，需要优化
4. **测试覆盖**: 需要测试多种 WiFi 芯片和场景

## 参考资料

1. Linux nl80211 文档: https://wireless.wiki.kernel.org/en/developers/documentation/nl80211
2. Linux cfg80211 文档: https://www.kernel.org/doc/html/latest/driver-api/80211/cfg80211.html
3. wpa_supplicant 源码: https://w1.fi/cgit/hostap/
4. ESP-IDF WiFi 文档: https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/wifi.html

## 更新历史

| 日期 | 更新内容 |
|------|----------|
| 2026-04-29 | 初始版本，创建实现计划 |
| 2026-04-29 | 实现阶段1-5: NETLINK_GENERIC、nl80211、cfg80211 核心代码 |
| 2026-04-29 | 实现阶段6: ESP32 cfg80211 驱动适配层 |
| 2026-04-29 | 实现阶段7: ESP32 通信接口实现 (SDIO/SPI) |
| 2026-04-29 | 实现阶段8: 用户态工具接口定义 (wpa_supplicant) |
