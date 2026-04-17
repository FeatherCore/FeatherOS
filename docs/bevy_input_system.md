# Bevy 输入系统分析文档

## 1. 概述

Bevy 的输入系统基于 ECS（Entity Component System）架构，采用**事件驱动**的设计模式。输入事件从底层平台（winit）产生，经过处理后转换为 Bevy 消息（Message），最终更新为可查询的资源（Resource）。

### 1.1 核心架构

```
winit (平台事件)
    ↓ 转换
Bevy 消息 (Message)
    ↓ 消费
输入系统 (System)
    ↓ 更新
资源 (Resource)
    ↓ 查询
游戏逻辑
```

### 1.2 核心组件

| 组件 | 文件路径 | 功能 |
|------|---------|------|
| 按钮输入核心 | `crates/bevy_input/src/button_input.rs` | 通用按钮状态管理 |
| 输入模块入口 | `crates/bevy_input/src/lib.rs` | 模块导出 |
| 鼠标输入 | `crates/bevy_input/src/mouse.rs` | 鼠标按键/移动/滚轮 |
| 键盘输入 | `crates/bevy_input/src/keyboard.rs` | 键盘按键处理 |
| 触摸输入 | `crates/bevy_input/src/touch.rs` | 多点触控支持 |
| 手柄输入 | `crates/bevy_input/src/gamepad.rs` | 游戏手柄 |
| winit 集成 | `crates/bevy_winit/src/lib.rs` | 平台事件转换 |

---

## 2. 核心数据结构

### 2.1 `ButtonInput<T>` - 通用按钮输入资源

```rust
pub struct ButtonInput<T: Clone + Eq + Hash + Send + Sync + 'static> {
    pressed: HashSet<T>,       // 当前按下的按钮
    just_pressed: HashSet<T>,  // 当前帧刚按下的按钮
    just_released: HashSet<T>, // 当前帧刚释放的按钮
}
```

**关键方法**：

| 方法 | 时间复杂度 | 功能 |
|------|-----------|------|
| `press(input)` | O(1)~ | 注册按下 |
| `release(input)` | O(1)~ | 注册释放 |
| `pressed(input)` | O(1)~ | 检查是否按下 |
| `just_pressed(input)` | O(1)~ | 检查当前帧是否刚按下 |
| `just_released(input)` | O(1)~ | 检查当前帧是否刚释放 |
| `clear()` | O(1)~ | 清空 just_pressed/just_released |
| `reset_all()` | O(n) | 清空所有状态 |

### 2.2 `ButtonState` - 按钮状态

```rust
pub enum ButtonState {
    Pressed,
    Released,
}
```

---

## 3. 鼠标输入

### 3.1 鼠标事件类型

```rust
// 鼠标按键事件
pub struct MouseButtonInput {
    button: MouseButton,  // 按键 (Left/Right/Middle/Back/Forward)
    state: ButtonState,   // 按下/释放
    window: Entity,       // 接收事件的窗口
}

// 鼠标移动事件
pub struct MouseMotion {
    delta: Vec2,  // 相对于上次的位置变化
}

// 鼠标滚轮事件
pub struct MouseWheel {
    unit: MouseScrollUnit,  // Line 或 Pixel
    x: f32,                 // 水平滚动
    y: f32,                 // 垂直滚动
    window: Entity,         // 接收事件的窗口
}
```

### 3.2 鼠标资源

```rust
// 鼠标按键状态
Res<ButtonInput<MouseButton>>

// 累积鼠标移动（每帧重置）
pub struct AccumulatedMouseMotion {
    delta: Vec2,
}

// 累积鼠标滚动（每帧重置）
pub struct AccumulatedMouseScroll {
    unit: MouseScrollUnit,
    delta: Vec2,
}
```

### 3.3 鼠标处理系统

```rust
pub fn mouse_button_input_system(
    mut mouse_button_input: ResMut<ButtonInput<MouseButton>>,
    mut mouse_button_input_events: MessageReader<MouseButtonInput>,
) {
    // 1. 清空 just_pressed/just_released
    mouse_button_input.bypass_change_detection().clear();
    
    // 2. 处理事件队列
    for event in mouse_button_input_events.read() {
        match event.state {
            ButtonState::Pressed => mouse_button_input.press(event.button),
            ButtonState::Released => mouse_button_input.release(event.button),
        }
    }
}

// 累积鼠标移动
pub fn accumulate_mouse_motion_system(
    mut mouse_motion_event: MessageReader<MouseMotion>,
    mut accumulated_mouse_motion: ResMut<AccumulatedMouseMotion>,
) {
    let mut delta = Vec2::ZERO;
    for event in mouse_motion_event.read() {
        delta += event.delta;
    }
    accumulated_mouse_motion.delta = delta;
}
```

---

## 4. 键盘输入

### 4.1 键盘事件

```rust
pub struct KeyboardInput {
    key_code: KeyCode,        // 物理键码 (位置相关)
    logical_key: Key,         // 逻辑键 (布局相关)
    state: ButtonState,       // 按下/释放
    text: Option<SmolStr>,    // 产生的文本
    repeat: bool,             // 是否自动重复
    window: Entity,           // 接收事件的窗口
}

// 窗口失去焦点事件
pub struct KeyboardFocusLost;
```

### 4.2 键盘码类型

#### `KeyCode` - 物理键码

表示按键的物理位置，与键盘布局无关。

```rust
pub enum KeyCode {
    KeyA, KeyB, KeyC,    // 字母键
    Digit0, Digit1,      // 数字键
    ArrowLeft, ArrowRight, // 方向键
    F1, F2, ... F35,     // 功能键
    ShiftLeft, ShiftRight, // 修饰键
    // ... 更多
}
```

#### `Key` - 逻辑键

表示按键产生的实际字符，受键盘布局影响。

```rust
pub enum Key {
    Character(SmolStr),  // 字符 (如 "a", "A", "1")
    Enter,               // 回车
    Space,               // 空格
    ArrowUp,             // 上箭头
    F1, F2, ... F35,     // 功能键
    Alt, Control, Shift, // 修饰键
    // ... 更多
}
```

### 4.3 键盘处理系统

```rust
pub fn keyboard_input_system(
    mut keycode_input: ResMut<ButtonInput<KeyCode>>,
    mut key_input: ResMut<ButtonInput<Key>>,
    mut keyboard_input_reader: MessageReader<KeyboardInput>,
    mut keyboard_focus_lost_reader: MessageReader<KeyboardFocusLost>,
) {
    // 1. 清空 just_pressed/just_released
    keycode_input.bypass_change_detection().clear();
    key_input.bypass_change_detection().clear();

    // 2. 处理键盘事件
    for event in keyboard_input_reader.read() {
        match event.state {
            ButtonState::Pressed => {
                keycode_input.press(event.key_code);
                key_input.press(event.logical_key.clone());
            }
            ButtonState::Released => {
                keycode_input.release(event.key_code);
                key_input.release(event.logical_key.clone());
            }
        }
    }

    // 3. 窗口失去焦点时释放所有按键
    if !keyboard_focus_lost_reader.is_empty() {
        keycode_input.release_all();
        keyboard_focus_lost_reader.clear();
    }
}
```

---

## 5. 触摸输入

### 5.1 触摸事件

```rust
pub struct TouchInput {
    phase: TouchPhase,  // 触摸阶段
    position: Vec2,     // 触摸位置
    window: Entity,     // 接收事件的窗口
    force: Option<ForceTouch>, // 按压力度
    id: u64,            // 触摸唯一 ID
}

pub enum TouchPhase {
    Started,   // 开始触摸
    Moved,     // 移动
    Ended,     // 结束触摸
    Canceled,  // 系统取消 (如窗口失焦)
}

pub enum ForceTouch {
    Calibrated {
        force: f64,
        max_possible_force: f64,
        altitude_angle: Option<f64>,
    },
    Normalized(f64),  // 归一化力度
}
```

### 5.2 触摸状态资源

```rust
pub struct Touches {
    pressed: HashMap<u64, Touch>,        // 当前按下的触摸
    just_pressed: HashMap<u64, Touch>,   // 当前帧刚按下的触摸
    just_released: HashMap<u64, Touch>,  // 当前帧刚释放的触摸
    just_canceled: HashMap<u64, Touch>,  // 当前帧刚取消的触摸
}

// 单个触摸信息
pub struct Touch {
    id: u64,
    start_position: Vec2,
    previous_position: Vec2,
    position: Vec2,
    start_force: Option<ForceTouch>,
    previous_force: Option<ForceTouch>,
    force: Option<ForceTouch>,
}

impl Touch {
    pub fn delta(&self) -> Vec2 {       // 当前位置 - 上次位置
        self.position - self.previous_position
    }
    pub fn distance(&self) -> Vec2 {    // 当前位置 - 起始位置
        self.position - self.start_position
    }
}
```

### 5.3 触摸处理系统

```rust
pub fn touch_screen_input_system(
    mut touch_state: ResMut<Touches>,
    mut touch_input_reader: MessageReader<TouchInput>,
) {
    // 1. 清空 just_* 状态
    touch_state.just_pressed.clear();
    touch_state.just_released.clear();
    touch_state.just_canceled.clear();

    if !touch_input_reader.is_empty() {
        // 2. 更新所有触摸的上一次位置
        for touch in touch_state.pressed.values_mut() {
            touch.previous_position = touch.position;
            touch.previous_force = touch.force;
        }

        // 3. 处理触摸事件
        for event in touch_input_reader.read() {
            touch_state.process_touch_event(event);
        }
    }
}

impl Touches {
    fn process_touch_event(&mut self, event: &TouchInput) {
        match event.phase {
            TouchPhase::Started => {
                self.pressed.insert(event.id, event.into());
                self.just_pressed.insert(event.id, event.into());
            }
            TouchPhase::Moved => {
                if let Some(mut new_touch) = self.pressed.get(&event.id).cloned() {
                    new_touch.position = event.position;
                    new_touch.force = event.force;
                    self.pressed.insert(event.id, new_touch);
                }
            }
            TouchPhase::Ended => {
                if let Some((_, v)) = self.pressed.remove_entry(&event.id) {
                    self.just_released.insert(event.id, v);
                } else {
                    self.just_released.insert(event.id, event.into());
                }
            }
            TouchPhase::Canceled => {
                if let Some((_, v)) = self.pressed.remove_entry(&event.id) {
                    self.just_canceled.insert(event.id, v);
                } else {
                    self.just_canceled.insert(event.id, event.into());
                }
            }
        }
    }
}
```

---

## 6. 整体数据流

### 6.1 完整流程图

```
┌─────────────────────────────────────────────────────────────────────┐
│                          winit 事件循环                               │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────────┐  │
│  │ 鼠标事件  │    │ 键盘事件  │    │ 触摸事件  │    │ 手柄事件 ... │  │
│  └─────┬────┘    └─────┬────┘    └─────┬────┘    └──────┬───────┘  │
│        │               │               │                │          │
│        ▼               ▼               ▼                ▼          │
│  ┌───────────────────────────────────────────────────────────────┐ │
│  │                     winit_runner.rs                           │ │
│  │  转换 winit 事件为 Bevy Message                               │ │
│  └──────────────────────────┬────────────────────────────────────┘ │
│                             │                                      │
└─────────────────────────────┼──────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      bevy_input crate                               │
│                                                                     │
│  ┌─────────────────┐    ┌─────────────────┐    ┌────────────────┐  │
│  │ mouse.rs        │    │ keyboard.rs     │    │ touch.rs       │  │
│  │                 │    │                 │    │                │  │
│  │ MessageReader   │    │ MessageReader   │    │ MessageReader  │  │
│  │       │         │    │       │         │    │       │        │  │
│  │       ▼         │    │       ▼         │    │       ▼        │  │
│  │ 处理系统        │    │ 处理系统        │    │ 处理系统       │  │
│  │       │         │    │       │         │    │       │        │  │
│  │       ▼         │    │       ▼         │    │       ▼        │  │
│  │ ButtonInput     │    │ ButtonInput     │    │ Touches        │  │
│  │ <MouseButton>   │    │ <KeyCode/Key>   │    │ Resource       │  │
│  │                 │    │                 │    │                │  │
│  │ Accumulated     │    │                 │    │                │  │
│  │ MouseMotion     │    │                 │    │                │  │
│  │                 │    │                 │    │                │  │
│  │ Accumulated     │    │                 │    │                │  │
│  │ MouseScroll     │    │                 │    │                │  │
│  └─────────────────┘    └─────────────────┘    └────────────────┘  │
│                                                                     │
└─────────────────────────────┬───────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      游戏逻辑系统                                    │
│                                                                     │
│  fn handle_input(                                                    │
│      keys: Res<ButtonInput<KeyCode>>,                                │
│      mouse: Res<ButtonInput<MouseButton>>,                           │
│      touch: Res<Touches>,                                            │
│  ) {                                                                 │
│      if keys.just_pressed(KeyCode::Space) { /* 跳跃 */ }            │
│      if mouse.pressed(MouseButton::Left) { /* 射击 */ }             │
│      if let Some(touch) = touch.iter().next() { /* 触摸逻辑 */ }    │
│  }                                                                   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 6.2 帧处理时序

```
Frame N:
┌────────────────────────────────────────────────────────────────┐
│ 1. 系统调度开始                                                 │
│                                                                │
│ 2. 驱动层收集平台事件 (winit)                                   │
│    └→ 发送 Message                                             │
│                                                                │
│ 3. 输入处理系统执行 (PreUpdate 阶段)                             │
│    ├── mouse_button_input_system()                             │
│    ├── accumulate_mouse_motion_system()                        │
│    ├── accumulate_mouse_scroll_system()                        │
│    ├── keyboard_input_system()                                 │
│    └── touch_screen_input_system()                             │
│                                                                │
│ 4. 用户系统执行 (Update 阶段)                                    │
│    └→ 查询 Res<ButtonInput<T>> / Res<Touches>                  │
│                                                                │
│ 5. 帧结束 - just_pressed/just_released 被清空                   │
└────────────────────────────────────────────────────────────────┘

Frame N+1:
  just_pressed = {}  ← 新的帧开始，状态被清空
```

---

## 7. 关键设计模式

### 7.1 事件与资源的分离

| 层次 | 类型 | 生命周期 | 用途 |
|------|------|---------|------|
| 事件 (Message) | `MouseButtonInput` | 瞬时 | 传输底层事件 |
| 资源 (Resource) | `ButtonInput<MouseButton>` | 持久 | 提供便捷查询 |

### 7.2 帧边界状态清理

```rust
// 每帧开始时清空 just_pressed/just_released
// 这确保了 just_* 只在触发的那一帧为 true
resource.clear();

// 处理事件，填充 just_pressed/just_released
for event in events {
    resource.press(event.input);
}
```

### 7.3 变化检测优化

```rust
// 使用 bypass_change_detection() 避免不必要的系统重执行
// clear() 会触发变化检测，但内部清空不需要通知其他系统
resource.bypass_change_detection().clear();
```

---

## 8. 常用 API

### 8.1 鼠标输入

```rust
fn mouse_system(
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
) {
    // 检查按键状态
    if mouse.pressed(MouseButton::Left) { }
    if mouse.just_pressed(MouseButton::Right) { }
    if mouse.just_released(MouseButton::Middle) { }
    
    // 获取鼠标移动
    let delta = motion.delta;
    
    // 获取滚轮滚动
    let scroll_delta = scroll.delta;
    match scroll.unit {
        MouseScrollUnit::Line => { } // 按行
        MouseScrollUnit::Pixel => { } // 按像素
    }
}
```

### 8.2 键盘输入

```rust
fn keyboard_system(
    keys: Res<ButtonInput<KeyCode>>,
    text_keys: Res<ButtonInput<Key>>,
) {
    // 使用 KeyCode (物理位置)
    if keys.pressed(KeyCode::KeyW) { }  // WASD 移动
    if keys.just_pressed(KeyCode::Escape) { } // 暂停
    
    // 使用 Key (逻辑字符)
    if text_keys.just_pressed(Key::Character("a")) { } // 输入 "a"
}
```

### 8.3 触摸输入

```rust
fn touch_system(touches: Res<Touches>) {
    // 获取所有活动触摸
    for touch in touches.iter() {
        let pos = touch.position();
        let delta = touch.delta();
        let distance = touch.distance();
    }
    
    // 检查特定触摸
    if let Some(touch) = touches.get_pressed(id) { }
    if touches.just_pressed(id) { }
    if touches.any_just_pressed() { }
    
    // 获取第一个触摸位置
    if let Some(pos) = touches.first_pressed_position() { }
}
```

### 8.4 组合键

```rust
fn combo_system(keys: Res<ButtonInput<KeyCode>>) {
    // Ctrl+C
    if keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
        && keys.pressed(KeyCode::KeyC) 
    {
        println!("Copy!");
    }
    
    // Ctrl+Shift+A
    if keys.all_pressed([
        KeyCode::ControlLeft,
        KeyCode::ShiftLeft,
        KeyCode::KeyA,
    ]) {
        println!("Select All!");
    }
}
```

---

## 9. 手柄输入

### 9.1 手柄资源

```rust
// 手柄按钮状态
Res<ButtonInput<GamepadButton>>

// 手柄轴值 (摇杆/扳机)
pub struct GamepadAxisChangedEvent {
    gamepad: Entity,
    axis: GamepadAxis,
    value: AxisValue,
}

// 手柄连接状态
pub struct GamepadConnectionEvent {
    gamepad: Entity,
    connection: GamepadConnection,
}
```

---

## 10. 与 LVGL 输入系统对比

| 特性 | Bevy | LVGL |
|------|------|------|
| 架构模式 | ECS (Resource/Message) | 回调驱动 |
| 事件模型 | Message 系统 | 回调函数 + 事件 |
| 状态管理 | HashSet 资源 | 结构体字段 |
| 帧处理 | 每帧清空 just_* | 定时器读取 |
| 多点触控 | 原生支持 | 需要手势识别器 |
| 键盘布局 | KeyCode/Key 分离 | 仅键码 |
| 变化检测 | ECS 内置 | 手动管理 |

---

## 11. 代码位置索引

| 功能 | 文件 | 主要类型/函数 |
|-----|------|-------------|
| 按钮输入 | `button_input.rs` | `ButtonInput<T>` |
| 鼠标处理 | `mouse.rs` | `mouse_button_input_system()`, `AccumulatedMouseMotion` |
| 键盘处理 | `keyboard.rs` | `keyboard_input_system()`, `KeyCode`, `Key` |
| 触摸处理 | `touch.rs` | `touch_screen_input_system()`, `Touches`, `TouchPhase` |
| 模块入口 | `lib.rs` | 模块导出 |

---

*文档版本: 1.0*
*基于 Bevy 源码分析生成*
