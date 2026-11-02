# MoyuyuPet

> show your earning every second!!!
>
> 我钱钱呢？？

一个本地优先的 AI 桌宠：感知 Qoder、DeepSeek Harness、Claude Code、Codex 等工具的
token 消耗与任务状态，用 2D 动画和少量评价气泡给出反馈。

当前状态：M1 宠物外壳、M2 Qoder 适配器、M3 dsh 适配器、M6 日常控制核心、
M7.1 任务感知气泡、M7.2 视觉升级已完成；阶段一“本地 token → 动画/气泡”闭环已打通。

## 特性

- 透明、置顶、可拖动的桌面宠物窗口
- Live2D Tororo 白猫，按 token 速率切换工作、庆祝、闲置、睡眠动作
- 模型缺失时自动回退内置 SVG 宠物
- 奶油猫应用图标（Tauri 全平台图标）
- 已接入 Qoder / Qoder CN 与 DeepSeek Harness 的本地 token 检测
- 气泡会结合会话标题吐槽当前任务，标题最多 24 字且可关闭
- 设置窗口、托盘控制、暂停、单实例与开机自启
- 本地优先：不联网、不读对话正文、不上传数据

## 技术栈

- Tauri 2 + Rust
- Vue 3 + TypeScript + Vite
- Live2D（pixi.js + pixi-live2d-display）
- Windows 10/11 x64 首发

## 快速开始

环境要求：

- Node.js 22+ 与 pnpm
- Rust stable（MSVC toolchain）
- Windows：WebView2、Visual Studio Build Tools（C++ 桌面开发）

```bash
pnpm install
pnpm tauri dev
```

构建：

```bash
pnpm build        # 仅构建前端
pnpm tauri build  # 构建完整桌面应用
```

## Live2D 模型

Live2D 使用 Live2D Inc. 的官方示例模型 Tororo（白猫）。模型与 Cubism Core
受 Live2D 授权约束，不随仓库分发，需要本地下载：

```powershell
powershell -ExecutionPolicy Bypass -File scripts/fetch-live2d.ps1 -AcceptLicense
```

下载后会放到 `public/live2d/`（已在 `.gitignore` 中忽略）。未下载或加载失败时，
应用会自动回退到内置 SVG 宠物。

授权与署名见 [docs/third-party/live2d-tororo.md](docs/third-party/live2d-tororo.md)。

## 文档

- [需求文档](docs/requirements.md)
- [落地方案](docs/implementation-plan.md)
- [里程碑与推进计划](docs/milestones.md)

## 项目结构

```text
src/          Vue 前端（Live2D/SVG 宠物、动画、气泡、设置）
src-tauri/    Rust 后端（窗口、适配器、事件聚合）
scripts/      Live2D 资源下载脚本
docs/         需求、方案、里程碑与第三方授权说明
```

## 隐私

MoyuyuPet 默认只在本地读取 AI 工具留下的元数据，用于驱动动画和气泡；不读取对话正文，
不联网，不上传任何内容。接入云端能力必须由用户显式开启。

## 许可证

[MIT](LICENSE)
