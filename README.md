# MoyuyuPet

> show your earning every second!!!
>
> 我钱钱呢？？

一个本地优先的 AI 桌宠：感知 Qoder、DeepSeek Harness、Claude Code、Codex 等工具的
token 消耗与任务状态，用 2D 动画和少量评价气泡给出反馈。

当前状态：可运行的 Tauri 骨架（M0 已完成），业务功能按里程碑推进中。

## 特性（规划）

- 透明、置顶、可拖动的桌面宠物窗口
- 根据 token 消耗速率驱动工作、庆祝、闲置等动画状态
- 与任务相关的低频评价气泡，默认只使用元数据
- 本地优先：不联网、不读对话正文、不上传数据
- 适配器架构：新增 AI 工具只需增加一个适配器

## 技术栈

- Tauri 2 + Rust
- Vue 3 + TypeScript + Vite
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

## 文档

- [需求文档](docs/requirements.md)
- [落地方案](docs/implementation-plan.md)
- [里程碑与推进计划](docs/milestones.md)

## 项目结构

```text
src/          Vue 前端（宠物窗口、动画、气泡）
src-tauri/    Rust 后端（窗口、适配器、事件聚合）
docs/         需求、方案与里程碑
```

## 隐私

MoyuyuPet 默认只在本地读取 AI 工具留下的元数据，用于驱动动画和气泡；不读取对话正文，
不联网，不上传任何内容。接入云端能力必须由用户显式开启。

## 许可证

[MIT](LICENSE)
