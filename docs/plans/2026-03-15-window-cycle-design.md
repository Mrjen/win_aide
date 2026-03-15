# 同应用多窗口循环切换 - 设计文档

## 概述

实现 Alt+` 在同一应用的多个窗口间循环切换的功能，类似 macOS 的 Cmd+` 行为。Alt+` 切换到下一个窗口，Alt+Shift+` 切换到上一个窗口。该功能作为可配置的系统热键集成到 Settings 面板中。

## 方案选择

**采用方案 A：扩展现有热键线程**。复用现有 `RegisterHotKey` + `PeekMessageW` 基础设施，改动最小且与现有架构自然融合。

## 数据模型变更

### Settings 新增字段

```rust
pub struct Settings {
    pub auto_start: bool,
    pub start_minimized: bool,
    pub dark_mode: bool,
    pub window_cycle: WindowCycleSettings,  // 新增
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WindowCycleSettings {
    pub enabled: bool,       // 是否启用，默认 true
    pub modifier: Modifier,  // 修饰键，默认 Alt
    pub key: char,           // 按键，默认 '`'
}
```

- `modifier + key` 用于下一个窗口
- `modifier + Shift + key` 自动作为反方向（上一个窗口），无需额外配置
- 使用 `#[serde(default)]` 保证旧配置文件向后兼容

### HotkeyEvent 扩展

```rust
pub enum HotkeyEvent {
    ShortcutTriggered { shortcut_id: String },
    WindowCycleNext,
    WindowCyclePrev,
}
```

## 热键注册与事件分发

### HotkeyCommand 新增指令

```rust
pub enum HotkeyCommand {
    RegisterAll(Vec<Shortcut>),
    UnregisterAll,
    RegisterWindowCycle { modifier: Modifier, key: char },  // 新增
    UnregisterWindowCycle,                                   // 新增
    Shutdown,
}
```

### 注册逻辑

- 窗口循环使用两个保留 ID（10001 和 10002），与用户快捷键自增 ID 空间隔开
- ID 10001: `modifier + key` → `WindowCycleNext`
- ID 10002: `modifier + MOD_SHIFT + key` → `WindowCyclePrev`
- 按键转换：`` ` `` (backtick) 对应 `VK_OEM_3 = 0xC0`，扩展 `key_to_vk()` 支持非字母按键

### key_to_vk 扩展

```rust
fn key_to_vk(key: char) -> u32 {
    match key {
        '`' => 0xC0,  // VK_OEM_3
        _ => key.to_ascii_uppercase() as u32,
    }
}
```

### 暂停行为

`UnregisterAll` 时同时注销窗口循环热键，恢复时根据 Settings 决定是否重新注册。

## 窗口循环核心逻辑

在 `launcher.rs` 中新增：

```rust
enum Direction { Next, Prev }

fn cycle_window(direction: Direction) {
    // 1. GetForegroundWindow() 获取当前活动窗口
    // 2. GetWindowThreadProcessId() 获取其进程 ID
    // 3. EnumWindows 收集该进程的所有可见窗口
    // 4. 如果窗口数 <= 1，直接返回
    // 5. 在列表中找到当前窗口的索引
    // 6. 根据 direction 计算目标索引（环形）
    //    - Next: (index + 1) % len
    //    - Prev: (index - 1 + len) % len
    // 7. activate_window(目标窗口)
}
```

### 新增 Win32 API

- `GetForegroundWindow` — 获取当前活动窗口

其余 API（`EnumWindows`、`GetWindowThreadProcessId`、`IsWindowVisible`、`SetForegroundWindow` 等）已有。

### 与现有代码的关系

- 复用现有 `activate_window()` 函数
- 新增 `find_all_windows_by_pid()` 函数，按进程 ID 收集所有可见窗口

## UI 设计

### Settings 面板扩展

在现有设置面板中追加窗口循环配置区域：

```
┌─ 设置面板 ──────────────────────────────────────────┐
│ ☑ 开机自启   ☑ 启动时最小化到托盘   ☑ 暗色模式      │
│                                                      │
│ ── 同应用窗口循环切换 ──                              │
│ ☑ 启用    修饰键: [Alt ▾]    按键: [`  ]             │
│ 提示: Alt+` 下一个窗口 / Alt+Shift+` 上一个窗口      │
└──────────────────────────────────────────────────────┘
```

- 开关：checkbox 控制 `enabled`
- 修饰键：下拉选择 Alt / Ctrl / Win
- 按键：文本输入框，单字符（默认 `` ` ``）
- 提示文字：动态显示当前配置对应的实际快捷键组合

### 冲突检测

修改时检测是否与用户已配置的快捷键冲突（同 modifier + key），同时检测 modifier + Shift + key 是否冲突。

## main.rs 集成

启动时：现有流程不变，如果 `settings.window_cycle.enabled` 则发送 `RegisterWindowCycle` 命令。

配置变更时：先 `UnregisterWindowCycle`，再根据新配置决定是否重新注册。

## 涉及文件

| 文件 | 变更类型 |
|------|----------|
| `packages/desktop/src/config.rs` | 新增 `WindowCycleSettings` 结构体，`Settings` 新增字段 |
| `packages/desktop/src/hotkey.rs` | 扩展 `HotkeyCommand`、`HotkeyEvent`，新增注册/注销逻辑，扩展 `key_to_vk` |
| `packages/desktop/src/launcher.rs` | 新增 `cycle_window` + `find_all_windows_by_pid`，扩展事件处理 |
| `packages/desktop/src/main.rs` | 启动时注册窗口循环热键，配置变更时同步 |
| `packages/desktop/src/views/home.rs` | 设置面板新增窗口循环配置 UI |
| `packages/desktop/Cargo.toml` | windows-rs features 可能需补充 `GetForegroundWindow` |
