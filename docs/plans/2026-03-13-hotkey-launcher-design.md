# Win Aide - 快捷键启动器设计文档

> 日期: 2026-03-13
> 状态: 已确认

## 1. 项目目标

将现有 Dioxus 多平台 demo 项目重构为一个 **Windows 快捷键启动器工具**，完全替代 AutoHotKey 脚本，实现：

- 通过 GUI 界面自定义「快捷键 → 应用程序」映射
- 全局快捷键监听：按下快捷键时启动应用或激活已有窗口
- 系统托盘常驻 + 开机自启

## 2. 技术方案

**Dioxus Desktop + windows-rs 直接调用 Win32 API**

- Dioxus 0.7.1 负责 GUI 渲染
- `windows-rs` 调用 Win32 API 实现全局快捷键注册、窗口查找/激活、进程启动
- 单 exe 分发，纯 Rust，零外部运行时依赖

## 3. 项目结构

```
win_aide/
├── Cargo.toml                    # workspace，两个成员：ui + desktop
├── package.json                  # TailwindCSS 构建
├── tailwind.css                  # Tailwind 入口
├── build-css.sh                  # CSS 编译脚本（只输出 desktop）
│
├── packages/
│   ├── ui/                       # 共享 UI 组件库
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── shortcut_list.rs  # 快捷键列表组件（表格）
│   │       ├── shortcut_form.rs  # 新增/编辑快捷键的表单弹窗
│   │       └── navbar.rs         # 工具栏
│   │
│   └── desktop/                  # Desktop 主应用
│       ├── Dioxus.toml
│       ├── Cargo.toml
│       ├── assets/
│       │   └── tailwind.css
│       └── src/
│           ├── main.rs           # 入口：初始化托盘、启动快捷键监听、渲染 GUI
│           ├── config.rs         # JSON 配置读写
│           ├── hotkey.rs         # Win32 全局快捷键注册/注销 + 消息循环
│           ├── launcher.rs       # LaunchOrActivate 逻辑
│           ├── tray.rs           # 系统托盘图标 + 右键菜单
│           └── views/
│               └── home.rs       # 主界面：快捷键管理列表
│
└── docs/plans/
    删除的包：web/, mobile/, api/
```

## 4. 数据模型

### 配置文件

存储位置：`~/.win_aide/config.json`

```json
{
  "version": 1,
  "shortcuts": [
    {
      "id": "a1b2c3d4",
      "name": "Chrome",
      "exe_name": "chrome.exe",
      "exe_path": "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
      "modifier": "Alt",
      "key": "C",
      "enabled": true
    }
  ],
  "settings": {
    "auto_start": true,
    "start_minimized": true
  }
}
```

### Rust 结构体

```rust
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct AppConfig {
    pub version: u32,
    pub shortcuts: Vec<Shortcut>,
    pub settings: Settings,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Shortcut {
    pub id: String,           // uuid
    pub name: String,         // 显示名称
    pub exe_name: String,     // 进程名（用于窗口匹配）
    pub exe_path: String,     // 完整路径（用于启动）
    pub modifier: Modifier,   // Alt / Ctrl / Win
    pub key: char,            // 字母键
    pub enabled: bool,        // 是否启用
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum Modifier {
    Alt,
    Ctrl,
    Win,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Settings {
    pub auto_start: bool,
    pub start_minimized: bool,
}
```

## 5. 核心模块设计

### 5.1 hotkey.rs - 全局快捷键

在独立线程中运行 Win32 消息循环：

- `RegisterHotKey` 注册全局快捷键
- `GetMessage` 循环监听 `WM_HOTKEY` 消息
- 通过 `std::sync::mpsc` channel 与主线程通信
- 配置变更时通过 channel 发送「注销旧快捷键 -> 注册新快捷键」指令
- 每个 Shortcut 映射到一个整数 hotkey ID

### 5.2 launcher.rs - 启动/激活

复刻 AHK 的 LaunchOrActivate 逻辑：

1. `EnumWindows` + `GetWindowThreadProcessId` + `QueryFullProcessImageName` 匹配进程
2. 找到窗口：`IsIconic` 检查是否最小化 -> `ShowWindow(SW_RESTORE)` -> `SetForegroundWindow` 激活
3. 未找到：`ShellExecute` / `CreateProcess` 启动应用
4. `SetForegroundWindow` 前调用 `AllowSetForegroundWindow` 确保前台切换成功

### 5.3 tray.rs - 系统托盘

使用 `tray-icon` crate：

- 左键双击：显示/隐藏主窗口
- 右键菜单：显示主窗口 / 暂停所有快捷键 / 退出

### 5.4 启动流程

```
main()
  1. 读取 config.json（不存在则创建默认配置）
  2. 初始化系统托盘
  3. spawn 快捷键监听线程，注册所有 enabled 的快捷键
  4. spawn launcher 处理线程，监听 channel
  5. 启动 Dioxus Desktop 渲染 GUI
     根据 start_minimized 决定是否显示窗口
```

## 6. GUI 界面

### 主界面

列表式布局，表格展示所有快捷键映射：

| 启用 | 快捷键 | 应用名称 | 路径 | 操作 |
|------|--------|---------|------|------|
| [x]  | Alt+C  | Chrome  | C:\...| 编辑 删除 |

顶部工具栏：[+ 添加快捷键] [设置]

### 新增/编辑弹窗

表单字段：
- 应用名称（文本输入）
- 进程名（文本输入）
- 路径（文本输入 + 浏览按钮，使用 `rfd` crate 弹出系统文件选择器）
- 快捷键（修饰键下拉选择 + 字母键输入）

### 交互规则

- 启用/禁用：点击复选框立即生效，实时注册/注销快捷键
- 编辑：弹出表单，保存后注销旧快捷键、注册新快捷键
- 删除：确认对话框，确认后注销并移除
- 冲突检测：保存时检查重复快捷键组合
- 关闭窗口：最小化到托盘，不退出

### 设置页面

- 开机自启（开/关）— 通过写入注册表 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` 实现
- 启动时最小化到托盘（开/关）

## 7. 依赖

| 用途 | Crate | 说明 |
|------|-------|------|
| GUI 框架 | `dioxus 0.7.1` | desktop feature |
| Win32 API | `windows-rs` | 官方 Rust 绑定 |
| 系统托盘 | `tray-icon` | Tauri 团队维护 |
| 文件选择器 | `rfd` | 系统原生对话框 |
| JSON 序列化 | `serde` + `serde_json` | 配置读写 |
| UUID | `uuid` | 条目 ID 生成 |
| 目录路径 | `dirs` | 用户目录获取 |

## 8. MVP 边界

### 包含

- 快捷键列表增删改查
- 启用/禁用单个快捷键
- 全局快捷键监听（单修饰键 + 字母）
- 启动/激活窗口（LaunchOrActivate）
- 系统托盘常驻 + 右键菜单
- 开机自启
- JSON 配置持久化
- 文件选择器选择 exe
- 快捷键冲突检测

### 不包含（未来扩展）

- 多修饰键组合（Ctrl+Alt+X）
- 同应用窗口循环切换
- 窗口分屏管理
- 应用图标提取显示
- 配置导入/导出
- 多语言支持
