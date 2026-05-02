# FHRE Hardware Codec Pipeline (V4.4)

本页记录资源解码“分阶段 + 硬件可插拔”的约定。目标是：当前默认纯软件路径稳定可用；未来在有硬件加速能力时只替换阶段实现。

## 一、统一返回与可观测类型

`ImageCache` 与相关资源入口按结果区分三种形态：

- `ResourceDecodeResult::Rendered(view)`：真实解码成功并可用；
- `ResourceDecodeResult::Placeholder(view, ImageDecodeErrorKind)`：解码失败但仍可用于渲染；
- `ResourceDecodeResult::Failed(CodecErrorKind)`：硬失败，调用方通常终止该资源流程；

说明：

- `as_result()` 将 `Rendered` 视为成功；
- `Placeholder` 会转成错误返回，但不会打断帧执行；
- `Failed` 仅在真正不可恢复失败时出现。
- V4.1 新增 `error_kind()` / `placeholder_kind()`，资源预热和 fixture 验收可直接读取错误类型，不必从 stats delta 反推。

## 二、Pipeline 分段

每次 `decode_resource_image_result` 前都可先拿到 pipeline plan：

- 基础读取/识别：`Read`, `Inspect`;
- 格式头处理：`Header` / 兼容性识别；
- 转换与打包：`Transform`, `ColorConvert`, `Pack`（按格式决定）；
- 缓存落盘：`CacheInsert`；
- 不支持阶段走 `Fallback`。

计划对象为 `CodecPipelinePlan<STAGES>`，带 `CodecPipelineStats`，并回写 `ImageCacheStats` / `RenderStats::codec_pipeline_*` 供 HUD 可视化。

V4.1/V4.4 API：

- `plan_resource_image_pipeline(bytes)`：默认 `CodecAcceleratorCapabilities::NONE`；
- `plan_resource_image_pipeline_with_caps(bytes, caps)`：用于平台层或测试显式声明 PNG/JPEG/FRAW 硬件候选阶段。
- `plan_resource_image_pipeline_job(bytes, caps)`：V4.2 新增，直接生成 `CodecPipelineJob<STAGES>`。

## 三、CodecPipelineJob（V4.2/V4.4 状态机）

`CodecPipelineJob<STAGES>` 是 plan 之后、真实硬件 decoder 之前的运行时骨架：

- `Planned`：已生成 `CodecPipelinePlan`，尚未准备硬件；
- `Prepared`：`prepare()` 已确认存在硬件候选，且没有 unsupported/overflow；
- `Submitted`：`submit()` 已进入提交态，未来对应硬件队列或中断等待；
- `Completed`：`complete()` 成功结束；
- `Fallback`：capability 不足、unsupported 或 overflow，调用方应走软件/placeholder 路径；
- `Failed`：准备或提交后出现真实硬失败。

状态规则：

- `Planned -> Prepared -> Submitted -> Completed` 是成功路径；
- `prepare()` 如果发现无硬件候选或 unsupported，进入 `Fallback(Unsupported)`；
- `prepare()` 如果发现 plan overflow，进入 `Fallback(Overflow)`；
- 非法状态跳转返回 `CodecPipelineJobError::InvalidTransition`；
- 当前 job 只表达调度状态，不新增 JPEG/PNG/SVG/TTF 完整解码能力。

V4.3 新增 backend 驱动入口：

- `prepare_with(backend)` / `submit_with(backend)` / `complete_with(backend)`；
- `CodecPipelineBackend` 是硬件 decoder 执行层 trait；
- `MockCodecBackend` 只验证 capability 与状态转移，不读取或修改真实资源字节。

V4.4 新增轻量 token：

- `CodecPipelineJobToken` 表示 mock executor 的提交 token；
- submit 成功后 token 进入 `submitted_token()`；
- complete 成功后 token 进入 `completed_token()`；
- token 只是状态/测试边界，不携带资源地址或 decoded buffer。

## 四、Mock Codec Executor（V4.4）

`MockCodecBackend` 用于在平台无关单测中验证 job 执行层：

- backend capabilities 必须与 job 生成时的 capabilities 一致，否则 `prepare_with()` 进入 `Fallback(Unsupported)`；
- 成功路径仍是 `Planned -> Prepared -> Submitted -> Completed`；
- `with_next_failure(Submit/Complete)` 可模拟提交或完成阶段硬失败，job 进入 `Failed`；
- submit failure 不产生 token；
- complete failure 保留 submitted token，completed token 为空；
- 非法状态跳转继续返回 `CodecPipelineJobError::InvalidTransition`；
- mock executor 不新增 JPEG/PNG/SVG/TTF 解码能力，真实 decoder 后续只替换 trait 实现。

## 五、资源类别与当前能力（V4.4）

目前默认（`CodecAcceleratorCapabilities::NONE`）下：

- FRAW：可完成 `Read/Inspect/Header/Pack/CacheInsert` 全链路；
- JPEG baseline/progressive（基础）：走软件解码，可按需走 fallback；
- PNG/TTF/SVG：当前默认保守路径：
  - 软件解码/解析可用时走 `Rendered`；
  - 暂未接入硬件加速/增量能力时标记相应阶段为 `fallback`；
  - 不支持的特性统一走占位，保证主流程连续执行。

## 六、降级与占位策略

`ImageDecodeErrorKind`：`Missing / Invalid / Truncated / Unsupported / Overflow`；

- `Missing`：资源未命中；
- `Invalid`：头部/字段非法；
- `Truncated`：长度不足；
- `Unsupported`：格式超范围；
- `Overflow`：缓存/尺寸/预算超限；

`decode_failures` 只记录已有 bytes 进入 decoder 后的真实失败；`decode_placeholders` 记录可继续渲染的占位成功。V4.1 起，missing resource 只计入 `load_failures`、`decode_placeholder_missing` 和 `codec_missing_resources`，不再混入 `decode_failures`。

`RenderStats` 同步暴露 `cache_decode_placeholders` 与 `cache_decode_placeholder_*`，用于和 `codec_*`、`codec_pipeline_*` 一起定位“资源不可用但 UI 仍能继续”的路径。

占位策略：

- 失败时若存在 `builtin_image` 占位图，返回 `Placeholder`；
- 不可提供占位图时返回 `Failed`；
- `prewarm` 接口保留 `Result` 语义，便于上层打印 mismatch；
- `get_or_load_with_placeholder` 保持“可渲染继续”语义，适合热路径。
- 当前不做负缓存；热路径应优先使用已预热的 cache/resolver，placeholder 主要用于资源缺失或格式不支持时保持画面连续。

## 七、可观测字段（HUD 优先）

- codec 失败：`decode_failures`, `decode_invalid`, `decode_truncated`, `decode_unsupported`, `decode_overflow`;
- 占位分类：`decode_placeholder_missing`, `decode_placeholder_invalid`, `decode_placeholder_truncated`, `decode_placeholder_unsupported`, `decode_placeholder_overflow`;
- RenderStats 映射：`cache_decode_placeholders`, `cache_decode_placeholder_missing`, `cache_decode_placeholder_invalid`, `cache_decode_placeholder_truncated`, `cache_decode_placeholder_unsupported`, `cache_decode_placeholder_overflow`;
- pipeline：`pipeline_candidates`, `pipeline_stages`, `pipeline_hardware_candidates`, `pipeline_fallbacks`, `pipeline_unsupported`, `pipeline_overflows`;

HUD 显示建议：
- 将 `decode_failures` 与 `decode_placeholder_*` 分屏展示；
- `pipeline_fallbacks` 上升而 `hardware_candidates` 低，说明当前设备仍在纯软件段；
- `decode_placeholder_*` 但 `decode_failures` 不抖动时，说明 UI 可继续运行但有可见质量降级；若同时出现 `decode_failures` 上升则需要查资源路径。

## 八、测试要求

新增/保留单测建议：

- 缓存命中：`Rendered` 后续命中复用视图；
- decode 缺失：`Missing` 触发占位；
- decode 无效：`Invalid` 可被分类；
- 逐步降级路径（`placeholder`）不影响软件渲染主循环。
- caps 路径：FRAW/JPEG/PNG 在 capability 允许时能产生 `pipeline_hardware_candidates`，能力不足时稳定 fallback/placeholder。
- job 路径：capability 支持时可完成 `prepare/submit/complete`；capability 不足时进入 `Fallback`；非法转移稳定返回 `InvalidTransition`。
- executor 路径：`MockCodecBackend` 可驱动 job 成功完成，可模拟 unsupported fallback 和提交失败进入 `Failed`。
- token 路径：submit 成功产生 token，complete 成功迁移到 completed token，complete failure 不误报完成。
