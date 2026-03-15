# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

这是一个基于 **Dioxus 0.7.1** 的 Windows 桌面应用（代号 win_aide），使用 Cargo workspace 管理，提供全局快捷键启动器功能。

## 常用命令

```bash
# 安装 dx CLI（首次需要）
curl -sSL http://dioxus.dev/install.sh | sh

# 开发运行（需先 cd 到 desktop 目录）
cd packages/desktop && dx serve    # Desktop 平台

# 构建检查
cargo check                        # 全工作区类型检查
cargo clippy                       # Lint（含 Dioxus 特定规则）
cargo build -p desktop             # 构建单个包
cargo test                         # 运行测试
```

## 架构

Cargo workspace 包含两个成员包，共享 `dioxus = "0.7.1"` 工作区依赖：

- **`packages/ui`** — 共享 UI 组件库（Navbar、ShortcutList、ShortcutForm、ProcessPicker）
- **`packages/desktop`** — Desktop 平台入口，使用 `dioxus/desktop` 特性，集成 Windows API 实现热键、进程管理、系统托盘等功能

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
