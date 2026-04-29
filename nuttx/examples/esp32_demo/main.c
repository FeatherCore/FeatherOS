/****************************************************************************
 * examples/esp32_demo/main.c
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
#include <unistd.h>
#include <string.h>
#include <errno.h>

#ifdef CONFIG_NET
#  include <net/if.h>
#  include <net/inet.h>
#  include <arpa/inet.h>
#endif

#include <nuttx/wireless/esp32_wifi.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define ESP32_EXAMPLE_SSID     "MyNetwork"
#define ESP32_EXAMPLE_PSK     "MyPassword"

#define ASCII_PROMPT   '>'
#define MAX_ARGS      3

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* Command handler type */

typedef CODE int (*cmd_handler_t)(int argc, FAR char *argv[]);

/* Command structure */

struct cmd_entry_s
{
  FAR const char *name;
  cmd_handler_t handler;
  FAR const char *usage;
};

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: cmd_help
 ****************************************************************************/

static int cmd_help(int argc, FAR char *argv[])
{
  UNUSED(argc);
  UNUSED(argv);

  printf("ESP32 WiFi Demo Commands:\n");
  printf("  init    - Initialize WiFi driver\n");
  printf("  scan    - Scan for networks\n");
  printf("  connect <ssid> [psk] - Connect to network\n");
  printf("  status  - Show connection status\n");
  printf("  disconnect - Disconnect from network\n");
  printf("  ifconfig - Show network interface\n");
  printf("  help   - Show this help message\n");
  return OK;
}

/****************************************************************************
 * Name: cmd_init
 ****************************************************************************/

static int cmd_init(int argc, FAR char *argv[])
{
  UNUSED(argc);
  UNUSED(argv);

  printf("Initializing ESP32 WiFi driver...\n");

  int ret = esp32_wifi_initialize();
  if (ret < 0)
    {
      printf("ERROR: Failed to initialize WiFi: %d\n", ret);
      return ret;
    }

  printf("ESP32 WiFi driver initialized\n");
  return OK;
}

/****************************************************************************
 * Name: cmd_scan
 ****************************************************************************/

#ifdef CONFIG_ESP32_WIFI_STA
static int cmd_scan(int argc, FAR char *argv[])
{
  UNUSED(argc);
  UNUSED(argv);

  printf("Scanning for networks...\n");

#ifdef CONFIG_ESP32_WIFI_STA_SCAN
  int ret = esp32_wifi_scan_request();
  if (ret < 0)
    {
      printf("ERROR: Scan failed: %d\n", ret);
      return ret;
    }

  printf("Scan initiated\n");
#else
  printf("Scan not supported\n");
#endif
  return OK;
}
#endif

/****************************************************************************
 * Name: cmd_connect
 ****************************************************************************/

#ifdef CONFIG_ESP32_WIFI_STA
static int cmd_connect(int argc, FAR char *argv[])
{
  FAR const char *ssid = NULL;
  FAR const char *psk = NULL;
  size_t ssid_len;
  int ret;

  UNUSED(psk);

  if (argc < 2)
    {
      ssid = ESP32_EXAMPLE_SSID;
#ifdef ESP32_EXAMPLE_PSK
      psk = ESP32_EXAMPLE_PSK;
#endif
    }
  else
    {
      ssid = argv[1];
      if (argc > 2)
        {
          psk = argv[2];
        }
    }

  ssid_len = strlen(ssid);
  printf("Connecting to SSID: %s\n", ssid);

  ret = esp32_wifi_connect_request(ssid, ssid_len, NULL);
  if (ret < 0)
    {
      printf("ERROR: Failed to connect: %d\n", ret);
      return ret;
    }

  printf("Connection initiated\n");
  return OK;
}
#endif

/****************************************************************************
 * Name: cmd_disconnect
 ****************************************************************************/

#ifdef CONFIG_ESP32_WIFI_STA
static int cmd_disconnect(int argc, FAR char *argv[])
{
  UNUSED(argc);
  UNUSED(argv);

  printf("Disconnecting...\n");

  int ret = esp32_wifi_disconnect_request();
  if (ret < 0)
    {
      printf("ERROR: Failed to disconnect: %d\n", ret);
      return ret;
    }

  printf("Disconnected\n");
  return OK;
}
#endif

/****************************************************************************
 * Name: cmd_status
 ****************************************************************************/

#ifdef CONFIG_ESP32_WIFI_STA
static int cmd_status(int argc, FAR char *argv[])
{
  struct esp32_wifi_status_s status;
  int ret;

  UNUSED(argc);
  UNUSED(argv);

  ret = esp32_wifi_get_status(&status);
  if (ret < 0)
    {
      printf("ERROR: Failed to get status: %d\n", ret);
      return ret;
    }

  printf("WiFi Status:\n");
  printf("  Connected: %s\n", status.connected ? "Yes" : "No");

  if (status.connected)
    {
      printf("  SSID: %.*s\n", status.ssid_len, status.ssid);
      printf("  RSSI: %d dBm\n", status.rssi);
      printf("  BSSID: %02x:%02x:%02x:%02x:%02x:%02x\n",
             status.bssid[0], status.bssid[1], status.bssid[2],
             status.bssid[3], status.bssid[4], status.bssid[5]);
    }

  return OK;
}
#endif

/****************************************************************************
 * Name: cmd_ifconfig
 ****************************************************************************/

#ifdef CONFIG_NET
static int cmd_ifconfig(int argc, FAR char *argv[])
{
  UNUSED(argc);
  UNUSED(argv);

  struct ifreq ifr;
  int sockfd;

  printf("Network Interfaces:\n");

  sockfd = socket(NET_AF_INET, NET_SOCK_DGRAM, 0);
  if (sockfd < 0)
    {
      printf("ERROR: socket failed: %d\n", errno);
      return sockfd;
    }

  strlcpy(ifr.ifr_name, "wlan0", IFNAMSIZ);
  if (ioctl(sockfd, SIOCGIFADDR, &ifr) < 0)
    {
      printf("  wlan0: not configured\n");
    }
  else
    {
      struct sockaddr_in *addr = (struct sockaddr_in *)&ifr.ifr_addr;
      printf("  wlan0: IP=%s\n", inet_ntoa(addr->sin_addr));
    }

  close(sockfd);
  return OK;
}
#else
static int cmd_ifconfig(int argc, FAR char *argv[])
{
  UNUSED(argc);
  UNUSED(argv);
  printf("Network support not enabled\n");
  return OK;
}
#endif

/****************************************************************************
 * Command table
 ****************************************************************************/

static const struct cmd_entry_s g_commands[] =
{
  {"init",       cmd_init,       "Initialize WiFi driver"},
  {"scan",       cmd_scan,       "Scan for networks"},
  {"connect",     cmd_connect,    "Connect to network"},
  {"status",     cmd_status,     "Show connection status"},
  {"disconnect", cmd_disconnect, "Disconnect"},
  {"ifconfig",   cmd_ifconfig,   "Show network interface"},
  {"help",       cmd_help,       "Show help"},
};

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: esp32_demo_main
 *
 * Description:
 *   Main entry point for ESP32 WiFi demo application.
 *
 ****************************************************************************/

int main(int argc, char *argv[])
{
  char line[128];
  char *args[MAX_ARGS];
  int nargs;
  size_t i;
  bool done = false;

  printf("\n");
  printf("ESP32 WiFi Demo Application\n");
  printf("============================\n");
  printf("\n");

  while (!done)
    {
      printf("%c ", ASCII_PROMPT);
      fflush(stdout);

      /* Read command line */

      if (fgets(line, sizeof(line), stdin) == NULL)
        {
          break;
        }

      /* Parse command */

      nargs = 0;
      char *ptr = line;
      while (*ptr && nargs < MAX_ARGS)
        {
          /* Skip whitespace */

          while (*ptr == ' ' || *ptr == '\t')
            {
              ptr++;
            }

          if (*ptr == '\n' || *ptr == '\0')
            {
              break;
            }

          /* Save argument start */

          args[nargs++] = ptr;

          /* Find end of argument */

          while (*ptr && *ptr != ' ' && *ptr != '\t' && *ptr != '\n')
            {
              ptr++;
            }

          if (*ptr == '\n' || *ptr == '\0')
            {
              *ptr = '\0';
              break;
            }

          *ptr++ = '\0';
        }

      if (nargs == 0)
        {
          continue;
        }

      /* Find and execute command */

      for (i = 0; i < sizeof(g_commands) / sizeof(g_commands[0]); i++)
        {
          if (strcmp(args[0], g_commands[i].name) == 0)
            {
              int ret = g_commands[i].handler(nargs - 1, &args[1]);
              UNUSED(ret);
              break;
            }
        }

      if (i >= sizeof(g_commands) / sizeof(g_commands[0]))
        {
          printf("Unknown command: %s\n", args[0]);
          cmd_help(0, NULL);
        }
    }

  printf("\nDemo finished\n");
  return 0;
}