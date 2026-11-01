# MoyuyuPet 里程碑与推进计划

| 项目 | 内容 |
| --- | --- |
| 版本 | v0.1 |
| 日期 | 2026-11-02 |
| 状态 | 执行中 |
| 关联文档 | [需求文档](./requirements.md) · [落地方案](./implementation-plan.md) |

本文档是 MoyuyuPet 的唯一进度看板。后续 session 推进任务时，以本文档中的
`NEXT` 里程碑为准，完成后更新状态、验收记录和进度日志。

## 1. 当前状态

| 项目 | 内容 |
| --- | --- |
| 最后更新 | 2026-11-02 |
| 当前里程碑 | M1 宠物外壳 |
| 当前阶段 | 阶段一：可用闭环 |
| 基线提交 | `6086a23`（Tauri 骨架 + 文档 + .gitignore） |
| 工作区 | 干净 |
| 下一步动作 | 验证点击穿透方案，搭宠物窗口与假事件状态机 |

## 2. 推进规则（给后续 session）

1. 开始前先读 [requirements.md](./requirements.md)、[implementation-plan.md](./implementation-plan.md) 和本文档。
2. 只推进状态为 `NEXT` 的最高优先级里程碑；依赖未完成时不得开始。
3. 开始后把该里程碑状态改为 `IN PROGRESS`，并把"当前里程碑"改成它。
4. 只做该里程碑范围内的任务；临时发现写入第 8 节"开放问题"，不擅自扩大范围。
5. 完成后必须运行第 6 节的验收方式，全部通过才能标记 `DONE`。
6. 更新本文档的状态、验收记录、进度日志；同步受影响的 `requirements.md` 和 `implementation-plan.md`。
7. 技术方案发生变化时，先改 `implementation-plan.md`，再回来更新本文档。
8. 测试数据必须脱敏；禁止提交真实会话、数据库、日志和项目路径。

状态含义：

| 状态 | 含义 |
| --- | --- |
| `DONE` | 已完成并通过验收 |
| `IN PROGRESS` | 正在推进 |
| `NEXT` | 下一个开始 |
| `TODO` | 未开始 |
| `BLOCKED` | 被依赖或决策阻塞，必须写明原因 |
| `CUT` | 本期不做，需写明原因 |

## 3. 阶段与里程碑总览

### 3.1 阶段划分

| 阶段 | 目标 | 包含里程碑 |
| --- | --- | --- |
| 阶段一：可用闭环 | 宠物能看、能动，Qoder 和 dsh 真实数据能驱动 | M1、M2、M3 |
| 阶段二：多工具覆盖 | 接入 Claude Code 和 Codex | M4、M5 |
| 阶段三：日常可用 | 设置、托盘、自启、隐私、稳定性 | M6 |
| 阶段四：体验增强 | OTel、本地模型、Live2D、打包发布 | M7 |

### 3.2 里程碑总览

| 里程碑 | 状态 | 依赖 | 核心产出 | 验收摘要 |
| --- | --- | --- | --- | --- |
| M0 工程基线 | `DONE` | 无 | Tauri 骨架、文档、gitignore | 构建与启动通过 |
| M1 宠物外壳 | `NEXT` | M0 | 透明窗口、五状态动画、假事件、气泡、点击穿透 | 假事件可驱动完整表现 |
| M2 Qoder 适配器 | `TODO` | M1 事件模型 | 只读轮询、水位、去重 | 真实 token 1.5 秒内触发 |
| M3 dsh 适配器 | `TODO` | M1 事件模型 | zstd 解析、事件映射 | turn/step 驱动工作与庆祝 |
| M4 Claude Code 适配器 | `TODO` | M1、收件箱 | hook、transcript 解析 | hook 写入后可自动连接 |
| M5 Codex 适配器 | `TODO` | M1 | SQLite / JSONL 双形态 | 至少一种形态可用 |
| M6 日常使用 | `TODO` | M1–M5 中已接入的适配器 | 设置、托盘、自启、单实例 | 清单全部通过 |
| M7 体验增强 | `TODO` | M6 | OTel、本地模型、Live2D、发布 | 按优先级逐项验收 |

M2 与 M3 在 M1 的事件模型稳定后可以并行推进；M4 与 M5 可以并行。

## 4. 里程碑详情

### M0 工程基线（DONE）

目标：从 WPF 迁移到可运行的 Tauri 2 + Vue 3 + TypeScript 骨架，并补齐项目文档。

交付物：

- [x] 删除 WPF 工程
- [x] Tauri 2 + Vue 3 + TypeScript 工程
- [x] 清理模板 demo，仅保留占位页
- [x] 适配 Tauri/Vite/Rust 的 `.gitignore`
- [x] 需求文档与落地方案

验收：

- [x] `pnpm build` 通过
- [x] `cargo check` 通过
- [x] `pnpm tauri build --debug --no-bundle` 通过
- [x] 应用可启动并显示窗口

证据：提交 `6086a23`。

### M1 宠物外壳（NEXT）

目标：在不接入真实 AI 工具的情况下，做出完整可看的宠物表现，并确定点击穿透方案。

范围内：

- 透明、置顶、无边框、不占任务栏的宠物窗口。
- 五状态动画：idle、working、celebrate、error、sleep。
- 假事件调试通道，用于驱动状态机。
- 气泡模板、冷却和优先级。
- 点击穿透原型与方案决策。
- 托盘最小可用、拖动、位置记忆。

范围外：

- 任何真实 AI 工具适配器。
- 设置窗口、开机自启、单实例。
- Live2D、音效、本地模型。

任务清单：

- [ ] 配置宠物窗口：`transparent`、`decorations: false`、`alwaysOnTop`、`skipTaskbar`、`resizable: false`
- [ ] Rust 侧窗口初始化：置顶、隐藏任务栏、位置钳制
- [ ] 点击穿透原型：对比方案 A（全局鼠标轮询+切换忽略）、B（紧凑窗口）、C（独立交互窗口），记录结论
- [ ] 实现拖动（`startDragging`）和右键/托盘入口
- [ ] 准备占位精灵图与帧配置（至少 idle、working、celebrate、error、sleep）
- [ ] `PetCanvas.vue`：按状态与强度播放动画
- [ ] `usePetActivity.ts`：监听 `pet://activity`，维护状态机
- [ ] 假事件命令/调试菜单：模拟 turn 开始、token 增量、turn 结束、错误
- [ ] 速率到强度的映射与平滑衰减
- [ ] `SpeechBubble.vue`：单气泡、30 字上限、冷却、优先级
- [ ] 气泡模板 `src/assets/bubbles/zh-CN.json`
- [ ] 托盘最小菜单：显示/隐藏、暂停/恢复、退出
- [ ] 窗口位置持久化与多显示器钳制
- [ ] 30 分钟连续运行检查

验收标准：

- [ ] 假事件能完整驱动 idle → working → celebrate → idle。
- [ ] error 状态优先级高于 working，并在 3 秒后恢复。
- [ ] 气泡同时最多一个，40 秒冷却生效，低优先级会被丢弃。
- [ ] 点击穿透方案确定，非交互区域不阻挡鼠标，宠物本体可点击和拖动。
- [ ] 窗口不抢焦点、不出现在任务栏、重启后位置合理。
- [ ] `pnpm build`、`cargo check`、`pnpm tauri dev` 均通过。
- [ ] 连续运行 30 分钟无崩溃、无明显内存增长。

验证方式：

```bash
pnpm build
cargo check --manifest-path src-tauri/Cargo.toml
pnpm tauri dev
```

决策记录（完成后填写）：

| 决策 | 结论 | 原因 |
| --- | --- | --- |
| 点击穿透方案 | 待定 | 需原型对比 |
| 占位精灵图规格 | 待定 | 需确认素材尺寸与帧率 |
| 气泡默认冷却 | 40 秒（暂定） | 避免打扰 |

### M2 Qoder 适配器（TODO）

目标：读取 Qoder / Qoder CN 本地数据库，把新增 token 消耗转换成 `TokensDelta`。

依赖：M1 的事件模型和假事件链路。

任务清单：

- [ ] 实现 `ActivitySource` trait 和 `QoderAdapter`
- [ ] 探测 Qoder CN / 国际版数据库路径，支持设置页覆盖
- [ ] 只读打开 `local.db`，设置 `busy_timeout`
- [ ] 轮询 `chat_message` 中水位之后、`role = 'assistant'` 且 `token_info` 非空的行
- [ ] 解析 `token_info`：`prompt_tokens`、`completion_tokens`、`cached_tokens`
- [ ] 明确 `cached_tokens` 与 `prompt_tokens` 的口径，并写入设置页说明
- [ ] 关联 `chat_session.session_title`，仅用于气泡
- [ ] 以 `rowid` + `gmt_create` 持久化水位
- [ ] 同消息去重，同 `request_id` 多消息允许累加
- [ ] 数据库被占用时复制 `local.db`、`-wal`、`-shm` 快照读取
- [ ] 不访问依赖 `vec0` 的 memory/embedding 表
- [ ] 解析失败只记录错误计数，不影响其他适配器
- [ ] 脱敏 fixture 与单元测试
- [ ] 适配器状态展示：已连接 / 未检测到 / 解析失败 / 仅 credits

验收标准：

- [ ] Qoder 产生新 token 后 1.5 秒内触发宠物 working。
- [ ] 重启应用不回放历史 token。
- [ ] 同一请求不重复计数。
- [ ] Qoder 运行中数据库被锁时不崩溃，能降级到快照。
- [ ] 无 Qoder 环境时显示"未检测到"，不影响假事件和其他适配器。
- [ ] 单元测试覆盖解析、水位推进和去重。

验证方式：真实 Qoder 对话 + fixture 测试 + 重启测试 + 锁库测试。

### M3 dsh 适配器（TODO）

目标：读取 DeepSeek Harness 的 zstd 会话文件，映射 turn/step/tool 和 usage。

依赖：M1 的事件模型。

任务清单：

- [ ] 监听 `~/.dsh/sessions`，对 `.zstd` 文件变化做防抖
- [ ] zstd 解码与 JSONL 解析，处理追加写入和半行
- [ ] 按 `seq` 去重，按会话文件持久化水位
- [ ] 事件映射：`turn/start`、`step/end`、`tool/call`、`turn/end`
- [ ] usage 映射：`inputTokens`、`outputTokens`、`cacheReadTokens`、`cacheWriteTokens`
- [ ] 标题从 `session/title` 获取，仅用于气泡
- [ ] 只读事件类型和 usage，不读取消息正文
- [ ] 解析失败保留上次水位并重试
- [ ] 脱敏 fixture 与单元测试
- [ ] 适配器状态展示

验收标准：

- [ ] dsh 任务开始时宠物进入 working，结束时进入 celebrate。
- [ ] 同一会话追加数据不重复计数。
- [ ] 损坏行/半帧被跳过，不影响后续事件。
- [ ] 不读取、不记录消息正文。
- [ ] 单元测试覆盖事件映射、seq 去重和重试。

后续可选：写 dsh 插件直接投递 `inbox/`，文件适配器保留兜底。

### M4 Claude Code 适配器（TODO）

目标：通过 hooks + transcript 接入 Claude Code。

依赖：M1 事件模型、本地收件箱（可随 M2 一并实现）。

任务清单：

- [ ] 定义并实现 `inbox/` 原子写入协议
- [ ] 提供 `moyuyu-hook`（独立小工具或脚本），把 hook 输入写入收件箱
- [ ] 支持 `Stop` / `SessionEnd` hook
- [ ] 提供自动写入 `settings.json` 的方案，修改前备份；同时提供手动配置说明
- [ ] 读取 `transcript_path` 中新增 assistant 消息的 `message.usage`
- [ ] 按 `message.id` / `requestId` 去重
- [ ] 适配器状态展示与错误计数
- [ ] 测试 fixture 与文档说明

验收标准：

- [ ] 安装 hook 后，Claude Code 完成一轮对话能触发宠物状态。
- [ ] token 数不重复计算。
- [ ] 不读取、不记录对话正文。
- [ ] 未安装 Claude Code 时显示"未检测到"。
- [ ] 自动配置失败时能回退到手动配置，且不改坏原 `settings.json`。

备注：当前开发机未安装 Claude Code，需要安装或使用 mock transcript 验证。

### M5 Codex 适配器（TODO）

目标：兼容 Codex 的 SQLite 形态与 CLI JSONL 形态。

依赖：M1 事件模型。

任务清单：

- [ ] 启动时探测 `state_5.sqlite` 与 `~/.codex/sessions`
- [ ] SQLite：轮询 `threads` 的 `updated_at_ms` 变化，读取 `tokens_used` 并做差
- [ ] SQLite：处理线程切换、重置和归档
- [ ] JSONL：解析 `token_count` 事件，处理累计值与缓存字段
- [ ] 可选支持 `config.toml` 的 `notify` 写入收件箱
- [ ] 标题/项目名仅用于气泡
- [ ] 适配器状态展示：已连接 / 已安装但暂无记录 / 不兼容
- [ ] 测试 fixture 与单元测试

验收标准：

- [ ] 至少一种 Codex 形态能真实驱动宠物。
- [ ] `threads` 为空时显示"已安装但暂无记录"，不报错。
- [ ] 累计 token 做差正确，重启不回放。
- [ ] 不读取消息正文。

备注：当前开发机 `threads` 表为 0 行，需要先产生 Codex 记录再验证。

### M6 日常使用（TODO）

目标：把宠物变成可以长期挂着使用的应用。

依赖：至少完成 M2、M3；M4/M5 可后补。

任务清单：

- [ ] 设置窗口：适配器开关、路径、轮询间隔、气泡频率、动画强度
- [ ] 托盘完整菜单：显示/隐藏、暂停/恢复、设置、退出
- [ ] 暂停：停止动画和气泡，适配器静默
- [ ] 单实例：重复启动唤醒已有窗口
- [ ] 开机自启，默认关闭
- [ ] 窗口位置记忆与多显示器恢复
- [ ] 适配器状态页：已连接 / 未检测到 / 解析失败 / 仅 credits
- [ ] 隐私说明页：每个工具读取的文件和字段
- [ ] 本地日志：只记录状态与错误，不记录正文
- [ ] 退出与数据清理流程
- [ ] 24 小时稳定性与性能检查

验收标准：

- [ ] 日常操作全部可用，无需改配置文件。
- [ ] 闲置 CPU ≤ 1%，工作态 CPU ≤ 5%，常驻内存 ≤ 250MB。
- [ ] 断网时核心功能不受影响。
- [ ] 关闭窗口隐藏到托盘，托盘可真正退出。
- [ ] 卸载/清空后不留水位、设置和日志。

### M7 体验增强（TODO）

目标：在不破坏稳定性的前提下增加体验项，按优先级逐项推进。

候选任务：

- [ ] Claude Code OpenTelemetry 精准用量（本地 OTLP）
- [ ] 本地小模型生成气泡文案（只传元数据）
- [ ] Live2D 模型与动作
- [ ] 音效与成就
- [ ] 每日 token 小结
- [ ] BongoCat 式全局键鼠镜像
- [ ] 多宠物 / 按工具分宠物
- [ ] 安装包、自动更新、代码签名
- [ ] macOS/Linux 支持

准入条件：M6 完成，且每项都能独立关闭，不影响核心链路。

## 5. 适配器完成定义（DoD）

新增或修改适配器时，必须同时满足：

- [ ] `probe()` 能区分已连接、未检测到、不兼容。
- [ ] `start()` 支持取消，暂停/退出不泄漏任务。
- [ ] 有水位与去重，重启不回放。
- [ ] 解析失败被隔离并计数，不影响其他适配器。
- [ ] 明确列出读取的文件与字段，写入隐私说明。
- [ ] 有脱敏 fixture 和单元测试。
- [ ] 在设置页显示健康状态。
- [ ] 更新 `requirements.md`、`implementation-plan.md` 和本文档。

## 6. 通用验收命令

```bash
# 前端类型检查与构建
pnpm build

# Rust 检查
cargo check --manifest-path src-tauri/Cargo.toml

# 调试构建
pnpm tauri build --debug --no-bundle

# 本地运行
pnpm tauri dev

# Rust 测试（在引入测试后启用）
cargo test --manifest-path src-tauri/Cargo.toml
```

手动验收基线：

- 窗口：透明、置顶、不进任务栏、可拖动、点击穿透正常。
- 动画：五状态可触发，强度随事件变化。
- 气泡：冷却、优先级、长度限制生效，不出现正文内容。
- 稳定性：工具未安装、数据库被锁、文件损坏时不崩溃。
- 隐私：断网可用，日志不含正文。

## 7. 进度日志

| 日期 | 里程碑 | 状态变化 | 提交 | 备注 |
| --- | --- | --- | --- | --- |
| 2026-11-02 | M0 | TODO → DONE | `6086a23` | Tauri 骨架、文档、gitignore 完成 |
| 2026-11-02 | M1 | TODO → NEXT | - | 等待开始 |

后续 session 完成里程碑时，在本表追加一行，并更新第 1、3 节的状态。

## 8. 开放问题与阻塞

| 编号 | 问题 | 影响里程碑 | 状态 |
| --- | --- | --- | --- |
| Q-01 | 宠物素材：占位精灵图规格、帧率、动作数量 | M1 | 待确认 |
| Q-02 | 点击穿透采用哪种方案 | M1 | 待原型验证 |
| Q-03 | 气泡默认频率与语气强度 | M1 | 暂定 40 秒 |
| Q-04 | Qoder 国际版路径与 CN 版差异 | M2 | 待确认 |
| Q-05 | Qoder `cached_tokens` 与 `prompt_tokens` 的口径 | M2 | 待确认 |
| Q-06 | Claude Code hook 是否允许自动修改 `settings.json` | M4 | 待确认 |
| Q-07 | Codex 优先支持桌面版还是 CLI 版 | M5 | 待确认 |
| Q-08 | 是否保留历史用量与每日小结 | M7 | 待确认 |
| Q-09 | 首发平台范围（仅 Windows 或跨平台） | M7 | 待确认 |

## 9. 给新 session 的启动说明

可以直接使用下面的提示词：

```text
请先阅读 docs/requirements.md、docs/implementation-plan.md 和 docs/milestones.md。
按 milestones.md 中状态为 NEXT 的里程碑推进；开始前确认依赖和开放问题，
完成后运行验收、更新里程碑状态和进度日志，并同步受影响的文档。
不要读取或提交真实会话、数据库和日志内容。
```
