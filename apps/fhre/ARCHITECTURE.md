# FHRE (Feather Hybrid Render Engine) Architecture

## Overview

FHRE is a lightweight ECS-based render engine inspired by Bevy's architecture, designed for embedded systems and minimal dependencies. It implements a subset of Bevy's patterns while maintaining no_std compatibility.

## Module Structure

```
fhre/
├── app/              - Application orchestrator (App, plugin management)
├── main_world/       - Main ECS world (entities, components, systems)
├── render_world/     - Render world (extracted data, render commands)
├── resources/        - Global resources (Time, Camera, etc.)
├── schedule/         - System scheduling and execution
├── plugin/           - Plugin system (Plugin, PluginGroup)
├── extract/          - Main→Render world extraction
├── pipeline/         - Rendering pipeline (software backend, batching)
├── node/             - Scene graph nodes (Transform, Camera, etc.)
├── animation/        - Animation system
├── event/            - Event system
├── picking/          - Input picking
└── math/             - Math types (Vec2, Vec3, Mat4, etc.)
```

## Core Architecture

### Dual World Pattern (Bevy-aligned)

FHRE follows Bevy's dual-world architecture:

1. **MainWorld** - Game logic, entity spawning, component manipulation
2. **RenderWorld** - Extracted render data, GPU commands

```
┌─────────────┐     extract()     ┌──────────────┐
│  MainWorld  │ ───────────────► │  RenderWorld │
│  (game)     │                  │  (render)    │
└─────────────┘                  └──────────────┘
```

### ECS Core

| Component | Description | Bevy Equivalent |
|-----------|-------------|-----------------|
| `Entity` | 64-bit entity identifier | `bevy_ecs::entity::Entity` |
| `Component` | Component trait | `bevy_ecs::component::Component` |
| `Query<T>` | Component query | `bevy_ecs::query::Query` |
| `Commands` | Deferred commands | `bevy_ecs::system::Commands` |
| `Res<T>`/`ResMut<T>` | Resource access | `bevy_ecs::system::Res/ResMut` |

### Schedule System

FHRE implements a simplified schedule system:

```
Startup → PreUpdate → Update → PostUpdate → Last
```

Compared to Bevy's full schedule:
```
First → PreStartup → Startup → PostStartup
     → PreUpdate → Update → PostUpdate
     → FixedPreUpdate → FixedUpdate → FixedPostUpdate
     → Last
```

## Unused Code (Reserved for Future)

The following items are defined but not yet used. They are reserved for future features:

### Commands System
- `InsertCommand::component_type_id` - Type tracking for debugging
- `InsertCommand::drop_fn` - Custom drop handling

### Change Detection (WIP)
- `MutState<T>` - State for `Mut<T>` SystemParam
- `RefState<T>` - State for `Ref<T>` SystemParam

**Status**: Change detection is partially implemented. Full implementation requires:
- Tick-based change tracking
- Integration with Query system
- Automatic tick updates on access

### Render World
- `RenderWorld::objects` - Direct object storage (currently uses ECS)
- `RenderWorld::viewport` - Viewport management
- `RenderWorld::phases` - Render phase execution
- `RenderWorld::use_phases` - Phase enable/disable flag

**Status**: Render phases are defined but not yet used for execution ordering.

### Schedule System
- `SystemSetConfig` - System set configuration
- `SystemSetConfig::new()` - Create config
- `SystemSetConfig::disabled()` - Disable set
- `SystemSetConfig::enabled()` - Enable set

**Status**: System sets are defined but configuration API is not connected.

### Pipeline/Batching
- `MAX_WAIT_TIME_MS` - Timeout for batch submission
- `GpuTaskCollector::last_submit_time` - Timing tracking
- `HybridScheduler::initialized` - Initialization flag
- `SoftwareBackend::build_blend_lut()` - Blend LUT builder

**Status**: Hybrid GPU/CPU scheduling is designed but not fully integrated.

### Timer
- `Timer` struct and all methods

**Status**: Timer is fully implemented but not used in current examples. Useful for:
- Animation timing
- Cooldowns
- Scheduled events

### Plugin System
- `Plugins::order` - Plugin execution order
- `Plugins::state` - Plugin state tracking
- `PluginGroupBuilder::group_name` - Group name for debugging

**Status**: Plugin ordering is tracked but not used for execution control.

## Comparison with Bevy

### What FHRE Has (Bevy-aligned)

| Feature | FHRE | Bevy | Notes |
|---------|------|------|-------|
| Entity-Component-System | ✅ | ✅ | Core ECS pattern |
| Dual World | ✅ | ✅ | MainWorld + RenderWorld |
| Plugin System | ✅ | ✅ | Plugin + PluginGroup |
| Schedule Labels | ✅ | ✅ | Stage-based execution |
| Commands | ✅ | ✅ | Deferred spawning |
| Resources | ✅ | ✅ | Global state |
| Query | ✅ | ✅ | Component iteration |
| Extract Pattern | ✅ | ✅ | Main→Render extraction |
| Animation | ✅ | ✅ | Animation curves/players |

### What FHRE Lacks (Not Implemented)

| Feature | FHRE | Bevy | Priority |
|---------|------|------|----------|
| Archetype Storage | ❌ | ✅ | High - Performance |
| Parallel Systems | ❌ | ✅ | Medium |
| Change Detection | ⚠️ | ✅ | High - WIP |
| Relations/Hierarchy | ❌ | ✅ | Medium |
| Observers | ❌ | ✅ | Low |
| Events (full) | ⚠️ | ✅ | Medium |
| Assets | ❌ | ✅ | High |
| Scene Loading | ❌ | ✅ | Medium |
| GPU Rendering | ⚠️ | ✅ | High - Software only |
| Windowing | ❌ | ✅ | External (SIM) |
| Input | ⚠️ | ✅ | Basic |
| UI | ⚠️ | ✅ | Basic |
| Audio | ❌ | ✅ | Low |
| Physics | ❌ | ✅ | External |
| Networking | ❌ | ✅ | External |

### Key Differences

1. **Storage Model**
   - Bevy: Archetype-based (components grouped by entity composition)
   - FHRE: Simple BTreeMap<TypeId, BTreeMap<EntityId, Component>>
   - Impact: FHRE is simpler but slower for large entity counts

2. **System Parameters**
   - Bevy: Full SystemParam derive macro, arbitrary tuples
   - FHRE: Manual impl for 1-9 parameters
   - Impact: FHRE requires explicit function wrappers

3. **no_std Support**
   - Bevy: Requires std
   - FHRE: no_std compatible (uses alloc)
   - Impact: FHRE works on embedded targets

4. **Render Backend**
   - Bevy: wgpu (Vulkan/Metal/DX12/WebGPU)
   - FHRE: Software rasterizer only
   - Impact: FHRE is portable but slower

5. **Dependencies**
   - Bevy: ~100 crates
   - FHRE: 0 external crates (self-contained)
   - Impact: FHRE is minimal but limited

## Design Decisions

### Why Simple Storage?

Archetype storage requires complex metadata and is optimized for cache-friendly iteration. For embedded systems with:
- Small entity counts (< 1000)
- Limited memory
- No SIMD

Simple BTreeMap storage is sufficient and easier to maintain.

### Why Software Rendering?

- No GPU driver dependencies
- Portable to any platform with a framebuffer
- Predictable performance
- Easier debugging

Future: Add optional GPU backend via wgpu or similar.

### Why no_std?

FeatherOS targets embedded systems where:
- No OS support
- Limited heap memory
- Static linking preferred

FHRE uses `alloc` crate for dynamic allocation but avoids std-only features.

## Future Work

### High Priority
1. **Change Detection** - Complete MutState/RefState implementation
2. **Asset Loading** - Image, mesh, animation data
3. **GPU Backend** - Optional wgpu integration
4. **Archetype Storage** - Performance optimization

### Medium Priority
1. **Parallel Systems** - Multi-threading support
2. **Full Event System** - Event readers/writers
3. **Hierarchy/Relations** - Parent-child relationships
4. **Scene Format** - Load/save scenes

### Low Priority
1. **Observers** - Event-driven architecture
2. **Hot Reloading** - Development experience
3. **Debug Tools** - Inspector, profiler

## References

- Bevy ECS: `/third/bevy/crates/bevy_ecs/`
- Bevy Render: `/third/bevy/crates/bevy_render/`
- Bevy App: `/third/bevy/crates/bevy_app/`
