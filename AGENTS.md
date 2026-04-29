# FHRE Repository Guide

## Build & Run
```bash
./nuttx/build.sh    # Build NuttX firmware with FHRE demo
./nuttx/nuttx       # Run simulator (then type 'fhre_rust' in nsh)
```

**X11 Required**: Simulator needs X11 display. If window shows no output, check X11 forwarding/display access.

## Architecture
- **Dual World Pattern** (Bevy-aligned): MainWorld (game logic) + RenderWorld (render commands)
- **Software Rendering**: No GPU dependencies, portable to embedded systems
- **no_std Compatible**: Uses `alloc` crate, works on bare metal

## Key Directories
- `apps/fhre/` - FHRE engine (ECS, rendering, animation)
- `apps/examples/fhre/` - Demo application
- `nuttx/` - NuttX RTOS integration

## FHRE Systems Execution Order
1. **Startup**: `setup_textures` → `setup` (spawn entities)
2. **PreUpdate**: `picking_system` → `button_interaction_system` → `input_system`
3. **Update**: `setup_animation` → `animation_control_system` → `model_switch_system` → `apply_animations`

## Critical Patterns
- **Texture Creation**: Use `images.add_with_event()` in `setup_textures()` for proper event handling
- **Model Switching**: Despawns old entities, spawns new with shared `AnimationClipHandle`
- **Animation**: `AnimationPlayer` component required on animatable entities

## Build Configuration
- Config: `sim:fhre` (NuttX simulator)
- Release: `opt-level = 'z'` (size optimization)
- Crate types: `staticlib`, `rlib` (embedded targets)

## Documentation

- [ARM Cortex-M55 and Cortex-M85 Support](docs/arm_cortex_m55_m85.md) - Implementation details for ARMv8.1-M CPUs
- [Renesas RA8P1 Porting Guide](docs/ra8p1_porting.md) - RA8P1 (Cortex-M85) porting documentation
- [STM32N6 Porting Guide](docs/stm32n6_porting.md) - STM32N6 porting documentation
- [Architecture](docs/ARCHITECTURE.md) - System architecture overview
- [FHRE Rendering Backend](docs/fhre_rendering_backend.md) - FHRE engine documentation
- [nl80211/cfg80211 Implementation Plan](docs/nl80211_cfg80211_implementation.md) - WiFi nl80211/cfg80211 architecture implementation

## WiFi nl80211/cfg80211 Implementation

**Goal**: Implement Linux-compatible WiFi architecture in NuttX to support:
- Standard nl80211 Netlink interface
- wpa_supplicant/hostapd user-space tools
- cfg80211 driver interface for WiFi chips

**Current Status**: Phase 1-8 Completed (Core Implementation)

**Key Components**:
| Component | Status | Description |
|-----------|--------|-------------|
| NETLINK_GENERIC | ✅ Implemented | Generic Netlink protocol |
| nl80211 | ✅ Implemented | 802.11 configuration interface |
| cfg80211 | ✅ Implemented | WiFi driver API layer |
| ESP32 Driver Core | ✅ Implemented | ESP32 cfg80211 driver adapter |
| ESP32 SDIO/SPI | ✅ Implemented | Low-level communication interface |
| ESP32 Events | ✅ Implemented | Event processing |
| ESP32 Commands | ✅ Implemented | Command protocol handling |
| wpa_supplicant Interface | ✅ Implemented | API interface for security tools |

**Implementation Plan**: See [docs/nl80211_cfg80211_implementation.md](docs/nl80211_cfg80211_implementation.md)

**Architecture Flow**:
```
wpa_supplicant/hostapd → libnl → NETLINK_GENERIC → nl80211 → cfg80211 → WiFi Driver
```

**Files Created**:
```
drivers/wireless/esp32/
├── esp32_wifi.c          # Core WiFi driver
├── esp32_sdio.c          # SDIO communication layer
├── esp32_spi.c           # SPI communication layer
├── esp_cfg80211.c        # cfg80211 operations
├── esp_cmd.c             # Command protocol handler
├── esp_event.c           # Event processor
├── esp_main.c            # Driver entry point
├── esp_cfg80211.h        # Driver header
├── esp_host_if.h         # Interface definitions
├── Kconfig              # Configuration options
└── Make.defs            # Build definitions

include/nuttx/wireless/
├── nl80211.h            # nl80211 definitions
├── cfg80211.h           # cfg80211 definitions
├── esp32_wifi.h         # ESP32 driver API
└── wpa_supplicant.h     # wpa_supplicant interface

examples/esp32_wifi_demo/
├── esp32_wifi_demo.c    # WiFi demo application
└── Makefile             # Build script
```