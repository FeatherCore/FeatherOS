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