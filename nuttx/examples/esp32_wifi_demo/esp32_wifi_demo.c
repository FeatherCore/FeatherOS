/****************************************************************************
 * examples/esp32_wifi_demo/esp32_wifi_demo.c
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
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <errno.h>
#include <debug.h>

#include <nuttx/wireless/esp32_wifi.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#ifdef CONFIG_DEBUG_WIRELESS_INFO
#  define demo_info(format, ...)  printf("demo: " format, ##__VA_ARGS__)
#else
#  define demo_info(format, ...)
#endif

#ifdef CONFIG_DEBUG_WIRELESS_ERROR
#  define demo_err(format, ...)   fprintf(stderr, "demo: " format, ##__VA_ARGS__)
#else
#  define demo_err(format, ...)
#endif

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct wifi_demo_context_s
{
  bool connected;                 /* Connection status */
};

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct wifi_demo_context_s g_wifi_demo_ctx;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: wifi_demo_usage
 *
 * Description:
 *   Show usage information.
 *
 ****************************************************************************/

static void wifi_demo_usage(const char *progname)
{
  printf("Usage: %s [command]\n", progname);
  printf("Commands:\n");
  printf("  init         - Initialize WiFi driver\n");
  printf("  scan         - Scan for available networks\n");
  printf("  connect SSID - Connect to specified network\n");
  printf("  disconnect   - Disconnect from network\n");
  printf("  status       - Show WiFi status\n");
  printf("  help         - Show this help message\n");
}

/****************************************************************************
 * Name: wifi_demo_init
 *
 * Description:
 *   Initialize WiFi driver and prepare for operations.
 *
 ****************************************************************************/

static int wifi_demo_init(void)
{
  int ret;

  demo_info("Initializing ESP32 WiFi driver\n");

  /* Initialize ESP32 WiFi driver */
  ret = esp32_wifi_initialize();
  if (ret < 0)
    {
      demo_err("Failed to initialize ESP32 WiFi driver: %d\n", ret);
      return ret;
    }

  demo_info("ESP32 WiFi driver initialized successfully\n");
  return OK;
}

/****************************************************************************
 * Name: wifi_demo_scan
 *
 * Description:
 *   Perform WiFi network scan.
 *
 ****************************************************************************/

static int wifi_demo_scan(void)
{
  int ret;

  demo_info("Initiating WiFi scan\n");

  /* Perform scan */
  ret = esp32_wifi_scan_request();
  if (ret < 0)
    {
      demo_err("WiFi scan failed: %d\n", ret);
      return ret;
    }

  demo_info("WiFi scan initiated\n");
  return OK;
}

/****************************************************************************
 * Name: wifi_demo_connect
 *
 * Description:
 *   Connect to specified WiFi network.
 *
 ****************************************************************************/

static int wifi_demo_connect(FAR const char *ssid)
{
  int ret;

  if (!ssid)
    {
      demo_err("SSID is required\n");
      return -EINVAL;
    }

  demo_info("Connecting to SSID: %s\n", ssid);

  /* Connect to network */
  ret = esp32_wifi_connect_request(ssid, strlen(ssid), NULL);
  if (ret < 0)
    {
      demo_err("WiFi connection failed: %d\n", ret);
      return ret;
    }

  g_wifi_demo_ctx.connected = true;
  demo_info("WiFi connection initiated\n");
  return OK;
}

/****************************************************************************
 * Name: wifi_demo_disconnect
 *
 * Description:
 *   Disconnect from current network.
 *
 ****************************************************************************/

static int wifi_demo_disconnect(void)
{
  int ret;

  demo_info("Disconnecting from WiFi network\n");

  /* Disconnect */
  ret = esp32_wifi_disconnect_request();
  if (ret < 0)
    {
      demo_err("WiFi disconnection failed: %d\n", ret);
      return ret;
    }

  g_wifi_demo_ctx.connected = false;
  demo_info("WiFi disconnected\n");
  return OK;
}

/****************************************************************************
 * Name: wifi_demo_status
 *
 * Description:
 *   Show current WiFi status.
 *
 ****************************************************************************/

static int wifi_demo_status(void)
{
  struct esp32_wifi_status_s wifi_status;
  int ret;

  ret = esp32_wifi_get_status(&wifi_status);
  if (ret < 0)
    {
      demo_err("Failed to get WiFi status: %d\n", ret);
      return ret;
    }

  printf("WiFi Status:\n");
  printf("  Connected: %s\n", wifi_status.connected ? "Yes" : "No");
  if (wifi_status.ssid_len > 0)
    {
      printf("  SSID: %.*s\n", (int)wifi_status.ssid_len, wifi_status.ssid);
    }
  printf("  BSSID: %02x:%02x:%02x:%02x:%02x:%02x\n",
         wifi_status.bssid[0], wifi_status.bssid[1], wifi_status.bssid[2],
         wifi_status.bssid[3], wifi_status.bssid[4], wifi_status.bssid[5]);
  printf("  RSSI: %d dBm\n", wifi_status.rssi);

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: esp32_wifi_demo_main
 *
 * Description:
 *   Main entry point for ESP32 WiFi demo application.
 *
 ****************************************************************************/

int main(int argc, char *argv[])
{
  int ret = OK;

  /* Parse command line arguments */

  if (argc < 2)
    {
      wifi_demo_usage(argv[0]);
      return EXIT_SUCCESS;
    }

  const char *command = argv[1];

  demo_info("ESP32 WiFi Demo Application\n");
  demo_info("===========================\n");

  if (strcmp(command, "help") == 0 || strcmp(command, "-h") == 0)
    {
      wifi_demo_usage(argv[0]);
      return EXIT_SUCCESS;
    }
  else if (strcmp(command, "init") == 0)
    {
      ret = wifi_demo_init();
    }
  else if (strcmp(command, "scan") == 0)
    {
      ret = wifi_demo_scan();
    }
  else if (strcmp(command, "connect") == 0)
    {
      if (argc < 3)
        {
          demo_err("SSID is required for connect command\n");
          wifi_demo_usage(argv[0]);
          return EXIT_FAILURE;
        }
      ret = wifi_demo_connect(argv[2]);
    }
  else if (strcmp(command, "disconnect") == 0)
    {
      ret = wifi_demo_disconnect();
    }
  else if (strcmp(command, "status") == 0)
    {
      ret = wifi_demo_status();
    }
  else
    {
      demo_err("Unknown command: %s\n", command);
      wifi_demo_usage(argv[0]);
      return EXIT_FAILURE;
    }

  if (ret < 0)
    {
      demo_err("Command failed: %d\n", ret);
      return EXIT_FAILURE;
    }

  demo_info("Command completed successfully\n");
  return EXIT_SUCCESS;
}

/****************************************************************************
 * Name: wifi_demo_init
 *
 * Description:
 *   Initialize WiFi driver and prepare for operations.
 *
 ****************************************************************************/

static int wifi_demo_init(void)
{
  int ret;

  demo_info("Initializing ESP32 WiFi driver\n");

  /* Initialize ESP32 WiFi driver */
  ret = esp32_wifi_initialize();
  if (ret < 0)
    {
      demo_err("Failed to initialize ESP32 WiFi driver: %d\n", ret);
      return ret;
    }

  demo_info("ESP32 WiFi driver initialized successfully\n");
  return OK;
}

/****************************************************************************
 * Name: wifi_demo_scan
 *
 * Description:
 *   Perform WiFi network scan.
 *
 ****************************************************************************/

static int wifi_demo_scan(void)
{
  int ret;

  demo_info("Initiating WiFi scan\n");

  /* Perform scan */
  ret = esp32_wifi_scan_request();
  if (ret < 0)
    {
      demo_err("WiFi scan failed: %d\n", ret);
      return ret;
    }

  demo_info("WiFi scan initiated\n");
  return OK;
}

/****************************************************************************
 * Name: wifi_demo_connect
 *
 * Description:
 *   Connect to specified WiFi network.
 *
 ****************************************************************************/

static int wifi_demo_connect(FAR const char *ssid)
{
  int ret;

  if (!ssid)
    {
      demo_err("SSID is required\n");
      return -EINVAL;
    }

  demo_info("Connecting to SSID: %s\n", ssid);

  /* Connect to network */
  ret = esp32_wifi_connect_request(ssid, strlen(ssid), NULL);
  if (ret < 0)
    {
      demo_err("WiFi connection failed: %d\n", ret);
      return ret;
    }

  g_wifi_demo_ctx.connected = true;
  demo_info("WiFi connection initiated\n");
  return OK;
}

/****************************************************************************
 * Name: wifi_demo_disconnect
 *
 * Description:
 *   Disconnect from current network.
 *
 ****************************************************************************/

static int wifi_demo_disconnect(void)
{
  int ret;

  demo_info("Disconnecting from WiFi network\n");

  /* Disconnect */
  ret = esp32_wifi_disconnect_request();
  if (ret < 0)
    {
      demo_err("WiFi disconnection failed: %d\n", ret);
      return ret;
    }

  g_wifi_demo_ctx.connected = false;
  demo_info("WiFi disconnected\n");
  return OK;
}

/****************************************************************************
 * Name: wifi_demo_status
 *
 * Description:
 *   Show current WiFi status.
 *
 ****************************************************************************/

static int wifi_demo_status(void)
{
  struct esp32_wifi_status_s wifi_status;
  int ret;

  ret = esp32_wifi_get_status(&wifi_status);
  if (ret < 0)
    {
      demo_err("Failed to get WiFi status: %d\n", ret);
      return ret;
    }

  printf("WiFi Status:\n");
  printf("  Connected: %s\n", wifi_status.connected ? "Yes" : "No");
  printf("  SSID: %.*s\n", (int)wifi_status.ssid_len, wifi_status.ssid);
  printf("  BSSID: %02x:%02x:%02x:%02x:%02x:%02x\n",
         wifi_status.bssid[0], wifi_status.bssid[1], wifi_status.bssid[2],
         wifi_status.bssid[3], wifi_status.bssid[4], wifi_status.bssid[5]);
  printf("  RSSI: %d dBm\n", wifi_status.rssi);

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: main
 *
 * Description:
 *   Main entry point for ESP32 WiFi demo application.
 *
 ****************************************************************************/

int main(int argc, char *argv[])
{
  int ret = OK;

  /* Parse command line arguments */

  if (argc < 2)
    {
      wifi_demo_usage(argv[0]);
      return EXIT_SUCCESS;
    }

  const char *command = argv[1];

  demo_info("ESP32 WiFi Demo Application\n");
  demo_info("===========================\n");

  if (strcmp(command, "help") == 0 || strcmp(command, "-h") == 0)
    {
      wifi_demo_usage(argv[0]);
      return EXIT_SUCCESS;
    }
  else if (strcmp(command, "init") == 0)
    {
      ret = wifi_demo_init();
    }
  else if (strcmp(command, "scan") == 0)
    {
      ret = wifi_demo_scan();
    }
  else if (strcmp(command, "connect") == 0)
    {
      if (argc < 3)
        {
          demo_err("SSID is required for connect command\n");
          wifi_demo_usage(argv[0]);
          return EXIT_FAILURE;
        }
      ret = wifi_demo_connect(argv[2]);
    }
  else if (strcmp(command, "disconnect") == 0)
    {
      ret = wifi_demo_disconnect();
    }
  else if (strcmp(command, "status") == 0)
    {
      ret = wifi_demo_status();
    }
  else
    {
      demo_err("Unknown command: %s\n", command);
      wifi_demo_usage(argv[0]);
      return EXIT_FAILURE;
    }

  if (ret < 0)
    {
      demo_err("Command failed: %d\n", ret);
      return EXIT_FAILURE;
    }

  demo_info("Command completed successfully\n");
  return EXIT_SUCCESS;
}