# MoyuyuPet 落地方案

| 项目 | 内容 |
| --- | --- |
| 版本 | v0.1（草案） |
| 日期 | 2026-11-02 |
| 状态 | 待评审 |
| 关联文档 | [需求文档](./requirements.md) |

## 1. 当前状态与目标

当前工程已完成从 WPF 到 Tauri 的迁移：

- Tauri 2 + Vue 3 + TypeScript + Vite。
- 应用名 `MoyuyuPet`，identifier `com.moyuyu.pet`。
- `pnpm build`、`cargo check`、debug 构建和窗口启动均已验证。
- 目前只有一个占位页面，没有业务逻辑。

本方案要解决的核心问题：把"读取本地 AI 工具 token/任务事件"与"宠物表现"解耦，先打通 Qoder 和 dsh 的闭环，再用同一套适配器框架扩展 Claude Code、Codex。

## 2. 总体架构

```
┌───────────────────────────── 宿主 AI 工具 ─────────────────────────────┐
│ Qoder CN       DeepSeek Harness       Claude Code       Codex           │
│ local.db       session.v4.jsonl.zstd  hooks/transcript  sqlite/jsonl    │
└──────┬───────────────┬──────────────────────┬───────────────┬──────────┘
       │               │                      │               │
       ▼               ▼                      ▼               ▼
┌──────────────────────── Rust 采集层（适配器） ──────────────────────────┐
│ QoderAdapter   DshAdapter   ClaudeAdapter   CodexAdapter   InboxAdapter │
│ 只读轮询        zstd 解析    hook/transcript  sqlite/jsonl   文件收件箱   │
└──────────────────────────────┬─────────────────────────────────────────┘
                               ▼
                    ┌─────────────────────┐
                    │ 归一化 + 去重 + 聚合 │  AiEvent / TokenUsage
                    └──────────┬──────────┘
                               ▼
                    ┌─────────────────────┐
                    │ Tauri event (IPC)   │  pet://activity
                    └──────────┬──────────┘
                               ▼
┌──────────────────────── Vue 表现层 ─────────────────────────────────────┐
│ 动画状态机（精灵图/Canvas）  气泡引擎（模板 + 元数据）  设置窗口  托盘     │
└─────────────────────────────────────────────────────────────────────────┘
```

原则：

1. 采集、解析、去重全部在 Rust 侧；前端不碰文件、数据库和日志。
2. 适配器只输出归一化事件，不直接操作 UI。
3. 表现层只消费事件，不关心数据来自哪个工具。
4. 任何适配器失败都只降级自身，不影响其他适配器和宿主工具。

## 3. 核心设计

### 3.1 事件模型

```rust
#[derive(Clone, Copy, Default, serde::Serialize)]
pub struct TokenUsage {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
}

#[derive(Clone, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AiEvent {
    TurnStarted {
        source: Source,
        session_id: String,
        project: Option<String>,
        title: Option<String>,
    },
    TokensDelta {
        source: Source,
        session_id: String,
        usage: TokenUsage,
    },
    ToolCalled {
        source: Source,
        session_id: String,
        tool: String,
    },
    TurnEnded {
        source: Source,
        session_id: String,
        usage: TokenUsage,
        ok: bool,
    },
}
```

约束：

- `usage` 一律表示"本次增量"，累计值必须在适配器内做差。
- `project`/`title` 只用于气泡文案，不落盘、不参与统计。
- `Source` 使用枚举（Qoder、Dsh、ClaudeCode、Codex、Inbox），方便过滤和展示。

### 3.2 适配器接口

```rust
pub trait ActivitySource: Send + Sync {
    fn id(&self) -> Source;
    fn probe(&self) -> AdapterStatus;
    fn start(&self, tx: tokio::sync::mpsc::Sender<AiEvent>)
        -> anyhow::Result<tokio::task::JoinHandle<()>>;
}
```

- `probe()`：检测路径/数据库/版本是否存在，返回已连接、未检测到、不兼容。
- `start()`：启动后台任务；使用取消令牌支持暂停/退出。
- 每个适配器独立文件，解析失败只记录错误计数并向 UI 报告，不 panic。
- 新增适配器的最小改动：一个模块 + 一个 fixture + 一条设置项。
- 实现顺序：M2 先落地 Qoder 独立模块；M3 引入 `ActivitySource` trait 统一 Qoder 与 dsh，避免只有一个实现时提前抽象。

### 3.3 增量水位与去重

| 数据源 | 水位/去重键 | 说明 |
| --- | --- | --- |
| Qoder | `chat_message.rowid` | 只处理水位之后的新消息；`gmt_create` 仅辅助展示 |
| dsh | 每个会话文件的 `seq` | 事件自带递增序号 |
| Claude Code | `message.id` / `requestId` | 同一请求可能拆成多条 assistant 消息 |
| Codex SQLite | `thread_id` + `updated_at_ms` + 上次 `tokens_used` | 累计值做差 |
| Codex JSONL | 文件 offset + `token_count` 序号 | 兼容 CLI 形态 |
| Inbox | 文件名 + 内容中的 `event_id` | hook/插件写入 |

水位保存在应用配置目录（Tauri app config dir）下的 `watermarks.json`，处理成功后延迟写盘。首次运行或数据源首次出现时，只初始化水位，不发射历史事件。

### 3.4 聚合与动画强度

- 聚合窗口 300ms（可配置 200–500ms）。
- `tokenRate = windowTokens / windowSeconds`。
- `intensity = clamp(tokenRate / 200.0, 0.15, 1.0)`，再做指数平滑，避免跳变。
- 连续 3 秒无事件回到 idle；连续 30 秒无事件进入 sleep。
- turn 结束后进入 celebrate，持续 2–3 秒，再回到 idle。
- 错误事件进入 error 状态，持续 3 秒。
- 多会话/多工具：首版合并为全局活跃度，同时在事件里保留 source，供气泡显示"来自 Qoder"。

### 3.5 本地收件箱（hook/插件入口）

- 目录：应用配置目录下的 `inbox/`。
- 写入协议：写 `xxx.json.tmp`，完成后 rename 为 `xxx.json`；监听方忽略 `.tmp`。
- 事件格式与 `AiEvent` 一致，另加 `event_id` 用于去重。
- 优点：不需要端口，不怕宠物未启动时丢事件，适合 hook 和 dsh 插件。
- 如后续需要 HTTP：仅绑定 `127.0.0.1`，使用随机端口和随机 Bearer token，token 存在本地设置中，默认关闭。

### 3.6 宠物状态机

```
sleep --event--> working
idle  --event--> working
working --3s 无事件--> idle
idle --30s 无事件--> sleep
working --turn_end--> celebrate --2.5s--> idle
任意 --error--> error --3s--> idle
paused: 所有状态静默，不接收动画事件
```

前端维护动画状态，Rust 只负责聚合和发射 `pet://activity`：

```ts
type PetActivity = {
  state: "idle" | "working" | "celebrate" | "error" | "sleep";
  intensity: number; // 0..1
  source?: "qoder" | "dsh" | "claude_code" | "codex";
  tokenRate: number;
};
```

### 3.7 气泡引擎

- 单气泡、最多 30 个汉字、默认冷却 40 秒。
- 优先级：错误 > 任务完成 > token 里程碑 > 任务开始 > 闲置闲聊。
- 输入只允许元数据：工具名、项目/任务标题、token 数、耗时、工具调用类型、文件数。
- 任务标题最多保留 24 字，优先选择带 `{task}` 的模板；设置中可关闭标题读取。
- 模板放在 `src/assets/bubbles/zh-CN.json`，示例：

```json
{
  "working": [
    "{tool} 又在狂吃 token 了…",
    "这个任务看起来不小，钱钱在飞",
    "敲敲敲，{tokens} token 没了"
  ],
  "celebrate": [
    "搞定！这次花了 {tokens} token",
    "任务完成，捡回一点钱钱",
    "{project} 又活过来了"
  ],
  "error": [
    "好像翻车了…要不要看看日志",
    "这次没跑通，先别急着花钱"
  ]
}
```

- 模板渲染时对项目名做截断/脱敏；默认不显示完整路径。
- P2 再接本地小模型（Ollama 或 llama.cpp sidecar），仍然只传元数据。

### 3.8 窗口与点击穿透

初始窗口配置建议：

```json
{
  "label": "pet",
  "width": 420,
  "height": 420,
  "transparent": true,
  "decorations": false,
  "alwaysOnTop": true,
  "skipTaskbar": true,
  "resizable": false,
  "shadow": false
}
```

点击穿透是 M1 需要验证的重点，候选方案：

- 方案 A（推荐先试）：窗口默认 `set_ignore_cursor_events(true)`；Rust 每 100ms 读取全局鼠标位置，进入宠物热区时关闭忽略，离开后恢复。热区用硬编码矩形或前端上报的包围盒。
- 方案 B：宠物本体窗口保持可交互，只把窗口做得足够紧，透明区域尽量小。
- 方案 C：宠物窗口完全穿透，另建一个很小的交互窗口。实现复杂，作为 A 的兜底。

拖动通过 `startDragging`；位置用 `tauri-plugin-window-state` 记忆并在多显示器变化后钳制到可见区域。关闭窗口默认隐藏到托盘，托盘菜单提供真正退出。

## 4. 工具接入方案

### 4.1 Qoder / Qoder CN（P0）

本机实测（Qoder CN）：

- 数据库：`%APPDATA%\QoderCN\SharedClientCache\cache\db\local.db`
- 表：`chat_message`
- 关键列：`id`、`session_id`、`request_id`、`role`、`content`、`summary`、`token_info`、`model_info`、`gmt_create`
- `token_info` 为 JSON：`prompt_tokens`、`completion_tokens`、`cached_tokens`、`max_input_tokens`
- 会话标题在 `chat_session.session_title`

实现：

1. 使用 `rusqlite` 以只读方式打开，设置 `busy_timeout`。
2. 每 1 秒轮询：`role = 'assistant' AND token_info IS NOT NULL AND token_info <> ''` 且水位之后的新行。
3. 以 `rowid` 为唯一增量主键，`gmt_create` 仅辅助展示，不参与水位。
4. 解析 `token_info`，映射为 `TokenUsage`；同一 `request_id` 的多条消息允许累加，但同一条消息只能计一次。M2 口径为 `total = prompt_tokens + completion_tokens`，`cached_tokens` 单独记录、暂不抵扣。
5. 会话标题通过 `session_id` 关联 `chat_session.session_title` 获取，截断 24 字后用于气泡；设置中可关闭。
6. 数据库被占用时：复制 `local.db`、`-wal`、`-shm` 到临时目录后读取快照。
7. 只查询 `chat_message`，不要遍历 `memory_common_embedding_*` 等依赖 `vec0` 扩展的表。
8. Qoder CLI/桌面 App 可能只有 credits 没有 token，需要识别并把状态标为"仅 credits，无法统计 token"（M2 尚未实现）。
9. M2 实际落在 `src-tauri/src/qoder.rs` 和 `events.rs`；M3 再整理为 `activity/` 目录和统一 trait。

M2 实现状态（2026-11-02）：

- 已自动探测 `QoderCN`、`Qoder`、`Qoder CN` 三种 APPDATA 路径。
- 首次运行将水位初始化为当前最大 rowid，避免回放历史。
- 已实现只读连接、`busy_timeout`、快照兜底、状态事件和 4 个单元测试。
- 临时数据库副本联调通过：新增 400 token 后水位推进并触发 working 动画与气泡。

待确认：

- Qoder 国际版路径可能是 `%APPDATA%\Qoder\...`，需要做成可配置并自动探测。
- `cached_tokens` 是否已包含在 `prompt_tokens` 内，需要在设置页展示口径说明，避免和用户预期不一致。

### 4.2 DeepSeek Harness / dsh（P0）

本机实测：

- 目录：`~/.dsh/sessions/<项目 slug>/session-<uuid>/session.v4.jsonl.zstd`
- 每个文件是 zstd 压缩的 JSONL，事件包含 `seq`、`time`、`type`、`data`。
- 已观察到的事件：`turn/start`、`turn/end`、`step/start`、`step/end`、`tool/call`、`tool/result`、`assistant/message`、`session/title`。
- usage 字段：`data.usage.{inputTokens, outputTokens, cacheReadTokens, cacheWriteTokens, totalTokens}`。

实现（M3 已落地）：

1. 轮询 `~/.dsh/sessions`（2 秒），对每个 `session.v4.jsonl.zstd` 用 mtime + size 判断是否变化；notify 防抖留到后续。
2. 文件变化时整体 zstd 解压并按行解析 JSON；损坏行/半行跳过。
3. 以每个文件的 `seq` 去重；水位按会话文件路径持久化到 `dsh-watermark.json`。
4. 事件映射：
   - `turn/start` → working（低强度）
   - `assistant/message` 的 `data.usage` → working + token 气泡
   - `tool/call` → working
   - `turn/end` 的 `data.reason.kind`：`completed` → celebrate；`failed`/`error` → error；其他 → idle
5. 如果应用在 turn 中途启动，先补算当前 turn 已产生的 usage，保证庆祝气泡的数据完整。
6. 隐私：只读取事件类型、seq、usage、turn reason 和 `session/title` 的标题，不读取 `assistant/message` 正文。
7. 首次运行初始化所有已有会话的水位，避免回放历史；新出现的会话从 seq 0 开始处理。
8. 标题从 `session/title` 事件的 `data.title` 提取，截断 24 字后用于气泡；设置中可关闭。

M3 实现状态（2026-11-02）：

- 真实 dsh 目录 5 个会话水位初始化成功。
- 临时会话 4 个事件处理完成，水位 seq 4，working 动画与气泡确认。
- 5 个 dsh 单元测试 + 4 个 Qoder 单元测试全部通过。
- 未采用 `notify` 和 dsh 插件，文件轮询适配器先行。

后续增强：

- dsh 是插件化运行时，P1/P2 可写一个 dsh 插件，在 turn/step 结束时直接写 `inbox/`，避免依赖内部文件格式。文件适配器保留为兜底。

### 4.3 Claude Code（P1）

本机未安装，按官方能力设计：

- transcript：`~/.claude/projects/<编码后的项目路径>/<session-id>.jsonl`
- assistant 消息的 `message.usage` 包含 `input_tokens`、`output_tokens`、`cache_creation_input_tokens`、`cache_read_input_tokens`。
- hooks：`SessionStart`、`UserPromptSubmit`、`Stop`、`SessionEnd` 等；hook 输入包含 `session_id`、`transcript_path`、`cwd`。
- OpenTelemetry（P2）：`CLAUDE_CODE_ENABLE_TELEMETRY=1`，指标名 `claude_code.token.usage`，OTLP 默认 `http/protobuf`，本地收 4318 端口。

实现优先级：

1. 提供 `moyuyu-hook`（小型可执行文件或脚本），在 `Stop`/`SessionEnd` 时把 hook 输入写入 `inbox/`。
2. 宠物收到 hook 后，读取 `transcript_path`，只取新增 assistant 消息的 usage；按 `message.id`/`requestId` 去重。
3. 设置页提供"一键写入 hook 配置"和"手动配置说明"两种方式，修改前备份 `settings.json`。
4. OTel 作为 P2：Rust 侧用 axum 接收 OTLP，解析指标并映射为 `TokensDelta`；只在用户开启时启动。

### 4.4 Codex（P1）

本机实测为 SQLite 形态：

- `~/.codex/state_5.sqlite` 的 `threads` 表包含 `id`、`tokens_used`、`rollout_path`、`title`、`first_user_message`、`updated_at_ms`、`model`、`cwd`。
- `~/.codex/logs_2.sqlite` 的 `logs` 表是运行日志，未观察到 token 字段。
- 本机 `threads` 目前 0 行，说明还没有可读的历史记录。

实现：

1. 启动时探测 Codex 形态：
   - 存在 `state_5.sqlite` → 使用 SQLite 适配器。
   - 存在 `~/.codex/sessions/**/*.jsonl` → 使用 JSONL 适配器（CLI 形态）。
2. SQLite：轮询 `threads` 中 `updated_at_ms` 变化的行，读取 `tokens_used` 并与上次值做差；标题用于气泡。
3. JSONL：监听 `token_count` 事件；注意字段可能是累计值，必须做差。
4. CLI 的 `config.toml` `notify` 可作为补充触发源，写入 `inbox/`；不修改用户配置，由设置页引导。
5. 未检测到数据时显示"已安装但暂无记录"，不要报错。

## 5. 技术选型

| 层面 | 选择 | 理由 |
| --- | --- | --- |
| 壳 | Tauri 2 | 已有骨架；透明/置顶/托盘/自启生态齐全 |
| 前端 | Vue 3 + TS | 已确定；组件和状态管理成本低 |
| 动画 | 精灵图 + Canvas 2D | 首版最轻、可控；Live2D 后置 |
| 采集 | Rust 常驻任务 | 文件/SQLite/zstd 处理不阻塞 UI |
| SQLite | `rusqlite`（bundled） | 只读访问 Qoder/Codex 数据库 |
| zstd | `zstd` crate | dsh 会话解压 |
| 文件监听 | `notify` crate | dsh、inbox 监听 |
| 异步 | `tokio` | 轮询、防抖、取消令牌 |
| 设置存储 | `tauri-plugin-store` | 简单可靠 |
| 单实例 | `tauri-plugin-single-instance` | 避免双宠物 |
| 自启 | `tauri-plugin-autostart` | Windows 原生 |
| 窗口位置 | `tauri-plugin-window-state` | 多屏/DPI 记忆 |
| 日志 | `tracing` + `tauri-plugin-log` | 本地排障，不记录正文 |
| 可选 HTTP | `axum` | 仅 P2，OTel 或插件回传 |

暂不引入：

- 代理拦截/中间人：侵入性强，且无法覆盖 IDE。
- 屏幕 OCR：不稳定且隐私风险高。
- 云端 LLM：默认不联网。
- Pinia/状态库：首版用 Vue 组合式 API 足够。

## 6. 目标工程结构

```text
MoyuyuPet/
├─ docs/
│  ├─ requirements.md
│  └─ implementation-plan.md
├─ src/                          # Vue 3 + TS
│  ├─ App.vue
│  ├─ components/
│  │  ├─ PetCanvas.vue
│  │  └─ SpeechBubble.vue
│  ├─ composables/
│  │  ├─ usePetActivity.ts
│  │  └─ useSpeechBubble.ts
│  ├─ services/
│  │  └─ tauriEvents.ts
│  └─ assets/
│     ├─ pet/                    # 精灵图与帧配置
│     └─ bubbles/zh-CN.json
├─ src-tauri/
│  ├─ src/
│  │  ├─ main.rs
│  │  ├─ lib.rs
│  │  ├─ activity/
│  │  │  ├─ mod.rs               # ActivitySource trait
│  │  │  ├─ model.rs             # AiEvent / TokenUsage
│  │  │  ├─ aggregator.rs        # 窗口聚合与速率
│  │  │  ├─ inbox.rs             # 本地收件箱
│  │  │  └─ adapters/
│  │  │     ├─ qoder.rs
│  │  │     ├─ dsh.rs
│  │  │     ├─ claude.rs
│  │  │     └─ codex.rs
│  │  ├─ settings.rs
│  │  └─ windows.rs
│  ├─ tests/fixtures/            # 脱敏样例数据
│  ├─ capabilities/default.json
│  └─ tauri.conf.json
├─ package.json
└─ pnpm-lock.yaml
```

## 7. 里程碑

> 进度状态、逐项任务和验收记录以 [milestones.md](./milestones.md) 为准；本节只保留概览。

| 里程碑 | 内容 | 验收 |
| --- | --- | --- |
| M0（已完成） | Tauri 2 + Vue 3 + TS 骨架 | 构建、启动通过 |
| M1 宠物外壳 | 透明窗口、托盘占位、精灵图五状态、假事件调试入口、气泡模板、点击穿透原型 | 假事件能驱动状态机；点击穿透方案验证通过 |
| M2 Qoder 适配器 | 只读轮询、水位、去重、设置路径、fixture 测试 | 真实 Qoder token 1.5 秒内触发；重启不回放 |
| M3 dsh 适配器 | zstd 解压、事件映射、seq 去重 | turn/step 能驱动 working/celebrate |
| M4 Claude Code | hook 安装器、hook shim、transcript 解析、inbox | 装好 Claude Code 后可自动连接；手动配置可用 |
| M5 Codex | SQLite 形态 + CLI 形态探测与解析 | 两种形态至少一种可用，失败时状态正确 |
| M6 日常使用 | 设置窗口、暂停、自启、单实例、位置记忆、适配器状态、隐私说明 | 清单全部通过 |
| M7 增强 | OTel、本地小模型、Live2D、音效、安装包 | 可选，按体验优先级排 |

建议先做 M1 + M2：本机已有 Qoder 数据，可以最快跑出"真实 token → 宠物反应"的闭环。

## 8. 测试计划

- 单元测试：
  - Qoder `token_info` 解析、缓存字段映射、水位推进。
  - dsh 事件映射、seq 去重、半行/损坏行跳过。
  - Claude transcript usage 解析与 requestId 去重。
  - Codex 累计值做差与重置处理。
  - 聚合窗口、速率、强度平滑。
- 集成测试：
  - 临时 SQLite 数据库 + 脱敏 fixture，验证增量读取。
  - 临时 zstd 会话文件 + 追加写入，验证防抖和重试。
  - inbox 原子写入 + 重名/损坏文件。
- 手动测试：
  - 点击穿透、拖动、托盘、暂停、隐藏/退出。
  - 100%/150%/200% DPI、多显示器、分辨率变化。
  - 断网、工具未安装、工具运行中删除/重建数据库。
- 性能测试：
  - 闲置 10 分钟 CPU/内存采样。
  - 连续工作 30 分钟，动画帧率与内存曲线。
  - 24 小时稳定性观察。

测试数据必须脱敏，仓库中禁止提交真实会话、token 明文或项目路径。

## 9. 风险与对策

| 风险 | 影响 | 对策 |
| --- | --- | --- |
| 工具格式随版本变化 | 适配器失效 | 能力探测、版本字段、健康状态、fixture 回归；失败只降级 |
| SQLite 被工具占用 | 读不到最新数据 | 只读连接 + busy_timeout + 临时快照兜底 |
| dsh zstd 追加/半帧 | 解析失败或重复 | 防抖、保留上次水位、seq 去重、失败重试 |
| token 口径不一致 | 统计与用户预期不符 | 设置页展示口径；缓存字段单独展示；不臆造费用 |
| 点击穿透体验差 | 宠物挡住操作或点不到 | M1 先做原型，A/B/C 方案对比后再定 |
| WebView2 内存偏高 | 常驻占用大 | 精灵图轻量方案、闲置降帧、及时释放模型资源 |
| 隐私争议 | 用户不信任 | 默认只读元数据、不联网、设置页透明说明、日志不落正文 |
| 工具条款限制 | 合规风险 | 只读本地数据、不修改工具、不做代理拦截、不上传 |
| 打包/签名 | SmartScreen 警告 | M7 处理安装包与签名；首版提供绿色版说明 |

## 10. 待确认问题

1. Qoder 国际版与 CN 版路径差异，是否都要首期支持？
2. dsh 采用文件适配器先行，还是直接投入插件开发？
3. Claude Code hook 安装是否允许自动修改 `settings.json`？
4. Codex 桌面版与 CLI 版哪个优先？
5. 点击穿透采用全局鼠标轮询方案是否可接受（约 10Hz）？
6. 精灵图素材来源、尺寸、帧率、动作数量。
7. 气泡默认频率、语气强度、是否显示项目名。
8. 是否需要"每日 token 小结"和历史数据保留策略。
9. 首发是否只发 Windows 安装包。

## 11. 文档维护

- 适配器格式或数据源发生变化时，必须同步更新本文件第 4 节和对应 fixture。
- 新增需求先改 `requirements.md`，再在本文件补充实现和验收方式。
- 每次版本迭代更新文档头部的版本号和日期。
