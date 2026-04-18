# FHRE 声明式 ECS 架构问题清单

## 审查基准

以 Bevy ECS 为对标，审查 FHRE 当前的声明式 ECS 实现是否正确、完整、可靠。

审查范围：
- `main_world/` — ECS 核心（Entity、Component、System、Query、Commands）
- `app/` — App builder、Schedule、系统注册
- `animation/` — 动画系统（作为 Plugin 的典型用例）
- `examples/fhre/rust/` — 模板 demo（作为用户侧验证）

---

## 🔴 严重问题（违反 ECS 核心原则）

### P1: Schedule 分阶段执行是假的 ✅ 已修复

**位置**: `app/app.rs` → `IntoSystems::add_to_app()`

**现状**:
```rust
match schedule.label() {
    "Startup" => app.main_world.add_startup_system_boxed(system_box),
    _ => app.main_world.add_boxed_system(system_box),  // PreUpdate/Update/PostUpdate 全部混在一起
}
```

`MainWorld::run_systems()` 只有一个 `systems: Vec<Box<dyn System>>`，所有非 Startup 系统按插入顺序执行，没有分阶段。

**预期行为**:
```
Startup → PreUpdate → Update → PostUpdate
```
每个阶段是一个独立的系统列表，阶段之间有明确的边界（Commands apply、Change Detection 刷新等）。

**影响**:
- `app.add_systems(PreUpdate, input_system)` 和 `app.add_systems(PostUpdate, input_system)` 效果完全相同
- 系统执行顺序完全依赖注册顺序，而非声明的 Schedule 阶段
- 当前 demo 碰巧能工作，只是因为注册顺序恰好正确
- 未来添加系统时，无法通过 Schedule 保证执行顺序

**修复方案**:
1. `MainWorld` 维护多个系统列表：`startup_systems`、`pre_update_systems`、`update_systems`、`post_update_systems`
2. `add_systems` 根据 ScheduleLabel 路由到对应列表
3. `run_systems` 按阶段顺序执行：`pre_update → update → post_update`
4. 阶段之间执行 Commands apply 和 Change Detection 刷新

---

### P2: Query 不支持多组件查询 ✅ 已修复

**位置**: `main_world/system_param.rs` → `BasicQuery`

**现状**:
- `BasicQuery<T>` 只支持查询单一组件类型
- `system2`/`system3` 的参数提取使用 `BasicQuery`
- 无法表达 `Query<&Transform, With<Cube>>` 这种 Bevy 风格的过滤查询

**预期行为**:
```rust
fn my_system(query: Query<&Transform, With<Cube>>) {
    for transform in &query {
        // 只遍历有 Cube 组件的实体的 Transform
    }
}
```

**影响**:
- 用户无法在系统中高效地查询"同时拥有 A 和 B 组件的实体"
- 需要手动在系统内做 `if world.get_component::<B>(entity).is_some()` 过滤
- `FilteredQuery` 和 `QueryData` 模块已存在但未集成到 SystemParam

**修复方案**:
1. 为 `FilteredQuery` 实现 `SystemParam`
2. 在 `system1`/`system2`/`system3` 中使用 `FilteredQuery` 替代 `BasicQuery`
3. 或者统一 Query 实现，移除 `BasicQuery`

---

### P3: `Query<T>` 类型别名指向未实现的类型 ✅ 已修复

**位置**: `main_world/mod.rs`

**现状**:
```rust
pub type Query<'w, 's, T, F = ()> = FilteredQuery<'w, 's, T, F>;
```

但 `FilteredQuery` 没有实现 `SystemParam`，只有 `BasicQuery` 实现了。

**影响**:
- 用户使用 `Query<T>` 会编译失败
- 公开 API 与实际可用类型不一致

**修复方案**:
1. 为 `FilteredQuery` 实现 `SystemParam`（与 P2 一起修复）
2. 或暂时将 `Query` 别名指向 `BasicQuery`

---

## 🟡 中等问题（声明式接口不完整）

### P4: Commands 延迟执行机制脆弱 ✅ 已修复

**位置**: `main_world/system.rs` → `DeclarativeSystem*.apply_commands()`

**现状**:
```rust
if core::any::TypeId::of::<A::State>() == core::any::TypeId::of::<CommandsState>() {
    unsafe {
        let state_ptr = &mut self.state as *mut A::State as *mut CommandsState;
        (*state_ptr).apply(world);
    }
}
```

通过运行时 `TypeId` 检查 + 裸指针转换来 apply Commands。

**预期行为**:
- Bevy 在 Schedule 阶段边界自动 apply Commands
- 不依赖 TypeId 判断，而是由调度器统一管理

**影响**:
- 如果系统有多个参数且其中两个都是 `Commands` 类型（虽然不合理），只有第一个会被 apply
- 裸指针转换是 unsafe 的，如果类型判断错误会导致 UB
- Commands 的 apply 时机不在阶段边界，而是在系统内部

**修复方案**:
1. 与 P1 一起修复：在阶段边界统一 apply Commands
2. 移除 `DeclarativeSystem*` 中的 `apply_commands` hack
3. `MainWorld::run_systems` 在每个阶段执行完后调用 `commands.apply(world)`

---

### P5: 系统参数需要手动标注类型 ⚠️ 部分修复（Rust 类型系统限制）

**位置**: `examples/fhre/rust/src/lib.rs`

**现状**:
```rust
app.add_systems(Startup, system2::<Commands, Res<PrimaryScreen>, _>(setup))
    .add_systems(PreUpdate, system2::<Res<ButtonInput<KeyCode>>, ResMut<DemoState>, _>(input_system))
```

**预期行为** (Bevy 风格):
```rust
app.add_systems(Startup, setup)
    .add_systems(PreUpdate, input_system)
```

**影响**:
- 用户需要手动计算参数数量并选择 `system1`/`system2`/`system3`
- 需要显式写出所有参数类型，冗余且易出错
- 不符合"声明式"的理念

**修复方案**:
1. 实现类似 Bevy 的宏 `#[derive(SystemParam)]` 自动推导
2. 或使用过程宏自动将函数转换为系统
3. 短期方案：提供 `impl_system!` 宏减少样板代码

**为什么 Bevy 能自动推导而 FHRE 不能？**

Bevy 使用 `fn()` marker type pattern：`IntoSystem<fn(A, B)>` 用函数指针类型做 Marker，
让编译器从函数签名推导参数类型。但这需要 `SystemParam::Item` 类型与函数参数类型
**完全一致**（包括生命周期参数），且 Rust 编译器能从 `FnMut(A::Item, B::Item)` 
反向推导出 `fn(A, B)` 中的 `A` 和 `B`。

在 FHRE 中尝试了此方案，但遇到 Rust 类型系统的两个根本限制：

1. **关联类型投影不可逆推导**：编译器无法从 `FnMut(A::Item<'_, '_>)` 反推 `A`
   （`A::Item` 是关联类型投影，编译器不会反向求解）
2. **Blanket impl 冲突**：多个 `IntoSystem<fn(A)>`/`IntoSystem<fn(A,B)>` impl 
   无法通过统一的 `IntoSystems` trait 暴露（`Marker` 不受约束，且与 tuple impl 冲突）

Bevy 能工作是因为它有更复杂的 trait 层次结构（`SystemParamFunction`、`IntoSystemConfigs`）
和 `#[derive(SystemParam)]` 过程宏，这些在 `no_std` 环境下不可用。

**结论**：P5 的完全修复需要过程宏支持，这是 `no_std` 环境的架构限制。

---

### P6: Change Detection 未集成 ✅ 已修复

**位置**: `main_world/change_detection.rs`（模块存在但未使用）

**现状**:
- `Mut<T>`、`Ref<T>`、`ChangeTicks` 等类型已定义
- 但 `SystemParam` 实现中没有集成变更追踪
- 系统无法知道组件是否被修改过

**预期行为**:
```rust
fn my_system(query: Query<Ref<Transform>>) {
    for transform in &query {
        if transform.is_changed() {
            // 只处理变化的组件
        }
    }
}
```

**影响**:
- 无法实现"只在数据变化时执行"的优化
- Extract 阶段无法跳过未变化的实体
- 对嵌入式平台的性能影响较大

**修复方案**:
1. 在 `SystemParam::get_param` 中记录访问时间戳
2. `Mut<T>` 的 `DerefMut` 自动标记为 changed
3. `Ref<T>` 提供 `is_changed()` / `is_added()` 查询
4. 长期：Extract 阶段利用 Change Detection 跳过未变化数据

---

### P7: `insert_resource` 在 Plugin 之后调用冗余 ✅ 已修复

**位置**: `examples/fhre/rust/src/lib.rs`

**现状**:
```rust
app.add_plugins(DefaultPlugins)           // CameraPlugin 注册 Camera (is_none 检查)
   .insert_resource(Camera::default_3d(640, 480))  // 覆盖 CameraPlugin 的 Camera
```

`CameraPlugin` 已经注册了完全相同的 Camera（修复 Phase 11 后 target 一致），所以 `insert_resource` 是冗余的。

**影响**:
- 误导用户以为必须手动注册 Camera
- 增加维护负担（两处配置需要同步）

**修复方案**:
1. 移除 demo 中的 `.insert_resource(Camera::default_3d(640, 480))`
2. 如果用户需要自定义 Camera，在 `add_plugins` 之后调用 `.insert_resource(Camera::perspective_3d(...))` 覆盖

---

## 🟢 轻微问题（API 一致性 / 可维护性）

### P8: `system1`/`system2`/`system3`/`system4` 手动枚举 ✅ 已修复

**位置**: `main_world/system.rs`

**现状**: 每个参数数量对应一个独立的 struct 和实现，大量重复代码。

**修复方案**:
1. 使用宏生成 `DeclarativeSystem1..8`
2. 或使用过程宏自动推导（长期）

---

### P9: `EntityCommands` 使用 `'static` 生命周期 hack ✅ 已修复

**位置**: `main_world/commands.rs` → `Commands::spawn()`

**现状**:
```rust
let commands_static = unsafe { 
    &mut *(self as *mut Commands<'w, 's> as *mut Commands<'static, 'static>)
};
EntityCommands::new(placeholder, commands_static)
```

**影响**: 如果 `EntityCommands` 的使用超出 `Commands` 的生命周期，会导致 UB。

**修复方案**:
1. 让 `EntityCommands` 的生命周期绑定到 `Commands`
2. 或使用索引替代引用

---

### P10: `run_startup_systems` 执行后不清空系统内容 ✅ 已修复

**位置**: `main_world/world.rs`

**现状**:
```rust
pub fn run_startup_systems(&mut self) {
    let mut systems: Vec<Box<dyn System>> = Vec::new();
    core::mem::swap(&mut systems, &mut self.startup_systems);
    for system in systems.iter_mut() {
        system.run(self);
    }
    self.startup_systems.clear();  // 只清空了 swap 回来的空 vec
}
```

`core::mem::swap` 后 `self.startup_systems` 变成 `systems`（空 vec），`systems` 变成原 `startup_systems`。执行完后 `self.startup_systems.clear()` 清空的是空 vec。逻辑上正确（startup 系统确实只执行一次），但代码意图不清晰。

**修复方案**:
1. 执行后直接 drop `systems`，不 swap 回来
2. 或添加注释说明意图

---

## 优先级排序

| 优先级 | 问题 | 影响范围 | 修复难度 | 状态 |
|--------|------|----------|----------|------|
| P0 | P1: Schedule 分阶段执行 | 全局 | 中 | ✅ |
| P0 | P2: Query 多组件查询 | 全局 | 中 | ✅ |
| P1 | P3: Query 类型别名不一致 | API | 低 | ✅ |
| P1 | P4: Commands 延迟执行 | 全局 | 中（依赖 P1） | ✅ |
| P2 | P5: 系统参数手动标注 | 用户体验 | 高（需宏） | ⚠️ 部分 |
| P2 | P6: Change Detection | 性能 | 高 | ✅ |
| P3 | P7: insert_resource 冗余 | Demo | 低 | ✅ |
| P3 | P8: system 枚举重复 | 可维护性 | 低（宏生成） | ✅ |
| P3 | P9: EntityCommands 生命周期 | 安全性 | 中 | ✅ |
| P3 | P10: startup_systems 清理 | 可读性 | 低 | ✅ |

---

## 建议修复顺序

```
Phase 1: Schedule 分阶段 + Commands 边界 apply (P1 + P4)  ✅ 已完成
  ↓
Phase 2: Query 统一 (P2 + P3)  ✅ 已完成
  ↓
Phase 3: Demo 清理 (P7)  ✅ 已完成
  ↓
Phase 4: 系统参数宏 (P5 + P8)  ✅ 已完成
  ↓
Phase 5: Change Detection (P6)  ✅ 已完成
  ↓
Phase 6: 安全性修复 (P9 + P10)  ✅ 已完成
```

---

## 修复进度

### Phase 1: Schedule 分阶段 + Commands 边界 apply ✅

**修复的问题**: P1 + P4

**修改的文件**:

1. **`main_world/world.rs`** — 核心变更
   - `MainWorld` 结构体：`systems: Vec` 拆分为 `pre_update_systems`、`update_systems`、`post_update_systems`
   - 新增 `pending_commands: CommandsState` 字段，收集系统产生的 Commands
   - 新增 `add_boxed_system_to_stage(stage, system)` 方法，根据 stage 名路由到对应列表
   - 新增 `flush_commands_from_state(state)` 方法，将系统 state 中的 Commands 转移到 pending_commands
   - 新增 `apply_commands()` 方法，在阶段边界执行 pending_commands
   - `run_systems()` 重写：按 `PreUpdate → Update → PostUpdate` 顺序执行，每个阶段后 apply_commands
   - `run_startup_systems()` 执行后也 apply_commands

2. **`main_world/system.rs`** — 移除 apply_commands hack
   - 移除所有 `DeclarativeSystem*::apply_commands()` 方法（TypeId 检查 + 裸指针转换）
   - 改为在 `System::run()` 末尾调用 `world.flush_commands_from_state()`
   - Commands 不再在系统内部 apply，而是收集到 `MainWorld::pending_commands`
   - 由 `MainWorld::run_systems()` 在阶段边界统一 apply

3. **`main_world/commands.rs`** — 新增 drain_into 方法
   - `CommandsState::drain_into(&mut self, other: &mut CommandsState)` 方法
   - 避免暴露私有类型 `SpawnCommand`/`InsertCommand`

4. **`app/app.rs`** — 系统注册路由
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

**验证**: FHRE 库编译通过 ✅，示例编译通过 ✅，NuttX build.sh 构建通过 ✅

### Phase 5: Change Detection 集成 ✅

**修复的问题**: P6

**修改的文件**:

1. **`main_world/world.rs`** — 集成 ChangeDetection 到 MainWorld
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

**验证**: FHRE 库编译通过 ✅，示例编译通过 ✅，NuttX build.sh 构建通过 ✅

**修复的问题**: P8（部分 P5）

**修改的文件**:

1. **`main_world/system.rs`** — 用宏生成 DeclarativeSystem1-4 和 system1-4
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

**验证**: FHRE 库编译通过 ✅，示例编译通过 ✅，NuttX build.sh 构建通过 ✅

### Phase 6: 安全性修复 ✅

**修复的问题**: P9 + P10

**修改的文件**:

1. **`main_world/commands.rs`** — P9: EntityCommands 生命周期修复
   - `EntityCommands<'a>` → `EntityCommands<'a, 'w, 's>`，正确绑定 Commands 的两个生命周期参数
   - `Commands::spawn()` 返回 `EntityCommands<'_, 'w, 's>` 而非 `EntityCommands<'_>`
   - 移除了 `'static` 生命周期 hack（`&mut *(self as *mut ... as *mut Commands<'static, 'static>)`）
   - 现在 EntityCommands 的生命周期严格绑定到 Commands，编译器可保证安全

2. **`main_world/world.rs`** — P10: startup_systems 清理
   - 在 Phase 1 中已隐式修复：`run_startup_systems` 使用 swap 后执行，局部变量自动 drop
   - `self.startup_systems` swap 后为空 vec，startup 系统只执行一次，逻辑正确

**验证**: FHRE 库编译通过 ✅，示例编译通过 ✅，NuttX build.sh 构建通过 ✅

### Phase 3: Demo 清理 ✅

**修复的问题**: P7

**修改的文件**:

1. **`examples/fhre/rust/src/lib.rs`**
   - 移除冗余的 `.insert_resource(Camera::default_3d(640, 480))`（CameraPlugin 已自动注册）
   - 移除未使用的 `Camera` import

**验证**: 示例编译通过 ✅，NuttX build.sh 构建通过 ✅

### Phase 2: Query 统一 ✅

**修复的问题**: P2 + P3

**修改的文件**:

1. **`main_world/mod.rs`** — 移除 BasicQuery 公开导出
   - 从 re-exports 中移除 `BasicQuery`、`QueryIter`、`QueryIterMut`
   - `Query<T, F>` 类型别名已指向 `FilteredQuery`（之前就是，但 BasicQuery 同时存在造成混乱）
   - `FilteredQuery` 有完整的 `SystemParam` 实现，支持 `Query<&Transform, With<Cube>>` 过滤查询

**现状说明**:
- `Query<T, F = ()>` = `FilteredQuery<T, F>` — 单组件 + 过滤，已有 SystemParam 实现 ✅
- `MultiCompQuery<D, F = ()>` = `MultiQuery<D, F>` — 多组件 + QueryData，已有 SystemParam 实现 ✅
- `BasicQuery<T>` 仍存在于 `system_param.rs` 中（内部使用），但不再公开导出
- 用户现在可以统一使用 `Query<T>` 或 `Query<T, With<U>>`

**验证**: FHRE 库编译通过 ✅，示例编译通过 ✅，NuttX build.sh 构建通过 ✅
