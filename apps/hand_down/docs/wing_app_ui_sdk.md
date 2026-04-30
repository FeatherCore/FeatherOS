# Wing AppUi SDK Notes

本文记录外部 Wing-managed Rust 应用当前推荐写法。目标是让应用保持轻量、声明式、无宏，并适合 NuttX task + 小屏 surface。

## Frame Loop

外部 App 推荐从 SDK facade 导入：

```rust
use wing::app_ui_sdk::prelude::*;
```

NuttX builtin 应用入口只需要创建应用状态，然后交给 `run_app_ui_rgb565_from_argv()`：

```rust
let mut app = MyApp::default();

unsafe {
    run_app_ui_rgb565_from_argv::<128, _>(
        argc,
        argv,
        Color::rgb(8, 14, 24),
        AppUiFramePacing::from_hz(30),
        &mut app,
    )
}
```

`run_app_ui_rgb565_from_argv()` 会从 NuttX `argc/argv` 打开 `WingSurface`，把 surface/open/render 错误转换成进程 exit code，并委托给 `run_app_ui_rgb565()`。

如果应用已经持有 `WingSurface`，也可以直接调用底层 runner：

```rust
match run_app_ui_rgb565::<128, _>(
    surface,
    Color::rgb(8, 14, 24),
    AppUiFramePacing::from_hz(30),
    &mut app,
) {
    Ok(()) => 0,
    Err(error) => error.exit_code(),
}
```

`run_app_ui_rgb565()` 会检查 RGB565 surface 格式，并在内部创建 `AppUiRuntime<N>` 和 `AppUiLoop`。它循环调用 `drive_rgb565()`，完成 input drain、command update、prepare、view compose、dirty render、frame stats 和 sleep hint。

当应用返回 `AppUiFlow::Exit` 或 `WingSurface` 被 drop 时，SDK 会在关闭 surface queue 前发送 close 事件。Shell 用这个事件回收 Wing-managed surface 并回到 Home，所以默认应用应该通过 `CMD_EXIT -> AppUiFlow::Exit` 结束，而不是在应用内部直接阻塞或循环等待 Shell 关闭。

如果应用需要自定义 loop、读取 `AppUiFrameResult` 或周期性上报 `AppUiFrameStats`，仍然可以直接使用 `AppUiLoop::drive_rgb565()`。

`src/app_ui_template.rs` 提供了一个可编译检查的最小模板：它使用 `app_ui_sdk::prelude::*`、`run_app_ui_rgb565_from_argv()`、typed signal、`AppUiScheduledApp` 和 `AppUiStagedSchedule`。当前模板不导出 `#[no_mangle]` builtin 入口，因此不会自动注册到 NuttX；把它变成真实 app 时，再在平台注册表中绑定具体入口名。

## Dirty Render

AppUi runtime 会用 `AppUiKey` 对比当前 frame 和上一帧的 spec。未变化的控件不会触发重绘；变化、出现或消失的控件会把旧 bounds 和新 bounds 加入 dirty region。

dirty region 是固定容量的多矩形列表，当前容量为 `APP_UI_DIRTY_RECT_CAPACITY`。正常情况下 runtime 会逐个 clip 渲染并提交多个小 dirty rect；如果列表溢出、frame spec 溢出，或者上一帧 spec 溢出，则退化为单个 union/full rect。这样热路径仍然没有 heap 分配，最坏情况也保持确定性。

`AppUiRenderResult` 和 `AppUiFrameResult` 会记录 `dirty_rects` 与 `dirty_overflowed`。自定义 loop 可以用这些字段做性能采样，例如观察动画控件是否只刷新局部区域。

## Frame Budget

`AppUiFrameResult::budget` 提供最近一帧的固定容量预算快照，`AppUiFrameStats::last_budget()` 会保留同一份信息，方便低频诊断或 System 页面采样。

当前预算字段覆盖四类热路径资源：

- `spec_count/spec_capacity`：当前声明式 frame spec 使用量。
- `previous_spec_count`：上一帧用于 diff 的 spec 使用量，容量与当前 frame 相同。
- `event_count/event_capacity`：本帧输入事件队列使用量。
- `command_count/command_capacity`：typed signal/command 队列使用量。
- `dirty_rects/dirty_rect_capacity`：局部刷新 dirty rect 使用量。

`spec_overflowed`、`previous_spec_overflowed`、`event_overflowed`、`command_overflowed` 和 `dirty_overflowed` 可以分别定位容量压力；`any_overflow()` 用于快速判断本帧是否触发过降级路径。所有字段都是定长数值，不需要 heap 或运行时字符串。

应用如果要把预算状态画到自己的 UI，可以实现 `AppUiApp::observe_frame()` 或 `AppUiScheduledApp::observe_scheduled_frame()`。这个 hook 在 `drive_rgb565()` 完成本帧 render/budget 计算后调用，适合把 `AppUiFrameResult::budget` 保存成应用状态；下一帧 `view_scheduled()` 再用静态标签显示它。`rust_surface_demo` 当前的 `APP BUDGET` 行就是这种模式。

## App Shape

应用状态是普通 struct。UI 由 `view_scheduled()` 每帧声明，输入不直接改状态，而是先映射为 typed signal：

```rust
const CMD_EXIT: AppUiSignalId = AppUiSignalId::new(900);
const CMD_SET_MODE: AppUiSignalId = AppUiSignalId::new(910);

type MySchedule = AppUiStagedSchedule<
    (
        fn(&mut MyApp, AppUiContext, AppUiCommand) -> AppUiFlow,
        fn(&mut MyApp, AppUiContext, AppUiCommand) -> AppUiFlow,
    ),
    fn(&mut MyApp, AppUiContext, AppUiCommand) -> AppUiFlow,
>;

impl AppUiScheduledApp<128> for MyApp {
    type Schedule = MySchedule;

    fn schedule(&mut self) -> Self::Schedule {
        AppUiStagedSchedule::new((flow_system, state_system), tick_system)
    }

    fn enqueue_scheduled_event(
        &mut self,
        _context: AppUiContext,
        gesture: AppUiGesture,
        commands: &mut AppUiCommandQueue,
    ) -> AppUiFlow {
        match gesture {
            AppUiGesture::Click { key, .. } if key == KEY_BACK => {
                commands.signal(CMD_EXIT, 0);
            }
            AppUiGesture::Click { key, .. } if is_mode_key(key) => {
                commands.signal(CMD_SET_MODE, mode_index(key) as i32);
            }
            _ => {}
        }

        AppUiFlow::Continue
    }

    fn view_scheduled(&self, context: AppUiContext, frame: &mut AppUiBuilder<'_, 128>) {
        compose_my_frame(frame, context.width(), context.height(), self.mode);
    }
}
```

## Stage Rules

`AppUiStagedSchedule<Input, Tick, Prepare>` 是静态阶段拆分：

- input stage 处理 `Gesture` 和 `Signal`。
- prepare stage 处理 `Prepare`，运行在 update 之后、view 之前。
- tick stage 处理 `Tick`，运行在帧完成之后。

不需要 prepare 时使用 `AppUiStagedSchedule::new(input, tick)`；需要 prepare 时使用 `with_prepare(input, tick, prepare)`.

## System Rules

系统函数显式接收应用状态、context 和 command，不做参数注入：

```rust
fn flow_system(app: &mut MyApp, _context: AppUiContext, command: AppUiCommand) -> AppUiFlow {
    match command {
        AppUiCommand::Signal(signal) if signal.is(CMD_EXIT) => AppUiFlow::Exit,
        _ => AppUiFlow::Continue,
    }
}

fn tick_system(app: &mut MyApp, _context: AppUiContext, command: AppUiCommand) -> AppUiFlow {
    if let AppUiCommand::Tick = command {
        app.toast_ticks = app.toast_ticks.saturating_sub(1);
    }

    AppUiFlow::Continue
}
```

保持约束：

- 外部 app 优先使用 `app_ui_sdk::prelude::*`，避免直接依赖 `app_ui.rs` 的内部组织。
- command ID 用 `AppUiSignalId`，控件 ID 用 `AppUiKey`。
- command queue 固定容量，不放字符串 payload。
- 控件 helper 只声明视觉和命中区，不保存应用状态。
- 每个可复用控件必须使用稳定 `AppUiKey`，否则 diff 会退化成删除旧节点再创建新节点。
- idle 应用实现 `frame_hint_scheduled()` 返回 `AppUiFrameHint::idle_default()`，动画应用保持 `active()`。
