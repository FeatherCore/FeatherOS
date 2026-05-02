# Wing 实现思路

本文定义重新实现 Wing 的方向。Wing 的 UI 风格对齐 `/home/uan-gpd/codes/windows-10-mobile-lvgl`，尤其是 Windows 10 Mobile / Lumia 风格的开始屏、通知面板、设置页、状态栏和导航手势。但实现方式不采用 LVGL 的对象树和回调模型，而是采用无宏、轻量、声明式 ECS。

当前最新方向：

```text
Wing = 小屏 Shell/UI runtime + 声明式 ECS
FHRE = no_std Rust 纯 3D 轻量游戏/图形引擎核心 + 绘制后端抽象 + scene/input/assets
```

FHRE 的“纯 3D 世界模型”不是历史参考，而是底层主线：Wing 的 UI 节点也应该是 FHRE 3D 世界中位于默认 Screen Canvas 平面 `z=0` 的对象。Wing 第一版不需要做复杂 3D UI，但它不能绕过 `Transform3D + Camera + Screen Canvas` 这套模型。FHRE 本身仍然要具备游戏运行时能力，后续 Wing 内的游戏应用可以直接基于 FHRE 开发。

## V4.7 对 FHRE EGL/OpenGL Worker 并行后端的适配边界

V4.7 继续 FHRE-first，Wing 不新增 route/page，不持有 EGL context、GL worker、packet、ring 或 fence：

- Wing 可以观察 FHRE 暴露的 `draw_chain_parallel_*`、`accel2d_ring_*`、`accel2d_fences_*` 统计，但不调用 `queue_draw_chain_with_contract()`，也不决定哪些 run 可并行。
- Wing 不生成或解析 `Accel2dCommandList`；`EglParallelAccelBackend` 的 worker submit、fence/readback、fallback/software replay 都由 FHRE backend 管理。
- Wing 不直接依赖 `sim-opengl-egl` feature；默认 `wing_build.sh` 不启用 OpenGL，不新增 EGL/OpenGL/pthread 链接依赖。
- 即使 `FHRE_SIM_OPENGL=egl` 编入 FHRE demo，Wing 仍不接管 NuttX sim 的 X11 framebuffer present 或 event loop。
- Wing 页面层继续只声明 UI tree，资源、placeholder、硬件/软件并行提交策略仍由 FHRE 管理。

## V4.6 对 FHRE Hybrid Parallel Renderer 的适配边界

V4.6 继续 FHRE-first，Wing 不新增 route/page，不持有 parallel scheduler、OpenGL context、packet、ring 或 fence：

- Wing 可以观察 FHRE 暴露的 `draw_chain_parallel_*`、draw-chain、executor/ring/fence stats，但不调用 `queue_draw_chain_with_contract()`。
- Wing 不生成或解析 `Accel2dCommandList`，也不决定哪些 run 可并行；overlap/barrier/fallback 由 FHRE `DrawList` 调度器处理。
- Wing 不直接依赖 `sim-opengl` 或 `sim-opengl-egl` feature；默认 `wing_build.sh` 不启用 OpenGL，不新增 GLX/EGL/OpenGL 链接依赖。
- `EglOpenGlAccel2dExecutor` 当前只是 FHRE 的 offscreen shim 边界，Wing 不参与 EGL pbuffer/surfaceless context、texture upload、FBO/readback 或 X11 event loop。
- Wing 页面层继续只声明 UI tree，资源、placeholder、硬件/软件并行提交策略仍由 FHRE 管理。

## V4.5 对 FHRE OpenGL/Executor 边界的适配边界

V4.5 继续 FHRE-first，Wing 不新增 route/page，不持有 OpenGL context、packet、ring 或 fence：

- Wing 可以观察 FHRE 暴露的 draw-chain、executor/ring/fence stats，但不生成或解析 `Accel2dCommandList`。
- Wing 不直接依赖 `sim-opengl` feature；默认 `wing_build.sh` 不启用 OpenGL，不新增 GLX/EGL/OpenGL 链接依赖。
- `SimOpenGlAccel2dExecutor` 只是 FHRE 的 renderer backend probe，Wing 不参与 context 创建、texture upload、FBO/readback 或 X11 event loop。
- Wing 页面层继续只声明 UI tree，资源、fallback、placeholder、硬件提交策略仍由 FHRE 管理。
- 当前仍不恢复 demo 线程内 `sim_x11events()`；NuttX sim 的 X11 event/update pumping 继续由 sim loop task 负责。

## V4.4 对 FHRE Descriptor Ring 的适配边界

V4.4 继续 FHRE-first，Wing 不新增 route/page，不实现私有 ring/fence 或 codec token：

- Wing 可以观察 FHRE 暴露的 `accel2d_ring_*` / `accel2d_fences_*` stats，但不生成或解析 `Accel2dSubmissionRing`。
- Wing 不直接持有 fence、framebuffer descriptor 或 source descriptor；硬件提交由 FHRE backend 管理。
- Wing 不拥有 `CodecPipelineJobToken`；资源准备、fallback、placeholder 仍由 FHRE 管理。
- `wing_build.sh` 仍是 FHRE V4.4 的兼容性回归入口，当前不新增 Windows 10 Mobile 风格页面实现。

## V4.3 对 FHRE 2D Packet 化的适配边界

V4.3 继续 FHRE-first，Wing 不新增 route/page，不实现私有硬件 packet 或 codec executor：

- Wing 可以观察 FHRE 暴露的 draw chain / codec stats，但不生成或解析 `Accel2dCommandList`。
- Wing 不直接持有 framebuffer/source descriptor，不参与 DMA2D/PXP/VG-Lite packet build。
- Wing 不拥有 `CodecPipelineBackend`；资源准备、fallback、placeholder 仍由 FHRE 管理。
- `wing_build.sh` 仍是 FHRE V4.3 的兼容性回归入口，当前不新增 Windows 10 Mobile 风格页面实现。

## V4.2 对 FHRE Mock 加速闭环的适配边界

V4.2 继续 FHRE-first，Wing 不新增 route/page，不实现私有硬件后端：

- Wing 可以观察 `MockAccelBackend` 带来的 `draw_chain_hw_runs/draw_chain_sw_runs/draw_chain_fallbacks`，但不生成或解析 `DrawChainImageDescriptor`。
- Wing 不拥有 codec job 状态机，只消费 FHRE 暴露的 `codec_pipeline_*` 与资源 cache stats。
- missing image、mask/layer fallback、placeholder 仍由 FHRE 底座处理；Wing 页面层继续只声明 UI tree。
- `wing_build.sh` 仍是 FHRE V4.2 的兼容性回归入口，当前不新增 Windows 10 Mobile 风格页面实现。

## V4.1 对 FHRE 底座清理的适配边界

V4.1 仍然 FHRE-first，Wing 不新增 route/page，不实现私有硬件后端：

- Wing 只消费 FHRE 输出的 `DrawCommand`、`RenderStats` 和资源 cache 结果，不生成私有 `DrawChainOpPayload`。
- Wing 不反查 FHRE draw-chain payload，不关心 DMA2D/GPU descriptor 如何生成；真实后端成功、fallback 或 unsupported 都由 FHRE stats 呈现。
- Wing 对 missing resource 的观察改看 `cache_load_failures` / `codec_missing_resources` / `cache_decode_placeholder_missing`；`cache_decode_failures` 不再包含 missing。
- `wing_build.sh` 继续是 FHRE V4.1 的兼容性回归入口，当前不新增 Windows 10 Mobile 风格页面实现。

## V4.0 对 FHRE 底座收口的适配边界

V4.0 阶段继续 FHRE-first，Wing 不新增页面能力，只做只读消费和回归验证：

- Wing 不实现私有 draw-chain、DMA2D/GPU 提交、mask/layer 合并或资源硬件 decode 策略。
- Wing 只通过 FHRE `RenderStats` 观察 `draw_chain_*`、`codec_pipeline_*`、`cache_decode_placeholders` 与 `cache_decode_placeholder_*`。
- 对资源缺失或格式不支持，Wing 页面仍只声明 `UiBuilder -> UiTree -> FHRE DrawCommand`；具体 `ResourceDecodeResult::Placeholder`、fallback image 和 cache stats 由 FHRE 处理。
- `wing_build.sh` 本阶段是 FHRE 改动的兼容性回归入口：必须继续同时编入 `fhre_demo` 与 `wing_demo`，但不因为 FHRE V4.0 增加新的 Wing route/page。

## V3.10 对 FHRE run 级链路调度的适配边界

V3.10 继续保持 Wing 不扩页面/route 的边界，只把 FHRE 的 run 级链路调度能力纳入 shell 观测和适配：

- Wing 不生成私有 draw chain，不关心如何拼接 Layer/Mask/Clip；页面层只输出 `UiBuilder -> UiTree -> FHRE DrawCommand`。
- `DrawList` 仍由 FHRE 生成 draw commands/chain。Wing 只消费 `DrawChain` run 统计（run/hw run/sw run/split/parallel hint）与现有 `RenderStats`（top chain task / chain fallback），用于定位卡顿来源。
- `RenderStats` 中新增 run 级计数作为 shell 判断条件：有提交 run 时优先看 `draw_chain_hw_runs`，有回退 run 时优先检查对应命令区间是否是局部回退，避免把整帧当作整体硬件失败。
- `wing_demo` 不持有私有 codec 或硬件 backend 接口，不新增 page，不私有 raster/mask/layer/framebuffer 解码路径；只验证 Home、AllApps、Settings、Notifications、AppSwitcher、FHRE Sample 在 FHRE 资源、draw chain、codec stats 框架下稳定运行。

## V3.11 run 级链路与 codec 并行提示的适配边界

V3.11 保持 V3.10 边界，但把可观测边界收紧为“只读观测 + 提示输入”，不把调度语义引入 Wing 页面层。

- Wing 只消费 FHRE 的 run 结果与 pipeline 统计，不引入 page 侧 run 合并、资源并行或硬件 fallback 策略：
  - `draw_chain_runs / draw_chain_hw_runs / draw_chain_sw_runs / draw_chain_splits / draw_chain_parallel_hints`
  - `codec_pipeline_candidates / codec_pipeline_fallbacks / codec_pipeline_unsupported / codec_pipeline_overflows`
  - `codec_prewarm_parallel_hints`
- 仅作为建议：`parallel_hints`（run 级）与 `codec_prewarm_parallel_hints`（资源预热）不执行，不改动当前页面 draw 顺序，不参与命令重排。
- `FHRE` 保持“复杂 decode 仅 prewarm/path cache”边界，Wing 继续不持有私有 PNG/JPEG/TTF/SVG 解析与 cache 管理。
- `fhre_demo` 的 HUD/HW-accel/codec 可观测口径是 Wing 定位的主要输入：当出现回退时先看 run 区间与对应 `top chain task / top fallback`，再看 `cache / pipeline` 趋势。

## V3.9 对 FHRE 硬件可插手管线的适配边界

V3.9 仍不扩 Wing route/page。Wing 只消费 FHRE 暴露的 draw/resource/cache/stats API，不拥有 DMA2D/GPU draw chain，也不拥有 PNG/JPEG/TTF/SVG 硬件 decoder stage。

- Wing 页面层继续只允许 `UiBuilder -> UiTree -> FHRE DrawCommand`，不得直接 decode 图片/SVG/字体，不得私有 raster/mask/layer/framebuffer。
- FHRE 新增的 `DrawChain` / `DrawChainOp` / `DrawChainStats` 只作为 backend 可接管边界和 HUD 观察入口。Wing 不生成私有 chain，不绕过 `DrawList`，也不按具体芯片分支页面绘制逻辑。
- FHRE 新增的 `CodecPipelinePlan` / `CodecStagePlan` / `CodecPipelineStats` 只描述资源 prewarm/decode 的硬件候选阶段。Wing 资源仍通过 FHRE `ResourceLoader + ImageCache/GlyphCache/GlyphRunCache/SvgDocumentCache` 进入 cache；draw 热路径 cache miss 只显示 fallback。
- `RenderStats` 中的 draw chain 和 codec pipeline counters 可用于 Shell 性能定位：如果后续接入真实 DMA2D/GPU/decoder backend，Wing 仍只看 FHRE stats 判断 submitted/fallback/unsupported，不在页面层复制策略。
- `wing_demo` 本阶段继续验证 Home、All Apps、Settings、Notifications、App Switcher 和 FHRE Sample 可运行，并加载 manifest 允许的小 PNG/FRAW 资源；大图缺失仍走 FHRE fallback。

## V3.8 对 FHRE 绘制/资源管线的适配边界

V3.8 仍不扩 Wing route/page。Wing 只作为 FHRE draw/resource/dirty/cache/stats 的 Shell 压力场景，验证 Home、All Apps、Settings、Notifications、App Switcher 和 FHRE Sample 能稳定消费 FHRE API。

- Wing 页面层继续只允许 `UiBuilder -> UiTree -> FHRE DrawCommand`，不得直接 decode PNG/JPEG/TTF/OTF/SVG，不得私有 raster/mask/layer/framebuffer。
- Wing 资源继续通过 FHRE `ResourceLoader`、`ImageCache`、`GlyphCache`、`GlyphRunCache`、`SvgCache` 或 `SvgDocumentCache` 的 prewarm/cache 边界进入绘制；draw 热路径 cache miss 只显示 fallback。
- FHRE V3.8 新增的 `SvgDocumentCache` / `SvgDocumentCacheStats` 和 `RenderStats::mark_svg_document_cache()` 是 Shell 调试时的共享观察入口。Wing 不复制 SVG document 解析逻辑，只消费 FHRE resolver/cache。
- NuttX sim 下的 present copy fallback、Kconfig `$APPSDIR` 和 ROMFS 资源安装策略是 Wing 稳定启动的底线；`wing_build.sh` 继续同时验证 `wing_demo` 和 `fhre_demo`。
- Wing ROMFS 资源改为 `apps/wing/resource/windows10_mobile/romfs_manifest.txt` allowlist + byte budget。默认轻量 profile 只安装小 PNG 图标、FRAW 图标和 license；W10M 大图、壁纸和照片只保留在源码树作参考，未显式列入 manifest 不会进入运行时镜像。
- `fhre_build.sh` / `wing_build.sh` 会在 sim profile 切换时清理旧 `etc/fhre`、`etc/wing` 和 `etctmp.*`，避免 stale ROMFS 把历史 Wing 大图或字体重新打进 FHRE/Wing 镜像。

## V3.6 对 FHRE 性能基线的适配边界

本轮 Wing 仍不扩 route/page，也不追像素级 W10M 复刻。Wing 只消费 FHRE V3.6 的 draw/cache/resolver 能力，并把 Shell 卡顿定位继续交给 FHRE HUD：

- Wing 页面层仍只允许 `UiBuilder -> UiTree -> FHRE DrawCommand`，不得直接 decode PNG/JPEG/TTF/OTF/SVG，也不得直接写 framebuffer。
- FHRE 新增的 `RenderStats::benchmark_summary()`、top dispatch task 和 top fallback task 是 Wing 调试时的主要性能入口：如果 Home/All Apps/Settings/Notifications/App Switcher 变卡，先看 FHRE HUD 中 image/text/vector/layer/mask/blur/present 哪一类成为 top task。
- Wing 资源仍必须通过 `ResourceLoader -> ImageCache/GlyphCache/SvgCache/GlyphRunCache::prewarm()` 进入 cache；draw 热路径 cache miss 只显示 fallback，不在页面层补私有 decode。
- Wing 不新增新的 Shell route；现有 Home、All Apps、Settings、Notifications、App Switcher、FHRE Sample 继续作为 V3.6 压力验收场景。

## V3.5 对 FHRE 图形能力的适配边界

本轮不扩 Wing 页面，不追像素级复刻。Wing 只消费 FHRE V3.5 的资源、字体、SVG、图片和 draw command 能力，继续保持页面层声明式：

```text
UiBuilder -> UiTree -> FHRE DrawCommand
```

页面层仍禁止：

- 直接 decode PNG/JPEG/TTF/OTF/SVG。
- 直接写 framebuffer。
- 引入 LVGL widget/object/style cascade。
- 为某个页面复制一套私有 raster/mask/layer 逻辑。

FHRE V3.5 对 Wing 的实际意义：

- Wing 的 PNG/JPEG/FRAW 资源继续通过 `ResourceLoader + ImageCache::prewarm()` 进入 cache；绘制热路径只查 `ImageResolver`。
- 复杂图片资源有更明确 fallback 边界：baseline CMYK/YCCK JPEG、EXIF orientation 和 progressive grayscale/YCbCr 子集可以由 FHRE 处理；progressive+CMYK、arithmetic coding、12-bit 等仍 fallback，不影响 Shell route。
- SVG 图标/资源可以逐步从简单 path 扩展到 shape/style/use、linear/radial gradient、简单 clipPath/mask path 和 filter 映射；V3.3 的 fill 与 stroke 都受有界 A8 `VectorMaskScratch` 约束，并通过 RenderStats 暴露成功/overflow/fallback 成本。超出固定容量或复杂 CSS/mask/filter 时仍由 FHRE fallback。
- 字体层已能识别 TrueType glyf 与 OTF/CFF，并具备 CFF Type2 最小 raster 和 Latin-only `GlyphRun` shaping 入口。V3.5 让 `GlyphIdResolver`、`GlyphRunResolver` 和固定容量 `GlyphRunCache` 组成 FHRE 公共文本入口，`DrawLabel` 优先按 glyph id 绘制 shaped `GlyphRun`，再生成固定容量 `TextLayout<N>`，承载 wrap、align、selection、underline、strikethrough；复杂脚本 shaping 还不是 Shell 页面能力。
- Wing 页面仍不应该直接解析字体文件。短文本 shaped run 的 hit/miss/overflow、glyph-id draw、codepoint fallback 和 selection fallback 已由 FHRE `RenderStats` 暴露，Shell 卡顿定位应先看 FHRE HUD，而不是在 Wing page 内新增私有文本缓存。
- Wing 本轮只做兼容适配，不新增 route/page；页面层仍不能直接 decode 图片、字体、SVG，也不能直接写 framebuffer。

## 当前实现进度

截至 2026-05-01，新的 Wing Rust 原型已经具备这些最小能力：

- `no_std` crate 可独立 `cargo check`。
- `wing_demo` 可打开 NuttX framebuffer 并调用 Wing 绘制入口。
- 当前 demo 仍是手写 Shell 视觉原型，但已经开始使用 FHRE 的 `Camera + Transform3D + RenderNode + DrawList`。
- 壁纸和主面板已经通过 FHRE Screen Canvas 模型投影到平面绘制命令。
- `UiKey` / `UiKind` / `UiSpec` / `UiBuilder<N>` 已具备最小固定容量声明式 UI 描述能力。
- `UiTree<N, DIRTY>` 已具备最小固定容量 UI 节点状态承载能力，内部节点实体由 FHRE `EntityWorld<N>` 管理，节点数据已拆为 FHRE `ComponentStorage<UiNode/Parent/Children/LayoutBox/LayoutResult/Visual/Opacity/Clip/ResolvedClip/TextNode/IconNode/Animation/Button, N>`。
- `apply_ui_frame()` 已可按稳定 `UiKey` 将本帧 `UiBuilder` spec 应用到 `UiTree`，并记录 created/updated/removed 和 dirty rect。
- `ActionId` / `UiHit` 已具备最小 action 映射和 screen-space hit-test 能力。
- `AppId` / `AppEntry` / `AppRegistry<N>` / `AppSurfaceState` 已具备固定容量 app 数据模型，Home tiles、All Apps、App Switcher 和 Sample App 开始接到同一组 app/action/id 语义上。
- `WingAssetId` / `WingAssetManifest` 已建立 Windows 10 Mobile 资源 id/path manifest。`/home/uan-gpd/codes/windows-10-mobile-lvgl/squareline/assets/*.png` 已导入到 `apps/wing/resource/windows10_mobile/assets`，但 NuttX sim ROMFS 只安装 `romfs_manifest.txt` 中通过 byte budget 的轻量资源；上游 MIT `LICENSE` 随资源保留。
- 上游只存在于 LVGL `native-with-alpha` C 数组里的图标已预提取为 FHRE `.fraw` raw 图片资源，当前包括 `padlock`、`settings_back`、`stars_ic`、`wifi_icon`、`wp_about`、`wp_account`、`wp_apps`、`wp_devices`、`wp_network`、`wp_personalization`、`wp_privacy`、`wp_time`，仓库路径为 `apps/wing/resource/windows10_mobile/raw`，ROMFS 路径为 `/etc/wing/resource/windows10_mobile/raw/*.fraw`。
- Wing 的 `UiKind::Image` / `ImageNode` 已接入 `UiBuilder -> UiTree -> FHRE DrawCommand::DrawImage*`，页面可以声明图片资源、fit 和 tint，而不直接读文件、不直接操作 framebuffer。
- `ShellMode` / `ShellAction` / `ShellState` 已具备最小 shell 状态承载和 action 分发能力；当前 route 已扩展为 `Home`、`AllApps`、`LockScreen`、`Cortana`、`Settings(SettingsRoute)`、`App(AppId)`、`Notifications`、`AppSwitcher`，通用 tile detail 仍作为演示页保留。
- Wing 已能消费 FHRE 的 `PointerEvent` / `KeyEvent` / `InputEvent`，将 pointer up 命中的 tile 转换成 `ShellAction`。
- `ShellActionQueue<N>` 已基于 FHRE `EventQueue` 建立，`WingDemoState::dispatch_input_queue()` 可批量消费 FHRE `InputQueue` 并产出 shell action。
- `wing_demo` 已具备最小 NuttX input glue：非阻塞读取 `/dev/input0` 触摸和 `/dev/kbd` 键盘，转换到 FHRE `InputQueue` 后再由 Wing action pump 消费。
- `wing_demo` 的 NuttX framebuffer glue 已改为优先使用 `yres_virtual >= yres * 2` 的双 framebuffer page pan：绘制发生在不可见页，帧末通过 `FBIOPAN_DISPLAY` 翻页；没有双 framebuffer 时才 fallback 到 `640x480/480x640x32` 静态离屏 backbuffer copy，避免 X11 sim 中直接写可见 framebuffer 导致 wallpaper、tile、overlay 的逐步绘制过程被用户看到而闪烁。
- `wing_demo` 已接入 FHRE `FrameClock` / `FramePolicy` / `PresentStats`：主循环按实际 draw + present 耗时计算 sleep，不再固定“画完再睡 16.6ms”；present skip、page-pan 和 copy present 会进入统计。
- `wing_demo` 已改用共享 FHRE NuttX runtime glue，framebuffer/page-pan/backbuffer/input/resource 逻辑不再和 `fhre_demo` 各维护一份。
- 共享 runtime 已修正 page-pan dirty present 的隐藏页一致性：dirty 帧先同步当前可见页到不可见页，再按 dirty clip 绘制，避免非 dirty 区域因隐藏页旧内容出现闪烁。
- `wing_demo` 的触摸输入读取已做第一版 move coalescing：同一轮 poll 中连续 move 只保留最后一次，Down/Up 仍立即进入队列，避免拖动页面时输入队列被 move flood 塞满。
- Wing 已开始消费 FHRE `GestureRecognizer` 输出：tap 选择 tile，下滑进入通知面板，上滑进入应用切换器，左右滑返回 Home。
- `ScrollState` 已接入 Shell：All Apps 和 Settings 拥有固定容量滚动状态，字段包括 offset、max_offset、dragging、last pointer y；滚动页面优先消费纵向拖动，左右滑仍返回 Home，键盘 Up/Down 可滚动当前列表。
- `wing_demo` 已保留持久 `WingDemoState`，每帧声明 UI，但 launcher 节点可以在 `UiTree` 中复用。
- launcher 主面板、标题、进度条和 tile 背景/文字已经开始通过 `UiBuilder -> UiTree/apply_ui_frame -> FHRE DrawCommand` 绘制，tile 已绑定稳定 `ActionId`，底部状态文本显示当前 shell action 状态。
- overlay 已具备独立 `UiTree`，Home handle、Tile detail、Notification、App Switcher 已通过 `UiBuilder -> overlay UiTree -> FHRE DrawCommand` 绘制。
- `UiKind::Circle` 已下沉到 FHRE `DrawCommand::FillCircle`，通知头像、应用卡片图形不再依赖 Wing 私有圆形直绘。
- `UiKind::Line` 已下沉到 FHRE `DrawCommand::StrokeLine`，状态栏小符号已经可以用声明式线段/圆/圆角矩形组合表达。
- `UiKind::Icon` / `IconNode` 已接入，launcher 主图标已从多条 primitive 组合替换成单个声明式 icon 节点，render build 输出 FHRE `DrawCommand::DrawSvgIcon`。
- status bar、launcher 主面板、tile、符号图形、Home handle、Notification、App Switcher 都已汇入 `UiBuilder -> UiTree -> DrawCommand` 或 `DrawList` 管线，Wing shell 主视觉不再直接调用 `Surface` 绘制 helper。
- FHRE 已提供最小 `EntityWorld` / `ComponentStorage` / `Schedule` / `GameRuntime`，Wing 的 `UiTree` 节点状态已经开始迁移到 FHRE tiny ECS，并已从单个 `UiNode` 组件拆成节点、布局输入、投影后的布局结果、视觉、透明度、裁剪、文本、图标、按钮组件。
- `UiSpec::with_opacity()` 已具备，`Opacity` 组件在 render build 阶段与颜色 alpha 合成，用于后续主题透明度和页面过渡动画。
- `UiSpec::with_clip()` 已具备，`Clip` 组件保存声明侧 clip rect，`ResolvedClip` 保存父级继承和自身 clip 交集后的最终 screen clip，render/hit-test/dirty 都使用 clipped bounds；通知面板内容已开始通过该管线裁剪。
- `UiSpec::with_animation()` 已具备，`Animation` 组件可保存 FHRE `Tween`，当前 launcher 图标已经使用 opacity pulse 验证声明式 animation -> dirty -> render build 路径。
- `UiSpec::with_parent()` 已具备，`Parent` / `Children` 组件可在 `UiTree` 内重建固定容量父子链表；当前 launcher tile 的文字和 icon 已开始挂接到 tile 父节点，先验证层级数据流，后续再接布局继承和页面 diff。
- `LayoutSpace::Parent` 已具备第一版父局部坐标能力，`UiSpec::with_local_parent()`、`UiBuilder::child_text()`、`UiBuilder::child_icon()` 可把 child 的局部 rect 投影到父节点 screen rect 内；launcher tile 的文字和 icon 已改成局部坐标声明。
- `GridLayout` / `LayoutRule::GridCell` 已具备第一版固定容量 grid layout，`UiBuilder::grid_tile_action()` 可把 tile 声明为父节点内的 grid cell；launcher 六个 tile 已从手写 x/y 坐标改为 grid layout。
- `StackLayout` / `LayoutRule::StackItem` 已具备第一版固定容量 row/column layout，通知页 quick actions 和通知卡片背景已从手写 x/y 偏移改为父节点内的 stack item 声明；quick action 文字也已挂到对应 tile 父节点内。
- `ContentInset` / `LayoutSpace::Content` 已具备第一版父 content 坐标能力，父节点可声明内容内边距，子节点可相对父节点 content rect 布局；launcher 标题、进度条、磁贴 grid 和状态文本已开始使用 content-space 声明。
- layout clip inheritance 已具备第一版：子节点最终 clip 会与父节点 `ResolvedClip` 取交集，当前 quick action 文本可同时受自身 tile clip 和通知面板 content clip 约束。
- `wing_demo` 的大块 UI 状态和 `UiBuilder` 缓冲已从 NuttX 任务栈迁入持久 `WingDemoState`，sim 配置中的 `CONFIG_EXAMPLES_WING_DEMO_RUST_STACKSIZE` 提升到 `32768`，避免 8KB 默认栈在启动阶段溢出。
- Wing demo 已增加第一版 Lumia/Windows 10 Mobile 风格 token：`LumiaMetrics` 根据屏幕短边推导状态栏、底部导航栏、Start padding、gap、tile size 和 debug 字体缩放；`LumiaTheme` 承载深色背景、强调色、半透明 panel、tile、文字和 nav bar 颜色。
- Home/Start 已从居中玻璃面板改为更接近 windows-10-mobile-lvgl 的全屏 Start panel：深色背景、透明内容层、三列磁贴比例、wide tile、小 tile、All apps row、live tile icon opacity 动画和稳定 `ActionId` hit-test。
- Home/Start 已开始使用上游 Windows 10 Mobile PNG 资源：Start tiles 默认只预热并安装小图标，例如 `phone_ic.png`、`people_ic.png`、`outlook_ic.png`、`message_ic.png`、`photos_ic.png`；大 tile 图和壁纸不进默认 ROMFS，命中时走 FHRE fallback。
- Wing demo 启动时会通过 FHRE `ImageCache::prewarm()` 预热并 pin 常用 W10M 图标，例如 status/nav/settings 资源；route 切换后会在 draw 前显式 prewarm 当前页面资源，绘制热路径的 image resolver 只查 `ImageCache::view()`，cache miss 显示 fallback，不在绘制中 decode。
- 本轮 Wing 不扩页面范围，只跟随 FHRE 图形能力 V3.x / LVGL draw task parity 适配：资源路径可继续通过 `ImageCache` 获得 PNG/FRAW/JPEG fallback 能力；真实 TTF glyph、OTF/CFF Type2 最小 raster、Latin-only `GlyphRun` shaping、`GlyphIdResolver`、固定容量 `GlyphRunCache` 和 `TextLayout`、可选 kerning、glyph advance label wrap、multi-path SVG document、linear/radial gradient、简单 clipPath/filter/mask 统计、真实有界 A8 vector mask scratch、group opacity/nested transform、stroke cap/join、textured mesh、dirty/present stats、`FillStyle`/`GradientStyle`/`BorderStyle`/`BorderAlign`/`ShadowStyle`/`LineStyle`/`ArcStyle`/`TextStyle`/`ImageDrawStyle`/`TriangleStyle`/`MaskSpec`/`LayerSpec` 等接口先在 `fhre_demo` 验收。`ImageDrawStyle::blend` 已进入 FHRE 真实像素合成，`DrawImageStyled::clip_radius` 只负责图片圆角裁剪，边框由单独 `DrawBorder` 声明；PNG/JPEG/TTF/SVG 子集继续由 FHRE 自研 codec/cache 承担。Wing 页面仍只声明 `UiBuilder -> UiTree -> FHRE DrawCommand`，不直接操作 framebuffer 或运行时 decode。FHRE HUD 现在显示上一帧 aggregate draw stats，并以 fill/border/line/arc/image/text/vector/triangle/layer/mask、codec missing/invalid/truncated/unsupported/overflow、draw chain candidate/fallback、codec pipeline candidate/fallback、progressive JPEG/CFF/OpenType/SVG feature counters、vector mask/glyph-run/glyph-id/text layout counters、per-task dispatch、top fallback/top chain task、fixture pass/mismatch、draw-task fallback 和 software/accelerated/fallback path 等口径定位 Shell 卡顿来源；fixture manifest 让 FHRE 的 PNG/FRAW/JPEG/TTF/SVG 正向和负向预热、fallback 统计可重复验收。
- FHRE 已新增真实 `BeginLayer/EndLayer`、`PushMask/PopMask`、`PushBitmapMask`、有界 `LayerScratch`、rounded/bitmap mask、blurred shadow、layer opacity/recolor/blur 合成、图片圆角 mask 裁剪、边框 inside/center/outside 语义、line cap/join 字段和顶点色 triangle；Wing 后续的 notification/settings/card 视觉应只消费这些 FHRE draw API，不在 Wing 页面层复制 LVGL widget/object/style cascade，也不直接触碰 framebuffer。
- Home 底部 handle 已替换为 Windows Phone 风格三键 nav bar：Back、Windows logo、Search 使用上游 `wp_back.png`、`wp_logo.png`、`wp_search.png` 资源，点击 Search 进入 Cortana overlay。
- Notification overlay 已改为全屏下拉式深色 panel：顶部标题与日期、两行三列 quick actions、通知卡片列表和底部 accent handle，布局仍通过固定容量 `GridLayout` / `StackLayout` / clip 管线生成。
- Notification quick actions 已从手绘占位 icon 切换为上游 PNG 资源，例如 `wifi_ic.png`、`bluetooth_ic.png`、`airplane_ic.png`、`brightness_ic.png` 和 `settings_ic.png`。
- Home 已新增 Settings 入口，`ACTION_SETTINGS` 会进入独立 `ShellMode::Settings`，不再落入通用 tile detail；左右滑或 Back/Home 键仍返回 Home。
- Home 已新增 All Apps 入口，`ACTION_ALL_APPS` 会进入独立 `ShellMode::AllApps`；All Apps 页面使用 `AppRegistry` 固定列表绘制完整纵向 app row，内容区带 clip 和滚动 offset，点击 Settings/Lock/News/Stars/Thermal/Sample App/功能 demo 会回到同一 action router。
- Lock Screen 页面已接入 `ShellMode::LockScreen`，使用时间日期和 raw `padlock.fraw`；旧 `img2.png` 壁纸属于大图参考资源，默认 ROMFS 不安装，资源缺失时仍保持 fallback 绘制。
- Cortana overlay 已接入 `ShellMode::Cortana`，由底部 Search nav action 打开，使用 Windows Phone 黑色搜索面板、搜索 glyph 和返回 Home 行为。
- All Apps 已开始读取 `ShellState::app_surface_state(AppId)`，可把 app 的 stopped/running/focused/preview 最小生命周期语义显示为 row subtitle 和右侧 `RUN/FOCUS/CARD` 状态标记；这让 Home tiles、All Apps、App Switcher 和 Sample App 共享同一 app state，而不是各画各的假状态。
- Settings 页面已作为第一版声明式 shell page 接入：全屏深色 panel、Windows Phone 风格标题区、设置列表 rows、亮度 slider 和底部 nav bar 都走 `UiBuilder -> UiTree -> FHRE DrawCommand`，设置主列表和子页面共享滚动状态，页面实现位于 `demo_settings/mod.rs`。
- Settings 页面已开始使用上游 PNG 资源：header 使用 `wp_settings.png`，System 使用 `wp_system.png`，Network 使用 `wifi_ic.png`，基础 setting row 使用 `settings_ic.png` / `brightness_ic.png`。
- `SettingsRoute` 已拆出 `Main`、`System`、`Personalization`、`Network`、`About`、`Account`、`Apps`、`Devices`、`Privacy`、`Time`；Settings rows 使用 `settings_back.fraw`、`wp_settings.png` 和 `wp_*.fraw` 图标，可点击进入对应子页面，子页面仍只通过 `UiBuilder -> UiTree -> FHRE DrawCommand` 绘制。
- FHRE `DrawSvgIcon` 的轻量 fallback 已新增 `SvgId(7)` settings gear，用于 Settings tile 和 Settings 页面 header；后续 SVG parser/cache 可替换该 fallback，而不改变 Wing 的 `IconNode`/`SvgId` 接口。
- Home 的 FHRE tile 已接入第一版全屏 `ShellMode::App(AppId)`：点击后进入 `APP_FHRE_SAMPLE` 的独立应用 surface，不再绘制 Home/状态栏背景；该页面用 FHRE `DrawList`、`DrawTriangle`、depth sort、line/grid/star primitive 绘制 pseudo-3D 图形演示，并叠加少量 Wing HUD，左右滑或 Back/Home 键返回 Home。
- App Switcher 已开始感知 `running_app: Option<AppId>`：当 FHRE Sample、News、Stars 或 Thermal 被启动后，应用切换器会展示 running app card；App Switcher 的 overlay tree 已接入 hit-test，点击 card 会重新进入对应 `ShellMode::App(AppId)`。`ShellState::app_surface_state()` 会把 App Switcher 中的运行应用暴露为 preview 状态，供 All Apps 和后续 task UI 复用。这仍是内置 route 原型，不是完整独立 task lifecycle。
- News/Stars/Thermal 内置 app 已继续验证 W10M 资源 fallback：`stars_ic.fraw` / `weather_ic.png` 等轻量资源进入默认 ROMFS，`news_image.png` / `news_tile.png` / `sky_bg.png` / `embedded_tile.png` 等大图只作参考，不安装时由 FHRE fallback 和 cache stats 暴露。
- Wing Rust crate 已完成第二轮目录模块化拆分：`lib.rs` 现在只声明模块和 re-export 公共能力，具体实现已下沉到 `action/mod.rs`、`animation/mod.rs`、`builder/mod.rs`、`demo/mod.rs`、`demo_home/mod.rs`、`demo_launcher/mod.rs`、`demo_notifications/mod.rs`、`demo_sample_app/mod.rs`、`demo_settings/mod.rs`、`demo_switcher/mod.rs`、`demo_detail/mod.rs`、`demo_overlay/mod.rs`、`demo_status/mod.rs`、`demo_wallpaper/mod.rs`、`key/mod.rs`、`layout/mod.rs`、`node/mod.rs`、`shell/mod.rs`、`spec/mod.rs`、`theme/mod.rs`、`tree/mod.rs`、`tree_layout/mod.rs`、`tree_render/mod.rs`。
- Wing 已把 `UiKey` 与布局规则从二级 `ui/key`、`ui/layout` 扁平为根模块 `key`、`layout`，使 `lib.rs` 直接声明所有当前顶层 `mod.rs`；`UiTree` 的节点组件类型下沉到 `node/mod.rs`，layout/clip 解析下沉到 `tree_layout/mod.rs`，render build/hit-test 下沉到 `tree_render/mod.rs`，`tree/mod.rs` 保持为 UI frame diff、组件写入、父子链和 dirty 状态机实现。
- `wing_demo` 的页面绘制也已拆成可独立演进的 shell page modules：wallpaper、status、launcher、all apps、home nav、notification、app switcher、tile detail、settings、sample app 和 overlay submit 分离。后续添加真实 app surface lifecycle 时应该沿用同一结构，而不是再把页面绘制堆回 `demo/mod.rs`。
- `prelude/mod.rs` 已作为外部 demo 和未来 Wing SDK 的常用公开能力入口；这也明确了 `lib.rs` 的定位是 crate root/API facade，而不是承载实现的长期文件。
- V3.9 阶段 Wing 不新增页面，只跟随 FHRE 的 `SvgDocumentCache`、image/glyph/glyph-run cache、draw chain stats、codec pipeline stats、dirty present 和 render stats 边界做兼容消费；Shell 页面仍不拥有图片/SVG/字体解析、mask/layer、硬件链或 framebuffer 逻辑。

当前视觉状态：

- Wing 的 UI 风格已经从“抽象玻璃卡片 demo”推进到 Lumia/Windows 10 Mobile 风格的第一版 Start shell：全屏 Start panel、强调色磁贴、底部三键 nav bar、下拉通知 quick actions 和通知卡片。
- 还没有真正完成 windows-10-mobile-lvgl 级别的像素相似度：当前已经能读取真实 PNG 资源，但字体度量、页面滚动、页面过渡动画、theme token 持久化和独立 app task/surface lifecycle 仍需要继续补齐。

当前还没有实现：

- 完整 diff/reorder/parent-child 同步。
- 滚动容器已有 All Apps/Settings 第一版 offset + clip，但还没有惯性、滚动条、nested scroll、edge handoff 和复用到所有未来页面。
- 可复用的 NuttX/Linux/SDL 真实输入事件到 FHRE `InputEvent` 的平台桥接模块。
- 更完整的 gesture overlay、drag progress 和边缘手势策略。
- 页面级 action router 和跨页面 command queue。
- 更完整的 Wing 级 dirty present：当前已把 status/launcher/overlay 三棵 `UiTree` 的 dirty region 合并给 `wing_demo` runtime，demo 可用上一帧 dirty union 作为本帧 clip；后续还需要把页面构建和渲染进一步拆开，做到同一帧精确 dirty pass。
- Notification/AppSwitcher 的完整真实数据模型和 page transition 状态。
- App UI SDK / Game SDK。
- 更完整的页面能力：All Apps/Settings 现在已有第一版滚动闭环和最小 app surface 状态展示，但还缺搜索、动态安装、页面转场和真实独立 task/surface lifecycle。结构上当前 `demo` 和 `tree` 已完成基础拆分，下一步应优先补真实数据和 transition，再视复杂度拆 `spec` 的 style/text/icon 子语义。
- W10M 参考大图还没有轻量化转换流水线；默认 ROMFS 已拒绝安装这些大图，后续需要补 downscale/quantize/FRAW atlas 或多 profile resource pack。

## 当前新增 API 边界

本轮需要保持稳定的公开入口：

- `AppId`：Shell 识别应用的稳定 id，后续动态安装也应映射到该层。
- `AppEntry`：固定 app metadata，包括 action、title、subtitle、icon 和 shell/app ownership。
- `AppRegistry<N>`：固定容量 app registry，当前为静态内置列表，后续可由 manifest/resource loader 填充。
- `AppSurfaceState`：最小应用 surface lifecycle 状态，当前用于明确 stopped/running/focused/preview 语义；`ShellState::app_surface_state(AppId)` 是 shell 页面查询该状态的稳定入口。
- `ScrollState`：固定容量滚动状态，当前由 `ShellState::all_apps_scroll` 和 `ShellState::settings_scroll` 持有；纵向 drag/键盘 Up/Down 更新 offset，页面绘制阶段根据内容高度设置 max offset。
- `WingAssetId`：Wing 资源稳定 id，可映射到 FHRE `ImageId`，避免页面代码硬编码 NuttX 文件路径；PNG 资源保持原 ID，新增 raw/FRAW 图标追加 ID。
- `WingAssetManifest`：Windows 10 Mobile 资源路径 manifest，当前映射到 `/etc/wing/resource/windows10_mobile/assets/*.png` 和 `/etc/wing/resource/windows10_mobile/raw/*.fraw`；实际进入 ROMFS 的文件还必须通过 `romfs_manifest.txt` allowlist 和 byte budget。
- `UiKind::Image` / `ImageNode`：声明式 image 节点，render build 输出 FHRE `DrawCommand::DrawImage`、`DrawImageFit` 或 `DrawImageTint`，实际 decode/cache/fit/tint 由 FHRE runtime 完成。
- `SettingsRoute`：Settings 子页面 route，当前包含 `Main`、`System`、`Network`、`Personalization`、`About`、`Account`、`Apps`、`Devices`、`Privacy`、`Time`，避免把所有设置页塞进一个布尔状态。
- `ShellMode::AllApps`、`ShellMode::LockScreen`、`ShellMode::Cortana`、`ShellMode::Settings(SettingsRoute)`、`ShellMode::App(AppId)`：当前 Shell 路由边界。

## 当前验收方式

- `cargo check --manifest-path apps/wing/rust/Cargo.toml` 必须通过。
- `cargo check --manifest-path apps/examples/wing_demo/rust/Cargo.toml` 必须通过。
- `nuttx/wing_build.sh` 必须通过。
- `wing_build.sh` 必须在 ROMFS 安装阶段执行 `romfs_manifest.txt` budget 检查；大图未列入 manifest 时不得进入镜像，列入但超预算必须构建失败。
- NSH 中运行 `wing_demo` 后，Home 应保持 Lumia/Windows 10 Mobile 风格；点击 All Apps 可进入完整纵向 app 列表并可拖动/键盘滚动；点击 Settings 可进入主设置页并进入 System/Personalization/Network/About/Account/Apps/Devices/Privacy/Time；点击 FHRE Sample 可进入全屏 FHRE app；上滑进入 App Switcher 后可通过 running app card 恢复应用。
- NSH 中运行 `wing_demo` 后，应能进入 Home、All Apps、Lock Screen、Cortana、Notifications、Settings 子页面、App Switcher、FHRE Sample、News、Stars、Thermal。
- NSH 中运行 `wing_demo` 时，Home/Notification/Settings/Lock/App 页面只应加载 manifest 允许的小 PNG/FRAW 资源；未安装的大图资源必须显示 FHRE fallback 视觉并保持 shell 可操作。
- NSH 中运行 `wing_demo` 时，连续拖动 All Apps/Settings 不应出现输入队列被 move flood 卡住；资源首次加载应优先命中 prewarm/pinned cache。
- NSH 中运行 `wing_demo` 时，Home/AllApps/Settings/Notification/App/Switcher 不新增页面能力，只验证共享 runtime、view-only image/SVG resolver、route prewarm、cache stats、FHRE draw chain/codec pipeline stats 和 dirty present adapter 是否让交互保持响应。

## Windows 10 Mobile 资源路径

运行时资源来源：

```text
source: /home/uan-gpd/codes/windows-10-mobile-lvgl/squareline/assets/*.png
repo:   apps/wing/resource/windows10_mobile/assets/*.png
ROMFS:  /etc/wing/resource/windows10_mobile/assets/*.png (manifest allowlist only)

source: /home/uan-gpd/codes/windows-10-mobile-lvgl/src/ui/images/*.c
repo:   apps/wing/resource/windows10_mobile/raw/*.fraw
ROMFS:  /etc/wing/resource/windows10_mobile/raw/*.fraw (manifest allowlist only)
```

这些 PNG/FRAW 是候选 UI 资源，不是截图预览。真正进入 NuttX ROMFS 的运行时资源必须列在 `apps/wing/resource/windows10_mobile/romfs_manifest.txt` 并通过 byte budget；未列入的大图只作为参考素材留在源码树。Wing 页面只使用 `WingAssetId`，demo/platform glue 通过 `ResourceLoader` 读取 ROMFS 文件，FHRE `PngDecoder/FrawDecoder + ImageCache` 负责解码和复用 decoded image。

当前第一阶段使用的资源包括：

- Start tiles: `phone_ic.png`、`people_ic.png`、`outlook_ic.png`、`message_ic.png`、`photos_ic.png`。
- Status / quick actions: `wifi_ic.png`、`bluetooth_ic.png`、`airplane_ic.png`、`battery_ic.png`、`brightness_ic.png`。
- Settings: `settings_ic.png`、`wp_settings.png`、`wp_system.png`。
- Raw settings/app icons: `settings_back.fraw`、`padlock.fraw`、`stars_ic.fraw`、`wifi_icon.fraw`、`wp_about.fraw`、`wp_account.fraw`、`wp_apps.fraw`、`wp_devices.fraw`、`wp_network.fraw`、`wp_personalization.fraw`、`wp_privacy.fraw`、`wp_time.fraw`。
- Wallpaper / app backgrounds: 大图暂不进入默认 ROMFS；页面命中缺失资源时必须显示 FHRE fallback，并通过 cache/codec stats 暴露。
- Nav resources: `wp_back.png`、`wp_logo.png`、`wp_search.png`、`wp_next.png`。

平台策略：

- 第一目标是 NuttX。
- Wing 不直接绑定 `/dev/fb0`，具体 framebuffer/input 应继续下沉到 FHRE 或 platform glue。
- 后续保留 Linux framebuffer、DRM/KMS、SDL host backend，以便小屏 SoC 或桌面调试。

参考路径：

- `/home/uan-gpd/codes/windows-10-mobile-lvgl`
- `/home/uan-gpd/codes/FeatherOS/apps/hand_down/examples/lvgl_window10`
- `/home/uan-gpd/codes/FeatherOS/apps/hand_down/docs/lvgl_window10_ui_implementation.md`
- `/home/uan-gpd/codes/FeatherOS/apps/hand_down/apps/wing_old`
- `/home/uan-gpd/codes/FeatherOS/apps/hand_down/apps/wing_old2`
- `/home/uan-gpd/codes/FeatherOS/apps/hand_down/docs/wing_app_ui_sdk.md`

## 定位

Wing 是小屏设备上的 Shell/UI runtime。

它负责：

- Home/Start 页面。
- 应用列表。
- 通知面板。
- 快捷操作面板。
- 设置应用。
- 应用切换器。
- 全屏应用 surface 管理。
- Shell 手势。
- 主题、字体、图标、资源选择。
- 声明式 UI 到 FHRE `RenderNode`、布局结果和 `DrawCommand` 的转换。

它不负责：

- 像素级 blend。
- 图片/SVG/字体底层解析。
- framebuffer 直接写入细节。
- LVGL 式 widget class 和 style cascade。
- 复杂桌面窗口系统。

一句话：

```text
Wing = Windows 10 Mobile 风格的小屏 Shell + 声明式 ECS UI
FHRE = 纯 3D 轻量游戏/图形引擎核心，Wing 构建在它之上
```

## 最新实现策略

Wing 不是 LVGL app，也不是旧 FHRE 的 demo。它应该从一个小而完整的 Shell 闭环开始：

```text
Home
  -> Settings
  -> Notification
  -> App Switcher
  -> Sample App
```

实现重点：

- 用声明式 view 函数描述页面。
- 用稳定 `UiKey` 做 diff。
- 用 ECS component 保存 UI 节点状态。
- 用系统函数处理输入、手势、状态变化。
- 用 layout 产生 Screen Canvas 平面上的 `Rect`、`z` 和层级。
- 用 render build 产生 FHRE 3D render node 和投影后的 `DrawCommand`。
- 资源、字体、SVG、图片解码走 FHRE。
- shell 不直接操作 framebuffer。

第一版 Wing shell 的视觉仍然是 2D UI，但语义上它是 FHRE 纯 3D 世界中的 `z=0` Screen Canvas UI。3D scene 和游戏逻辑属于 FHRE 能力，可以在 Wing 游戏应用中逐步使用，不要求 Shell 自身一开始做复杂 3D 视觉。

## 和 windows-10-mobile-lvgl 对齐什么

要对齐的是体验和视觉，不是代码结构。

### UI 风格目标

从 windows-10-mobile-lvgl 中提取这些核心体验：

- 深色/壁纸背景。
- 状态栏：时间、电量、网络状态。
- Start tiles：磁贴入口，强调色，可透明。
- All apps：纵向应用列表。
- Notification panel：下拉通知面板，顶部快捷操作。
- Settings：列表入口 + 子页面，内容纵向滚动。
- Accent color：一组强调色。
- Nav bar：返回、Home、搜索/功能键。
- 动画：通知面板上下滑、页面淡入、磁贴轻微动态。

当前已落地的 windows-10-mobile-lvgl 对齐点：

- `ui_homeScreen.c` 中的全屏 Start panel、顶部 status、底部 nav panel，被收敛成 Wing 的 `LumiaMetrics` + `LumiaTheme` + `draw_launcher()` + `draw_home_handle()`。
- Start panel 使用固定容量声明式节点表达 wide tile / small tile，不照搬 LVGL flex object tree。
- notification panel 使用全屏 overlay、quick action grid 和 bottom accent handle，对齐 windows-10-mobile-lvgl 的视觉结构，但仍走 Wing ECS UI tree。
- 所有这些结构都仍在 FHRE Screen Canvas 平面 `z=0` 上绘制，后续可以与 3D scene / game app 共存。

### 第一版 Shell 页面

第一版 Wing demo 应该包含：

```text
Home / Start
  -> Tiles
  -> All apps entry

Notification
  -> Quick actions
  -> Notification cards

App Switcher
  -> Running app cards

Settings
  -> Main settings list
  -> System page
  -> Personalization page
  -> Network page

Sample App
  -> full screen app surface
```

不要一开始复制 windows-10-mobile-lvgl 的全部页面。先选择一个优秀闭环：

```text
Home -> Settings -> Personalization -> Back -> Home
Home -> Notification -> Quick action -> Settings
Home -> Sample App -> gesture back -> Home
```

## 不采用 LVGL 的地方

Wing 不采用：

- `lv_obj_t` object tree。
- `lv_obj_set_style_*` 分散式 style 修改。
- `LV_EVENT_*` 回调驱动业务。
- SquareLine 生成大段全局变量。
- 运行时 widget class 继承。

Wing 采用：

- `World` 保存实体和组件。
- `Resource` 保存全局状态。
- `System` 显式更新状态。
- `View` 每帧声明 UI spec。
- `Diff` 根据稳定 key 更新 ECS。
- `RenderBuild` 从 ECS 生成 FHRE 可消费的 `RenderNode` 和 `DrawCommand`。

数据流：

```text
Input events
  -> GestureSystem
  -> ShellState / AppState
  -> Declarative View
  -> UiSpec diff
  -> ECS components
  -> Layout
  -> RenderNode / DrawCommand
  -> FHRE dirty/clip/backend
```

## 声明式 UI 设计

核心不是宏，而是稳定 key + builder。

```rust
pub struct UiKey(u32);

pub struct UiSpec {
    pub key: UiKey,
    pub kind: UiKind,
    pub rect: Rect,
    pub style: StyleId,
    pub text: Option<TextId>,
    pub icon: Option<IconId>,
    pub action: Option<ActionId>,
}

pub struct UiBuilder<const N: usize> {
    specs: [UiSpec; N],
    len: usize,
}
```

页面声明类似：

```rust
fn view_home(shell: &ShellState, theme: &Theme, ui: &mut UiBuilder<128>) {
    ui.tile(KEY_SETTINGS, "Settings", IconId::Settings, shell.tile_rect(0));
    ui.tile(KEY_NOTIFICATIONS, "Notify", IconId::Bell, shell.tile_rect(1));
    ui.text(KEY_CLOCK, shell.time_text(), theme.clock_style(), shell.clock_rect());
}
```

要求：

- 每个控件有稳定 `UiKey`。
- 控件 helper 不保存业务状态。
- 输入只产生 action/signal。
- 状态变化只发生在系统中。
- diff 失败或容量溢出时退化为全量重建。

## ECS 组件

Wing 可以复用 FHRE 提供的 tiny ECS helper，但 UI 组件语义属于 Wing。FHRE 不应该决定 Wing 的页面结构、导航结构或 shell 状态机。

第一阶段组件：

```rust
pub struct UiNode {
    pub key: UiKey,
    pub kind: UiKind,
    pub seen: bool,
}

pub struct LayoutBox {
    pub space: LayoutSpace,
    pub content_inset: ContentInset,
    pub rule: LayoutRule,
    pub transform: Transform3D,
    pub size: Size,
    pub line_to: Point,
}

pub struct LayoutResult {
    pub rect: Rect,
    pub depth: Fixed16,
    pub line_to: Point,
}

pub struct Visual {
    pub color: Color,
    pub radius: u16,
    pub scale: u16,
}

pub struct Opacity {
    pub value: u8,
}

pub struct Parent {
    pub key: UiKey,
    pub entity: Option<Entity>,
}

pub struct Children {
    pub first_child: Option<Entity>,
    pub next_sibling: Option<Entity>,
}

pub struct Clip {
    pub rect: Rect,
    pub projected: Rect,
}

pub struct ResolvedClip {
    pub rect: Rect,
}

pub struct TextNode {
    pub text: TextId,
    pub font: FontId,
}

pub struct IconNode {
    pub icon: IconId,
}

pub struct Animation {
    pub opacity: Option<Tween>,
}

pub struct Button {
    pub action: ActionId,
    pub state: ButtonState,
}
```

Shell 领域组件：

```rust
pub struct HomeTile;
pub struct NotificationCard;
pub struct QuickActionTile;
pub struct AppPreviewCard;
pub struct SettingsRow;
pub struct AppSurface;
```

资源：

```rust
pub struct ShellState {
    pub mode: ShellMode,
    pub previous_mode: ShellMode,
    pub nav_stack: FixedStack<Route, 8>,
    pub overlay_progress: u8,
}

pub struct Theme {
    pub accent: Color,
    pub background: BackgroundId,
    pub tile_opacity: u8,
    pub nav_opacity: u8,
}

pub struct AppRegistry;
pub struct NotificationStore;
pub struct InputState;
pub struct GestureState;
pub struct SettingsStore;
```

## 调度阶段

系统函数显式注册，不做参数注入：

```rust
pub type SystemFn = fn(&mut WingRuntime);
```

推荐阶段：

```text
Input
  poll_platform_input
  gesture_system

Update
  shell_action_system
  app_lifecycle_system
  notification_system
  settings_system

Animation
  transition_system
  live_tile_system

View
  compose_active_route
  apply_ui_diff

Layout
  layout_system
  clip_system

RenderBuild
  build_draw_list
  compute_dirty_region

Render
  fhre_backend_draw
```

可以先在 `wing_demo` 中手写循环，等稳定后再沉淀成 `WingRuntime`。

## 页面和手势

### ShellMode

```rust
pub enum ShellMode {
    Home,
    AllApps,
    LockScreen,
    Cortana,
    Notification,
    AppSwitcher,
    App(AppId),
    Settings(SettingsRoute),
}
```

### 手势规则

基础规则：

- Home 下滑：打开通知面板。
- Home 上滑：打开应用切换器或应用列表。
- App 内下滑：打开通知面板 overlay。
- App 内上滑：打开应用切换器 overlay。
- 左滑/右滑：返回上一层。
- 如果当前 app 已在根页面，左滑/右滑退出 app 回 Home。
- 设置应用内部优先使用自己的返回键；没有返回键时使用手势。

手势系统输出 action：

```rust
pub enum ShellAction {
    OpenNotification,
    OpenAppSwitcher,
    OpenApp(AppId),
    OpenSettings(SettingsRoute),
    Back,
    Home,
    ToggleQuickAction(QuickActionId),
}
```

## Home / Start 设计

视觉参考 windows-10-mobile-lvgl 的 Start tiles，但要适合手表/小屏。

第一版 layout：

```text
┌────────────────────────────┐
│ 10:08             icons 72 │
│                            │
│  WING                      │
│                            │
│ [Phone] [Msg ]             │
│ [Mail ] [Set ]             │
│ [News         ]            │
│ [Photos       ]            │
│                            │
│        nav handle          │
└────────────────────────────┘
```

特点：

- 竖屏优先，兼容 480x640。
- 磁贴使用强调色和半透明背景。
- 图标用 FHRE SVG icon cache。
- live tile 第一阶段只做简单文字/图片切换。
- 背景可用渐变或 FHRE 图片资源。

磁贴数据：

```rust
pub struct AppTile {
    pub app: AppId,
    pub title: &'static str,
    pub icon: IconId,
    pub size: TileSize,
    pub live: LiveTileKind,
}
```

## Notification 页面

对齐 windows-10-mobile-lvgl 的通知面板：

- 顶部状态信息。
- 快捷操作网格。
- 亮度条。
- 通知卡片。
- 半透明面板。

第一版：

```text
┌────────────────────────────┐
│ NOTIFICATIONS              │
│ [brightness bar]           │
│ [WiFi] [BT] [AIR]          │
│ [DND ] [LITE] [SYNC]       │
│                            │
│ Message                  now│
│ Mail                     2m │
│ System                   8m │
└────────────────────────────┘
```

数据结构：

```rust
pub struct QuickAction {
    pub id: QuickActionId,
    pub title: &'static str,
    pub icon: IconId,
    pub checked: bool,
}

pub struct Notification {
    pub app: AppId,
    pub title: FixedStr<24>,
    pub body: FixedStr<48>,
    pub time: FixedStr<8>,
}
```

## Settings 应用

Settings 是默认应用，但第一阶段可以作为 Wing 内置 route 实现。后续如果走独立 NuttX task，也要保持同一套声明式 App UI。

第一版路由：

```rust
pub enum SettingsRoute {
    Main,
    System,
    Personalization,
    Network,
    About,
    Account,
    Apps,
    Devices,
    Privacy,
    Time,
}
```

页面参考 windows-10-mobile-lvgl：

- Main: search box + settings rows。
- System: brightness slider、timeout dropdown。
- Personalization: accent color、tile opacity、background。
- Network: WiFi switch、network list。
- About: version、renderer、build info。

设置页面不要用大图背景；它应该是高可读列表界面。

## 应用模型

第一阶段：

- app 是 Wing demo 内部 route。
- `SampleApp` 用全屏 UI 展示应用运行状态。
- back gesture 退出 app。
- `AppSwitcher` 保存并展示最近启动的 `running_app`，第一版用声明式 card + 线框 preview 表达 app surface，并可点击回到 Sample App。
- game app 可以作为 `SampleGame` route 验证 FHRE 的 game loop、输入和 sprite/camera 能力。

第二阶段：

- 支持 Wing-managed app surface。
- 外部 app 可以通过 NuttX task 启动。
- Shell 负责 app stack 和 surface 生命周期。

第三阶段：

- 独立 app 使用 AppUi SDK。
- app 只声明自己的 UI，Shell 管理通知、手势 overlay、返回 Home。
- 游戏 app 可以使用 FHRE Game SDK，Shell 只管理生命周期、全屏 surface、返回手势和系统 overlay。

## 资源模型

Wing 不直接解析复杂资源。它通过 FHRE 资源能力使用：

- `FontId`
- `ImageId`
- `SvgId`
- `IconId`
- `GlyphResolver`

资源路径建议：

```text
/etc/wing/resource/
├── fonts/
├── images/
└── icons/
```

但第一阶段图标优先用 SVG，背景图可以先用渐变或一张已缩放图片。

当前边界：

- FHRE 已在 `/etc/fhre/resource/fonts/dejavu_sans.ttf` 验收真实 TTF glyph，Wing 后续只通过 `FontId + GlyphResolver` 使用字形缓存，不在 Shell 页面里解析字体。
- FHRE 已在 `/etc/fhre/resource/svg/*.svg` 验收 Bootstrap SVG fixtures，并通过 `SvgResolver + DrawSvgDocument` 进入统一 dirty/stats 绘制管线；Wing 仍通过 `SvgId` / `IconNode` 声明图标，具体 SVG parser/cache 属于 FHRE。
- FHRE 已在 `/etc/fhre/resource/images/demo_photo.jpg` 验收 JPEG 子集和 textured triangle；Wing 页面只消费 `ImageId` 和 `DrawImage*`，不会在页面层引入 JPEG/mesh 特例。
- FHRE Draw V2.7 已提供 LVGL draw 层对齐命令矩阵：styled fill/gradient、border align、box shadow、line cap/join、arc、label、styled image、rounded image mask、mask stack、bitmap mask、bounded layer、blur、vector document、flat/gradient/textured triangle，并通过 `DrawTaskKind` / `DrawTaskCounters` / `DrawFeatureFlags` / `DrawBackendDispatch` / `RenderBackend::draw_dispatched_command()` 支持后续 backend 能力协商和按任务覆写。顶层 draw pass 和 layer 内部 draw pass 都会进入同一 dispatch/stats 管线。Wing 后续应通过 `UiBuilder -> UiTree -> DrawCommand` 消费这些能力改善 W10M 视觉，不在 Shell 页面中新增私有 mask/layer/blur/framebuffer 绘制。

图标来源：

- `/home/uan-gpd/codes/icons/icons`
- Bootstrap Icons 风格统一、文件小、适合作为 SVG icon set。

## 主题系统

对齐 Windows 10 Mobile 的 accent color。

```rust
pub struct Theme {
    pub accent: Color,
    pub bg: Color,
    pub panel: Color,
    pub panel_alt: Color,
    pub text: Color,
    pub text_dim: Color,
    pub tile_opacity: u8,
    pub nav_opacity: u8,
}
```

内置 accent：

- Cyan
- Cobalt
- Teal
- Lime
- Amber
- Red
- Magenta
- Violet

不要让整个 UI 变成单一色块。背景、面板、文字、强调色要分层。

## 与 FHRE 的边界

Wing 只生成 FHRE 可以消费的数据：

```text
Wing UI ECS
  -> LayoutBox + Visual + Opacity + Clip + ResolvedClip + TextNode + IconNode
  -> FHRE Entity + Transform3D + Screen Canvas primitive
  -> Camera projection
  -> DrawCommand / RenderCommand
  -> FHRE dirty/clip/backend
```

FHRE 不知道 “SettingsRow” 或 “NotificationCard”。它只看到：

- rect
- Transform3D
- Screen Canvas z=0
- node bounds
- text
- icon
- image
- clip
- opacity
- draw command order

Wing 也不应该直接操作 framebuffer。所有绘制走 FHRE。

## 当前模块结构

```text
apps/wing/rust/src/
├── lib.rs                  # crate facade: mod declarations + public re-exports
├── action/mod.rs           # Shell actions and built-in SvgId mapping
├── animation/mod.rs        # Wing-side animation component wrapper
├── builder/mod.rs          # fixed-capacity declarative UiSpec builder
├── demo/mod.rs             # WingDemoState and shell frame dispatch
├── demo_wallpaper/mod.rs   # FHRE DrawList wallpaper scene
├── demo_status/mod.rs      # status bar view
├── demo_launcher/mod.rs    # Lumia/Windows 10 Mobile Start tiles
├── demo_home/mod.rs        # bottom nav bar
├── demo_notifications/mod.rs # notification overlay view
├── demo_switcher/mod.rs    # app switcher overlay view
├── demo_detail/mod.rs      # tile detail overlay view
├── demo_overlay/mod.rs     # overlay UiTree submit/render helper
├── demo_sample_app/mod.rs  # full screen FHRE app surface demo
├── demo_settings/mod.rs    # Settings shell page
├── key/mod.rs              # stable UiKey
├── layout/mod.rs           # LayoutSpace, ContentInset, GridLayout, StackLayout
├── node/mod.rs             # ECS components used by UiTree
├── shell/mod.rs            # ShellMode, ShellState, ShellActionQueue
├── spec/mod.rs             # UiSpec and UiKind
├── theme/mod.rs            # Lumia metrics and theme tokens
├── tree/mod.rs             # UI diff, component writes, hierarchy, dirty state
├── tree_layout/mod.rs      # parent/content layout and clip inheritance
└── tree_render/mod.rs      # DrawCommand build and hit-test
```

这个结构保持两个边界：`demo_*` 是当前 shell demo 的页面实现，后续会逐步沉淀成真正的 view/page modules；`tree_*` 是 Wing UI runtime 的核心，不应该依赖具体 demo 页面。

## wing_demo 目标

`wing_demo` 是 UI 综合演示，不是 FHRE primitive demo。

第一版 wing_demo 应该展示：

- Home tiles。
- Notification panel。
- App switcher。
- Settings main/system/personalization。
- 一个 sample app。
- 下滑/上滑/左右滑/点击。
- SVG icon。
- 字体渲染。
- dirty region debug 数据。

`wing_build.sh` 中同时编入 `fhre_demo` 和 `wing_demo`：

```text
nsh> fhre_demo
nsh> wing_demo
```

这样调试 Wing 视觉或交互时，可以随时跑 `fhre_demo` 判断底层绘制是否正常。

## 里程碑

### Milestone 0: 当前框架

- `apps/wing` 和 `apps/examples/wing_demo` 目录有效。
- `wing_build.sh` 能构建。
- `wing_demo` 能独立运行。
- `fhre_demo` 也能被 wing build 编进去。

### Milestone 1: 声明式 UI spec

- `UiKey`：已具备最小实现。
- `UiSpec`：已具备最小实现。
- `UiBuilder`：已具备固定容量最小实现。
- `UiTree`：已具备固定容量最小节点存储，内部已使用 FHRE `EntityWorld + ComponentStorage` 承载 UI 节点实体，并已拆出 `UiNode`、`LayoutBox`、`LayoutResult`、`Visual`、`Opacity`、`Clip`、`ResolvedClip`、`TextNode`、`IconNode`、`Animation`、`Button` 组件。
- `apply_ui_frame`：已具备按 key 复用节点和 dirty 记录的最小实现。
- button/action 映射：已具备最小 `ActionId` 和 `UiTree::hit_test()`。
- fixed-capacity spec 数组：已具备最小实现。

### Milestone 2: Shell state 和页面

- `ShellMode`：已具备最小实现。
- `ShellAction`：已具备最小实现。
- `ShellState`：已具备最小 action 分发和状态承载。
- `ShellActionQueue`：已具备固定容量 action 队列，可从 FHRE `InputQueue` 批量泵入。
- NuttX sim input glue：`wing_demo` 已能读取 `/dev/input0` 和 `/dev/kbd` 并进入 action pump。
- 基础手势：tap/select、swipe down notification、swipe up app switcher、left/right back home 已具备。
- Home 页面。
- Status bar：已迁移到声明式 `UiBuilder -> UiTree`。
- Launcher 符号图形：已迁移到声明式 `UiBuilder -> UiTree`，并已从内置 primitive 组合收敛为 `UiKind::Icon` / `IconNode`，当前由 FHRE `DrawSvgIcon` 的轻量 vector fallback 绘制；tile 本身已开始用 `GridLayout` 声明，tile 内文字和 icon 已开始用 `LayoutSpace::Parent` 局部坐标声明，launcher 面板内容已开始用 `LayoutSpace::Content` 声明，后续替换为 SVG parser/cache。
- Notification 页面：已具备最小 overlay 原型，并已迁移到声明式 `UiBuilder -> UiTree`。
- App Switcher 页面：已具备最小 overlay 原型，并已迁移到声明式 `UiBuilder -> UiTree`。
- Lock Screen / Cortana：已具备第一版独立 route，分别使用 W10M 壁纸/raw lock icon 和黑色搜索 overlay。
- Settings main 页面：已具备第一版独立 `ShellMode::Settings` 和声明式页面，包含 header、设置 rows、亮度 slider、底部 nav bar，已覆盖 System/Network/Personalization/About/Account/Apps/Devices/Privacy/Time 子页面，仍需补滚动和状态持久化。
- Sample App：已具备第一版独立 `ShellMode::App(AppId)`，Home 的 FHRE tile 可进入全屏应用 surface，内容使用 FHRE `DrawList` 和 triangle/line primitive 绘制 pseudo-3D 场景；News/Stars/Thermal 也已作为内置 app surface 原型接入，左右滑或 Back/Home 返回 Home；仍需接入真实独立 task lifecycle 和 app switcher surface snapshot。
- App Switcher card interaction：已具备第一版 overlay hit-test，FHRE app card 可点击回到 Sample App。
- Back/Home action：已具备 key action 的最小处理，仍待真实平台输入桥接。

### Milestone 3: Layout 和 dirty

- 简单 grid layout：已具备第一版 `GridLayout` / `LayoutRule::GridCell`，当前 launcher tile 已使用。
- row/column stack layout：已具备第一版 `StackLayout` / `LayoutRule::StackItem`，当前通知页 quick actions 和通知卡片背景已使用。
- 父局部坐标 layout：已具备第一版，`LayoutSpace::Parent` 根据 `Parent` 组件解析 child screen-space layout，当前 launcher tile 子文字和 icon 已使用。
- 父 content 坐标 layout：已具备第一版，`ContentInset` 保存在父节点 `LayoutBox`，`LayoutSpace::Content` 根据父节点 content rect 解析 child screen-space layout，当前 launcher 面板内容已使用。
- 投影后的 `LayoutResult` 组件：已具备，`apply_ui_frame()` 写入，render/hit-test/dirty 使用缓存后的 screen-space 结果。
- 显式 `Opacity` 组件：已具备，颜色 alpha 与组件 opacity 在生成 `DrawCommand` 时合成。
- 显式 `Clip` + 解析后 `ResolvedClip`：已具备，render build 生成 per-command effective clip，hit-test 和 dirty 使用 clipped bounds；子节点会继承父级 clip 并取交集。
- 显式 `IconNode` 组件：已具备，保存 `SvgId` 并在 render build 输出 `DrawCommand::DrawSvgIcon`。
- 显式 `Animation` 组件：已具备，保存 FHRE `Tween`，当前用于 launcher icon opacity pulse，并在每帧自动标记动画节点 dirty。
- 页面切换 dirty。
- 控件变化 dirty。
- notification clip：通知面板内容已开始使用 `UiSpec::with_clip()` 和 `Clip` 组件裁剪。
- render stats。

### Milestone 4: Windows 10 Mobile 风格增强

- Start tiles 尺寸系统。
- All apps list。
- accent color。
- nav bar。
- live tile 基础动画。
- settings personalization。

### Milestone 5: App surface

- 内置 sample app 全屏：已具备第一版 FHRE 图形演示 app surface。
- app switcher card：已具备第一版 FHRE app card、running app 状态和点击恢复。
- gesture overlay。
- back gesture 退出 app：已具备左右滑和 Back/Home 返回 Home 的最小路径。
- 为独立 app task 预留接口。

### Milestone 6: AppUi SDK

- 外部 app 声明式 view。
- typed signal。
- fixed command queue。
- app UI dirty diff。
- Shell/app surface lifecycle。

## 验收标准

每个阶段都要满足：

- `./wing_build.sh` 成功。
- `wing_demo` 和 `fhre_demo` 都能在 NSH 中执行。
- Wing 不直接依赖 LVGL。
- Wing 不直接写 framebuffer。
- Rust core 保持 `no_std`，不引入外部 crate。
- UI 页面使用稳定 key 声明，不靠全局 mutable widget 指针。
- 默认 UI 至少在 320x480、480x640 两类竖屏比例下可用。
- 视觉上能看出 Windows 10 Mobile/Lumia 风格，而不是通用 dashboard。
