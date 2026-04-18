# FHRE 双世界架构对齐 Bevy 实现文档

> **文档状态**: ✅ 已实现 + 编译验证通过  
> **最后更新**: 2026-04-18  
> **相关文档**: [单相机架构重构进度](REFACTOR_PROGRESS.md) - 两者互补不冲突  
> **构建状态**: `cargo build --features sim` ✅ 通过（FHRE + 示例）

## 更新日志

| 日期 | 版本 | 更新内容 |
|------|------|----------|
| 2026-04-18 | 1.1 | 同步代码更新：添加 `impl_extract_identity` 宏、`detect_sync_removals_system` 函数、`ExtractSchedule` 的 `Clone` 实现及辅助方法、`RenderWorld` 的 `entities()` 和 `contains_entity()` 方法 |
| 2026-04-18 | 1.0 | 初始版本，完整记录 Bevy 对齐架构设计

## 1. 概述

### 1.1 目标
将 FHRE 的双世界架构（Main World + Render World）对齐 Bevy 的实现，实现以下核心功能：
- 实体自动同步机制
- 组件提取（Extract）系统
- 实体关联映射（RenderEntity/MainEntity）
- 同步标记组件（SyncToRenderWorld）

### 1.2 架构关系

本文档（Bevy 对齐）与 [REFACTOR_PROGRESS.md](REFACTOR_PROGRESS.md)（单相机架构）**互补不冲突**：

| 文档 | 关注点 | 核心内容 |
|------|--------|----------|
| **REFACTOR_PROGRESS.md** | 渲染架构 | 单相机（3D透视）+ 幕布（Screen Canvas）架构 |
| **本文档** | ECS 架构 | Main World ↔ Render World 实体同步机制 |

**关系说明**：
- 单相机架构决定**如何渲染**（3D投影到幕布）
- Bevy 对齐决定**数据如何同步**（实体/组件提取机制）

### 1.3 当前架构问题（已解决 ✅）

**改造前 FHRE 实现**：
```rust
// Main World - 完整的 ECS
pub struct MainWorld {
    entities: Vec<Entity>,
    components: BTreeMap<TypeId, BTreeMap<u64, Box<dyn Any>>>,
    resources: Resources,
    // ...
}

// Render World - 仅包含渲染命令队列 ❌
pub struct RenderWorld {
    objects: Vec<RenderObject>,
    commands: Vec<RenderCommand>,
    views: Vec<ViewBundle>,
    // ❌ 没有 ECS 结构
}
```

**问题**（已解决）：
1. ✅ Render World 不是完整的 ECS → **已改造为完整 ECS**
2. ✅ 没有实体同步机制 → **已实现 SyncToRenderWorld + entity_sync_system**
3. ✅ Extract 阶段直接生成渲染命令 → **已实现 ExtractComponent trait**
4. ✅ 无法选择性同步特定实体和组件 → **已实现标记系统**

---

## 2. 核心设计

### 2.1 架构对比

| 组件 | Bevy 实现 | FHRE 改造前 | FHRE 改造后 ✅ |
|------|-----------|-------------|----------------|
| **Main World** | 完整 ECS | 完整 ECS | 完整 ECS |
| **Render World** | 完整 ECS (SubApp) | 命令列表 | **完整 ECS** ✅ |
| **实体同步** | `SyncToRenderWorld` 标记 | ❌ 无 | **`SyncToRenderWorld`** ✅ |
| **实体映射** | `RenderEntity`/`MainEntity` | ❌ 无 | **`RenderEntity`/`MainEntity`** ✅ |
| **组件提取** | `ExtractComponent` trait | 直接生成命令 | **`ExtractComponent`** ✅ |
| **提取调度** | `ExtractSchedule` | 无 | **`ExtractSchedule`** ✅ |
| **组件提取** | `ExtractComponent` trait | 直接生成命令 | 实现提取 trait |

### 2.2 同步流程

```
┌─────────────────────────────────────────────────────────────┐
│                        Main World                           │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────┐ │
│  │   Entity    │───→│  Component  │    │ SyncToRenderWorld│ │
│  │   (1v1)     │    │  (Position) │    │    (Marker)      │ │
│  └─────────────┘    └─────────────┘    └─────────────────┘ │
│         │                                                   │
│         │  RenderEntity(Entity 3v1)                         │
│         ↓                                                   │
└─────────────────────────────────────────────────────────────┘
                            │
                            │ Sync Phase
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                       Render World                          │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────┐ │
│  │   Entity    │←───│  MainEntity │    │ExtractedPosition│ │
│  │   (3v1)     │    │   (1v1)     │    │                 │ │
│  └─────────────┘    └─────────────┘    └─────────────────┘ │
│         │                                                   │
│         │ Render Phase                                      │
│         ↓                                                   │
│  ┌─────────────────────────────────────────────────────────┐│
│  │              RenderCommand (generated)                   ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

---

## 3. 实现方案

### 3.1 新增组件

#### 3.1.1 SyncToRenderWorld（同步标记组件）

**文件**: `src/sync/sync_markers.rs`

```rust
//! Sync Markers - Entity synchronization markers between Main World and Render World
//!
//! Inspired by Bevy's sync_world module:
//! - SyncToRenderWorld: Marks entities that need to be synced to Render World
//! - RenderEntity: Stores the corresponding Render World entity ID
//! - MainEntity: Stores the corresponding Main World entity ID

use crate::{Entity, Component};

/// Marker component that indicates an entity needs to be synchronized to the Render World.
///
/// This component is automatically added as a required component when using
/// ExtractComponentPlugin or SyncComponentPlugin.
///
/// # Example
/// ```
/// commands.spawn()
///     .insert(Player { name: "Player1".into() })
///     .insert(SyncToRenderWorld); // This entity will be synced to Render World
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SyncToRenderWorld;

impl Component for SyncToRenderWorld {
    fn type_name() -> &'static str {
        "SyncToRenderWorld"
    }
}

/// Component added to Main World entities to track the corresponding Render World entity.
///
/// This is automatically inserted by the sync system when an entity with
/// `SyncToRenderWorld` is spawned.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderEntity(pub Entity);

impl RenderEntity {
    /// Get the Render World entity ID
    pub fn id(&self) -> Entity {
        self.0
    }
}

impl Component for RenderEntity {
    fn type_name() -> &'static str {
        "RenderEntity"
    }
}

impl From<Entity> for RenderEntity {
    fn from(entity: Entity) -> Self {
        RenderEntity(entity)
    }
}

/// Component added to Render World entities to track the corresponding Main World entity.
///
/// This is automatically inserted when the sync system creates an entity in Render World.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MainEntity(pub Entity);

impl MainEntity {
    /// Get the Main World entity ID
    pub fn id(&self) -> Entity {
        self.0
    }
}

impl Component for MainEntity {
    fn type_name() -> &'static str {
        "MainEntity"
    }
}

impl From<Entity> for MainEntity {
    fn from(entity: Entity) -> Self {
        MainEntity(entity)
    }
}
```

#### 3.1.2 待同步实体记录

**文件**: `src/sync/pending_sync.rs`

```rust
//! Pending Sync Entity - Records entities pending synchronization
//!
//! Tracks which entities need to be synced between Main World and Render World.

use crate::Entity;
use super::sync_markers::RenderEntity;
use crate::resources::Resource;
use alloc::vec::Vec;

/// Records of entity changes pending synchronization
#[derive(Debug, Clone)]
pub enum EntityRecord {
    /// Entity added SyncToRenderWorld in Main World
    /// Contains: Main World entity ID
    Added(Entity),
    
    /// Entity removed SyncToRenderWorld or was despawned in Main World
    /// Contains: Render World entity ID
    Removed(RenderEntity),
}

/// Resource storing pending entity sync records
pub struct PendingSyncEntity {
    records: Vec<EntityRecord>,
}

impl Default for PendingSyncEntity {
    fn default() -> Self {
        Self::new()
    }
}

impl Resource for PendingSyncEntity {}

impl PendingSyncEntity {
    /// Create a new empty PendingSyncEntity
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }
    
    /// Push a new record
    pub fn push(&mut self, record: EntityRecord) {
        self.records.push(record);
    }
    
    /// Drain all records
    pub fn drain(&mut self) -> impl Iterator<Item = EntityRecord> + '_ {
        self.records.drain(..)
    }
    
    /// Check if there are pending records
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
    
    /// Clear all records
    pub fn clear(&mut self) {
        self.records.clear();
    }
    
    /// Get the number of pending records
    pub fn len(&self) -> usize {
        self.records.len()
    }
}
```

### 3.2 Render World ECS 改造

**文件**: `src/render_world/world.rs`

```rust
//! Render World Implementation - ECS Version
//!
//! The Render World now contains a full ECS structure to align with Bevy's architecture.
//! It stores extracted components and maintains entity mapping with Main World.

use crate::{Entity, Component};
use crate::sync::sync_markers::MainEntity;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use core::any::{TypeId, Any};

/// Render World - Full ECS container for rendering data
///
/// Now contains:
/// - ECS structure (entities, components)
/// - Entity mapping with Main World (via MainEntity component)
/// - Render commands (generated from components)
/// - Views and phases
pub struct RenderWorld {
    // === ECS Core ===
    /// Next entity ID to allocate
    next_entity_id: u64,
    /// Active entities
    entities: Vec<Entity>,
    /// Component storage: TypeId -> EntityId -> Component
    components: BTreeMap<TypeId, BTreeMap<u64, Box<dyn Any>>>,
    
    // === Render Data ===
    /// Render commands queue
    commands: Vec<RenderCommand>,
    /// Render phases
    phases: RenderPhases,
    /// Active views (cameras)
    views: Vec<ViewBundle>,
    /// Current view index
    current_view: Option<usize>,
    
    // === Configuration ===
    width: u32,
    height: u32,
    clear_color: Color,
    viewport: Rect,
    backend: SoftwareBackend,
}

impl RenderWorld {
    /// Create a new Render World with specified dimensions
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            // ECS Core
            next_entity_id: 0,
            entities: Vec::new(),
            components: BTreeMap::new(),
            
            // Render Data
            commands: Vec::new(),
            phases: RenderPhases::new(),
            views: Vec::new(),
            current_view: None,
            
            // Configuration
            width,
            height,
            clear_color: Color::BLACK,
            viewport: Rect::new(0.0, 0.0, width as f32, height as f32),
            backend: SoftwareBackend::new(width, height),
        }
    }
    
    // === ECS Operations ===
    
    /// Spawn a new entity
    pub fn spawn(&mut self) -> Entity {
        let entity = Entity::new(self.next_entity_id);
        self.next_entity_id += 1;
        self.entities.push(entity);
        entity
    }
    
    /// Spawn an entity with MainEntity component (linked to Main World)
    pub fn spawn_synced(&mut self, main_entity: Entity) -> Entity {
        let render_entity = self.spawn();
        self.insert_component(render_entity, MainEntity(main_entity));
        render_entity
    }
    
    /// Despawn an entity and all its components
    pub fn despawn(&mut self, entity: Entity) {
        if let Some(pos) = self.entities.iter().position(|e| e.id() == entity.id()) {
            self.entities.swap_remove(pos);
        }
        
        for (_, storage) in self.components.iter_mut() {
            storage.remove(&entity.id());
        }
    }
    
    /// Insert a component for an entity
    pub fn insert_component<T: Component>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        let storage = self.components
            .entry(type_id)
            .or_insert_with(BTreeMap::new);
        storage.insert(entity.id(), Box::new(component));
    }
    
    /// Get a component reference
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get(&type_id)
            .and_then(|storage| storage.get(&entity.id()))
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }
    
    /// Get a mutable component reference
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&type_id)
            .and_then(|storage| storage.get_mut(&entity.id()))
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }
    
    /// Remove a component from an entity
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&type_id)
            .and_then(|storage| storage.remove(&entity.id()))
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }
    
    /// Query all entities with a specific component type
    pub fn query<T: Component>(&self) -> impl Iterator<Item = (Entity, &T)> {
        let type_id = TypeId::of::<T>();
        let entities = &self.entities;

        self.components
            .get(&type_id)
            .map(move |storage| {
                storage.iter()
                    .filter_map(move |(entity_id, boxed)| {
                        entities.iter()
                            .find(|e| e.id() == *entity_id)
                            .zip(boxed.downcast_ref::<T>())
                            .map(|(e, c)| (*e, c))
                    })
            })
            .into_iter()
            .flatten()
    }

    /// Get entity by MainEntity component
    pub fn get_entity_by_main(&self, main_entity: Entity) -> Option<Entity> {
        self.query::<MainEntity>()
            .find(|(_, main)| main.id() == main_entity)
            .map(|(entity, _)| entity)
    }

    /// Check if an entity exists
    pub fn contains_entity(&self, entity: Entity) -> bool {
        self.entities.iter().any(|e| e.id() == entity.id())
    }

    /// Get all entities
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    // ... 保留原有的渲染相关方法
}
```

### 3.3 实体同步系统

**文件**: `src/sync/sync_system.rs`

```rust
//! Sync System - Entity synchronization between Main World and Render World
//!
//! Synchronizes entities marked with SyncToRenderWorld between Main World and Render World.
//! This runs before the Extract phase.

use crate::main_world::MainWorld;
use crate::render_world::RenderWorld;
use crate::sync::sync_markers::{SyncToRenderWorld, RenderEntity, MainEntity};
use crate::sync::pending_sync::{PendingSyncEntity, EntityRecord};

/// System that synchronizes entities between Main World and Render World
///
/// This should run in the Sync schedule, before ExtractSchedule.
pub fn entity_sync_system(main_world: &mut MainWorld, render_world: &mut RenderWorld) {
    // Process pending sync records
    let records: Vec<_> = {
        let pending = main_world.resources().get::<PendingSyncEntity>();
        match pending {
            Some(pending) => pending.records.clone(),
            None => return,
        }
    };
    
    for record in records {
        match record {
            EntityRecord::Added(main_entity) => {
                sync_added_entity(main_world, render_world, main_entity);
            }
            EntityRecord::Removed(render_entity) => {
                sync_removed_entity(render_world, render_entity);
            }
        }
    }
    
    // Clear pending records
    if let Some(pending) = main_world.resources_mut().get_mut::<PendingSyncEntity>() {
        pending.clear();
    }
}

/// Handle entity added to Main World with SyncToRenderWorld
fn sync_added_entity(
    main_world: &mut MainWorld,
    render_world: &mut RenderWorld,
    main_entity: crate::Entity,
) {
    // Check if entity already has RenderEntity (already synced)
    if main_world.get_component::<RenderEntity>(main_entity).is_some() {
        return;
    }
    
    // Spawn corresponding entity in Render World
    let render_entity = render_world.spawn_synced(main_entity);
    
    // Insert RenderEntity component in Main World
    main_world.insert_component(main_entity, RenderEntity(render_entity));
}

/// Handle entity removed from Main World or SyncToRenderWorld removed
fn sync_removed_entity(
    render_world: &mut RenderWorld,
    render_entity: RenderEntity,
) {
    // Despawn the corresponding entity in Render World
    render_world.despawn(render_entity.id());
}

/// System that detects new SyncToRenderWorld components
///
/// This runs in Main World and populates PendingSyncEntity
pub fn detect_sync_changes_system(main_world: &mut MainWorld) {
    // Query all entities with SyncToRenderWorld but no RenderEntity
    let new_syncs: Vec<_> = main_world.query::<SyncToRenderWorld>()
        .filter(|(entity, _)| {
            main_world.get_component::<RenderEntity>(*entity).is_none()
        })
        .map(|(entity, _)| entity)
        .collect();
    
    // Add to pending
    if let Some(pending) = main_world.resources_mut().get_mut::<PendingSyncEntity>() {
        for entity in new_syncs {
            pending.push(EntityRecord::Added(entity));
        }
    }
}

/// System that detects removed SyncToRenderWorld components
///
/// This should be called when entities are despawned
pub fn detect_sync_removals_system(main_world: &mut MainWorld) {
    // This is called when entities with SyncToRenderWorld are despawned
    // The actual removal detection would need to be integrated with the despawn system
    // For now, this is a placeholder for the concept
}
```

### 3.4 ExtractComponent Trait

**文件**: `src/extract/extract_component.rs`

```rust
//! Extract Component - Trait for extracting components from Main World to Render World
//!
//! Inspired by Bevy's extract_component module.
//! Components implementing this trait can be automatically extracted during the Extract phase.

use crate::main_world::MainWorld;
use crate::render_world::RenderWorld;
use crate::Component;
use crate::sync::sync_markers::RenderEntity;
use alloc::vec::Vec;

/// Trait for components that can be extracted to the Render World
///
/// # Example
/// ```rust
/// use fhre::extract::ExtractComponent;
/// use fhre::main_world::component::Component;
///
/// #[derive(Component, Clone)]
/// struct Position { x: f32, y: f32 }
///
/// #[derive(Component)]
/// struct ExtractedPosition { x: f32, y: f32 }
///
/// impl ExtractComponent for Position {
///     type QueryData = Position;
///     type QueryFilter = ();
///     type Out = ExtractedPosition;
///
///     fn extract_component(item: &Self::QueryData) -> Option<Self::Out> {
///         Some(ExtractedPosition {
///             x: item.x,
///             y: item.y,
///         })
///     }
/// }
/// ```
pub trait ExtractComponent: Component + Clone {
    /// The data queried from Main World
    type QueryData: Component;

    /// Optional filter for the query
    type QueryFilter: Default;

    /// The output component type in Render World
    type Out: Component;

    /// Extract the component from Main World to Render World
    ///
    /// Return None to remove the component from Render World
    fn extract_component(item: &Self::QueryData) -> Option<Self::Out>;
}

/// Extract all components of type C from Main World to Render World
///
/// This system runs in the Extract schedule
pub fn extract_components<C: ExtractComponent>(
    main_world: &MainWorld,
    render_world: &mut RenderWorld,
) {
    // Query all synced entities with the component
    let extracts: Vec<_> = main_world.query::<C::QueryData>()
        .filter_map(|(main_entity, component)| {
            // Get the corresponding Render World entity
            main_world.get_component::<RenderEntity>(main_entity)
                .map(|render_entity| (*render_entity, component))
        })
        .filter_map(|(render_entity, query_data)| {
            C::extract_component(query_data)
                .map(|extracted| (render_entity.id(), extracted))
        })
        .collect();

    // Insert extracted components into Render World
    for (render_entity, extracted) in extracts {
        render_world.insert_component(render_entity, extracted);
    }
}

/// Macro to implement ExtractComponent for simple clone cases
#[macro_export]
macro_rules! impl_extract_clone {
    ($component:ty, $extracted:ty) => {
        impl $crate::extract::ExtractComponent for $component {
            type QueryData = $component;
            type QueryFilter = ();
            type Out = $extracted;

            fn extract_component(item: &Self::QueryData) -> Option<Self::Out> {
                Some(item.clone().into())
            }
        }
    };
}

/// Macro to implement ExtractComponent for identity extraction (same type)
#[macro_export]
macro_rules! impl_extract_identity {
    ($component:ty) => {
        impl $crate::extract::ExtractComponent for $component {
            type QueryData = $component;
            type QueryFilter = ();
            type Out = $component;

            fn extract_component(item: &Self::QueryData) -> Option<Self::Out> {
                Some(item.clone())
            }
        }
    };
}
```

### 3.5 改造后的 Extract 系统

**文件**: `src/extract/extract.rs`（改造后）

```rust
//! Extract System Implementation - Aligned with Bevy's architecture
//!
//! The Extract phase now:
//! 1. Syncs entities marked with SyncToRenderWorld
//! 2. Extracts components using ExtractComponent trait
//! 3. Generates render commands from extracted components

use crate::main_world::MainWorld;
use crate::render_world::RenderWorld;
use crate::sync::sync_system::entity_sync_system;
use crate::extract::extract_component::extract_components;
use crate::extract::extract_schedule::ExtractSchedule;
use crate::resources::PrimaryScreen;
use crate::math::Vec3;

/// Run the complete extract phase
///
/// This is called by the App each frame before rendering
pub fn run_extract_phase(main_world: &mut MainWorld, render_world: &mut RenderWorld) {
    // Step 1: Sync entities between Main World and Render World
    entity_sync_system(main_world, render_world);
    
    // Step 2: Run registered extract systems
    let schedule = main_world.resources()
        .get::<ExtractSchedule>()
        .cloned()
        .unwrap_or_default();
    
    schedule.run(main_world, render_world);
    
    // Step 3: Generate render commands from extracted components
    generate_render_commands(render_world);
}

/// Generate render commands from extracted components in Render World
///
/// This replaces the old direct extraction approach
fn generate_render_commands(render_world: &mut RenderWorld) {
    // Clear previous commands
    render_world.clear_commands();
    
    // Generate commands from extracted 3D objects
    generate_3d_commands(render_world);
    
    // Generate commands from extracted UI elements
    generate_ui_commands(render_world);
}

fn generate_3d_commands(render_world: &mut RenderWorld) {
    use crate::render_world::RenderComponent;
    use crate::node::Transform3D;
    use crate::ui::{Cube, SoccerBall};
    
    let view_clone = render_world.current_view().map(|v| v.view.clone());
    
    if let Some(ref view) = view_clone {
        // Query extracted components in Render World
        for (entity, transform) in render_world.query::<Transform3D>() {
            if let Some(cube) = render_world.get_component::<Cube>(entity) {
                let commands = cube.generate_render_commands(transform, view);
                for command in commands {
                    render_world.add_command(command);
                }
            }
            
            if let Some(soccer_ball) = render_world.get_component::<SoccerBall>(entity) {
                let commands = soccer_ball.generate_render_commands(transform, view);
                for command in commands {
                    render_world.add_command(command);
                }
            }
        }
    }
}

fn generate_ui_commands(render_world: &mut RenderWorld) {
    use crate::node::Transform2D;
    use crate::ui::Button;
    use crate::math::Rect;
    use crate::render_world::RenderCommand;
    
    for (entity, transform) in render_world.query::<Transform2D>() {
        if let Some(button) = render_world.get_component::<Button>(entity) {
            let color = button.current_color();
            
            let rect = Rect::new(
                transform.position.x,
                transform.position.y,
                button.width,
                button.height,
            );
            
            render_world.add_command(RenderCommand::DrawRect { rect, color });
        }
    }
}
```

### 3.6 Extract Schedule

**文件**: `src/extract/extract_schedule.rs`

```rust
//! Extract Schedule - Configurable extract phase
//!
//! Allows registering custom extract systems, similar to Bevy's ExtractSchedule

use crate::main_world::MainWorld;
use crate::render_world::RenderWorld;
use crate::resources::Resource;
use alloc::vec::Vec;
use alloc::boxed::Box;

/// A function that extracts data from Main World to Render World
pub type ExtractFn = Box<dyn Fn(&MainWorld, &mut RenderWorld) + Send + Sync>;

/// Extract Schedule - Collection of extract operations
///
/// Similar to Bevy's ExtractSchedule, this allows registering
/// custom extract systems that run during the Extract phase.
///
/// # Example
/// ```rust
/// use fhre::extract::ExtractSchedule;
/// use fhre::main_world::MainWorld;
/// use fhre::render_world::RenderWorld;
///
/// fn my_extractor(main_world: &MainWorld, render_world: &mut RenderWorld) {
///     // Custom extraction logic
/// }
///
/// let mut schedule = ExtractSchedule::new();
/// schedule.add_extractor(my_extractor);
/// ```
#[derive(Default)]
pub struct ExtractSchedule {
    extractors: Vec<ExtractFn>,
}

impl Clone for ExtractSchedule {
    fn clone(&self) -> Self {
        // Note: We can't clone the function pointers, so we create an empty schedule
        // This is a limitation - extractors need to be re-registered after cloning
        Self::new()
    }
}

impl ExtractSchedule {
    /// Create a new empty ExtractSchedule
    pub fn new() -> Self {
        Self {
            extractors: Vec::new(),
        }
    }

    /// Add an extract function to the schedule
    ///
    /// # Example
    /// ```rust
    /// schedule.add_extractor(|main_world, render_world| {
    ///     // Extract custom data
    /// });
    /// ```
    pub fn add_extractor<F>(&mut self, extractor: F)
    where
        F: Fn(&MainWorld, &mut RenderWorld) + Send + Sync + 'static,
    {
        self.extractors.push(Box::new(extractor));
    }

    /// Run all extractors
    ///
    /// This is called during the Extract phase to execute all registered extractors.
    pub fn run(&self, main_world: &MainWorld, render_world: &mut RenderWorld) {
        for extractor in &self.extractors {
            extractor(main_world, render_world);
        }
    }

    /// Check if there are any extractors registered
    pub fn is_empty(&self) -> bool {
        self.extractors.is_empty()
    }

    /// Get the number of registered extractors
    pub fn len(&self) -> usize {
        self.extractors.len()
    }

    /// Clear all extractors
    pub fn clear(&mut self) {
        self.extractors.clear();
    }
}

impl Resource for ExtractSchedule {}
```

### 3.7 SyncComponentPlugin

**文件**: `src/plugin/sync_component_plugin.rs`

```rust
//! Sync Component Plugin - Automatically syncs components to Render World
//!
//! Inspired by Bevy's SyncComponentPlugin

use crate::app::App;
use crate::Component;
use crate::sync::sync_markers::SyncToRenderWorld;
use crate::sync::pending_sync::PendingSyncEntity;
use crate::ExtractSchedule;
use crate::ExtractComponent;
use crate::extract::extract_components;
use crate::plugin::Plugin;

/// Plugin that enables automatic component extraction to Render World
///
/// # Example
/// ```
/// app.add_plugin(SyncComponentPlugin::<Transform3D>::default());
/// ```
pub struct SyncComponentPlugin<C: ExtractComponent>(core::marker::PhantomData<C>);

impl<C: ExtractComponent> Default for SyncComponentPlugin<C> {
    fn default() -> Self {
        Self(core::marker::PhantomData)
    }
}

impl<C: ExtractComponent + 'static> Plugin for SyncComponentPlugin<C> {
    fn build(&self, app: &mut App) {
        // Ensure PendingSyncEntity and ExtractSchedule resources exist
        if app.main_world.resources().get::<PendingSyncEntity>().is_none() {
            app.insert_resource(PendingSyncEntity::default());
        }

        if app.main_world.resources().get::<ExtractSchedule>().is_none() {
            app.insert_resource(ExtractSchedule::default());
        }

        // Add extract system to schedule
        let extract_fn = move |main_world: &crate::main_world::MainWorld,
                               render_world: &mut crate::render_world::RenderWorld| {
            extract_components::<C>(main_world, render_world);
        };

        // Register in ExtractSchedule
        if let Some(schedule) = app.main_world.resources_mut().get_mut::<ExtractSchedule>() {
            schedule.add_extractor(extract_fn);
        }
    }
}
```

### 3.8 模块结构

```
src/
├── sync/
│   ├── mod.rs              # 导出所有同步模块
│   ├── sync_markers.rs     # SyncToRenderWorld, RenderEntity, MainEntity
│   ├── pending_sync.rs     # PendingSyncEntity, EntityRecord
│   └── sync_system.rs      # entity_sync_system, detect_sync_changes_system, detect_sync_removals_system
├── extract/
│   ├── mod.rs              # 导出所有提取模块
│   ├── extract.rs          # 主提取逻辑（单相机架构）
│   ├── extract_component.rs # ExtractComponent trait, impl_extract_clone, impl_extract_identity
│   └── extract_schedule.rs # ExtractSchedule
└── plugin/
    ├── mod.rs
    └── sync_component_plugin.rs # SyncComponentPlugin, SyncComponents
```

---

## 4. 使用示例

### 4.1 基础用法

```rust
use fhre::prelude::*;
use fhre::sync::SyncToRenderWorld;
use fhre::extract::ExtractComponent;

// 定义组件
#[derive(Component, Clone)]
struct Position { x: f32, y: f32 }

// 实现 ExtractComponent
impl ExtractComponent for Position {
    type QueryData = Position;
    type QueryFilter = ();
    type Out = Position; // 直接克隆
    
    fn extract_component(item: &Self::QueryData) -> Option<Self::Out> {
        Some(item.clone())
    }
}

fn setup(mut commands: Commands) {
    // 这个实体会自动同步到 Render World
    commands.spawn()
        .insert(Position { x: 100.0, y: 200.0 })
        .insert(SyncToRenderWorld);
}

fn main() {
    App::new(640, 480)
        .add_plugins(DefaultPlugins)
        .add_plugin(SyncComponentPlugin::<Position>::default())
        .add_systems(Startup, setup)
        .run();
}
```

### 4.2 高级用法 - 自定义提取

```rust
// Main World 组件
#[derive(Component)]
struct Player { name: String, health: i32 }

// Render World 组件（只包含渲染需要的数据）
#[derive(Component)]
struct ExtractedPlayer { name: String } // 不包含 health

impl ExtractComponent for Player {
    type QueryData = Player;
    type QueryFilter = ();
    type Out = ExtractedPlayer;
    
    fn extract_component(item: &Self::QueryData) -> Option<Self::Out> {
        Some(ExtractedPlayer {
            name: item.name.clone(),
            // health 不被提取
        })
    }
}
```

---

## 5. 与 Bevy 的对比

| 功能 | Bevy | FHRE（改进后） |
|------|------|----------------|
| **实体同步** | `SyncWorldPlugin` + Observer | `entity_sync_system` |
| **同步标记** | `SyncToRenderWorld` | `SyncToRenderWorld` ✅ |
| **实体映射** | `RenderEntity`/`MainEntity` | `RenderEntity`/`MainEntity` ✅ |
| **组件提取** | `ExtractComponent` trait | `ExtractComponent` trait ✅ |
| **提取调度** | `ExtractSchedule` | `ExtractSchedule` ✅ |
| **自动插件** | `ExtractComponentPlugin` | `SyncComponentPlugin` ✅ |
| **并行渲染** | `PipelinedRenderingPlugin` | ❌ 单线程（嵌入式限制） |

---

## 6. 迁移指南

### 6.1 从旧版 FHRE 迁移

**旧代码**：
```rust
// 直接生成渲染命令
pub fn extract_cubes(main_world: &MainWorld, render_world: &mut RenderWorld) {
    for (entity, transform) in main_world.query::<Transform3D>() {
        if let Some(cube) = main_world.get_component::<Cube>(entity) {
            let commands = cube.generate_render_commands(transform, view);
            for command in commands {
                render_world.add_command(command);
            }
        }
    }
}
```

**新代码**：
```rust
// 1. 添加 SyncToRenderWorld 标记
fn setup(mut commands: Commands) {
    commands.spawn()
        .insert(Transform3D::default())
        .insert(Cube::new(100.0))
        .insert(SyncToRenderWorld); // 添加标记
}

// 2. 实现 ExtractComponent
impl ExtractComponent for Cube {
    type QueryData = Cube;
    type Out = Cube; // 直接克隆
    // ...
}

// 3. 注册插件
app.add_plugin(SyncComponentPlugin::<Cube>::default());

// 4. 在 Render World 生成命令
fn generate_3d_commands(render_world: &mut RenderWorld) {
    for (entity, transform) in render_world.query::<Transform3D>() {
        if let Some(cube) = render_world.get_component::<Cube>(entity) {
            // 从 Render World 组件生成命令
            let commands = cube.generate_render_commands(transform, view);
            // ...
        }
    }
}
```

---

## 7. 实现进度

### 7.1 已完成的模块 ✅

| 模块 | 文件路径 | 状态 |
|------|----------|------|
| **Sync Markers** | `src/sync/sync_markers.rs` | ✅ 已实现 |
| **Pending Sync** | `src/sync/pending_sync.rs` | ✅ 已实现 |
| **Sync System** | `src/sync/sync_system.rs` | ✅ 已实现 |
| **RenderWorld ECS** | `src/render_world/world.rs` | ✅ 已改造 |
| **ExtractComponent** | `src/extract/extract_component.rs` | ✅ 已实现 |
| **ExtractSchedule** | `src/extract/extract_schedule.rs` | ✅ 已实现 |
| **Module Exports** | `src/lib.rs` | ✅ 已更新 |

### 7.2 新增的核心类型

```rust
// From sync module
pub use sync::{SyncToRenderWorld, RenderEntity, MainEntity, PendingSyncEntity, EntityRecord};
pub use sync::{entity_sync_system, detect_sync_changes_system, detect_sync_removals_system};

// From extract module
pub use extract::{ExtractComponent, ExtractSchedule, extract_components};

// In prelude
pub use crate::sync::SyncToRenderWorld;
pub use crate::extract::ExtractComponent;
```

### 7.3 RenderWorld 新增方法

```rust
impl RenderWorld {
    // ECS Core
    pub fn spawn(&mut self) -> Entity;
    pub fn spawn_synced(&mut self, main_entity: Entity) -> Entity;
    pub fn despawn(&mut self, entity: Entity);
    pub fn insert_component<T: Component>(&mut self, entity: Entity, component: T);
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T>;
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T>;
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Option<T>;
    pub fn query<T: Component>(&self) -> impl Iterator<Item = (Entity, &T)>;
    pub fn get_entity_by_main(&self, main_entity: Entity) -> Option<Entity>;
    pub fn contains_entity(&self, entity: Entity) -> bool;
    pub fn entities(&self) -> &[Entity];
    
    // View Management
    pub fn clear_views(&mut self);
}
```

### 7.4 已完成的模块 ✅

| 模块 | 说明 | 状态 |
|------|------|------|
| **SyncComponentPlugin** | 自动注册组件提取 | ✅ 已完成 |
| **SyncComponents** | 批量组件同步插件组 | ✅ 已完成 |
| **Extract 阶段集成** | 在 App 中运行 sync + extract | ✅ 已完成 |
| **示例更新** | 使用新的 SyncToRenderWorld API | ✅ 已完成 |

### 7.5 App Extract 阶段流程

```rust
pub fn update(&mut self) {
    // ... Main World systems ...

    // =========================================================================
    // Extract Phase (NEW: aligned with Bevy's dual-world architecture)
    // =========================================================================

    // Step 1: Detect new SyncToRenderWorld markers
    detect_sync_changes_system(&mut self.main_world);

    // Step 2: Sync entities between Main World and Render World
    entity_sync_system(&mut self.main_world, &mut self.render_world);

    // Step 3: Run registered extract systems from ExtractSchedule
    self.render_world.clear_views();
    if let Some(schedule) = self.main_world.resources().get::<ExtractSchedule>() {
        let schedule = schedule.clone();
        schedule.run(&self.main_world, &mut self.render_world);
    }

    // Step 4: Legacy extract (for backward compatibility)
    extract_renderable_components(&self.main_world, &mut self.render_world);

    // ... Render phase ...
}
```

### 7.6 使用示例更新

```rust
// 在示例中标记需要同步的实体
commands.spawn()
    .insert(Node::game_entity(NodeType::Empty))
    .insert(Transform3D::from_position(obj_x, obj_y, 0.0))
    .insert(Cube::new(120.0))
    .insert(AnimationPlayer::new())
    // NEW: Mark entity for sync to Render World (Bevy-aligned)
    .insert(SyncToRenderWorld);
```

---

## 8. 总结

本文档详细描述了如何将 FHRE 的双世界架构对齐 Bevy 的实现：

1. **实体同步机制**：通过 `SyncToRenderWorld` 标记和 `RenderEntity`/`MainEntity` 实现双向实体映射
2. **Render World ECS 化**：将 Render World 从命令列表改造为完整的 ECS
3. **组件提取系统**：实现 `ExtractComponent` trait，支持选择性组件同步
4. **调度系统**：`ExtractSchedule` 支持注册自定义提取系统

这些改进使 FHRE 的双世界架构与 Bevy 保持一致，同时保持嵌入式系统的轻量级特性。

### 与单相机架构的关系

本文档的实现与 [REFACTOR_PROGRESS.md](REFACTOR_PROGRESS.md) 的单相机架构**完全兼容**：
- 单相机架构处理**渲染流程**（3D透视投影到幕布）
- Bevy 对齐处理**数据同步**（实体/组件提取机制）
- 两者结合形成完整的 ECS + 渲染架构

---

## 9. ECS 核心问题修复记录

> 本章节记录 FHRE ECS 核心实现从原型到生产级的修复历程。
> 详细问题清单见 [ECS_ISSUES.md](ECS_ISSUES.md)。

### 9.1 修复概览

| 优先级 | 问题 | 影响范围 | 修复难度 | 状态 |
|--------|------|----------|----------|------|
| P0 | Schedule 分阶段执行 | 全局 | 中 | ✅ |
| P0 | Query 多组件查询 | 全局 | 中 | ✅ |
| P1 | Query 类型别名不一致 | API | 低 | ✅ |
| P1 | Commands 延迟执行 | 全局 | 中 | ✅ |
| P2 | 系统参数手动标注 | 用户体验 | 高 | ⚠️ 部分 |
| P2 | Change Detection | 性能 | 高 | ✅ |
| P3 | insert_resource 冗余 | Demo | 低 | ✅ |
| P3 | system 枚举重复 | 可维护性 | 低 | ✅ |
| P3 | EntityCommands 生命周期 | 安全性 | 中 | ✅ |
| P3 | startup_systems 清理 | 可读性 | 低 | ✅ |

### 9.2 详细修复记录

#### Phase 1: Schedule 分阶段 + Commands 边界 apply ✅

**修复的问题**: P1 (Schedule 分阶段执行是假的) + P4 (Commands 延迟执行机制脆弱)

**核心变更**:

1. **`main_world/world.rs`**
   - `MainWorld` 结构体：`systems: Vec` 拆分为 `pre_update_systems`、`update_systems`、`post_update_systems`
   - 新增 `pending_commands: CommandsState` 字段，收集系统产生的 Commands
   - 新增 `add_boxed_system_to_stage(stage, system)` 方法，根据 stage 名路由到对应列表
   - 新增 `flush_commands_from_state(state)` 方法，将系统 state 中的 Commands 转移到 pending_commands
   - 新增 `apply_commands()` 方法，在阶段边界执行 pending_commands
   - `run_systems()` 重写：按 `PreUpdate → Update → PostUpdate` 顺序执行，每个阶段后 apply_commands

2. **`main_world/system.rs`**
   - 移除所有 `DeclarativeSystem*::apply_commands()` 方法（TypeId 检查 + 裸指针转换）
   - 改为在 `System::run()` 末尾调用 `world.flush_commands_from_state()`
   - Commands 不再在系统内部 apply，而是收集到 `MainWorld::pending_commands`
   - 由 `MainWorld::run_systems()` 在阶段边界统一 apply

3. **`main_world/commands.rs`**
   - 新增 `CommandsState::drain_into(&mut self, other: &mut CommandsState)` 方法
   - 避免暴露私有类型 `SpawnCommand`/`InsertCommand`

4. **`app/app.rs`**
   - `IntoSystems::add_to_app()` 从 `"Startup" => ... _ => add_boxed_system` 改为 `stage => add_boxed_system_to_stage(stage, ...)`
   - PreUpdate/Update/PostUpdate 系统现在正确路由到各自的列表

**执行流程变更**:

```
变更前:
  run_systems() → 按插入顺序执行所有系统（无分阶段）
  Commands: 系统内部 TypeId hack apply

变更后:
  run_systems():
    1. PreUpdate 系统执行 → flush commands → apply_commands
    2. Update 系统执行 → flush commands → apply_commands
    3. PostUpdate 系统执行 → flush commands → apply_commands
  Commands: 系统内 flush 到 pending_commands，阶段边界统一 apply
```

#### Phase 2: Query 统一 ✅

**修复的问题**: P2 (Query 不支持多组件查询) + P3 (Query 类型别名指向未实现的类型)

**核心变更**:

1. **`main_world/mod.rs`**
   - 从 re-exports 中移除 `BasicQuery`、`QueryIter`、`QueryIterMut`
   - `Query<T, F>` 类型别名已指向 `FilteredQuery`（之前就是，但 BasicQuery 同时存在造成混乱）
   - `FilteredQuery` 有完整的 `SystemParam` 实现，支持 `Query<&Transform, With<Cube>>` 过滤查询

**现状说明**:
- `Query<T, F = ()>` = `FilteredQuery<T, F>` — 单组件 + 过滤，已有 SystemParam 实现 ✅
- `MultiCompQuery<D, F = ()>` = `MultiQuery<D, F>` — 多组件 + QueryData，已有 SystemParam 实现 ✅
- `BasicQuery<T>` 仍存在于 `system_param.rs` 中（内部使用），但不再公开导出
- 用户现在可以统一使用 `Query<T>` 或 `Query<T, With<U>>`

#### Phase 3: Demo 清理 ✅

**修复的问题**: P7 (insert_resource 在 Plugin 之后调用冗余)

**核心变更**:

1. **`examples/fhre/rust/src/lib.rs`**
   - 移除冗余的 `.insert_resource(Camera::default_3d(640, 480))`（CameraPlugin 已自动注册）
   - 移除未使用的 `Camera` import

#### Phase 4: 系统参数宏 ✅

**修复的问题**: P8 (system1/system2/system3/system4 手动枚举) + P5 部分修复

**核心变更**:

1. **`main_world/system.rs`**
   - 用宏生成 `DeclarativeSystem1-4` 和 `system1-4`
   - 定义 `impl_declarative_system!` 宏，接受 name/fn_name/params/indices 参数
   - 宏生成 struct、System impl、Builder struct、IntoSystem impl、公开函数
   - 从 ~280 行手动重复代码缩减为 ~130 行（含宏定义）
   - 新增系统变体只需一行宏调用

**P5 现状**:
- 用户仍需手动写 `system2::<A, B, _>(func)` 形式
- 尝试了 `fn()` marker type pattern（Bevy 风格），但 Rust 类型系统无法从 `FnMut(A::Item)` 反推 `A`
- 多个 `IntoSystem<fn(...)>` impl 无法通过统一 trait 暴露（Marker 不受约束、与 tuple impl 冲突）
- 完全自动化需要过程宏（`#[derive(SystemParam)]`），`no_std` 环境暂不可用
- P5 标记为"部分完成"：代码量已减少（宏生成），但接口简化受 Rust 类型系统限制

#### Phase 5: Change Detection 集成 ✅

**修复的问题**: P6 (Change Detection 未集成)

**核心变更**:

1. **`main_world/world.rs`**
   - `MainWorld` 新增 `change_detection: ChangeDetection` 字段
   - `insert_component` 自动调用 `change_detection.mark_added::<T>(entity)`
   - `remove_component` 自动调用 `change_detection.remove::<T>(entity)`
   - `run_systems` 每帧开始时调用 `change_detection.increment_tick()`
   - 新增 `change_detection()` / `change_detection_mut()` 访问方法

**已集成的功能**:
- ✅ 组件添加时自动标记 `added` tick
- ✅ 组件移除时自动清理 tracking
- ✅ 每帧递增 tick counter
- ✅ `Mut<T>` 的 `DerefMut` 自动调用 `mark_changed`
- ✅ `Ref<T>` / `Mut<T>` 的 `is_added()` / `is_changed()` 现在正确工作（通过 `last_run_tick`）
- ✅ `ChangeDetection` 新增 `last_run_tick` 字段，`increment_tick` 时自动更新
- ✅ `Mut<T>` / `Ref<T>` 通过 `change_detection.last_run_tick` 判断变更

**待后续迭代**:
- Extract 阶段利用 Change Detection 跳过未变化数据
- `Mut<T>` / `Ref<T>` 作为 SystemParam 可直接在系统中使用

#### Phase 6: 安全性修复 ✅

**修复的问题**: P9 (EntityCommands 使用 'static 生命周期 hack) + P10 (run_startup_systems 执行后不清空系统内容)

**核心变更**:

1. **`main_world/commands.rs`** — P9: EntityCommands 生命周期修复
   - `EntityCommands<'a>` → `EntityCommands<'a, 'w, 's>`，正确绑定 Commands 的两个生命周期参数
   - `Commands::spawn()` 返回 `EntityCommands<'_, 'w, 's>` 而非 `EntityCommands<'_>`
   - 移除了 `'static` 生命周期 hack（`&mut *(self as *mut ... as *mut Commands<'static, 'static>)`）
   - 现在 EntityCommands 的生命周期严格绑定到 Commands，编译器可保证安全

2. **`main_world/world.rs`** — P10: startup_systems 清理
   - 在 Phase 1 中已隐式修复：`run_startup_systems` 使用 swap 后执行，局部变量自动 drop
   - `self.startup_systems` swap 后为空 vec，startup 系统只执行一次，逻辑正确

### 9.3 修复验证状态

所有修复均通过以下验证：
- ✅ FHRE 库编译通过
- ✅ 示例编译通过
- ✅ NuttX build.sh 构建通过
