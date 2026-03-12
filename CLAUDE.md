# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

这是一个基于 **Dioxus 0.7.1** 的多平台 Rust 项目（代号 win_aide），使用 Cargo workspace 管理，支持 Web、Desktop 和 Mobile 三个平台。

## 常用命令

```bash
# 安装 dx CLI（首次需要）
curl -sSL http://dioxus.dev/install.sh | sh

# 开发运行（需先 cd 到对应平台目录）
cd packages/web && dx serve        # Web 平台
cd packages/desktop && dx serve    # Desktop 平台
cd packages/mobile && dx serve --platform android   # Android
cd packages/mobile && dx serve --platform ios        # iOS

# 构建检查
cargo check                        # 全工作区类型检查
cargo check -p web                 # 检查单个包
cargo clippy                       # Lint（含 Dioxus 特定规则）
cargo build -p desktop             # 构建单个包
cargo test                         # 运行测试
```

## 架构

Cargo workspace 包含四个成员包，共享 `dioxus = "0.7.1"` 工作区依赖：

- **`packages/ui`** — 跨平台共享 UI 组件库（Hero、Navbar、Echo），不依赖任何平台特定功能
- **`packages/web`** — Web 平台入口，使用 `dioxus/web` 特性，编译为 WebAssembly
- **`packages/desktop`** — Desktop 平台入口，使用 `dioxus/desktop` 特性
- **`packages/mobile`** — Mobile 平台入口，使用 `dioxus/mobile` 特性
- **`packages/api`**（未注册到 workspace）— 全栈服务器函数，使用 `dioxus/fullstack` + `dioxus/server`

每个平台包结构一致：`main.rs`（路由定义 + 启动）→ `views/`（Home、Blog 页面组件）。三个平台共享相同的路由结构（`/` 和 `/blog/:id`），但视图实现可独立演化。

## Dioxus 0.7 关键约定

**必须参考 `AGENTS.md`** — 包含完整的 Dioxus 0.7 API 文档。0.7 版本重构了所有 API，`cx`、`Scope`、`use_state` 已移除。

核心要点：
- 组件用 `#[component]` 宏，返回 `Element`，UI 用 `rsx!` 宏
- 状态：`use_signal()` 替代旧版 `use_state`，`use_memo()` 做记忆化
- Props 必须是 owned 类型（`String` 而非 `&str`），实现 `PartialEq + Clone`；用 `ReadOnlySignal` 包装使 props 响应式
- 路由：`#[derive(Routable)]` 枚举 + `#[route("/path")]` + `#[layout(Component)]`
- 资源引用：`asset!("/assets/...")` 宏，路径相对项目根目录
- 全栈服务器函数：`#[post]` / `#[get]` 宏

## Clippy 规则

`clippy.toml` 配置了 Dioxus 专用的 await-holding 检查：**不要在 await 点持有 `GenerationalRef`、`GenerationalRefMut` 或 `WriteLock`**，否则会导致借用死锁。
