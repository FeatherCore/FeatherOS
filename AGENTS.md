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
- [Architecture](docs/ARCHITECTURE.md) - System architecture overview
- [FHRE Rendering Backend](docs/fhre_rendering_backend.md) - FHRE engine documentation