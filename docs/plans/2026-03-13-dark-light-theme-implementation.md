# 暗色/亮色主题切换 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 Win Aide 桌面应用添加暗色/亮色模式切换，支持导航栏图标 + 设置面板 checkbox 双入口，持久化到 config.json。

**Architecture:** 在 Tailwind CSS 源文件中定义亮色默认变量 + `.dark` 类覆盖暗色变量。Rust 端在 `Settings` 新增 `dark_mode` 字段，App 根组件根据该值动态切换根 `<div>` 的 `dark` class。所有组件中的硬编码颜色 class 替换为语义化 token。

**Tech Stack:** Rust, Dioxus 0.7, Tailwind CSS v4, serde

---

### Task 1: CSS 变量体系 — 亮色默认 + .dark 覆盖

**Files:**
- Modify: `tailwind.css`

**Step 1: 替换 tailwind.css 为双主题变量**

将现有 `@theme` 改为亮色默认值，新增 `.dark` 选择器覆盖为暗色值，并添加新的语义 token（text-primary、text-secondary、text-muted、border-default、border-subtle）。

```css
@import "tailwindcss";

@theme {
  --color-bg-primary: #fafaf8;
  --color-bg-card: #f0efe9;
  --color-accent: #5b6fa3;
  --color-accent-focus: #4a5d8e;
  --color-text-primary: #1a1a1a;
  --color-text-secondary: #6b7280;
  --color-text-muted: #9ca3af;
  --color-border-default: #d1d5db;
  --color-border-subtle: #e5e7eb;
}

.dark {
  --color-bg-primary: #0f1116;
  --color-bg-card: #1e222d;
  --color-accent: #91a4d2;
  --color-accent-focus: #6d85c6;
  --color-text-primary: #ffffff;
  --color-text-secondary: #9ca3af;
  --color-text-muted: #6b7280;
  --color-border-default: #374151;
  --color-border-subtle: #1f2937;
}
```

**Step 2: 重新编译 Tailwind CSS**

Run: `npx @tailwindcss/cli -i tailwind.css -o packages/desktop/assets/tailwind.css`
Expected: 输出文件更新，无报错

**Step 3: Commit**

```bash
git add tailwind.css packages/desktop/assets/tailwind.css
git commit -m "feat: 添加双主题 CSS 变量体系（亮色默认 + .dark 覆盖）"
```

---

### Task 2: config.rs — Settings 新增 dark_mode 字段

**Files:**
- Modify: `packages/desktop/src/config.rs`

**Step 1: 给 Settings 添加 dark_mode 字段**

在 `Settings` 结构体中新增 `dark_mode: bool`。使用 `#[serde(default = "default_dark_mode")]` 确保旧配置文件反序列化兼容。

```rust
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Settings {
    pub auto_start: bool,
    pub start_minimized: bool,
    #[serde(default = "default_dark_mode")]
    pub dark_mode: bool,
}

fn default_dark_mode() -> bool {
    true
}
```

**Step 2: 更新 Default impl**

在 `AppConfig::default()` 中加入 `dark_mode: true`：

```rust
settings: Settings {
    auto_start: false,
    start_minimized: true,
    dark_mode: true,
},
```

**Step 3: 更新 test_default_config 测试**

在 `test_default_config` 中加入断言：

```rust
assert!(config.settings.dark_mode);
```

**Step 4: 更新 test_serialize_deserialize 测试**

在测试中 `Settings` 构造添加 `dark_mode: true`：

```rust
settings: Settings {
    auto_start: true,
    start_minimized: true,
    dark_mode: true,
},
```

**Step 5: 运行测试**

Run: `cargo test -p desktop`
Expected: 全部通过

**Step 6: Commit**

```bash
git add packages/desktop/src/config.rs
git commit -m "feat: Settings 新增 dark_mode 字段，默认暗色，兼容旧配置"
```

---

### Task 3: main.rs — 根组件动态 dark class + dark_mode signal

**Files:**
- Modify: `packages/desktop/src/main.rs`

**Step 1: 添加 dark_mode signal 并传递给 Home**

在 `App` 组件中，从 config 读取 `dark_mode` 初始值创建 signal，传给 Home：

```rust
let mut dark_mode = use_signal(|| config().settings.dark_mode);
```

**Step 2: 根 div 动态 class**

将根 `<div>` 的 class 改为动态拼接：

```rust
div {
    class: if dark_mode() { "dark bg-bg-primary text-text-primary font-sans min-h-screen" } else { "bg-bg-primary text-text-primary font-sans min-h-screen" },
    ...
}
```

注意：原来是 `text-white`，改为 `text-text-primary`。

**Step 3: Home 组件传入 dark_mode**

```rust
Home {
    config: config,
    paused: paused,
    dark_mode: dark_mode,
    on_config_changed: move |new_config: AppConfig| {
        dark_mode.set(new_config.settings.dark_mode);
        config.set(new_config.clone());
        // ... 其余逻辑不变
    },
}
```

**Step 4: 编译检查**

Run: `cargo check -p desktop`
Expected: 编译报错（Home 还没有 dark_mode prop），这是预期的，下一步修复

**Step 5: Commit**

```bash
git add packages/desktop/src/main.rs
git commit -m "feat: App 根组件添加 dark_mode signal 和动态 dark class"
```

---

### Task 4: home.rs — 接收 dark_mode prop + 双入口切换

**Files:**
- Modify: `packages/desktop/src/views/home.rs`

**Step 1: Home 组件签名添加 dark_mode prop**

```rust
#[component]
pub fn Home(
    config: Signal<AppConfig>,
    paused: Signal<bool>,
    dark_mode: Signal<bool>,
    on_config_changed: EventHandler<AppConfig>,
) -> Element {
```

**Step 2: 导航栏添加主题切换按钮**

在 Navbar 内右侧按钮组中，"设置"按钮前面加一个主题切换按钮：

```rust
div { class: "flex gap-2",
    button {
        class: "px-3 py-1.5 bg-accent text-text-primary rounded hover:bg-accent-focus transition-colors cursor-pointer text-sm",
        onclick: move |_| {
            form_data.set(ShortcutFormData::default());
            editing_id.set(None);
            conflict_msg.set(None);
            show_form.set(true);
        },
        "+ 添加快捷键"
    }
    button {
        class: "px-3 py-1.5 text-text-secondary hover:text-text-primary border border-border-default rounded hover:bg-border-default transition-colors cursor-pointer text-sm",
        title: if dark_mode() { "切换到亮色模式" } else { "切换到暗色模式" },
        onclick: move |_| {
            let mut cfg = config();
            cfg.settings.dark_mode = !cfg.settings.dark_mode;
            save_and_notify(cfg);
        },
        if dark_mode() { "☀" } else { "🌙" }
    }
    button {
        class: "px-3 py-1.5 text-text-secondary hover:text-text-primary border border-border-default rounded hover:bg-border-default transition-colors cursor-pointer text-sm",
        onclick: move |_| show_settings.set(!show_settings()),
        "设置"
    }
}
```

**Step 3: 设置面板添加暗色模式 checkbox**

在设置面板的 `flex gap-6` div 中，在最后一个 label 后面添加：

```rust
label { class: "flex items-center gap-2 text-sm text-text-secondary cursor-pointer",
    input {
        r#type: "checkbox",
        checked: config().settings.dark_mode,
        class: "w-4 h-4 accent-accent",
        onchange: move |_| {
            let mut cfg = config();
            cfg.settings.dark_mode = !cfg.settings.dark_mode;
            save_and_notify(cfg);
        },
    }
    "暗色模式"
}
```

**Step 4: 替换 home.rs 中所有硬编码颜色 class**

逐一替换（所有实例）：
- `text-white` → `text-text-primary`
- `text-gray-300` → `text-text-secondary`
- `text-gray-400` → `text-text-secondary`
- `border-gray-700` → `border-border-default`
- `hover:text-white` → `hover:text-text-primary`
- `hover:bg-gray-700` → `hover:bg-border-default`

注意：`bg-red-600`、`hover:bg-red-500`、`text-red-*`、`bg-yellow-*`、`text-yellow-*` 保持不变。

**Step 5: 编译检查**

Run: `cargo check -p desktop`
Expected: PASS

**Step 6: Commit**

```bash
git add packages/desktop/src/views/home.rs
git commit -m "feat: Home 组件添加主题切换按钮和设置面板 checkbox，替换硬编码颜色"
```

---

### Task 5: navbar.rs — 替换硬编码颜色 class

**Files:**
- Modify: `packages/ui/src/navbar.rs`

**Step 1: 替换 class**

```rust
div { class: "flex items-center justify-between px-4 py-3 border-b border-border-default",
```

仅 `border-gray-700` → `border-border-default`。

**Step 2: 编译检查**

Run: `cargo check -p ui`
Expected: PASS

**Step 3: Commit**

```bash
git add packages/ui/src/navbar.rs
git commit -m "feat: Navbar 组件替换硬编码边框颜色为语义 token"
```

---

### Task 6: shortcut_list.rs — 替换硬编码颜色 class

**Files:**
- Modify: `packages/ui/src/shortcut_list.rs`

**Step 1: 替换所有硬编码颜色 class**

逐一替换：
- 表头：`text-gray-400` → `text-text-secondary`，`border-gray-700` → `border-border-default`
- 数据行：`border-gray-800` → `border-border-subtle`，`hover:bg-gray-800/50` → `hover:bg-bg-card`
- 应用名称：`text-white` → `text-text-primary`
- 路径：`text-gray-400` → `text-text-secondary`
- 编辑按钮：`text-gray-300` → `text-text-secondary`，`hover:text-white` → `hover:text-text-primary`，`hover:bg-gray-700` → `hover:bg-border-default`
- 空状态：`text-gray-500` → `text-text-muted`

注意：红色系（删除按钮）保持不变。

**Step 2: 编译检查**

Run: `cargo check -p ui`
Expected: PASS

**Step 3: Commit**

```bash
git add packages/ui/src/shortcut_list.rs
git commit -m "feat: ShortcutList 组件替换硬编码颜色为语义 token"
```

---

### Task 7: shortcut_form.rs — 替换硬编码颜色 class

**Files:**
- Modify: `packages/ui/src/shortcut_form.rs`

**Step 1: 替换所有硬编码颜色 class**

逐一替换：
- 标题：`text-white` → `text-text-primary`
- label：`text-gray-400` → `text-text-secondary`
- input：`border-gray-700` → `border-border-default`，`text-white` → `text-text-primary`
- select：`border-gray-700` → `border-border-default`，`text-white` → `text-text-primary`
- "+" 符号：`text-gray-400` → `text-text-secondary`
- 浏览按钮：`bg-gray-700` → `bg-border-default`，`text-white` → `text-text-primary`，`hover:bg-gray-600` → `hover:bg-text-secondary`
- 取消按钮：`text-gray-300` → `text-text-secondary`，`hover:text-white` → `hover:text-text-primary`
- 保存按钮：`text-white` → `text-text-primary`（在 accent 背景上）

注意：红色冲突提示保持不变。`bg-black/60` 遮罩保持不变。

**Step 2: 编译检查**

Run: `cargo check -p ui`
Expected: PASS

**Step 3: Commit**

```bash
git add packages/ui/src/shortcut_form.rs
git commit -m "feat: ShortcutForm 组件替换硬编码颜色为语义 token"
```

---

### Task 8: 重新编译 Tailwind + 全量验证

**Files:**
- Regenerate: `packages/desktop/assets/tailwind.css`

**Step 1: 重新编译 Tailwind CSS（包含所有新 class）**

Run: `npx @tailwindcss/cli -i tailwind.css -o packages/desktop/assets/tailwind.css`
Expected: 输出成功，文件包含新的语义 class（`text-text-primary`、`border-border-default` 等）和 `.dark` 选择器

**Step 2: 全量编译检查**

Run: `cargo check`
Expected: 全工作区编译通过

**Step 3: 运行测试**

Run: `cargo test -p desktop`
Expected: 全部通过

**Step 4: Commit**

```bash
git add packages/desktop/assets/tailwind.css
git commit -m "chore: 重新编译 Tailwind CSS，包含双主题语义 class"
```

---

### Task 9: 手动验证

**Step 1: 启动应用**

Run: `cd packages/desktop && dx serve`

**Step 2: 验证暗色模式（默认）**

检查：页面背景深色 `#0f1116`，文字白色，按钮蓝紫色

**Step 3: 点击导航栏 ☀ 图标**

检查：切换到亮色，背景 `#fafaf8`，文字深色，图标变为 🌙

**Step 4: 打开设置面板**

检查：「暗色模式」checkbox 未选中

**Step 5: 勾选「暗色模式」checkbox**

检查：切换回暗色，导航栏图标同步变为 ☀

**Step 6: 重启应用**

检查：主题偏好已持久化，重启后保持上次选择
