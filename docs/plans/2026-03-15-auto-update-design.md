# 自动更新功能设计

## 概述

为 win_aide 添加 GitHub Release 自动更新功能，支持定时检查、应用内弹窗通知、下载新版并自动替换重启。

## 需求

- GitHub 公开仓库：`Mrjen/win_aide`
- 定时自动检查（每 4 小时）+ 手动检查入口
- 检测到新版后，应用内弹窗显示更新信息
- 用户确认后自动下载、替换、重启
- 单 exe 分发模式

## 方案选择

**方案 A：纯 Rust 自实现**（已选择）

使用 `reqwest` 调 GitHub API + `semver` 比较版本 + `.bat` 脚本自替换。依赖少、完全可控、与现有架构无缝集成。

## 架构设计

### 新增模块

`packages/desktop/src/updater.rs` — 更新逻辑核心模块，职责：

1. 版本检查 — 调用 GitHub API 获取最新 Release
2. 版本比较 — semver 对比
3. 下载 — 流式下载 exe 到临时目录，提供进度
4. 自替换 — 生成 bat 脚本完成替换和重启

`packages/ui/src/update_dialog.rs` — 更新弹窗 UI 组件

### 新增依赖

```toml
# packages/desktop/Cargo.toml
reqwest = { version = "0.12", features = ["json", "stream"] }
semver = "1"
futures-util = "0.3"
```

### 数据流

```
定时器/手动触发
  → updater::check_update()
  → GitHub API 返回 Release JSON
  → 解析 tag_name，semver 比较
  → 有新版本 → 通知 UI 显示弹窗
  → 用户点击"立即更新"
  → updater::download_update() 下载 exe（带进度）
  → updater::apply_update() 生成 bat 脚本并执行
  → 当前进程退出，bat 等待后替换 exe 并重启
```

## 版本检查与比较

### GitHub API

`GET https://api.github.com/repos/Mrjen/win_aide/releases/latest`

关注字段：

```rust
struct ReleaseInfo {
    tag_name: String,      // "v1.0.1"
    name: String,          // Release 标题
    body: String,          // 更新日志（Markdown）
    assets: Vec<Asset>,    // 附件列表
}

struct Asset {
    name: String,                    // "win_aide-v1.0.1.exe"
    browser_download_url: String,    // 下载地址
    size: u64,                       // 文件大小（字节）
}
```

### 版本比较逻辑

1. 从 `tag_name` 去掉前缀 `v`，得到 `"1.0.1"`
2. 当前版本从 `env!("CARGO_PKG_VERSION")` 编译期获取
3. 用 `semver::Version::parse()` 解析
4. 远程版本 > 当前版本 → 有更新
5. 从 `assets` 中查找 `.exe` 附件作为下载目标

### 定时检查

- `tokio::time::interval`，每 4 小时检查一次
- 启动后延迟 30 秒首次检查
- 在 `use_future` 中运行
- 网络错误静默忽略

## 下载与进度

### 下载流程

1. `reqwest` GET 请求获取文件流
2. 从 `Content-Length` 获取总大小
3. 下载到 `std::env::temp_dir().join("win_aide_update.exe")`
4. 逐块读取（8KB chunks），通过 `Signal` 更新进度

### 状态机

```rust
enum UpdateState {
    Idle,                          // 无更新
    Checking,                      // 检查中
    Available(ReleaseInfo),        // 有新版本
    Downloading { progress: f64 }, // 下载中 0.0~1.0
    Ready,                         // 下载完成
    Error(String),                 // 出错
}
```

通过全局 `Signal<UpdateState>` 共享。

### UI 表现

| 状态 | UI |
|------|----|
| Idle | 不显示 |
| Checking | "检查中..." + 转圈 |
| Available | 弹窗：版本号、更新日志、"立即更新"/"稍后提醒" |
| Downloading | 进度条 + 百分比 |
| Ready | "安装并重启"按钮 |
| Error | 错误信息 + "重试" |

## 自替换与重启

### 替换流程

1. 获取当前 exe 路径：`std::env::current_exe()`
2. 在临时目录生成 `win_aide_updater.bat`：

```bat
@echo off
timeout /t 2 /nobreak >nul
copy /y "%TEMP%\win_aide_update.exe" "当前exe路径"
start "" "当前exe路径"
del "%~f0"
```

3. `std::process::Command` 启动 bat（`CREATE_NO_WINDOW`）
4. 当前进程 `std::process::exit(0)`

### 安全保障

- 下载后校验文件大小与 `asset.size` 一致
- `copy` 失败不破坏旧版本
- 替换失败时用户下次启动仍是旧版

### 权限

- 不做 UAC 提权，保持简单
- 适用于用户自定义目录的单 exe 分发场景

## UI 集成

### 触发入口

1. **主页面** — 导航栏区域添加版本/更新状态指示，点击手动检查
2. **托盘菜单** — 右键菜单添加"检查更新"选项

### 定时检查集成

在 `home.rs` 新增 `use_future` 运行定时检查循环。

### 配置

不新增配置项，更新检查始终开启，保持简单。
