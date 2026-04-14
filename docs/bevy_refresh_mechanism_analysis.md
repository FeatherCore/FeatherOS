# Bevy 刷新机制分析文档

## 1. 概述

Bevy 是一个基于 Rust 的数据驱动游戏引擎，采用 ECS (Entity-Component-System) 架构。本文档详细分析 Bevy 的刷新机制，包括应用主循环、渲染流程和事件处理。

---

## 2. 项目结构

```
bevy/
├── crates/
│   ├── bevy_app/              # 应用核心
│   │   ├── src/
│   │   │   ├── app.rs         # App 主结构
│   │   │   ├── main_schedule.rs   # 主调度配置
│   │   │   └── schedule_runner.rs # 调度运行器
│   │   └── Cargo.toml
│   │
│   ├── bevy_render/           # 渲染系统
│   │   ├── src/
│   │   │   ├── lib.rs         # 渲染主模块
│   │   │   ├── render_graph/  # 渲染图
│   │   │   └── view/          # 视图管理
│   │   └── Cargo.toml
│   │
│   ├── bevy_winit/            # 窗口系统
│   │   ├── src/
│   │   │   ├── lib.rs         # winit 插件
│   │   │   ├── state.rs       # 应用状态管理
│   │   │   └── system.rs      # 窗口系统
│   │   └── Cargo.toml
│   │
│   └── bevy_ecs/              # ECS 核心
│       └── src/
│           ├── schedule/      # 调度系统
│           └── system/        # 系统执行
│
└── Cargo.toml                 # 工作区配置
```

---

## 3. 应用主循环 (App Loop)

### 3.1 应用生命周期

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Bevy 应用生命周期                               │
│                                                                             │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐  │
│  │  Build   │──▶│  Finish  │──▶│  Cleanup │──▶│  Update  │──▶│  Exit    │  │
│  │   构建    │   │  完成     │   │  清理     │   │  更新     │   │  退出    │  │
│  └──────────┘   └──────────┘   └──────────┘   └────┬─────┘   └──────────┘  │
│                                                    │                        │
│                                                    └──────────────────────▶│
│                                                           (循环执行)        │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 应用主结构 (App)

**文件**: `crates/bevy_app/src/app.rs`

```rust
pub struct App {
    /// ECS 世界，存储所有实体和组件
    pub world: World,
    /// 调度集合
    pub schedules: Schedules,
    /// 子应用集合
    pub sub_apps: SubApps,
    /// 插件生命周期状态
    pub plugins_state: PluginsState,
    /// 应用运行器
    pub runner: Box<dyn FnOnce(App) -> AppExit>,
    /// 插件注册表
    pub plugin_registry: Vec<Box<dyn Plugin>>,
    /// 插件名称集合
    pub plugin_name_added: HashSet<Box<str>>,
}

impl App {
    /// 运行应用
    pub fn run(&mut self) -> AppExit {
        // 1. 完成插件构建
        self.finish();
        // 2. 清理插件
        self.cleanup();
        // 3. 执行 runner
        let runner = std::mem::replace(&mut self.runner, Box::new(run_once));
        (runner)(std::mem::take(self))
    }
    
    /// 单次更新
    pub fn update(&mut self) {
        // 运行主调度
        self.world.run_schedule(Main);
        // 更新子应用
        for sub_app in self.sub_apps.values_mut() {
            sub_app.update();
        }
    }
}
```

### 3.3 主调度配置

**文件**: `crates/bevy_app/src/main_schedule.rs`

```rust
/// 主调度标签
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Main;

/// 主调度顺序
#[derive(Resource, Debug)]
pub struct MainScheduleOrder {
    pub labels: Vec<InternedScheduleLabel>,
}

impl Default for MainScheduleOrder {
    fn default() -> Self {
        Self {
            labels: vec![
                PreStartup.intern(),
                Startup.intern(),
                PostStartup.intern(),
                First.intern(),
                PreUpdate.intern(),
                StateTransition.intern(),
                RunFixedMainLoop.intern(),
                Update.intern(),
                PostUpdate.intern(),
                Last.intern(),
            ],
        }
    }
}
```

### 3.4 调度执行顺序

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              主调度执行顺序                                  │
│                                                                             │
│  PreStartup ──▶ Startup ──▶ PostStartup                                     │
│       │                                                    (仅执行一次)      │
│       ▼                                                                     │
│  First ──▶ PreUpdate ──▶ StateTransition ──▶ RunFixedMainLoop              │
│                                                               │             │
│                                                               ▼             │
│  Update ──▶ PostUpdate ──▶ Last                                             │
│       │                                                                     │
│       └────────────────────────────────────────────────────────────────▶    │
│                           (每帧循环执行)                                     │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Winit 窗口循环 (窗口模式)

### 4.1 Winit 事件循环

**文件**: `crates/bevy_winit/src/state.rs`

```rust
pub(crate) struct WinitAppRunnerState {
    /// 运行的应用
    app: App,
    /// 退出值
    app_exit: Option<AppExit>,
    /// 当前更新模式
    update_mode: UpdateMode,
    /// 是否接收到窗口事件
    window_event_received: bool,
    /// 是否接收到设备事件
    device_event_received: bool,
    /// 是否接收到用户事件
    user_event_received: bool,
    /// 是否请求重绘
    redraw_requested: bool,
    /// 上次重绘后是否已更新
    ran_update_since_last_redraw: bool,
    /// 等待时间是否已过
    wait_elapsed: bool,
    /// 启动时强制更新次数
    startup_forced_updates: u32,
    /// 应用生命周期状态
    lifecycle: AppLifecycle,
}
```

### 4.2 更新模式 (UpdateMode)

```rust
pub enum UpdateMode {
    /// 连续更新模式 (游戏模式)
    Continuous,
    
    /// 响应式更新模式 (GUI模式)
    Reactive {
        /// 最大等待时间
        wait: Duration,
        /// 是否响应窗口事件
        react_to_window_events: bool,
        /// 是否响应设备事件
        react_to_device_events: bool,
        /// 是否响应用户事件
        react_to_user_events: bool,
    },
}
```

### 4.3 事件处理流程

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          Winit 事件处理流程                                  │
│                                                                             │
│  ┌─────────────┐                                                            │
│  │ new_events  │  新事件到达                                                 │
│  └──────┬──────┘                                                            │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                            │
│  │   resumed   │  应用恢复 (Android)                                         │
│  └──────┬──────┘                                                            │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐                     │
│  │ window_event│───▶│ device_event│───▶│  user_event │                     │
│  │  窗口事件    │    │  设备事件    │    │  用户事件    │                     │
│  └──────┬──────┘    └─────────────┘    └─────────────┘                     │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                            │
│  │about_to_wait│  准备等待下一帧                                             │
│  └──────┬──────┘                                                            │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                            │
│  │redraw_req   │  请求重绘                                                   │
│  └──────┬──────┘                                                            │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                            │
│  │  suspended  │  应用挂起 (Android)                                         │
│  └─────────────┘                                                            │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 4.4 重绘处理逻辑

```rust
fn redraw_requested(&mut self, event_loop: &ActiveEventLoop) {
    // 1. 确定是否应该更新
    let mut should_update = self.should_update(update_mode);
    
    // 2. 启动时强制更新
    if self.startup_forced_updates > 0 {
        self.startup_forced_updates -= 1;
        should_update = true;
    }
    
    // 3. 生命周期处理
    if self.lifecycle == AppLifecycle::WillSuspend {
        self.lifecycle = AppLifecycle::Suspended;
        should_update = true;
    }
    
    if self.lifecycle == AppLifecycle::WillResume {
        self.lifecycle = AppLifecycle::Running;
        should_update = true;
        self.redraw_requested = true;
    }
    
    // 4. 执行应用更新
    if should_update {
        if !self.ran_update_since_last_redraw || all_invisible {
            self.run_app_update();
            self.ran_update_since_last_redraw = true;
        } else {
            self.redraw_requested = true;
        }
    }
    
    // 5. 设置控制流
    match update_mode {
        UpdateMode::Continuous => {
            // 连续模式：有窗口可见时使用 Wait，否则 Poll
            event_loop.set_control_flow(ControlFlow::Wait);
            self.redraw_requested = true;
        }
        UpdateMode::Reactive { wait, .. } => {
            // 响应式模式：设置等待超时
            if self.wait_elapsed {
                self.redraw_requested = true;
                event_loop.set_control_flow(ControlFlow::WaitUntil(next));
            }
        }
    }
    
    // 6. 请求窗口重绘
    if self.redraw_requested && self.lifecycle != AppLifecycle::Suspended {
        for window in winit_windows.windows.values() {
            window.request_redraw();
        }
        self.redraw_requested = false;
    }
}
```

---

## 5. 渲染流程

### 5.1 渲染系统集合

**文件**: `crates/bevy_render/src/lib.rs`

```rust
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum RenderSystems {
    /// 提取命令
    ExtractCommands,
    /// 准备网格
    PrepareMeshes,
    /// 创建视图
    CreateViews,
    /// 特化管线
    Specialize,
    /// 准备视图
    PrepareViews,
    /// 排队
    Queue,
    /// 阶段排序
    PhaseSort,
    /// 准备资源
    Prepare,
    /// 渲染
    Render,
    /// 清理
    Cleanup,
    /// 最终清理
    PostCleanup,
}
```

### 5.2 渲染调度配置

```rust
impl Render {
    pub fn base_schedule() -> Schedule {
        let mut schedule = Schedule::new(Self);
        
        schedule.configure_sets(
            (
                ExtractCommands,
                PrepareMeshes,
                CreateViews,
                Specialize,
                PrepareViews,
                Queue,
                PhaseSort,
                Prepare,
                Render,
                Cleanup,
                PostCleanup,
            )
                .chain(),
        );
        
        // 准备资源子链
        schedule.configure_sets(
            (
                PrepareResources,
                PrepareResourcesBatchPhases,
                PrepareResourcesWritePhaseBuffers,
                PrepareResourcesCollectPhaseBuffers,
                PrepareResourcesFlush,
                PrepareBindGroups,
            )
                .chain()
                .in_set(Prepare),
        );
        
        schedule
    }
}
```

### 5.3 渲染流程图

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              渲染流程                                        │
│                                                                             │
│  Main World                              Render World                       │
│  (主世界)                                (渲染世界)                          │
│       │                                        │                            │
│       │ ExtractSchedule                        │                            │
│       │ (提取阶段)                              │                            │
│       ▼                                        ▼                            │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ ExtractCommands                                                      │   │
│  │ - 从主世界提取需要渲染的实体                                          │   │
│  │ - 复制变换、材质、网格等数据                                            │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│       │                                        │                            │
│       ▼                                        ▼                            │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ PrepareMeshes                                                        │   │
│  │ - 准备网格数据                                                        │   │
│  │ - 上传到 GPU                                                          │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│       │                                        │                            │
│       ▼                                        ▼                            │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ CreateViews                                                          │   │
│  │ - 创建相机视图                                                        │   │
│  │ - 计算视锥体                                                          │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│       │                                        │                            │
│       ▼                                        ▼                            │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ Specialize                                                           │   │
│  │ - 特化渲染管线                                                        │   │
│  │ - 根据材质特性选择着色器                                               │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│       │                                        │                            │
│       ▼                                        ▼                            │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ PrepareViews                                                         │   │
│  │ - 准备视图相关资源                                                    │   │
│  │ - 设置渲染目标                                                        │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│       │                                        │                            │
│       ▼                                        ▼                            │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ Queue                                                                │   │
│  │ - 将渲染项加入队列                                                    │   │
│  │ - 按渲染阶段分类                                                       │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│       │                                        │                            │
│       ▼                                        ▼                            │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ PhaseSort                                                            │   │
│  │ - 对渲染项排序                                                        │   │
│  │ - 按深度、材质等排序                                                   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│       │                                        │                            │
│       ▼                                        ▼                            │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ Prepare                                                              │   │
│  │ - 准备渲染资源                                                        │   │
│  │ - 绑定组、缓冲区                                                       │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│       │                                        │                            │
│       ▼                                        ▼                            │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ Render                                                               │   │
│  │ - 执行渲染图                                                          │   │
│  │ - 提交 GPU 命令                                                        │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│       │                                        │                            │
│       ▼                                        ▼                            │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ Cleanup / PostCleanup                                                │   │
│  │ - 清理临时实体                                                        │   │
│  │ - 释放资源                                                            │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 6. 调度运行器 (Schedule Runner)

### 6.1 运行模式

**文件**: `crates/bevy_app/src/schedule_runner.rs`

```rust
pub enum RunMode {
    /// 循环运行模式
    Loop {
        /// 每次调度完成后等待的最小时间
        wait: Option<Duration>,
    },
    /// 仅运行一次
    Once,
}

impl Default for RunMode {
    fn default() -> Self {
        RunMode::Loop { wait: None }
    }
}
```

### 6.2 循环运行逻辑

```rust
impl Plugin for ScheduleRunnerPlugin {
    fn build(&self, app: &mut App) {
        let run_mode = self.run_mode;
        app.set_runner(move |mut app: App| {
            // 1. 完成插件初始化
            if app.plugins_state() != PluginsState::Cleaned {
                while app.plugins_state() == PluginsState::Adding {
                    tick_global_task_pools_on_main_thread();
                }
                app.finish();
                app.cleanup();
            }
            
            // 2. 根据运行模式执行
            match run_mode {
                RunMode::Once => {
                    app.update();
                    AppExit::Success
                }
                RunMode::Loop { wait } => {
                    loop {
                        let start_time = Instant::now();
                        
                        // 执行更新
                        app.update();
                        
                        // 检查退出
                        if let Some(exit) = app.should_exit() {
                            return exit;
                        }
                        
                        // 计算等待时间
                        let end_time = Instant::now();
                        if let Some(wait) = wait {
                            let exe_time = end_time - start_time;
                            if exe_time < wait {
                                sleep(wait - exe_time);
                            }
                        }
                    }
                }
            }
        });
    }
}
```

---

## 7. 双世界架构

### 7.1 主世界与渲染世界

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              双世界架构                                      │
│                                                                             │
│  ┌───────────────────────────────┐  ┌───────────────────────────────┐      │
│  │         Main World            │  │        Render World           │      │
│  │         (主世界)               │  │        (渲染世界)              │      │
│  │                               │  │                               │      │
│  │  ┌─────────────────────────┐  │  │  ┌─────────────────────────┐  │      │
│  │  │   游戏逻辑系统           │  │  │  │   渲染系统               │  │      │
│  │  │   - 玩家控制             │  │  │  │   - 网格渲染             │  │      │
│  │  │   - 物理模拟             │  │  │  │   - 光照计算             │  │      │
│  │  │   - AI 行为              │  │  │  │   - 后处理               │  │      │
│  │  └─────────────────────────┘  │  │  └─────────────────────────┘  │      │
│  │                               │  │                               │      │
│  │  ┌─────────────────────────┐  │  │  ┌─────────────────────────┐  │      │
│  │  │   游戏状态组件           │  │  │  │   渲染资源               │  │      │
│  │  │   - Transform           │  │  │  │   - GpuMesh             │  │      │
│  │  │   - Velocity            │  │  │  │   - GpuMaterial         │  │      │
│  │  │   - Health              │  │  │  │   - BindGroup           │  │      │
│  │  └─────────────────────────┘  │  │  └─────────────────────────┘  │      │
│  │                               │  │                               │      │
│  └───────────────┬───────────────┘  └───────────────┬───────────────┘      │
│                  │                                  │                       │
│                  │      ExtractSchedule             │                       │
│                  │      (提取调度)                   │                       │
│                  └──────────────────────────────────▶                       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 7.2 提取流程

```rust
/// 提取系统示例
fn extract_meshes(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &Handle<Mesh>), Changed<Transform>>,
) {
    for (entity, transform, mesh) in query.iter() {
        // 将主世界的数据提取到渲染世界
        commands.spawn((
            ExtractedTransform(*transform),
            mesh.clone(),
        ));
    }
}
```

---

## 8. 关键文件清单

| 文件路径 | 功能描述 |
|---------|---------|
| `crates/bevy_app/src/app.rs` | App 主结构和生命周期管理 |
| `crates/bevy_app/src/main_schedule.rs` | 主调度配置 |
| `crates/bevy_app/src/schedule_runner.rs` | 调度运行器实现 |
| `crates/bevy_winit/src/lib.rs` | Winit 插件入口 |
| `crates/bevy_winit/src/state.rs` | 应用状态管理和事件循环 |
| `crates/bevy_render/src/lib.rs` | 渲染系统主模块 |
| `crates/bevy_ecs/src/schedule/mod.rs` | ECS 调度系统 |

---

## 9. 刷新机制总结

### 9.1 三种运行模式

| 模式 | 适用场景 | 特点 |
|------|---------|------|
| **ScheduleRunner** | 无窗口应用、服务器 | 固定频率循环 |
| **Winit (Continuous)** | 游戏 | 连续渲染，最大化帧率 |
| **Winit (Reactive)** | GUI 应用 | 事件驱动，节省资源 |

### 9.2 刷新流程图

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              完整刷新流程                                    │
│                                                                             │
│  1. 事件循环 (Winit)                                                         │
│     │                                                                       │
│     ▼                                                                       │
│  2. 确定更新模式 (Continuous/Reactive)                                       │
│     │                                                                       │
│     ▼                                                                       │
│  3. 检查是否需要更新 (should_update)                                         │
│     │                                                                       │
│     ▼                                                                       │
│  4. 执行 App::update()                                                       │
│     │                                                                       │
│     ├──▶ 4.1 运行主调度 (Main Schedule)                                      │
│     │       │                                                               │
│     │       ├──▶ First / PreUpdate / Update / PostUpdate / Last             │
│     │       │                                                               │
│     │       └──▶ 子应用更新                                                  │
│     │               │                                                       │
│     │               └──▶ Render World 更新                                   │
│     │                       │                                               │
│     │                       ├──▶ Extract (提取)                              │
│     │                       ├──▶ Prepare (准备)                              │
│     │                       ├──▶ Queue (排队)                                │
│     │                       └──▶ Render (渲染)                               │
│     │                                                                       │
│     ▼                                                                       │
│  5. 设置控制流 (ControlFlow)                                                 │
│     │                                                                       │
│     ├──▶ Wait (等待事件)                                                     │
│     ├──▶ WaitUntil (超时等待)                                                │
│     └──▶ Poll (立即继续)                                                     │
│     │                                                                       │
│     ▼                                                                       │
│  6. 请求重绘 (request_redraw)                                                │
│     │                                                                       │
│     └──────────────────────────────────────────────────────────────────▶    │
│                              (循环到步骤 1)                                  │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 9.3 性能优化要点

1. **使用 Reactive 模式**: GUI 应用在空闲时节省 CPU/GPU 资源
2. **提取阶段优化**: 只提取变化的数据到渲染世界
3. **并行执行**: 利用 Bevy 的并行调度系统
4. **控制帧率**: 通过 `wait` 参数限制最大帧率

---

## 10. 使用示例

### 10.1 连续模式 (游戏)

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(WinitSettings {
            update_mode: UpdateMode::Continuous,
            ..default()
        })
        .add_systems(Update, game_logic)
        .run();
}
```

### 10.2 响应式模式 (GUI)

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(WinitSettings {
            update_mode: UpdateMode::Reactive {
                wait: Duration::from_millis(16), // 60 FPS max
                react_to_window_events: true,
                react_to_device_events: true,
                react_to_user_events: true,
            },
            ..default()
        })
        .add_systems(Update, ui_update)
        .run();
}
```

### 10.3 无窗口模式

```rust
use bevy::prelude::*;
use bevy::app::ScheduleRunnerPlugin;
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs(1)))
        .add_systems(Update, server_tick)
        .run();
}
```
