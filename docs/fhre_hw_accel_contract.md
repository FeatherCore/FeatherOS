# FHRE Hardware Accel Contract (V4.7)

本文定义 FHRE 与硬件后端（或平台加速层）之间的最小提交契约。目标是：后端接收“可提交运行片段”，失败时可安全回退，且统计可观测。

## 一、提交入口与成功语义

`RenderBackend` 暴露两级提交入口：

- `submit_draw_chain(chain, stats)`：旧入口，保留兼容。
- `submit_draw_chain_with_contract(chain, contract, stats)`：V4 新入口，建议所有后端优先实现该入口。

后端返回：

- `DrawChainSubmitResult::Submitted`：整段成功提交；
- `DrawChainSubmitResult::Unsupported`：该后端不支持；
- `DrawChainSubmitResult::Fallback`：运行时不可提交（部分能力缺失、状态/对齐约束不满足、临时故障等）。

FHRE 调度器约定：

- 先按 `DrawChainRun` 级别提交一次；
- 如果返回 `Unsupported/Fallback`，仅回退该 `run` 的命令范围；
- 对不含 `mask/layer` 状态的 render run，可继续对单 `op` 做细粒度尝试，确保“一个 op 回退不拖垮整帧”的边界；
- 对含 `mask/layer` 状态的 run，提交失败时必须整体回退当前 run，避免硬件端状态提交成功但软件端内容回退时丢失 mask/layer 栈。

V4.6/V4.7 额外提供 `ParallelRenderBackend`，用于 run 级延迟提交和 barrier 收敛：

- `queue_draw_chain_with_contract(chain, contract, stats)`：只 queue 当前 run，不要求立即写 framebuffer；
- `pending_draw_chain_bounds()`：返回 pending run 的合并 bounds，用于判断后续软件/硬件 run 是否可并行；
- `flush_pending_draw_chain(stats)`：提交并完成 pending run，对调度器而言是 fence/barrier；
- `ParallelDrawChainSubmitResult::Queued` 表示已入队但未完成，`Submitted` 表示后端同步完成，`Fallback/Unsupported` 由调度器回退软件。

parallel 调度约定：

- 只允许无 mask/layer、bounds 已知、与 pending run 不相交的 run 并行 queue；
- pending hardware run 与后续软件 fallback run 不相交时，CPU 软件路径可以先绘制，并记录 `draw_chain_parallel_software_runs`；
- overlap、mask/layer、未知 bounds、纯状态 marker 或最终收尾必须先 flush pending fence；
- flush 失败只影响当前 pending 批次，后续仍按软件回放收敛；
- 最终 framebuffer 必须与纯软件 `Surface` draw order 完全一致。
- V4.7 的 `EglParallelAccelBackend` 会在 Rust 侧保留 pending packet；如果 EGL worker 在 fence/readback 阶段失败，后端会用这些 packet 软件回放 pending 批次，避免 GL 失败导致 framebuffer 丢 run。

## 二、Run Contract（最小状态）

`DrawChainRunContract` 在提交时仅携带可重现硬件调度所需最小信息：

- `descriptor.command_start`：首条命令索引；
- `descriptor.command_end`：末端索引（开区间）；
- `descriptor.has_clip` / `has_mask` / `has_layer`：状态标记；
- `descriptor.bounds`：run 的几何边界；
- `kinds[] + len`：该 run 的 op kind 序列（仅候选范围）；
- `command_end`/`command_start` 用于提交器重建最小可重放区域。

### contract 约束

- 只允许提交 `run` 中可提交的连续片段；
- `kinds` 长度与 `len` 必须一致；
- `draw chain` 与 `kinds` 之间保持原始顺序关系；
- 未携带 `clip/mask/layer` 时按空状态处理；
- 后端不得假设 frame 全局状态，需基于 `descriptor` 自洽渲染。
- `has_mask/has_layer` 为 true 的 run 必须被视为自包含状态片段；框架会尽量把 enter + render + exit 放入同一个 run，超出 `max_chain_ops` 时该 run 会回退软件。
- V4.1 起，合成 clip marker 的 `command_len` 为 0；真实可重放命令仍保留 command span。纯 marker run 的 `command_start == command_end`，后端应通过 payload 消费状态，而不是依赖软件命令范围。

## 三、Op Payload（V4.1/V4.6 固定字段）

`DrawChainOp` 除 `kind/task/bounds/clip/command_start/command_len` 外携带 `DrawChainOpPayload`，这是后端接管的主入口：

- `FillRect { rect, color }`：对应 `SolidFill` / `AlphaFill`；
- `Image { rect, image, opacity, fit, tint }`：对应 `ImageBlit` / `ImageBlend`；
- `Clip { clip }`：合成 clip marker 或显式 clip 状态；
- `Mask { spec }`：`MaskEnter` 的第一版硬件可识别状态；
- `Layer { rect, spec }`：`LayerEnter` 的第一版硬件可识别状态；
- `None`：退出 marker 或必须软件处理的 fallback。

后端实现建议：

- 优先用 payload 生成 DMA2D/GPU descriptor，不反查 `DrawCommand`；
- `FallbackRange` 和 payload 不匹配时直接返回 `Fallback`；
- V4.6 已提供 `Accel2dCommandList + Accel2dExecutor + Accel2dSubmissionRing + ParallelRenderBackend`，用于把 payload-only 路径编译成更接近真实硬件提交的 packet/executor/ring/fence/parallel barrier；它仍不绑定具体 MCU 外设。

## 四、2D Packet、Executor、Submission Ring 与 Parallel Flush（V4.7）

`Accel2dCommandList<N>` 是 draw chain 和真实硬件后端之间的固定容量 packet 层：

- `Accel2dCommand::SetClip`：设置当前裁剪状态；
- `Accel2dCommand::Fill`：矩形填充，携带 `rect/clip/color`；
- `Accel2dCommand::Image`：图片 blit/blend，携带目标 rect、clip、`ImageId`、源 descriptor、opacity、fit、tint。

descriptor 边界：

- `Accel2dFramebufferDescriptor`：目标 framebuffer 的 `data/len/width/height/stride/bpp/PixelFormat/clip`；
- `Accel2dImageSourceDescriptor`：源图像的 `data/len/width/height/stride/ImageFormat`；
- packet build 只读取 `DrawChainOpPayload`、`DrawChainRunContract`、framebuffer descriptor 与 `ImageResolver/builtin_image`；
- packet execute 只消费 `Accel2dCommandList`，真实 DMA/GPU 后端后续可替换该执行层。

`Accel2dExecutor<CMDS>` 是 V4.5/V4.6 的执行层边界：

- `capabilities()`：返回 fill/image/clip、target/source format、sync fence、async fence 能力；
- `queue(packet)`：把已 build 的 packet 入队，返回 `Accel2dFence`；
- `flush()`：提交 queued packet；
- `execute_submitted()`：当前 mock/sim 同步执行 submitted packet；真实硬件后端可改为提交 descriptor 或推进 DMA/GPU 队列；
- `complete_fence(fence)`：完成 fence，真实后端可映射为 interrupt/poll completion。

`Accel2dExecutorError` 稳定映射为：

- `Unsupported`：能力缺口，例如格式不支持；
- `Fallback`：后端主动要求软件回放；
- `Overflow`：ring 或 executor 容量不足；
- `SubmitFailed`：提交失败；
- `FenceFailed`：fence 状态错误或完成失败；
- `Ring(Accel2dRingError)`：保留 ring 细节用于调试。

当前公共执行器：

- `SurfaceAccel2dExecutor`：offscreen/readback 等价执行器，写入 `Surface`，用于 mock 和测试。
- `SimOpenGlAccel2dExecutor`：`sim-opengl` feature 下的 OpenGL-like packet mapping probe，默认不编译，不链接 GL。
- `EglOpenGlAccel2dExecutor`：`sim-opengl-egl` feature 下的 C/FFI shim 边界；真实 worker 未在构建时启用时稳定返回 `Unsupported`。
- `EglParallelAccelBackend<RUNS, OPS>`：`sim-opengl-egl` feature 下的真实 host/sim EGL/OpenGL worker parallel backend。它只消费 `Accel2dCommandList`，queue 阶段复制 run bounds、framebuffer snapshot、source image bytes 与 command packet，worker 线程独占 EGL context/FBO，barrier flush 时按提交顺序 readback 并写回 framebuffer descriptor。

`Accel2dSubmissionRing<RUNS, CMDS>` 是 packet 之后的固定容量提交层：

- `queue(packet)`：只接受已 build 成功且非空的 packet，返回 `Accel2dFence`；
- `flush()`：按入队顺序把 queued submission 标记为 submitted；
- `complete_fence(fence)`：把 submitted fence 标记为 completed；
- `Accel2dSubmission<CMDS>` 保存 sequence、fence、state 与 packet；
- `Accel2dRingError` 覆盖 overflow、empty submission、invalid fence、not submitted、already completed。

ring 规则：

- queue 失败不写 framebuffer；
- flush 不改变 submission 顺序；
- complete 只能作用于 submitted fence；
- 当前 mock 后端同步 flush/execute/complete；真实硬件后端可把 flush 映射为 descriptor 提交，把 complete 映射为中断或轮询完成。
- V4.6 的 parallel backend 可以先 queue 多个不相交 packet，再在 barrier 时按 ring 顺序 flush/execute/complete。

build 失败规则：

- 空 chain、contract mismatch、packet 容量不足、空 framebuffer、源图像越界、missing image、payload 不匹配均返回错误；
- `Mask/Layer/FallbackRange` 当前不进入 packet，返回 fallback；
- 错误必须发生在写 framebuffer 前，保证失败 run/op 可安全软件回放。

## 五、真实 EGL/OpenGL Worker（V4.7，可选）

`sim-opengl-egl` 现在包含默认关闭的真实 worker executor：

- C shim：`apps/fhre/rust/src/egl_shim.c`；
- Rust 后端：`EglParallelAccelBackend<RUNS, OPS>`；
- 构建开关：Cargo feature `sim-opengl-egl`；
- 真实 worker gate：构建时设置 `FHRE_SIM_OPENGL=egl` 或 `FHRE_EGL_WORKER_ENABLE=1`；
- 默认 `cargo test`、`fhre_build.sh`、`wing_build.sh` 不编入真实 worker，不新增默认 GL present 路径。

worker 规则：

- worker thread 独占 EGL display/context/pbuffer/FBO，不复用 NuttX sim 的 X11 `Display`；
- Rust 主线程只负责 packet build、worker submit、pending bounds 和 barrier flush；
- `queue_draw_chain_with_contract()` 成功后立即返回 `Queued`，不写 framebuffer；
- pending GL run 与后续软件 run 不相交时，软件路径可继续绘制；
- overlap、mask/layer、纯 marker、最终收尾由 `DrawList::execute_parallel_tracked_on()` 触发 barrier；
- `flush_pending_draw_chain()` 等 worker 完成后按 run bounds readback 到 framebuffer descriptor；
- GL submit/readback/fence 失败时，不让调度器丢 pending run：Rust 后端使用保存的 pending `Accel2dCommandList` 软件回放 pending 批次并返回 `Fallback`。

第一版 packet 支持：

- `SetClip -> glScissor`；
- `Fill -> opaque glClear/scissor 或 alpha quad`；
- `Image -> texture upload + textured quad` 的最小 `Stretch` 子集；
- target/source format：RGB565 / RGB888 / RGBA8888，source 额外允许 A8。

当前不支持：

- mask/layer/vector/text/triangle/3D 的 GL packet；
- direct GL window present；
- NuttX sim X11 event loop 接管；
- MCU/MPU OpenGL ES 平台适配。

## 六、MockAccelBackend 与 MockParallelAccelBackend（V4.6/V4.7 验证后端）

`MockAccelBackend` 是平台无关调试/测试后端，包装 `Surface` 并实现 `RenderBackend`：

- capabilities 声明 `draw_chain = true`、`accelerated_2d = true`、`chain_run_fence = true`；
- `chain_draw_features` 仅覆盖 `Fill/Image`，`Clip` 通过状态 marker 支持；
- `Mask/Layer/FallbackRange` 不作为链内硬件提交目标，继续软件回退；
- 成功提交后 `RenderStats.draw_chain_hw_runs` 上升，软件命令不计入 `commands_seen`。

提交规则：

- 提交流程：先 build `Accel2dCommandList`，再通过 `SurfaceAccel2dExecutor` queue、flush、执行 submitted packet、complete fence；
- 任一 op 不支持或资源缺失，整次提交返回 `Fallback`，不会提前修改 framebuffer；
- stateless run 失败后，调度器仍可继续对单 op 重试；已通过 missing image + 两端 fill 的像素测试验证“失败 op 不拖垮前后片段”；
- packet build 期间只读 `DrawChainOpPayload`、`DrawChainRunContract` 和 `ImageResolver/builtin_image`，不得反查 `DrawCommand`；
- packet execute 期间只读 `Accel2dCommandList`，便于真实硬件后端复用同一提交边界。
- ring/fence 当前同步完成；`draw_chain_parallel_hints` 表示可并行提交候选。

`MockParallelAccelBackend<RUNS, OPS>` 是 V4.6 的确定性并行验证后端：

- queue 阶段只 build `Accel2dCommandList` 并入 `Accel2dSubmissionRing`，不写 framebuffer；
- `pending_draw_chain_bounds()` 返回 pending packet 的合并 run bounds；
- flush 阶段按 submission 顺序执行 packet 到 `Surface` 并 complete fence；
- 用于验证 disjoint hardware queue、software-while-hw、overlap barrier、stateful barrier 和 fallback stats；
- 不使用 OS 线程，不代表真实 GPU 并发，只验证真实后端应满足的提交/收敛契约。

## 七、Sim OpenGL / EGL Packet Probe（V4.7，可选）

`sim-opengl` 是默认关闭的 FHRE feature，用于验证 OpenGL-like 后端边界：

- 默认 `fhre_build.sh` / `wing_build.sh` 不启用该 feature，不新增 OpenGL 链接依赖；
- 可通过 `FHRE_SIM_OPENGL=1 ./nuttx/fhre_build.sh` 让 FHRE crate/demo 编入 probe feature；
- 当前 probe 不替换 NuttX sim 的 X11 framebuffer/present 路径，不恢复 demo 线程内 `sim_x11events()`；
- `SetClip` 映射为 scissor；
- `Fill` 映射为矩形 fill quad；
- `Image` 映射为 texture quad，保留 source format/stride、opacity、fit、tint；
- 当前执行仍委托 `SurfaceAccel2dExecutor` 做 offscreen/readback 等价像素结果，真实 GLX/EGL/OpenGL ES 后端后续只替换 execute 层；
- mask/layer/vector/text/triangle/3D 不进入 GL packet，继续按现有 fallback 或软件路径处理。

`sim-opengl-egl` 是默认关闭的 C/FFI shim feature：

- 通过 `apps/fhre/rust/src/egl_shim.c` 建立 host/sim EGL/OpenGL offscreen worker；
- build script 会在显式 feature 和构建 gate 同时打开时探测 host EGL/OpenGL headers/libs，存在时链接 `EGL/GL/pthread` 并编入真实 worker；否则编入 unavailable stub；
- `FHRE_SIM_OPENGL=egl ./nuttx/fhre_build.sh` 会让 FHRE crate/demo 编入真实 worker，普通 `--features sim-opengl-egl` 仅验证 API/stub 路径；
- 真实实现使用 EGL pbuffer + desktop OpenGL，render 到 offscreen target，再按 run bounds readback 到 framebuffer descriptor；
- 不复用 NuttX sim 的 X11 `Display`，不恢复 demo 线程内 `sim_x11events()`；
- `FHRE_SIM_OPENGL=egl ./nuttx/fhre_build.sh` 只启用 feature/链接边界，默认构建仍不受影响。

## 八、Image Descriptor（V4.2/V4.7）

`DrawChainImageDescriptor` 是 `DrawChainOpPayload::Image` 的硬件友好解析结果：

- `rect` / `clip`：目标区域与本 op 有效裁剪；
- `image` / `view`：原始 `ImageId` 与解析后的 `ImageView`；
- `view.format` / `view.stride` / `view.data` / `view.len`：后续 DMA/GPU descriptor 的源图像字段；
- `opacity` / `fit` / `tint`：blend/scale/tint 参数。

`resolve_draw_chain_image_descriptor(payload, clip, resolver)` 规则：

- 只接受 `DrawChainOpPayload::Image`；
- 优先解析 `builtin_image(image)`，再走外部 `ImageResolver`；
- 资源缺失、空图像或 payload 类型不匹配时返回错误，由提交者在写 framebuffer 前 fallback；
- 软件路径仍负责 missing image placeholder，不把 placeholder 偷塞进硬件 descriptor。
- V4.3 起，image descriptor 会继续下沉成 `Accel2dImageSourceDescriptor`，packet 中保存 source stride/format/address/length 与 blend 参数；V4.4 ring 只转运 packet，不改写 source descriptor。

## 九、能力位与 run 形成

`BackendCapabilities` 里新增/使用以下硬件链化能力：

- `draw_chain`：是否支持 draw-chain 提交流程；
- `max_chain_ops`：单 run 最大 op 数；
- `chain_draw_features`：当前硬件支持的 chain op 组合能力；
- `chain_continuous_submit`：是否支持一次提交跨多个候选 run（当前默认为分 run）；
- `chain_run_fence`：是否支持 run 粒度 fence；只有为 true 时才产生 `draw_chain_parallel_hints`；
- `chain_mix_alpha/clip/mask/layer`：是否支持 run 内混合/裁剪/遮罩/图层互操作；

`DrawChainOpKind` 目前 V4 最小支持集：

- `SolidFill`, `AlphaFill`, `ImageBlit`, `ImageBlend`, `Clip`, `LayerEnter`, `LayerExit`, `MaskEnter`, `MaskExit`, `FallbackRange`.

`FallbackRange` 仅用于软件回退分支，不应作为可提交候选。

## 十、可观测字段（用于 HUD / 调试）

- `RenderStats`：`draw_chain_candidates / draw_chain_ops / draw_chain_submitted / draw_chain_fallbacks / draw_chain_runs / draw_chain_hw_runs / draw_chain_sw_runs / draw_chain_splits / draw_chain_parallel_hints`.
- V4.4/V4.7 ring/fence：`accel2d_ring_submissions / accel2d_ring_flushes / accel2d_fences_issued / accel2d_fences_completed / accel2d_ring_overflows`.
- V4.6/V4.7 parallel：`draw_chain_parallel_queued / draw_chain_parallel_completed / draw_chain_parallel_barriers / draw_chain_parallel_software_runs / draw_chain_parallel_fallbacks`.
- 后端返回 `Unsupported` 与 `Fallback` 需分开记账：
  - Unsupported：能力缺口；
  - Fallback：当前 run 运行时不满足提交约束；
- `commands_seen / commands_drawn` 主要统计软件执行路径，`run` 级成功提交的命令不计入“seen”。

## 十一、回退策略（建议）

后端优先返回：

1. `Submitted`：执行硬件命令路径；
2. `Fallback`：立即回退到软件路径；
3. `Unsupported`：记录能力缺失并回退到软件路径；

对于 `Fallback`：

- stateless render run 会继续尝试单-op 提交，便于定位最小失败片段；
- stateful `mask/layer` run 会直接软件重放整个 run，保证回退画面与软件基线一致。

## 十二、测试要求（V4.7 验收）

- 至少 2~3 个单测覆盖：
  - run 级拆分 + run 级回退；
  - run 回退仅影响该 run/op（单 op 继续走软件）；
  - `mask/layer` stateful run 失败时整体软件重放；
  - contract 信息字段可被后端使用且不依赖外部不可见状态。
  - `ProbeChainBackend` 只读 payload 和 contract，不反查 `DrawCommand`，且合成 clip marker 的 command span 为空。
  - `MockAccelBackend` 的 fill/alpha/clip/image 像素输出与纯软件 `Surface` 一致。
  - missing image、mask/layer、fallback payload 在提交前 fallback，软件回放后像素与纯软件基线一致。
  - `Accel2dCommandList` 能携带 framebuffer/source stride、format、clip、blend 参数。
  - packet build 的 overflow、missing image、空 framebuffer、payload mismatch、mask/layer/fallback payload 都在写像素前失败。
  - `Accel2dSubmissionRing` 可按顺序入队多个 packet，并按顺序 flush/complete fence。
  - ring overflow、非法 fence、重复完成、未提交完成稳定返回错误。
  - mock backend 的 ring/fence stats 与 draw-chain hw/sw/fallback 统计一致。
  - `Accel2dExecutor` 能报告能力、queue/flush/execute/complete，并把 ring/fence 错误映射为稳定 executor error。
  - `SurfaceAccel2dExecutor` 的 fill/alpha/clip/image 输出与纯软件 `Surface` 一致。
  - `sim-opengl` feature 打开时，OpenGL-like probe 可把 packet 映射为 scissor/fill quad/texture quad，并通过 readback 等价路径得到与软件一致的像素。
  - `MockParallelAccelBackend` 可 queue 两个 disjoint hardware run，延迟 flush 后像素与软件一致。
  - pending hardware run 与 disjoint software fallback run 可重叠调度，stats 记录 software-while-hw。
  - overlap run、mask/layer stateful run 或 marker 边界触发 barrier，最终 draw order 正确。
  - `sim-opengl-egl` feature 打开但构建 gate 未打开时会稳定返回 `Unsupported`，构建时打开 `FHRE_SIM_OPENGL=egl` 或 `FHRE_EGL_WORKER_ENABLE=1` 时 `EglParallelAccelBackend` 可执行/回放 pending packet，最终像素与软件一致。
  - EGL worker ring overflow、GL fence/readback failure 必须在不污染 framebuffer 的前提下 fallback 或软件回放。
