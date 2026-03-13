# Win Aide 快捷键启动器 - 实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将现有 Dioxus 多平台 demo 重构为 Windows 快捷键启动器工具，替代 AutoHotKey 脚本。

**Architecture:** Dioxus 0.7.1 Desktop 渲染 GUI 配置界面，后台线程通过 windows-rs 调用 Win32 API 实现全局快捷键注册（RegisterHotKey）和窗口启动/激活（EnumWindows + SetForegroundWindow）。系统托盘常驻通过 tray-icon crate 实现。配置以 JSON 文件持久化到用户目录。

**Tech Stack:** Rust, Dioxus 0.7.1 (desktop), windows-rs, tray-icon, rfd, serde/serde_json, uuid, dirs

**Design Doc:** `docs/plans/2026-03-13-hotkey-launcher-design.md`

---

## Task 1: 清理 Workspace 和删除多余平台包

**Files:**
- Modify: `Cargo.toml` (workspace root)
- Modify: `build-css.sh`
- Delete: `packages/web/` (entire directory)
- Delete: `packages/mobile/` (entire directory)
- Delete: `packages/api/` (entire directory)

**Step 1: 删除 web、mobile、api 包目录**

```bash
rm -rf packages/web packages/mobile packages/api
```

**Step 2: 更新 workspace Cargo.toml**

将 `Cargo.toml` 修改为：

```toml
[workspace]
resolver = "2"
members = [
    "packages/ui",
    "packages/desktop",
]

[workspace.dependencies]
dioxus = { version = "0.7.1" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# workspace
ui = { path = "packages/ui" }
```

**Step 3: 简化 build-css.sh**

```bash
#!/bin/bash
set -e

echo "Building TailwindCSS..."
npx @tailwindcss/cli -i tailwind.css -o packages/desktop/assets/tailwind.css --minify
echo "TailwindCSS build complete."
```

**Step 4: 验证 workspace 编译**

Run: `cargo check`
Expected: 编译通过（可能有 unused import 警告，后续任务会处理）

**Step 5: 提交**

```bash
git add -A
git commit -m "refactor: 清理 workspace，移除 web/mobile/api 包"
```

---

## Task 2: 添加项目依赖

**Files:**
- Modify: `packages/desktop/Cargo.toml`
- Modify: `packages/ui/Cargo.toml`

**Step 1: 更新 desktop Cargo.toml**

```toml
[package]
name = "desktop"
version = "0.1.0"
edition = "2021"

[dependencies]
dioxus = { workspace = true, features = ["router"] }
ui = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { version = "1", features = ["v4"] }
dirs = "6"
tray-icon = "0.19"
muda = "0.16"
rfd = "0.15"

[dependencies.windows]
version = "0.58"
features = [
    "Win32_UI_WindowsAndMessaging",
    "Win32_UI_Input_KeyboardAndMouse",
    "Win32_Foundation",
    "Win32_System_Threading",
    "Win32_System_ProcessStatus",
    "Win32_Security",
    "Win32_System_Registry",
]

[features]
default = ["desktop"]
desktop = ["dioxus/desktop"]
```

**Step 2: 更新 ui Cargo.toml**

```toml
[package]
name = "ui"
version = "0.1.0"
edition = "2021"

[dependencies]
dioxus = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
```

**Step 3: 验证依赖解析**

Run: `cargo check`
Expected: 依赖下载并编译通过

**Step 4: 提交**

```bash
git add packages/desktop/Cargo.toml packages/ui/Cargo.toml Cargo.toml
git commit -m "feat: 添加 windows-rs、tray-icon、rfd 等核心依赖"
```

---

## Task 3: 创建 config 模块（数据模型 + JSON 读写）

**Files:**
- Create: `packages/desktop/src/config.rs`
- Test file: 内嵌 `#[cfg(test)] mod tests`

**Step 1: 编写 config 模块测试**

在 `packages/desktop/src/config.rs` 中编写测试：

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AppConfig {
    pub version: u32,
    pub shortcuts: Vec<Shortcut>,
    pub settings: Settings,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Shortcut {
    pub id: String,
    pub name: String,
    pub exe_name: String,
    pub exe_path: String,
    pub modifier: Modifier,
    pub key: char,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Modifier {
    Alt,
    Ctrl,
    Win,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Settings {
    pub auto_start: bool,
    pub start_minimized: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            shortcuts: Vec::new(),
            settings: Settings {
                auto_start: false,
                start_minimized: true,
            },
        }
    }
}

impl Modifier {
    pub fn display_name(&self) -> &str {
        match self {
            Modifier::Alt => "Alt",
            Modifier::Ctrl => "Ctrl",
            Modifier::Win => "Win",
        }
    }

    pub fn all() -> &'static [Modifier] {
        &[Modifier::Alt, Modifier::Ctrl, Modifier::Win]
    }
}

/// 获取配置文件目录 ~/.win_aide/
pub fn config_dir() -> PathBuf {
    let home = dirs::home_dir().expect("无法获取用户目录");
    home.join(".win_aide")
}

/// 获取配置文件路径 ~/.win_aide/config.json
pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

/// 加载配置，文件不存在则返回默认配置并创建文件
pub fn load_config() -> AppConfig {
    let path = config_path();
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        let config = AppConfig::default();
        save_config(&config);
        config
    }
}

/// 保存配置到 JSON 文件
pub fn save_config(config: &AppConfig) {
    let dir = config_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir).expect("无法创建配置目录");
    }
    let path = config_path();
    let json = serde_json::to_string_pretty(config).expect("序列化配置失败");
    fs::write(&path, json).expect("写入配置文件失败");
}

/// 检查快捷键是否冲突（同一 modifier + key 组合）
pub fn has_conflict(shortcuts: &[Shortcut], modifier: &Modifier, key: char, exclude_id: Option<&str>) -> bool {
    shortcuts.iter().any(|s| {
        s.modifier == *modifier
            && s.key.to_ascii_uppercase() == key.to_ascii_uppercase()
            && exclude_id.map_or(true, |id| s.id != id)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn temp_config_path() -> PathBuf {
        env::temp_dir().join(format!("win_aide_test_{}", uuid::Uuid::new_v4()))
    }

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.version, 1);
        assert!(config.shortcuts.is_empty());
        assert!(!config.settings.auto_start);
        assert!(config.settings.start_minimized);
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = AppConfig {
            version: 1,
            shortcuts: vec![Shortcut {
                id: "test-id".to_string(),
                name: "Chrome".to_string(),
                exe_name: "chrome.exe".to_string(),
                exe_path: "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe".to_string(),
                modifier: Modifier::Alt,
                key: 'C',
                enabled: true,
            }],
            settings: Settings {
                auto_start: true,
                start_minimized: true,
            },
        };

        let json = serde_json::to_string_pretty(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_has_conflict() {
        let shortcuts = vec![
            Shortcut {
                id: "1".to_string(),
                name: "Chrome".to_string(),
                exe_name: "chrome.exe".to_string(),
                exe_path: "chrome.exe".to_string(),
                modifier: Modifier::Alt,
                key: 'C',
                enabled: true,
            },
        ];

        // 相同组合应冲突
        assert!(has_conflict(&shortcuts, &Modifier::Alt, 'C', None));
        // 大小写不敏感
        assert!(has_conflict(&shortcuts, &Modifier::Alt, 'c', None));
        // 不同修饰键不冲突
        assert!(!has_conflict(&shortcuts, &Modifier::Ctrl, 'C', None));
        // 不同字母不冲突
        assert!(!has_conflict(&shortcuts, &Modifier::Alt, 'V', None));
        // 排除自身不冲突
        assert!(!has_conflict(&shortcuts, &Modifier::Alt, 'C', Some("1")));
    }

    #[test]
    fn test_modifier_display() {
        assert_eq!(Modifier::Alt.display_name(), "Alt");
        assert_eq!(Modifier::Ctrl.display_name(), "Ctrl");
        assert_eq!(Modifier::Win.display_name(), "Win");
    }
}
```

**Step 2: 在 main.rs 中注册模块**

在 `packages/desktop/src/main.rs` 顶部添加：
```rust
mod config;
```

**Step 3: 运行测试验证**

Run: `cargo test -p desktop`
Expected: 全部通过

**Step 4: 提交**

```bash
git add packages/desktop/src/config.rs packages/desktop/src/main.rs
git commit -m "feat: 创建 config 模块，实现数据模型和 JSON 配置读写"
```

---

## Task 4: 创建 hotkey 模块（Win32 全局快捷键注册）

**Files:**
- Create: `packages/desktop/src/hotkey.rs`

**Step 1: 实现 hotkey 模块**

```rust
use crate::config::{Modifier, Shortcut};
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_WIN,
    HOT_KEY_MODIFIERS,
};
use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, WM_HOTKEY};

/// 快捷键触发事件
#[derive(Debug, Clone)]
pub struct HotkeyEvent {
    pub shortcut_id: String,
}

/// 发送给快捷键线程的指令
pub enum HotkeyCommand {
    /// 注册一组快捷键（会先注销所有旧的）
    RegisterAll(Vec<Shortcut>),
    /// 停止监听并退出线程
    Shutdown,
}

fn modifier_to_win32(modifier: &Modifier) -> HOT_KEY_MODIFIERS {
    match modifier {
        Modifier::Alt => MOD_ALT | MOD_NOREPEAT,
        Modifier::Ctrl => MOD_CONTROL | MOD_NOREPEAT,
        Modifier::Win => MOD_WIN | MOD_NOREPEAT,
    }
}

fn key_to_vk(key: char) -> u32 {
    key.to_ascii_uppercase() as u32
}

/// 启动快捷键监听线程
/// 返回 (command_sender, event_receiver)
pub fn start_hotkey_listener() -> (mpsc::Sender<HotkeyCommand>, mpsc::Receiver<HotkeyEvent>) {
    let (cmd_tx, cmd_rx) = mpsc::channel::<HotkeyCommand>();
    let (event_tx, event_rx) = mpsc::channel::<HotkeyEvent>();

    thread::spawn(move || {
        let mut registered_ids: HashMap<i32, String> = HashMap::new();
        let mut next_id: i32 = 1;

        loop {
            // 检查是否有新指令（非阻塞）
            if let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    HotkeyCommand::RegisterAll(shortcuts) => {
                        // 先注销所有旧快捷键
                        for &id in registered_ids.keys() {
                            unsafe {
                                let _ = UnregisterHotKey(HWND::default(), id);
                            }
                        }
                        registered_ids.clear();
                        next_id = 1;

                        // 注册新快捷键
                        for shortcut in &shortcuts {
                            if !shortcut.enabled {
                                continue;
                            }
                            let id = next_id;
                            next_id += 1;
                            let modifiers = modifier_to_win32(&shortcut.modifier);
                            let vk = key_to_vk(shortcut.key);

                            unsafe {
                                if RegisterHotKey(HWND::default(), id, modifiers, vk).is_ok() {
                                    registered_ids.insert(id, shortcut.id.clone());
                                }
                            }
                        }
                    }
                    HotkeyCommand::Shutdown => {
                        // 注销所有快捷键
                        for &id in registered_ids.keys() {
                            unsafe {
                                let _ = UnregisterHotKey(HWND::default(), id);
                            }
                        }
                        break;
                    }
                }
            }

            // 检查快捷键消息（带超时，避免阻塞检查指令）
            let mut msg = MSG::default();
            unsafe {
                // PeekMessageW 非阻塞检查消息
                if windows::Win32::UI::WindowsAndMessaging::PeekMessageW(
                    &mut msg,
                    HWND::default(),
                    WM_HOTKEY,
                    WM_HOTKEY,
                    windows::Win32::UI::WindowsAndMessaging::PM_REMOVE,
                )
                .as_bool()
                {
                    if msg.message == WM_HOTKEY {
                        let hotkey_id = msg.wParam.0 as i32;
                        if let Some(shortcut_id) = registered_ids.get(&hotkey_id) {
                            let _ = event_tx.send(HotkeyEvent {
                                shortcut_id: shortcut_id.clone(),
                            });
                        }
                    }
                }
            }

            // 短暂休眠避免 CPU 空转
            thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    (cmd_tx, event_rx)
}
```

**Step 2: 在 main.rs 中注册模块**

```rust
mod hotkey;
```

**Step 3: 验证编译**

Run: `cargo check -p desktop`
Expected: 编译通过

**Step 4: 提交**

```bash
git add packages/desktop/src/hotkey.rs packages/desktop/src/main.rs
git commit -m "feat: 创建 hotkey 模块，实现 Win32 全局快捷键注册和监听"
```

---

## Task 5: 创建 launcher 模块（启动/激活窗口）

**Files:**
- Create: `packages/desktop/src/launcher.rs`

**Step 1: 实现 launcher 模块**

```rust
use crate::config::Shortcut;
use crate::hotkey::HotkeyEvent;
use std::collections::HashMap;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::sync::mpsc;
use std::thread;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, TRUE};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AllowSetForegroundWindow, EnumWindows, GetWindowThreadProcessId, IsIconic, IsWindowVisible,
    SetForegroundWindow, ShowWindow, SW_RESTORE,
};

struct FindWindowData {
    exe_name: String,
    found_hwnd: Option<HWND>,
}

/// EnumWindows 回调：查找匹配 exe_name 的可见窗口
unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data = &mut *(lparam.0 as *mut FindWindowData);

    // 跳过不可见窗口
    if !IsWindowVisible(hwnd).as_bool() {
        return TRUE;
    }

    let mut process_id: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut process_id));

    if process_id == 0 {
        return TRUE;
    }

    // 打开进程获取可执行文件路径
    if let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) {
        let mut buffer = [0u16; 1024];
        let mut size = buffer.len() as u32;
        if QueryFullProcessImageNameW(process, PROCESS_NAME_FORMAT(0), &mut buffer, &mut size)
            .is_ok()
        {
            let path = OsString::from_wide(&buffer[..size as usize]);
            let path_str = path.to_string_lossy().to_lowercase();
            if path_str.ends_with(&data.exe_name.to_lowercase()) {
                data.found_hwnd = Some(hwnd);
                return BOOL(0); // 停止枚举
            }
        }
    }

    TRUE
}

/// 查找运行中的窗口
fn find_window_by_exe(exe_name: &str) -> Option<HWND> {
    let mut data = FindWindowData {
        exe_name: exe_name.to_string(),
        found_hwnd: None,
    };

    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut data as *mut FindWindowData as isize),
        );
    }

    data.found_hwnd
}

/// 激活已有窗口
fn activate_window(hwnd: HWND) {
    unsafe {
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        let _ = AllowSetForegroundWindow(windows::Win32::UI::WindowsAndMessaging::ASFW_ANY);
        let _ = SetForegroundWindow(hwnd);
    }
}

/// 启动新进程
fn launch_process(exe_path: &str) {
    use std::process::Command;
    let _ = Command::new(exe_path).spawn();
}

/// LaunchOrActivate：如果应用已运行则激活窗口，否则启动
pub fn launch_or_activate(shortcut: &Shortcut) {
    if let Some(hwnd) = find_window_by_exe(&shortcut.exe_name) {
        activate_window(hwnd);
    } else {
        launch_process(&shortcut.exe_path);
    }
}

/// 启动 launcher 处理线程
/// 监听 hotkey 事件，根据配置执行 launch_or_activate
pub fn start_launcher(
    event_rx: mpsc::Receiver<HotkeyEvent>,
    shortcuts: Vec<Shortcut>,
) -> mpsc::Sender<Vec<Shortcut>> {
    let (update_tx, update_rx) = mpsc::channel::<Vec<Shortcut>>();

    thread::spawn(move || {
        let mut shortcut_map: HashMap<String, Shortcut> = shortcuts
            .into_iter()
            .map(|s| (s.id.clone(), s))
            .collect();

        loop {
            // 检查配置更新
            if let Ok(new_shortcuts) = update_rx.try_recv() {
                shortcut_map = new_shortcuts
                    .into_iter()
                    .map(|s| (s.id.clone(), s))
                    .collect();
            }

            // 检查快捷键事件
            if let Ok(event) = event_rx.recv_timeout(std::time::Duration::from_millis(50)) {
                if let Some(shortcut) = shortcut_map.get(&event.shortcut_id) {
                    launch_or_activate(shortcut);
                }
            }
        }
    });

    update_tx
}
```

**Step 2: 在 main.rs 中注册模块**

```rust
mod launcher;
```

**Step 3: 验证编译**

Run: `cargo check -p desktop`
Expected: 编译通过

**Step 4: 提交**

```bash
git add packages/desktop/src/launcher.rs packages/desktop/src/main.rs
git commit -m "feat: 创建 launcher 模块，实现 LaunchOrActivate 窗口查找和激活"
```

---

## Task 6: 创建 tray 模块（系统托盘）

**Files:**
- Create: `packages/desktop/src/tray.rs`

**Step 1: 实现 tray 模块**

```rust
use muda::{Menu, MenuItem, PredefinedMenuItem};
use tray_icon::{
    menu::MenuEvent, Icon, TrayIcon, TrayIconBuilder,
};

/// 托盘菜单项 ID
pub const MENU_SHOW: &str = "show";
pub const MENU_PAUSE: &str = "pause";
pub const MENU_QUIT: &str = "quit";

/// 托盘事件
#[derive(Debug, Clone, PartialEq)]
pub enum TrayEvent {
    Show,
    TogglePause,
    Quit,
}

/// 创建默认图标（简单的彩色方块）
fn create_default_icon() -> Icon {
    // 16x16 RGBA 图标（蓝紫色，与主题 accent 色一致）
    let size = 16u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for _ in 0..size * size {
        rgba.push(0x91); // R
        rgba.push(0xa4); // G
        rgba.push(0xd2); // B
        rgba.push(0xFF); // A
    }
    Icon::from_rgba(rgba, size, size).expect("无法创建托盘图标")
}

/// 初始化系统托盘
/// 返回 TrayIcon 实例（必须保持存活否则托盘图标会消失）
pub fn create_tray() -> TrayIcon {
    let menu = Menu::new();

    let show_item = MenuItem::with_id(MENU_SHOW, "显示主窗口", true, None);
    let pause_item = MenuItem::with_id(MENU_PAUSE, "暂停所有快捷键", true, None);
    let quit_item = MenuItem::with_id(MENU_QUIT, "退出", true, None);

    let _ = menu.append(&show_item);
    let _ = menu.append(&PredefinedMenuItem::separator());
    let _ = menu.append(&pause_item);
    let _ = menu.append(&PredefinedMenuItem::separator());
    let _ = menu.append(&quit_item);

    TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Win Aide - 快捷键启动器")
        .with_icon(create_default_icon())
        .build()
        .expect("无法创建系统托盘")
}

/// 轮询托盘菜单事件（非阻塞）
pub fn poll_tray_event() -> Option<TrayEvent> {
    if let Ok(event) = MenuEvent::receiver().try_recv() {
        match event.id.0.as_str() {
            MENU_SHOW => Some(TrayEvent::Show),
            MENU_PAUSE => Some(TrayEvent::TogglePause),
            MENU_QUIT => Some(TrayEvent::Quit),
            _ => None,
        }
    } else {
        None
    }
}
```

**Step 2: 在 main.rs 中注册模块**

```rust
mod tray;
```

**Step 3: 验证编译**

Run: `cargo check -p desktop`
Expected: 编译通过

**Step 4: 提交**

```bash
git add packages/desktop/src/tray.rs packages/desktop/src/main.rs
git commit -m "feat: 创建 tray 模块，实现系统托盘图标和右键菜单"
```

---

## Task 7: 重构 UI 组件库 — 移除旧组件，创建 ShortcutList

**Files:**
- Delete: `packages/ui/src/hero.rs`
- Delete: `packages/ui/assets/header.svg`
- Modify: `packages/ui/src/lib.rs`
- Modify: `packages/ui/src/navbar.rs`
- Create: `packages/ui/src/shortcut_list.rs`

**Step 1: 删除旧组件文件**

```bash
rm -f packages/ui/src/hero.rs packages/ui/assets/header.svg
```

**Step 2: 创建 shortcut_list.rs**

```rust
use dioxus::prelude::*;

use crate::Modifier;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct ShortcutRow {
    pub id: String,
    pub name: String,
    pub exe_name: String,
    pub exe_path: String,
    pub modifier: String,
    pub key: char,
    pub enabled: bool,
}

#[component]
pub fn ShortcutList(
    shortcuts: Vec<ShortcutRow>,
    on_toggle: EventHandler<String>,
    on_edit: EventHandler<String>,
    on_delete: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "w-full",
            // 表头
            div { class: "grid grid-cols-[50px_120px_1fr_1fr_100px] gap-2 px-4 py-2 text-sm text-gray-400 border-b border-gray-700",
                span { "启用" }
                span { "快捷键" }
                span { "应用名称" }
                span { "路径" }
                span { "操作" }
            }

            // 数据行
            for shortcut in shortcuts.iter() {
                {
                    let id_toggle = shortcut.id.clone();
                    let id_edit = shortcut.id.clone();
                    let id_delete = shortcut.id.clone();
                    rsx! {
                        div {
                            class: "grid grid-cols-[50px_120px_1fr_1fr_100px] gap-2 px-4 py-3 items-center border-b border-gray-800 hover:bg-gray-800/50 transition-colors",
                            // 启用复选框
                            div {
                                input {
                                    r#type: "checkbox",
                                    checked: shortcut.enabled,
                                    class: "w-4 h-4 cursor-pointer accent-accent",
                                    onchange: move |_| on_toggle.call(id_toggle.clone()),
                                }
                            }
                            // 快捷键
                            span { class: "text-accent font-mono text-sm",
                                "{shortcut.modifier} + {shortcut.key}"
                            }
                            // 应用名称
                            span { class: "text-white truncate", "{shortcut.name}" }
                            // 路径
                            span { class: "text-gray-400 text-sm truncate", "{shortcut.exe_path}" }
                            // 操作按钮
                            div { class: "flex gap-2",
                                button {
                                    class: "px-2 py-1 text-sm text-gray-300 hover:text-white hover:bg-gray-700 rounded transition-colors cursor-pointer",
                                    onclick: move |_| on_edit.call(id_edit.clone()),
                                    "编辑"
                                }
                                button {
                                    class: "px-2 py-1 text-sm text-red-400 hover:text-red-300 hover:bg-red-900/30 rounded transition-colors cursor-pointer",
                                    onclick: move |_| on_delete.call(id_delete.clone()),
                                    "删除"
                                }
                            }
                        }
                    }
                }
            }

            // 空状态
            if shortcuts.is_empty() {
                div { class: "text-center text-gray-500 py-12",
                    p { class: "text-lg mb-2", "暂无快捷键配置" }
                    p { class: "text-sm", "点击上方「添加快捷键」开始配置" }
                }
            }
        }
    }
}
```

**Step 3: 更新 navbar.rs（简化为工具栏）**

```rust
use dioxus::prelude::*;

#[component]
pub fn Navbar(children: Element) -> Element {
    rsx! {
        div { class: "flex items-center justify-between px-4 py-3 border-b border-gray-700",
            {children}
        }
    }
}
```

**Step 4: 更新 lib.rs**

```rust
//! 跨平台共享 UI 组件库

// 从 desktop config 模块复用的类型（在 UI 层面只需要 Modifier 的显示名）
pub use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Modifier {
    Alt,
    Ctrl,
    Win,
}

impl Modifier {
    pub fn display_name(&self) -> &str {
        match self {
            Modifier::Alt => "Alt",
            Modifier::Ctrl => "Ctrl",
            Modifier::Win => "Win",
        }
    }
}

mod navbar;
pub use navbar::Navbar;

mod shortcut_list;
pub use shortcut_list::{ShortcutList, ShortcutRow};
```

**Step 5: 验证编译**

Run: `cargo check -p ui`
Expected: 编译通过

**Step 6: 提交**

```bash
git add -A
git commit -m "refactor: 重构 UI 组件库，移除 Hero，创建 ShortcutList 表格组件"
```

---

## Task 8: 创建 ShortcutForm 组件（新增/编辑弹窗）

**Files:**
- Create: `packages/ui/src/shortcut_form.rs`
- Modify: `packages/ui/src/lib.rs`

**Step 1: 实现 shortcut_form.rs**

```rust
use dioxus::prelude::*;

use crate::Modifier;

#[derive(Debug, Clone, PartialEq)]
pub struct ShortcutFormData {
    pub id: Option<String>,
    pub name: String,
    pub exe_name: String,
    pub exe_path: String,
    pub modifier: Modifier,
    pub key: String,
}

impl Default for ShortcutFormData {
    fn default() -> Self {
        Self {
            id: None,
            name: String::new(),
            exe_name: String::new(),
            exe_path: String::new(),
            modifier: Modifier::Alt,
            key: String::new(),
        }
    }
}

#[component]
pub fn ShortcutForm(
    initial: ShortcutFormData,
    conflict_message: Option<String>,
    on_save: EventHandler<ShortcutFormData>,
    on_cancel: EventHandler<()>,
    on_browse: EventHandler<()>,
) -> Element {
    let mut name = use_signal(|| initial.name.clone());
    let mut exe_name = use_signal(|| initial.exe_name.clone());
    let mut exe_path = use_signal(|| initial.exe_path.clone());
    let mut modifier = use_signal(|| initial.modifier.clone());
    let mut key = use_signal(|| initial.key.clone());

    let title = if initial.id.is_some() { "编辑快捷键" } else { "添加快捷键" };
    let initial_id = initial.id.clone();

    rsx! {
        // 遮罩层
        div {
            class: "fixed inset-0 bg-black/60 flex items-center justify-center z-50",
            onclick: move |_| on_cancel.call(()),

            // 弹窗
            div {
                class: "bg-bg-card rounded-lg p-6 w-[450px] shadow-xl",
                onclick: move |e| e.stop_propagation(),

                h2 { class: "text-xl font-semibold text-white mb-6", "{title}" }

                // 应用名称
                div { class: "mb-4",
                    label { class: "block text-sm text-gray-400 mb-1", "应用名称" }
                    input {
                        r#type: "text",
                        class: "w-full bg-bg-primary border border-gray-700 rounded px-3 py-2 text-white focus:border-accent-focus focus:outline-none",
                        placeholder: "例如：Chrome",
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                    }
                }

                // 进程名
                div { class: "mb-4",
                    label { class: "block text-sm text-gray-400 mb-1", "进程名" }
                    input {
                        r#type: "text",
                        class: "w-full bg-bg-primary border border-gray-700 rounded px-3 py-2 text-white focus:border-accent-focus focus:outline-none",
                        placeholder: "例如：chrome.exe",
                        value: "{exe_name}",
                        oninput: move |e| exe_name.set(e.value()),
                    }
                }

                // 路径
                div { class: "mb-4",
                    label { class: "block text-sm text-gray-400 mb-1", "可执行文件路径" }
                    div { class: "flex gap-2",
                        input {
                            r#type: "text",
                            class: "flex-1 bg-bg-primary border border-gray-700 rounded px-3 py-2 text-white focus:border-accent-focus focus:outline-none text-sm",
                            placeholder: "C:\\Program Files\\...",
                            value: "{exe_path}",
                            oninput: move |e| exe_path.set(e.value()),
                        }
                        button {
                            class: "px-3 py-2 bg-gray-700 text-white rounded hover:bg-gray-600 transition-colors cursor-pointer text-sm",
                            onclick: move |_| on_browse.call(()),
                            "浏览..."
                        }
                    }
                }

                // 快捷键
                div { class: "mb-6",
                    label { class: "block text-sm text-gray-400 mb-1", "快捷键" }
                    div { class: "flex gap-2 items-center",
                        select {
                            class: "bg-bg-primary border border-gray-700 rounded px-3 py-2 text-white focus:border-accent-focus focus:outline-none cursor-pointer",
                            value: "{modifier().display_name()}",
                            onchange: move |e| {
                                let val = e.value();
                                modifier.set(match val.as_str() {
                                    "Ctrl" => Modifier::Ctrl,
                                    "Win" => Modifier::Win,
                                    _ => Modifier::Alt,
                                });
                            },
                            option { value: "Alt", "Alt" }
                            option { value: "Ctrl", "Ctrl" }
                            option { value: "Win", "Win" }
                        }
                        span { class: "text-gray-400 text-lg", "+" }
                        input {
                            r#type: "text",
                            class: "w-16 bg-bg-primary border border-gray-700 rounded px-3 py-2 text-white text-center uppercase focus:border-accent-focus focus:outline-none",
                            placeholder: "A",
                            maxlength: 1,
                            value: "{key}",
                            oninput: move |e| {
                                let val = e.value();
                                if let Some(c) = val.chars().last() {
                                    if c.is_ascii_alphabetic() {
                                        key.set(c.to_ascii_uppercase().to_string());
                                    }
                                } else {
                                    key.set(String::new());
                                }
                            },
                        }
                    }
                }

                // 冲突提示
                if let Some(msg) = &conflict_message {
                    div { class: "mb-4 p-3 bg-red-900/30 border border-red-700 rounded text-red-300 text-sm",
                        "{msg}"
                    }
                }

                // 按钮
                div { class: "flex justify-end gap-3",
                    button {
                        class: "px-4 py-2 text-gray-300 hover:text-white transition-colors cursor-pointer",
                        onclick: move |_| on_cancel.call(()),
                        "取消"
                    }
                    button {
                        class: "px-4 py-2 bg-accent text-white rounded hover:bg-accent-focus transition-colors cursor-pointer",
                        onclick: {
                            let initial_id = initial_id.clone();
                            move |_| {
                                on_save.call(ShortcutFormData {
                                    id: initial_id.clone(),
                                    name: name(),
                                    exe_name: exe_name(),
                                    exe_path: exe_path(),
                                    modifier: modifier(),
                                    key: key(),
                                });
                            }
                        },
                        "保存"
                    }
                }
            }
        }
    }
}
```

**Step 2: 更新 lib.rs 导出**

在 `packages/ui/src/lib.rs` 末尾添加：

```rust
mod shortcut_form;
pub use shortcut_form::{ShortcutForm, ShortcutFormData};
```

**Step 3: 验证编译**

Run: `cargo check -p ui`
Expected: 编译通过

**Step 4: 提交**

```bash
git add packages/ui/src/shortcut_form.rs packages/ui/src/lib.rs
git commit -m "feat: 创建 ShortcutForm 弹窗组件，支持新增/编辑快捷键"
```

---

## Task 9: 重构 desktop 主入口和 Home 视图

**Files:**
- Rewrite: `packages/desktop/src/main.rs`
- Rewrite: `packages/desktop/src/views/home.rs`
- Modify: `packages/desktop/src/views/mod.rs`
- Delete: `packages/desktop/src/views/blog.rs`

**注意：** 此任务中 `config.rs` 中的 `Modifier` 类型需要与 `ui` 包中的 `Modifier` 统一。将 `config.rs` 中的 `Modifier` 改为复用 `ui::Modifier`。

**Step 1: 更新 config.rs，复用 ui::Modifier**

删除 `config.rs` 中的 `Modifier` enum 定义和 `display_name`/`all` 方法，改为：

```rust
pub use ui::Modifier;
```

同时确保 `config.rs` 顶部的 import 中没有重复的 `Modifier` 定义。

**Step 2: 删除 blog.rs**

```bash
rm packages/desktop/src/views/blog.rs
```

**Step 3: 更新 views/mod.rs**

```rust
mod home;
pub use home::Home;
```

**Step 4: 重写 views/home.rs**

```rust
use dioxus::prelude::*;
use ui::{Navbar, ShortcutForm, ShortcutFormData, ShortcutList, ShortcutRow};

use crate::config::{self, AppConfig, Modifier, Shortcut};

#[component]
pub fn Home(
    config: Signal<AppConfig>,
    on_config_changed: EventHandler<AppConfig>,
) -> Element {
    let mut show_form = use_signal(|| false);
    let mut editing_id = use_signal(|| None::<String>);
    let mut conflict_msg = use_signal(|| None::<String>);
    let mut form_data = use_signal(ShortcutFormData::default);
    let mut show_settings = use_signal(|| false);
    let mut delete_confirm = use_signal(|| None::<String>);

    // 将 config shortcuts 转换为 UI 行
    let rows: Vec<ShortcutRow> = config()
        .shortcuts
        .iter()
        .map(|s| ShortcutRow {
            id: s.id.clone(),
            name: s.name.clone(),
            exe_name: s.exe_name.clone(),
            exe_path: s.exe_path.clone(),
            modifier: s.modifier.display_name().to_string(),
            key: s.key,
            enabled: s.enabled,
        })
        .collect();

    let save_and_notify = move |new_config: AppConfig| {
        config::save_config(&new_config);
        on_config_changed.call(new_config);
    };

    rsx! {
        div { class: "flex flex-col h-screen",
            // 工具栏
            Navbar {
                div { class: "flex items-center gap-2",
                    h1 { class: "text-lg font-semibold text-white", "Win Aide" }
                    span { class: "text-sm text-gray-400", "快捷键启动器" }
                }
                div { class: "flex gap-2",
                    button {
                        class: "px-3 py-1.5 bg-accent text-white rounded hover:bg-accent-focus transition-colors cursor-pointer text-sm",
                        onclick: move |_| {
                            form_data.set(ShortcutFormData::default());
                            editing_id.set(None);
                            conflict_msg.set(None);
                            show_form.set(true);
                        },
                        "+ 添加快捷键"
                    }
                    button {
                        class: "px-3 py-1.5 text-gray-300 hover:text-white border border-gray-700 rounded hover:bg-gray-700 transition-colors cursor-pointer text-sm",
                        onclick: move |_| show_settings.toggle(),
                        "设置"
                    }
                }
            }

            // 设置面板（展开/折叠）
            if show_settings() {
                div { class: "px-4 py-3 bg-bg-card border-b border-gray-700",
                    h3 { class: "text-sm font-semibold text-gray-300 mb-3", "设置" }
                    div { class: "flex gap-6",
                        label { class: "flex items-center gap-2 text-sm text-gray-300 cursor-pointer",
                            input {
                                r#type: "checkbox",
                                checked: config().settings.auto_start,
                                class: "w-4 h-4 accent-accent",
                                onchange: move |_| {
                                    let mut cfg = config();
                                    cfg.settings.auto_start = !cfg.settings.auto_start;
                                    save_and_notify(cfg);
                                },
                            }
                            "开机自启"
                        }
                        label { class: "flex items-center gap-2 text-sm text-gray-300 cursor-pointer",
                            input {
                                r#type: "checkbox",
                                checked: config().settings.start_minimized,
                                class: "w-4 h-4 accent-accent",
                                onchange: move |_| {
                                    let mut cfg = config();
                                    cfg.settings.start_minimized = !cfg.settings.start_minimized;
                                    save_and_notify(cfg);
                                },
                            }
                            "启动时最小化到托盘"
                        }
                    }
                }
            }

            // 快捷键列表
            div { class: "flex-1 overflow-y-auto",
                ShortcutList {
                    shortcuts: rows,
                    on_toggle: move |id: String| {
                        let mut cfg = config();
                        if let Some(s) = cfg.shortcuts.iter_mut().find(|s| s.id == id) {
                            s.enabled = !s.enabled;
                        }
                        save_and_notify(cfg);
                    },
                    on_edit: move |id: String| {
                        let cfg = config();
                        if let Some(s) = cfg.shortcuts.iter().find(|s| s.id == id) {
                            form_data.set(ShortcutFormData {
                                id: Some(s.id.clone()),
                                name: s.name.clone(),
                                exe_name: s.exe_name.clone(),
                                exe_path: s.exe_path.clone(),
                                modifier: s.modifier.clone(),
                                key: s.key.to_string(),
                            });
                            editing_id.set(Some(s.id.clone()));
                            conflict_msg.set(None);
                            show_form.set(true);
                        }
                    },
                    on_delete: move |id: String| {
                        delete_confirm.set(Some(id));
                    },
                }
            }

            // 删除确认对话框
            if let Some(del_id) = delete_confirm() {
                div {
                    class: "fixed inset-0 bg-black/60 flex items-center justify-center z-50",
                    div { class: "bg-bg-card rounded-lg p-6 w-[350px] shadow-xl",
                        h3 { class: "text-lg text-white mb-4", "确认删除" }
                        p { class: "text-gray-300 text-sm mb-6", "确定要删除这个快捷键配置吗？此操作不可撤销。" }
                        div { class: "flex justify-end gap-3",
                            button {
                                class: "px-4 py-2 text-gray-300 hover:text-white transition-colors cursor-pointer",
                                onclick: move |_| delete_confirm.set(None),
                                "取消"
                            }
                            button {
                                class: "px-4 py-2 bg-red-600 text-white rounded hover:bg-red-500 transition-colors cursor-pointer",
                                onclick: move |_| {
                                    let mut cfg = config();
                                    cfg.shortcuts.retain(|s| s.id != del_id);
                                    save_and_notify(cfg);
                                    delete_confirm.set(None);
                                },
                                "删除"
                            }
                        }
                    }
                }
            }

            // 新增/编辑弹窗
            if show_form() {
                ShortcutForm {
                    initial: form_data(),
                    conflict_message: conflict_msg(),
                    on_save: move |data: ShortcutFormData| {
                        // 校验
                        if data.name.is_empty() || data.exe_path.is_empty() || data.key.is_empty() {
                            conflict_msg.set(Some("请填写所有必填字段".to_string()));
                            return;
                        }
                        let key_char = data.key.chars().next().unwrap();

                        // 冲突检测
                        let cfg = config();
                        if config::has_conflict(&cfg.shortcuts, &data.modifier, key_char, data.id.as_deref()) {
                            conflict_msg.set(Some(format!("快捷键 {} + {} 已被占用", data.modifier.display_name(), key_char)));
                            return;
                        }

                        let mut cfg = config();
                        if let Some(edit_id) = &data.id {
                            // 编辑已有
                            if let Some(s) = cfg.shortcuts.iter_mut().find(|s| &s.id == edit_id) {
                                s.name = data.name;
                                s.exe_name = data.exe_name;
                                s.exe_path = data.exe_path;
                                s.modifier = data.modifier;
                                s.key = key_char;
                            }
                        } else {
                            // 新增
                            cfg.shortcuts.push(Shortcut {
                                id: uuid::Uuid::new_v4().to_string(),
                                name: data.name,
                                exe_name: data.exe_name,
                                exe_path: data.exe_path,
                                modifier: data.modifier,
                                key: key_char,
                                enabled: true,
                            });
                        }
                        save_and_notify(cfg);
                        show_form.set(false);
                    },
                    on_cancel: move |_| show_form.set(false),
                    on_browse: move |_| {
                        // 使用 rfd 文件选择器
                        let file = rfd::FileDialog::new()
                            .add_filter("可执行文件", &["exe"])
                            .pick_file();
                        if let Some(path) = file {
                            let path_str = path.to_string_lossy().to_string();
                            // 从路径提取文件名作为 exe_name
                            let file_name = path.file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default();
                            form_data.with_mut(|d| {
                                d.exe_path = path_str;
                                if d.exe_name.is_empty() {
                                    d.exe_name = file_name.clone();
                                }
                                if d.name.is_empty() {
                                    // 去掉 .exe 后缀作为默认名称
                                    d.name = file_name.trim_end_matches(".exe").to_string();
                                }
                            });
                        }
                    },
                }
            }
        }
    }
}
```

**Step 5: 重写 main.rs**

```rust
use dioxus::prelude::*;

mod config;
mod hotkey;
mod launcher;
mod tray;
mod views;

use config::AppConfig;
use views::Home;

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    // 加载配置
    let initial_config = config::load_config();

    // 启动快捷键监听
    let (hotkey_cmd_tx, hotkey_event_rx) = hotkey::start_hotkey_listener();

    // 注册初始快捷键
    let _ = hotkey_cmd_tx.send(hotkey::HotkeyCommand::RegisterAll(
        initial_config.shortcuts.clone(),
    ));

    // 启动 launcher
    let launcher_update_tx = launcher::start_launcher(
        hotkey_event_rx,
        initial_config.shortcuts.clone(),
    );

    // 创建系统托盘
    let _tray = tray::create_tray();

    // 启动 Dioxus Desktop
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let config = use_signal(|| config::load_config());

    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        div { class: "bg-bg-primary text-white font-sans min-h-screen",
            Home {
                config: config,
                on_config_changed: move |new_config: AppConfig| {
                    config.set(new_config);
                    // TODO: 通知 hotkey 和 launcher 线程更新
                },
            }
        }
    }
}
```

**Step 6: 验证编译**

Run: `cargo check -p desktop`
Expected: 编译通过（可能有 unused variable 警告，如 `hotkey_cmd_tx`、`launcher_update_tx`，后续会接入）

**Step 7: 提交**

```bash
git add -A
git commit -m "feat: 重构 desktop 主入口和 Home 视图，实现快捷键管理界面"
```

---

## Task 10: 接入快捷键和 launcher 线程通信

**Files:**
- Modify: `packages/desktop/src/main.rs`

**Step 1: 用 use_context 共享线程通信 sender**

重写 `main.rs`，将 `hotkey_cmd_tx` 和 `launcher_update_tx` 通过 `use_context_provider` 传入组件树：

```rust
use dioxus::prelude::*;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

mod config;
mod hotkey;
mod launcher;
mod tray;
mod views;

use config::AppConfig;
use views::Home;

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

/// 通过 context 共享的后台通信通道
#[derive(Clone)]
pub struct BackendChannels {
    pub hotkey_tx: Arc<Mutex<Sender<hotkey::HotkeyCommand>>>,
    pub launcher_tx: Arc<Mutex<Sender<Vec<config::Shortcut>>>>,
}

fn main() {
    let initial_config = config::load_config();

    let (hotkey_cmd_tx, hotkey_event_rx) = hotkey::start_hotkey_listener();
    let _ = hotkey_cmd_tx.send(hotkey::HotkeyCommand::RegisterAll(
        initial_config.shortcuts.clone(),
    ));

    let launcher_update_tx = launcher::start_launcher(
        hotkey_event_rx,
        initial_config.shortcuts.clone(),
    );

    let _tray = tray::create_tray();

    // 用静态变量传递 channels 给 Dioxus
    // (dioxus::launch 不接受闭包参数，需要通过全局状态)
    BACKEND_CHANNELS
        .lock()
        .unwrap()
        .replace(BackendChannels {
            hotkey_tx: Arc::new(Mutex::new(hotkey_cmd_tx)),
            launcher_tx: Arc::new(Mutex::new(launcher_update_tx)),
        });

    dioxus::launch(App);
}

static BACKEND_CHANNELS: std::sync::Mutex<Option<BackendChannels>> = std::sync::Mutex::new(None);

#[component]
fn App() -> Element {
    let channels = use_context_provider(|| {
        BACKEND_CHANNELS.lock().unwrap().take().expect("BackendChannels 未初始化")
    });

    let config = use_signal(|| config::load_config());

    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        div { class: "bg-bg-primary text-white font-sans min-h-screen",
            Home {
                config: config,
                on_config_changed: move |new_config: AppConfig| {
                    config.set(new_config.clone());

                    // 通知快捷键线程更新注册
                    if let Ok(tx) = channels.hotkey_tx.lock() {
                        let _ = tx.send(hotkey::HotkeyCommand::RegisterAll(
                            new_config.shortcuts.clone(),
                        ));
                    }

                    // 通知 launcher 线程更新配置
                    if let Ok(tx) = channels.launcher_tx.lock() {
                        let _ = tx.send(new_config.shortcuts);
                    }
                },
            }
        }
    }
}
```

**Step 2: 验证编译**

Run: `cargo check -p desktop`
Expected: 编译通过

**Step 3: 提交**

```bash
git add packages/desktop/src/main.rs
git commit -m "feat: 接入 hotkey 和 launcher 线程通信，配置变更实时生效"
```

---

## Task 11: 实现开机自启（注册表操作）

**Files:**
- Create: `packages/desktop/src/autostart.rs`
- Modify: `packages/desktop/src/views/home.rs` (接入 autostart)

**Step 1: 实现 autostart.rs**

```rust
use windows::Win32::System::Registry::{
    RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY_CURRENT_USER, KEY_SET_VALUE,
    KEY_QUERY_VALUE, REG_SZ, RegQueryValueExW,
};
use windows::core::{w, PCWSTR};

const RUN_KEY: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const APP_NAME: PCWSTR = w!("WinAide");

/// 设置开机自启
pub fn set_auto_start(enabled: bool) {
    unsafe {
        let mut hkey = Default::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY, 0, KEY_SET_VALUE, &mut hkey).is_ok() {
            if enabled {
                // 获取当前 exe 路径
                if let Ok(exe_path) = std::env::current_exe() {
                    let path_str = exe_path.to_string_lossy().to_string();
                    let wide: Vec<u16> = path_str.encode_utf16().chain(std::iter::once(0)).collect();
                    let bytes: &[u8] = std::slice::from_raw_parts(
                        wide.as_ptr() as *const u8,
                        wide.len() * 2,
                    );
                    let _ = RegSetValueExW(hkey, APP_NAME, 0, REG_SZ, Some(bytes));
                }
            } else {
                let _ = RegDeleteValueW(hkey, APP_NAME);
            }
        }
    }
}

/// 检查是否已设置开机自启
pub fn is_auto_start_enabled() -> bool {
    unsafe {
        let mut hkey = Default::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY, 0, KEY_QUERY_VALUE, &mut hkey).is_ok() {
            RegQueryValueExW(hkey, APP_NAME, None, None, None, None).is_ok()
        } else {
            false
        }
    }
}
```

**Step 2: 在 main.rs 中注册模块**

```rust
mod autostart;
```

**Step 3: 在 home.rs 设置面板中接入 autostart**

在 `on_config_changed` 回调中，当 `auto_start` 设置变更时调用 `autostart::set_auto_start`。在 Home 组件的设置区域 auto_start checkbox 的 `onchange` 中：

```rust
onchange: move |_| {
    let mut cfg = config();
    cfg.settings.auto_start = !cfg.settings.auto_start;
    crate::autostart::set_auto_start(cfg.settings.auto_start);
    save_and_notify(cfg);
},
```

**Step 4: 验证编译**

Run: `cargo check -p desktop`
Expected: 编译通过

**Step 5: 提交**

```bash
git add packages/desktop/src/autostart.rs packages/desktop/src/main.rs packages/desktop/src/views/home.rs
git commit -m "feat: 实现开机自启功能，通过注册表 HKCU Run 键管理"
```

---

## Task 12: 更新 TailwindCSS 配置和构建

**Files:**
- Modify: `tailwind.css`

**Step 1: 更新 tailwind.css 主题色**

```css
@import "tailwindcss";

@theme {
  --color-bg-primary: #0f1116;
  --color-bg-card: #1e222d;
  --color-accent: #91a4d2;
  --color-accent-focus: #6d85c6;
}
```

（主题色不变，确认 source 路径包含新组件文件）

**Step 2: 重新构建 CSS**

Run: `bash build-css.sh`
Expected: TailwindCSS build complete.

**Step 3: 验证完整构建**

Run: `cargo build -p desktop`
Expected: 构建成功

**Step 4: 提交**

```bash
git add -A
git commit -m "chore: 更新 TailwindCSS 构建配置"
```

---

## Task 13: 最终集成验证

**Step 1: 完整编译检查**

Run: `cargo clippy -- -W clippy::all`
Expected: 无 error，可能有少量 warning

**Step 2: 运行测试**

Run: `cargo test -p desktop`
Expected: 所有 config 测试通过

**Step 3: 启动应用验证**

Run: `cd packages/desktop && dx serve`
Expected:
- 窗口打开，显示快捷键管理列表界面
- 点击「添加快捷键」弹出表单
- 系统托盘显示图标
- 填写表单并保存，列表中出现新条目
- 快捷键可以触发启动/激活应用

**Step 4: 最终提交**

修复所有 clippy 警告后：

```bash
git add -A
git commit -m "feat: Win Aide 快捷键启动器 MVP 完成"
```

---

## 任务依赖图

```
Task 1 (清理 workspace)
  └── Task 2 (添加依赖)
       ├── Task 3 (config 模块) ──┐
       ├── Task 4 (hotkey 模块) ──┤
       ├── Task 5 (launcher 模块)─┤
       └── Task 6 (tray 模块) ───┤
            ├── Task 7 (UI ShortcutList) ──┐
            └── Task 8 (UI ShortcutForm) ──┤
                 └── Task 9 (main.rs + home 视图)
                      └── Task 10 (线程通信接入)
                           └── Task 11 (开机自启)
                                └── Task 12 (TailwindCSS)
                                     └── Task 13 (集成验证)
```

**可并行的任务：** Task 3/4/5/6 互相独立，可以并行开发。Task 7/8 也可并行。
