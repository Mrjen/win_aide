# 从运行程序中选择 — 设计文档

日期: 2026-03-13

## 目标

在快捷键表单中新增"从运行程序选择"功能，让用户可以直接从当前运行的进程列表中选择目标程序，无需手动浏览文件系统查找 exe 路径。

## 设计决策

| 决策点 | 选择 | 理由 |
|--------|------|------|
| 入口形式 | 并排按钮（在"浏览..."旁） | 不影响现有功能，操作路径清晰 |
| 显示信息 | 程序名 + exe 路径 + 图标 | 信息充足，方便辨认 |
| 进程过滤 | 只显示有可见窗口的进程 | 匹配"启动/激活窗口"的使用场景 |
| 去重策略 | 按 exe_path 去重 | 避免多进程程序（Chrome）重复显示 |
| 搜索功能 | 有，实时过滤 | 实现成本低，提升体验 |
| 技术方案 | 纯 Win32 API | 复用 launcher.rs 现有逻辑，零新依赖 |

## 数据层

### 新增模块 `packages/desktop/src/process.rs`

数据结构（定义在 `packages/ui` 中保持平台无关）：

```rust
pub struct ProcessInfo {
    pub name: String,           // 显示名（去掉 .exe 后缀）
    pub exe_name: String,       // 进程名（如 chrome.exe）
    pub exe_path: String,       // 完整路径
    pub icon: Option<Vec<u8>>,  // RGBA 图标数据（32x32）
}
```

核心函数（在 `packages/desktop/src/process.rs`）：

```rust
pub fn list_windowed_processes() -> Vec<ProcessInfo>
```

实现步骤：
1. `EnumWindows` 回调遍历顶级窗口
2. `IsWindowVisible` 过滤不可见窗口
3. `GetWindowThreadProcessId` → `OpenProcess` → `QueryFullProcessImageNameW` 获取路径
4. `ExtractIconExW` + `GetIconInfo` + `GetBitmapBits` 提取 32x32 图标转 RGBA
5. `HashMap<String, ProcessInfo>` 按 exe_path 去重
6. 按 name 字母排序后返回

## UI 组件

### `packages/ui/src/process_picker.rs` — 进程选择弹窗

```
┌─────────────────────────────────┐
│  从运行中的程序选择        [✕]   │
├─────────────────────────────────┤
│  🔍 搜索程序...                  │
├─────────────────────────────────┤
│  [icon] Chrome                  │
│         C:\Program Files\...     │
│─────────────────────────────────│
│  [icon] Visual Studio Code      │
│         C:\Users\...\Code.exe    │
│  ...（可滚动）                   │
└─────────────────────────────────┘
```

Props：
- `processes: Vec<ProcessInfo>` — 进程列表（由 home.rs 传入）
- `on_select: EventHandler<ProcessInfo>` — 选择回调
- `on_cancel: EventHandler<()>` — 取消回调

内部状态：
- `search_query: Signal<String>` — 搜索关键字

行为：
- 搜索框实时过滤 name 和 exe_path（不区分大小写）
- 点击某行触发 `on_select`
- 图标以 base64 data URI 渲染；无图标时显示默认占位图标
- 样式复用现有 modal 风格，遵循暗色/亮色主题 CSS 变量

### `shortcut_form.rs` 改动

- 在"浏览..."按钮旁新增"从运行程序选择"按钮
- 点击后通知 home.rs 显示 ProcessPicker
- 选择后自动填充 name、exe_name、exe_path（与"浏览"按钮逻辑一致）

## 数据流

```
用户点击"从运行程序选择"
  → shortcut_form 触发事件通知 home.rs
  → home.rs 调用 spawn(async { list_windowed_processes() })
  → 显示 ProcessPicker 弹窗（含加载状态）
  → 用户搜索/点击某行 → on_select(ProcessInfo)
  → shortcut_form 填充 name/exe_name/exe_path
  → 关闭弹窗
```

## 跨层架构

- `ProcessInfo` 定义在 `packages/ui`（纯数据结构，平台无关）
- `list_windowed_processes()` 定义在 `packages/desktop/src/process.rs`（Win32 依赖）
- `ProcessPicker` 组件在 `packages/ui`（通过 props 接收数据）
- `home.rs` 负责协调调用和数据传递

## 错误处理

- 进程枚举失败 → 弹窗显示"无法获取进程列表"
- 进程列表为空 → 显示"未找到运行中的程序"空状态
- 图标提取失败 → 显示默认占位图标

## 不做的事情

- 不做进程图标缓存（每次打开重新获取，保证实时性）
- 不做自动刷新（关闭重开即可刷新）
- 不修改现有 launcher.rs 逻辑
