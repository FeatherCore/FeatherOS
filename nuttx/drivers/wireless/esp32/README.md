# ESP32 cfg80211 WiFi Driver

## 概述

ESP32 cfg80211 WiFi 驱动为 NuttX RTOS 提供了完整的 ESP32 WiFi 支持。该驱动基于 Espressif 的 esp-hosted-ng 架构设计，允许 ESP32 作为协处理器通过 SDIO 或 SPI 接口与主控制器通信，并提供标准的 cfg80211 API 接口，支持 wpa_supplicant 和 hostapd 等用户空间工具。

## 特性

- **完整的 cfg80211 接口**: 支持标准的 Linux WiFi 配置 API
- **双接口支持**: SDIO 和 SPI 通信接口
- **多模式支持**: Station (STA) 和 Access Point (AP) 模式
- **安全协议**: WEP, WPA/WPA2 (PSK)
- **扫描功能**: 主动和被动扫描
- **电源管理**: 传输功率控制

## 文件结构

```
drivers/wireless/esp32/
├── esp32_main.c          # 主入口和初始化
├── esp32_netdev.c       # 网络设备接口
├── esp32_boot.c         # 固件启动协议
├── esp32_cfg80211.c     # cfg80211 操作实现
├── esp32_sdio.c         # SDIO 通信层
├── esp32_sdio_ops.c     # SDIO 接口操作
├── esp32_spi.c          # SPI 通信层
├── esp32_spi_ops.c      # SPI 接口操作
├── esp_cmd.c            # 命令协议处理
├── esp_event.c          # 事件处理
├── esp_cfg80211.h       # 内部定义和常量
├── esp_host_if.h        # 主机接口定义
├── Kconfig              # 配置选项
├── Make.defs           # 构建定义
└── README.md           # 本文档

nuttx/net/wireless/
├── cfg80211.c          # cfg80211 核心实现
└── nl80211.c          # nl80211 Netlink 接口

nuttx/include/nuttx/wireless/
├── cfg80211.h         # cfg80211 API 头文件
└── nl80211.h         # nl80211 定义头文件
```

## 配置选项

### 基本配置

```
CONFIG_ESP32_WIFI=y                 # 启用 ESP32 WiFi 驱动
CONFIG_ESP32_WIFI_SDIO=y           # 启用 SDIO 接口
CONFIG_ESP32_WIFI_SPI=n            # 启用 SPI 接口 (二选一)
```

### SDIO 配置

```
CONFIG_ESP32_WIFI_SDIO_DEVMINOR=0      # SDIO 设备次设备号
CONFIG_ESP32_WIFI_SDIO_FUNCTION=1      # SDIO 功能号
```

### SPI 配置

```
CONFIG_ESP32_WIFI_SPI_DEVMINOR=0       # SPI 设备次设备号
CONFIG_ESP32_WIFI_SPI_INTR_PIN=-1      # 中断 GPIO 引脚
CONFIG_ESP32_WIFI_SPI_CS_PIN=2         # 片选 GPIO 引脚
CONFIG_ESP32_WIFI_SPI_FREQ=10000000    # SPI 频率 (Hz)
```

### 功能配置

```
CONFIG_ESP32_WIFI_STA=y               # 启用 Station 模式
CONFIG_ESP32_WIFI_AP=n                # 启用 AP 模式
CONFIG_ESP32_WIFI_P2P=n               # 启用 P2P 模式
```

### 高级配置

```
CONFIG_ESP32_WIFI_NINTERFACES=2      # 最大接口数
CONFIG_ESP32_WIFI_RX_BUFFER_SIZE=2048 # 接收缓冲区大小
CONFIG_ESP32_WIFI_SCAN_TIMEOUT=10     # 扫描超时 (秒)
CONFIG_ESP32_WIFI_CONNECT_TIMEOUT=15 # 连接超时 (秒)
CONFIG_ESP32_WIFI_DEBUG=n            # 调试输出
```

## 已实现的 cfg80211 操作

| 操作 | 状态 | 说明 |
|------|------|------|
| add_virtual_intf | ✅ | 添加虚拟接口 |
| del_virtual_intf | ✅ | 删除虚拟接口 |
| change_virtual_intf | ✅ | 更改接口类型 |
| scan | ✅ | 触发扫描 |
| connect | ✅ | 连接到 AP |
| disconnect | ✅ | 断开连接 |
| add_key | ✅ | 添加加密密钥 |
| del_key | ✅ | 删除加密密钥 |
| set_default_key | ✅ | 设置默认密钥 |
| start_ap | ✅ | 启动 AP |
| stop_ap | ✅ | 停止 AP |
| change_beacon | ✅ | 更改信标 |
| add_station | ✅ | 添加站点 |
| del_station | ✅ | 删除站点 |
| set_tx_power | ✅ | 设置发射功率 |
| get_tx_power | ✅ | 获取发射功率 |
| mgmt_tx | ⏳ | 管理帧发送 (待完善) |
| get_station | ⏳ | 获取站点信息 (待完善) |

## ESP32 协处理器通信协议

### 命令类型

| 命令 | 代码 | 说明 |
|------|------|------|
| INIT_INTERFACE | 1 | 初始化接口 |
| SET_MAC | 2 | 设置 MAC 地址 |
| GET_MAC | 3 | 获取 MAC 地址 |
| SCAN_REQUEST | 4 | 扫描请求 |
| STA_CONNECT | 5 | 连接请求 |
| DISCONNECT | 6 | 断开连接 |
| ADD_KEY | 8 | 添加密钥 |
| DEL_KEY | 9 | 删除密钥 |
| SET_MODE | 22 | 设置模式 |
| AP_CONFIG | 24 | AP 配置 |

### 事件类型

| 事件 | 代码 | 说明 |
|------|------|------|
| SCAN_RESULT | 1 | 扫描结果 |
| STA_CONNECT | 2 | 连接成功 |
| STA_DISCONNECT | 3 | 断开连接 |
| AUTH_RX | 4 | 收到认证帧 |
| ASSOC_RX | 5 | 收到关联帧 |
| AP_MGMT_RX | 6 | AP 收到管理帧 |

## 数据包格式

### Payload Header

```
+--------+--------+--------+--------+--------+--------+--------+--------+
|IF_TYPE |IF_NUM  |FLAGS   |PKT_TYPE|RESERVE|  LEN  | OFFSET |CHECKSUM|
|  4bit  |  4bit  |  8bit  |  8bit  |  8bit  | 16bit  | 16bit  | 16bit  |
+--------+--------+--------+--------+--------+--------+--------+--------+
```

## 使用示例

### NuttX 配置

```bash
cd nuttx
make menuconfig
# 依次选择:
# Device Drivers -> Wireless LAN support -> ESP32 WiFi support
# Device Drivers -> Wireless LAN support -> SDIO interface support
# Networking support -> Wireless -> IEEE 802.11 Driver Support
```

### 初始化驱动

```c
#include <nuttx/wireless/esp32_wifi.h>

/* 初始化 ESP32 WiFi 驱动 */
int ret = esp32_wifi_initialize();
if (ret < 0) {
    printf("Failed to initialize ESP32 WiFi: %d\n", ret);
    return ret;
}
```

### 连接 WiFi

```c
/* 连接 WiFi 网络 */
const char *ssid = "MyNetwork";
const uint8_t *bssid = NULL;  /* 不指定 BSSID */

ret = esp32_wifi_connect(ssid, strlen(ssid), bssid);
if (ret < 0) {
    printf("Failed to connect: %d\n", ret);
    return ret;
}
```

### 扫描网络

```c
/* 扫描可用网络 */
ret = esp32_wifi_scan();
if (ret < 0) {
    printf("Failed to scan: %d\n", ret);
    return ret;
}
```

## 依赖关系

- **NuttX Netlink**: 用于 nl80211 通信
- **NuttX 网络子系统**: 用于网络设备注册
- **GPIO 驱动**: 用于硬件复位
- **SPI/SDIO 驱动**: 用于底层通信
- **cfg80211 子系统**: 提供标准化 WiFi 接口

## 测试状态

| 组件 | 状态 |
|------|------|
| 编译测试 | ✅ 通过 |
| 模块加载 | ✅ 通过 |
| 硬件通信 | ⏳ 待测试 |
| WiFi 扫描 | ⏳ 待测试 |
| WiFi 连接 | ⏳ 待测试 |
| 数据传输 | ⏳ 待测试 |

## 已知限制

1. **WPA3**: 暂不支持
2. **企业认证**: 暂不支持
3. **P2P 模式**: 功能待完善
4. **6GHz 频段**: 暂不支持

## 后续计划

1. 完善管理帧发送功能
2. 添加 WPA3 支持
3. 实现 OTA 固件更新
4. 优化数据传输性能
5. 添加蓝牙 LE 支持

## 参考资料

- [Espressif esp-hosted](https://github.com/espressif/esp-hosted)
- [Linux cfg80211 Documentation](https://www.kernel.org/doc/html/latest/wireless/cfg80211.html)
- [NuttX Wireless Subsystem](../wireless/index.html)