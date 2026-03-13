# 暗色/亮色主题切换设计

## 需求

为 Win Aide 桌面应用添加暗色/亮色模式切换功能。

- 切换入口：导航栏快捷图标 + 设置面板 checkbox
- 持久化：保存到 config.json，重启后记住选择
- 亮色风格：柔和暖白

## 方案：CSS 自定义属性 + Rust 切换

在 `tailwind.css` 中定义两套语义化 CSS 变量（亮色默认 + `.dark` 覆盖），组件统一使用语义 class。Rust 端只在根 `<div>` 切换 `dark` class。

## CSS 变量体系

### 亮色（默认）

| Token | 值 | 用途 |
|---|---|---|
| `--color-bg-primary` | `#fafaf8` | 页面背景 |
| `--color-bg-card` | `#f0efe9` | 卡片/面板背景 |
| `--color-accent` | `#5b6fa3` | 按钮强调色 |
| `--color-accent-focus` | `#4a5d8e` | 悬停/焦点 |
| `--color-text-primary` | `#1a1a1a` | 主文字 |
| `--color-text-secondary` | `#6b7280` | 次要文字 |
| `--color-text-muted` | `#9ca3af` | 弱文字 |
| `--color-border-default` | `#d1d5db` | 主边框 |
| `--color-border-subtle` | `#e5e7eb` | 浅边框 |

### 暗色（`.dark` 覆盖）

| Token | 值 |
|---|---|
| `--color-bg-primary` | `#0f1116` |
| `--color-bg-card` | `#1e222d` |
| `--color-accent` | `#91a4d2` |
| `--color-accent-focus` | `#6d85c6` |
| `--color-text-primary` | `#ffffff` |
| `--color-text-secondary` | `#9ca3af` |
| `--color-text-muted` | `#6b7280` |
| `--color-border-default` | `#374151` |
| `--color-border-subtle` | `#1f2937` |

## Rust 层

### config.rs

`Settings` 新增 `dark_mode: bool`，默认 `true`。

### main.rs App 组件

- 从 config 读取 `dark_mode` 初始值
- 根 `<div>` 动态 class：有 dark_mode 时加 `dark`
- `dark_mode` 作为 `Signal<bool>` 通过 props 传给 Home

## UI 层

### 导航栏

按钮组中加主题切换按钮（☀/🌙），点击切换 dark_mode signal 并保存配置。

### 设置面板

新增「暗色模式」checkbox，与 dark_mode 双向绑定。

### 组件 class 替换

| 现有 class | 替换为 |
|---|---|
| `text-white` | `text-text-primary` |
| `text-gray-300` | `text-text-secondary` |
| `text-gray-400` | `text-text-secondary` |
| `text-gray-500` | `text-text-muted` |
| `border-gray-700` | `border-border-default` |
| `border-gray-800` | `border-border-subtle` |
| `hover:bg-gray-700` | `hover:bg-border-default` |
| `hover:bg-gray-800/50` | `hover:bg-bg-card` |
| `bg-gray-700` | `bg-border-default` |

不动的部分：红色系、黄色系、accent 系列。

## 涉及文件

- `tailwind.css` — 变量定义
- `packages/desktop/src/config.rs` — Settings 字段
- `packages/desktop/src/main.rs` — 根组件 dark class 切换
- `packages/desktop/src/views/home.rs` — 设置面板 + 导航栏按钮 + class 替换
- `packages/ui/src/navbar.rs` — class 替换
- `packages/ui/src/shortcut_list.rs` — class 替换
- `packages/ui/src/shortcut_form.rs` — class 替换
