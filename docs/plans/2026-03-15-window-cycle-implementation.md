# 同应用多窗口循环切换 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现 Alt+` / Alt+Shift+` 在同一应用的多个窗口间循环切换，作为可配置系统热键集成到 Settings 面板。

**Architecture:** 扩展现有热键线程，新增 `WindowCycleNext` / `WindowCyclePrev` 事件类型。在 launcher 线程中实现窗口循环逻辑（`GetForegroundWindow` → `EnumWindows` 按 PID 收集 → 环形切换）。Settings 面板新增配置区域。

**Tech Stack:** Rust, Dioxus 0.7.1, windows-rs 0.58 (Win32 API), serde

---

### Task 1: 数据模型 — WindowCycleSettings 结构体

**Files:**
- Modify: `packages/desktop/src/config.rs:1-49` (结构体定义区域)

**Step 1: 新增 WindowCycleSettings 结构体和 Default 实现**

在 `config.rs` 中 `fn default_dark_mode()` 之后添加：

```rust
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WindowCycleSettings {
    pub enabled: bool,
    pub modifier: Modifier,
    pub key: char,
}

impl Default for WindowCycleSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            modifier: Modifier::Alt,
            key: '`',
        }
    }
}
```

**Step 2: 在 Settings 中添加 window_cycle 字段**

修改 `Settings` 结构体，在 `dark_mode` 字段后添加：

```rust
#[serde(default)]
pub window_cycle: WindowCycleSettings,
```

**Step 3: 更新 AppConfig::default()**

在 `Default for AppConfig` 的 settings 初始化中添加：

```rust
settings: Settings {
    auto_start: false,
    start_minimized: true,
    dark_mode: true,
    window_cycle: WindowCycleSettings::default(),
},
```

**Step 4: 更新测试**

更新 `test_default_config` 添加断言：

```rust
assert!(config.settings.window_cycle.enabled);
assert_eq!(config.settings.window_cycle.modifier, Modifier::Alt);
assert_eq!(config.settings.window_cycle.key, '`');
```

更新 `test_serialize_deserialize` 的 Settings 构造，添加 `window_cycle` 字段：

```rust
settings: Settings {
    auto_start: true,
    start_minimized: true,
    dark_mode: true,
    window_cycle: WindowCycleSettings::default(),
},
```

**Step 5: 运行测试**

Run: `cargo test -p desktop`
Expected: 所有测试通过

**Step 6: 提交**

```bash
git add packages/desktop/src/config.rs
git commit -m "feat: 添加 WindowCycleSettings 数据模型"
```

---

### Task 2: 热键类型扩展 — HotkeyEvent 和 HotkeyCommand

**Files:**
- Modify: `packages/desktop/src/hotkey.rs:1-26` (类型定义区域)

**Step 1: 将 HotkeyEvent 从 struct 改为 enum**

替换现有的 `HotkeyEvent` struct：

```rust
/// 快捷键触发事件
#[derive(Debug, Clone)]
pub enum HotkeyEvent {
    /// 用户快捷键触发
    ShortcutTriggered { shortcut_id: String },
    /// 窗口循环：下一个
    WindowCycleNext,
    /// 窗口循环：上一个
    WindowCyclePrev,
}
```

**Step 2: 扩展 HotkeyCommand enum**

在 `UnregisterAll` 和 `Shutdown` 之间添加两个新变体：

```rust
pub enum HotkeyCommand {
    /// 注册一组快捷键（会先注销所有旧的）
    RegisterAll(Vec<Shortcut>),
    /// 注销所有快捷键（暂停）
    UnregisterAll,
    /// 注册窗口循环热键
    RegisterWindowCycle { modifier: Modifier, key: char },
    /// 注销窗口循环热键
    UnregisterWindowCycle,
    /// 停止监听并退出线程
    Shutdown,
}
```

**Step 3: 编译检查**

Run: `cargo check -p desktop`
Expected: 编译错误 — `launcher.rs` 中 `event.shortcut_id` 不再可用（因为 HotkeyEvent 从 struct 变成了 enum）。这是预期的，Task 5 会修复。

---

### Task 3: 热键注册逻辑 — 窗口循环热键

**Files:**
- Modify: `packages/desktop/src/hotkey.rs:6-124` (imports + 线程实现)

**Step 1: 更新 imports，添加 MOD_SHIFT**

在 import 行添加 `MOD_SHIFT`：

```rust
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT,
    MOD_SHIFT, MOD_WIN,
};
```

**Step 2: 添加保留 ID 常量**

在 `key_to_vk` 函数之前添加：

```rust
/// 窗口循环热键的保留 ID（与用户快捷键自增 ID 空间隔开）
const WINDOW_CYCLE_NEXT_ID: i32 = 10001;
const WINDOW_CYCLE_PREV_ID: i32 = 10002;
```

**Step 3: 扩展 key_to_vk 支持特殊按键**

```rust
fn key_to_vk(key: char) -> u32 {
    match key {
        '`' => 0xC0, // VK_OEM_3 (backtick)
        _ => key.to_ascii_uppercase() as u32,
    }
}
```

**Step 4: 在线程循环中添加 window_cycle_registered 状态和命令处理**

在 `thread::spawn` 闭包内，`let mut next_id: i32 = 1;` 后添加：

```rust
let mut window_cycle_registered = false;
```

在 `match cmd` 中 `UnregisterAll` 分支内，`registered_ids.clear();` 之后添加注销窗口循环热键的逻辑：

```rust
HotkeyCommand::UnregisterAll => {
    for &id in registered_ids.keys() {
        unsafe {
            let _ = UnregisterHotKey(HWND::default(), id);
        }
    }
    registered_ids.clear();
    // 同时注销窗口循环热键
    if window_cycle_registered {
        unsafe {
            let _ = UnregisterHotKey(HWND::default(), WINDOW_CYCLE_NEXT_ID);
            let _ = UnregisterHotKey(HWND::default(), WINDOW_CYCLE_PREV_ID);
        }
        window_cycle_registered = false;
    }
}
```

在 `Shutdown` 分支的 `for` 循环之后、`break;` 之前，添加：

```rust
if window_cycle_registered {
    unsafe {
        let _ = UnregisterHotKey(HWND::default(), WINDOW_CYCLE_NEXT_ID);
        let _ = UnregisterHotKey(HWND::default(), WINDOW_CYCLE_PREV_ID);
    }
}
```

在 `Shutdown` 分支之后添加两个新分支：

```rust
HotkeyCommand::RegisterWindowCycle { modifier, key } => {
    // 先注销旧的
    if window_cycle_registered {
        unsafe {
            let _ = UnregisterHotKey(HWND::default(), WINDOW_CYCLE_NEXT_ID);
            let _ = UnregisterHotKey(HWND::default(), WINDOW_CYCLE_PREV_ID);
        }
    }
    let mod_flags = modifier_to_win32(&modifier);
    let vk = key_to_vk(key);
    unsafe {
        let next_ok = RegisterHotKey(
            HWND::default(),
            WINDOW_CYCLE_NEXT_ID,
            mod_flags,
            vk,
        )
        .is_ok();
        let prev_ok = RegisterHotKey(
            HWND::default(),
            WINDOW_CYCLE_PREV_ID,
            mod_flags | MOD_SHIFT,
            vk,
        )
        .is_ok();
        window_cycle_registered = next_ok || prev_ok;
    }
}
HotkeyCommand::UnregisterWindowCycle => {
    if window_cycle_registered {
        unsafe {
            let _ = UnregisterHotKey(HWND::default(), WINDOW_CYCLE_NEXT_ID);
            let _ = UnregisterHotKey(HWND::default(), WINDOW_CYCLE_PREV_ID);
        }
        window_cycle_registered = false;
    }
}
```

**Step 5: 更新消息处理，分发窗口循环事件**

将现有的消息处理块替换为：

```rust
if msg.message == WM_HOTKEY {
    let hotkey_id = msg.wParam.0 as i32;
    match hotkey_id {
        WINDOW_CYCLE_NEXT_ID => {
            let _ = event_tx.send(HotkeyEvent::WindowCycleNext);
        }
        WINDOW_CYCLE_PREV_ID => {
            let _ = event_tx.send(HotkeyEvent::WindowCyclePrev);
        }
        _ => {
            if let Some(shortcut_id) = registered_ids.get(&hotkey_id) {
                let _ = event_tx.send(HotkeyEvent::ShortcutTriggered {
                    shortcut_id: shortcut_id.clone(),
                });
            }
        }
    }
}
```

**Step 6: 编译检查**

Run: `cargo check -p desktop`
Expected: 仍有 `launcher.rs` 编译错误（Task 4-5 修复）

---

### Task 4: 窗口循环核心逻辑

**Files:**
- Modify: `packages/desktop/src/launcher.rs:1-105`

**Step 1: 添加 GetForegroundWindow import**

更新 imports：

```rust
use windows::Win32::UI::WindowsAndMessaging::{
    AllowSetForegroundWindow, EnumWindows, GetForegroundWindow, GetWindowThreadProcessId,
    IsIconic, IsWindowVisible, SetForegroundWindow, ShowWindow, ASFW_ANY, SW_RESTORE,
};
```

**Step 2: 添加 Direction enum**

在 `FindWindowData` 之前添加：

```rust
/// 窗口循环方向
pub enum Direction {
    Next,
    Prev,
}
```

**Step 3: 添加 CollectWindowsData 和回调**

在 `find_window_by_exe` 函数之后添加：

```rust
struct CollectWindowsData {
    target_pid: u32,
    windows: Vec<HWND>,
}

/// EnumWindows 回调：收集指定进程的所有可见窗口
unsafe extern "system" fn collect_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data = &mut *(lparam.0 as *mut CollectWindowsData);

    if !IsWindowVisible(hwnd).as_bool() {
        return TRUE;
    }

    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));

    if pid == data.target_pid {
        data.windows.push(hwnd);
    }

    TRUE
}

/// 查找指定进程 ID 的所有可见窗口
fn find_all_windows_by_pid(pid: u32) -> Vec<HWND> {
    let mut data = CollectWindowsData {
        target_pid: pid,
        windows: Vec::new(),
    };

    unsafe {
        let _ = EnumWindows(
            Some(collect_windows_callback),
            LPARAM(&mut data as *mut CollectWindowsData as isize),
        );
    }

    data.windows
}
```

**Step 4: 添加 cycle_window 函数**

在 `find_all_windows_by_pid` 之后添加：

```rust
/// 在同一应用的多个窗口间循环切换
pub fn cycle_window(direction: Direction) {
    unsafe {
        let active = GetForegroundWindow();
        if active.0.is_null() {
            return;
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(active, Some(&mut pid));
        if pid == 0 {
            return;
        }

        let windows = find_all_windows_by_pid(pid);
        if windows.len() <= 1 {
            return;
        }

        let Some(index) = windows.iter().position(|&w| w == active) else {
            return;
        };

        let target_index = match direction {
            Direction::Next => (index + 1) % windows.len(),
            Direction::Prev => (index + windows.len() - 1) % windows.len(),
        };

        activate_window(windows[target_index]);
    }
}
```

**Step 5: 编译检查**

Run: `cargo check -p desktop`
Expected: 仍有 `launcher.rs` start_launcher 编译错误（event.shortcut_id 不存在，Task 5 修复）

---

### Task 5: Launcher 事件分发

**Files:**
- Modify: `packages/desktop/src/launcher.rs:107-141` (start_launcher 函数)

**Step 1: 更新事件处理逻辑**

将 `start_launcher` 中的事件处理块替换为 match 表达式：

```rust
if let Ok(event) = event_rx.recv_timeout(std::time::Duration::from_millis(50)) {
    match event {
        HotkeyEvent::ShortcutTriggered { shortcut_id } => {
            if let Some(shortcut) = shortcut_map.get(&shortcut_id) {
                launch_or_activate(shortcut);
            }
        }
        HotkeyEvent::WindowCycleNext => {
            cycle_window(Direction::Next);
        }
        HotkeyEvent::WindowCyclePrev => {
            cycle_window(Direction::Prev);
        }
    }
}
```

**Step 2: 编译检查**

Run: `cargo check -p desktop`
Expected: 通过（所有编译错误已修复）

**Step 3: 提交**

```bash
git add packages/desktop/src/hotkey.rs packages/desktop/src/launcher.rs
git commit -m "feat: 实现窗口循环热键注册和核心切换逻辑"
```

---

### Task 6: main.rs 集成

**Files:**
- Modify: `packages/desktop/src/main.rs:31-75` (main 函数)
- Modify: `packages/desktop/src/main.rs:101-131` (App 暂停/恢复逻辑)
- Modify: `packages/desktop/src/main.rs:141-158` (on_config_changed)

**Step 1: 启动时注册窗口循环热键**

在 `main()` 中 `RegisterAll` 发送之后（第 41 行之后）添加：

```rust
// 注册窗口循环热键
if initial_config.settings.window_cycle.enabled {
    let wc = &initial_config.settings.window_cycle;
    let _ = hotkey_cmd_tx.send(hotkey::HotkeyCommand::RegisterWindowCycle {
        modifier: wc.modifier.clone(),
        key: wc.key,
    });
}
```

**Step 2: 更新暂停恢复逻辑**

在 `TogglePause` 的恢复分支中（`else` 块，约第 112-121 行），在 `RegisterAll` 发送之后添加窗口循环重新注册：

```rust
} else {
    let cfg = config::load_config();
    if let Ok(tx) = channels.hotkey_tx.lock() {
        let _ = tx.send(hotkey::HotkeyCommand::RegisterAll(cfg.shortcuts));
        // 恢复窗口循环热键
        if cfg.settings.window_cycle.enabled {
            let wc = &cfg.settings.window_cycle;
            let _ = tx.send(hotkey::HotkeyCommand::RegisterWindowCycle {
                modifier: wc.modifier.clone(),
                key: wc.key,
            });
        }
    }
    // ... 托盘文字更新保持不变
}
```

**Step 3: 更新 on_config_changed**

在 `on_config_changed` 闭包中，现有 `RegisterAll` 发送之后添加窗口循环同步：

```rust
// 通知快捷键线程更新注册
if let Ok(tx) = channels.hotkey_tx.lock() {
    let _ = tx.send(hotkey::HotkeyCommand::RegisterAll(
        new_config.shortcuts.clone(),
    ));
    // 同步窗口循环热键
    let _ = tx.send(hotkey::HotkeyCommand::UnregisterWindowCycle);
    if new_config.settings.window_cycle.enabled {
        let wc = &new_config.settings.window_cycle;
        let _ = tx.send(hotkey::HotkeyCommand::RegisterWindowCycle {
            modifier: wc.modifier.clone(),
            key: wc.key,
        });
    }
}
```

**Step 4: 编译检查**

Run: `cargo check -p desktop`
Expected: 通过

**Step 5: 提交**

```bash
git add packages/desktop/src/main.rs
git commit -m "feat: main.rs 集成窗口循环热键注册和生命周期管理"
```

---

### Task 7: Settings 面板 UI

**Files:**
- Modify: `packages/desktop/src/views/home.rs:184-226` (设置面板区域)

**Step 1: 在设置面板中添加窗口循环配置区域**

在现有设置面板的 `div { class: "flex flex-wrap gap-x-8 gap-y-3", ... }` 闭合之后、外层 `div` 闭合之前，添加新的配置区域：

```rust
// ── 窗口循环切换设置 ──
div { class: "mt-4 pt-4 border-t border-border-subtle",
    div { class: "flex items-center gap-2 mb-3",
        span { class: "text-xs font-medium text-text-muted uppercase tracking-wide", "同应用窗口循环切换" }
    }
    div { class: "flex flex-wrap items-center gap-x-6 gap-y-3",
        // 启用开关
        label { class: "inline-flex items-center gap-2.5 text-sm text-text-secondary cursor-pointer select-none",
            input {
                r#type: "checkbox",
                checked: config().settings.window_cycle.enabled,
                onchange: move |_| {
                    let mut cfg = config();
                    cfg.settings.window_cycle.enabled = !cfg.settings.window_cycle.enabled;
                    save_and_notify(cfg);
                },
            }
            "启用"
        }
        // 修饰键选择
        label { class: "inline-flex items-center gap-2 text-sm text-text-secondary",
            span { "修饰键" }
            select {
                class: "px-2 py-1 bg-bg-input border border-border-default rounded text-sm text-text-primary cursor-pointer",
                value: config().settings.window_cycle.modifier.display_name(),
                onchange: move |e: Event<FormData>| {
                    let mut cfg = config();
                    cfg.settings.window_cycle.modifier = match e.value().as_str() {
                        "Ctrl" => ui::Modifier::Ctrl,
                        "Win" => ui::Modifier::Win,
                        _ => ui::Modifier::Alt,
                    };
                    save_and_notify(cfg);
                },
                option { value: "Alt", selected: config().settings.window_cycle.modifier == ui::Modifier::Alt, "Alt" }
                option { value: "Ctrl", selected: config().settings.window_cycle.modifier == ui::Modifier::Ctrl, "Ctrl" }
                option { value: "Win", selected: config().settings.window_cycle.modifier == ui::Modifier::Win, "Win" }
            }
        }
        // 按键输入
        label { class: "inline-flex items-center gap-2 text-sm text-text-secondary",
            span { "按键" }
            input {
                r#type: "text",
                class: "w-12 px-2 py-1 bg-bg-input border border-border-default rounded text-sm text-text-primary text-center",
                value: config().settings.window_cycle.key.to_string(),
                maxlength: 1,
                onchange: move |e: Event<FormData>| {
                    if let Some(ch) = e.value().chars().next() {
                        let mut cfg = config();
                        cfg.settings.window_cycle.key = ch;
                        save_and_notify(cfg);
                    }
                },
            }
        }
    }
    // 提示信息
    p { class: "mt-2 text-xs text-text-muted",
        {format!(
            "{}+{} 下一个窗口 / {}+Shift+{} 上一个窗口",
            config().settings.window_cycle.modifier.display_name(),
            config().settings.window_cycle.key,
            config().settings.window_cycle.modifier.display_name(),
            config().settings.window_cycle.key,
        )}
    }
}
```

**Step 2: 编译检查**

Run: `cargo check -p desktop`
Expected: 通过

**Step 3: 提交**

```bash
git add packages/desktop/src/views/home.rs
git commit -m "feat: Settings 面板添加窗口循环切换配置 UI"
```

---

### Task 8: 构建验证与手动测试

**Step 1: 运行全部测试**

Run: `cargo test -p desktop`
Expected: 所有测试通过

**Step 2: 构建**

Run: `cargo build -p desktop`
Expected: 构建成功

**Step 3: 手动测试清单**

1. 启动应用，打开设置面板，确认窗口循环切换配置区域显示正确
2. 打开多个相同应用的窗口（如多个文件资源管理器窗口）
3. 按 Alt+` 验证切换到下一个窗口
4. 按 Alt+Shift+` 验证切换到上一个窗口
5. 在设置中取消勾选"启用"，验证热键失效
6. 修改修饰键为 Ctrl，验证 Ctrl+` 生效
7. 暂停所有快捷键，验证窗口循环也暂停
8. 恢复快捷键，验证窗口循环恢复
9. 确认只有一个窗口时按热键无反应（不报错）

**Step 4: 提交最终验证通过**

```bash
git add -A
git commit -m "feat: 完成同应用多窗口循环切换功能"
```
