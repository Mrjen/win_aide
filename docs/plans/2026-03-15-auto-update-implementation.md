# 自动更新功能实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 win_aide 添加 GitHub Release 自动更新功能，支持定时检查 + 手动检查、应用内弹窗通知、自动下载替换重启。

**Architecture:** 在 `packages/desktop/src/updater.rs` 实现更新核心逻辑（检查、下载、替换），在 `packages/ui/src/update_dialog.rs` 实现更新弹窗 UI 组件。通过 `Signal<UpdateState>` 在后台逻辑和 UI 间共享状态。自替换通过 `.bat` 脚本解决 Windows 文件锁问题。

**Tech Stack:** reqwest (HTTP), semver (版本比较), futures-util (流处理), Dioxus 0.7.1 (UI), tokio (异步运行时)

---

### Task 1: 添加依赖

**Files:**
- Modify: `packages/desktop/Cargo.toml`

**Step 1: 添加 reqwest、semver、futures-util 依赖**

在 `packages/desktop/Cargo.toml` 的 `[dependencies]` 部分添加：

```toml
reqwest = { version = "0.12", features = ["json", "stream"] }
semver = "1"
futures-util = "0.3"
```

**Step 2: 验证依赖可以解析**

Run: `cargo check -p desktop 2>&1 | head -5`
Expected: 编译开始（可能较慢因为首次下载依赖），无依赖解析错误

**Step 3: Commit**

```bash
git add packages/desktop/Cargo.toml
git commit -m "feat: 添加自动更新功能依赖 reqwest、semver、futures-util"
```

---

### Task 2: 实现 updater 核心模块 — 数据结构与版本检查

**Files:**
- Create: `packages/desktop/src/updater.rs`
- Modify: `packages/desktop/src/main.rs` (添加 `mod updater;`)

**Step 1: 创建 updater.rs，定义数据结构**

```rust
use serde::Deserialize;

/// GitHub Release API 响应（仅取需要的字段）
#[derive(Debug, Clone, Deserialize)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

/// 更新状态机
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateState {
    Idle,
    Checking,
    Available(UpdateInfo),
    Downloading { progress: f64 },
    Ready,
    Error(String),
}

/// 供 UI 显示的更新信息（从 ReleaseInfo 提取）
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateInfo {
    pub version: String,
    pub name: String,
    pub body: String,
    pub download_url: String,
    pub size: u64,
}

impl Default for UpdateState {
    fn default() -> Self {
        Self::Idle
    }
}

const GITHUB_API_URL: &str = "https://api.github.com/repos/Mrjen/win_aide/releases/latest";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
```

**Step 2: 实现 check_update 函数**

在 `updater.rs` 中继续添加：

```rust
use dioxus::prelude::*;

/// 检查是否有新版本可用，更新 state Signal
pub async fn check_update(state: &mut Signal<UpdateState>) {
    state.set(UpdateState::Checking);

    match fetch_latest_release().await {
        Ok(Some(info)) => state.set(UpdateState::Available(info)),
        Ok(None) => state.set(UpdateState::Idle),
        Err(e) => {
            // 网络错误静默处理，回到 Idle（不打扰用户）
            eprintln!("检查更新失败: {e}");
            state.set(UpdateState::Idle);
        }
    }
}

/// 调用 GitHub API 获取最新 Release，比较版本
async fn fetch_latest_release() -> Result<Option<UpdateInfo>, String> {
    let client = reqwest::Client::builder()
        .user_agent("win_aide-updater")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let release: ReleaseInfo = client
        .get(GITHUB_API_URL)
        .send()
        .await
        .map_err(|e| format!("请求失败: {e}"))?
        .json()
        .await
        .map_err(|e| format!("解析 JSON 失败: {e}"))?;

    // 解析版本号
    let remote_ver_str = release.tag_name.strip_prefix('v').unwrap_or(&release.tag_name);
    let remote_ver = semver::Version::parse(remote_ver_str)
        .map_err(|e| format!("解析远程版本号失败: {e}"))?;
    let current_ver = semver::Version::parse(CURRENT_VERSION)
        .map_err(|e| format!("解析当前版本号失败: {e}"))?;

    if remote_ver <= current_ver {
        return Ok(None);
    }

    // 查找 .exe 附件
    let asset = release
        .assets
        .iter()
        .find(|a| a.name.ends_with(".exe"))
        .ok_or_else(|| "Release 中未找到 .exe 文件".to_string())?;

    Ok(Some(UpdateInfo {
        version: remote_ver_str.to_string(),
        name: release.name.unwrap_or_else(|| release.tag_name.clone()),
        body: release.body.unwrap_or_default(),
        download_url: asset.browser_download_url.clone(),
        size: asset.size,
    }))
}
```

**Step 3: 在 main.rs 中注册模块**

在 `packages/desktop/src/main.rs` 的 mod 声明区域添加：

```rust
mod updater;
```

（添加在 `mod views;` 之后即可）

**Step 4: 验证编译**

Run: `cargo check -p desktop`
Expected: 编译通过，无错误

**Step 5: Commit**

```bash
git add packages/desktop/src/updater.rs packages/desktop/src/main.rs
git commit -m "feat: 实现更新检查核心逻辑（GitHub API + semver 比较）"
```

---

### Task 3: 实现下载与自替换逻辑

**Files:**
- Modify: `packages/desktop/src/updater.rs`

**Step 1: 添加下载函数**

在 `updater.rs` 中添加：

```rust
use futures_util::StreamExt;
use std::path::PathBuf;

/// 下载新版本 exe 到临时目录
pub async fn download_update(
    state: &mut Signal<UpdateState>,
    download_url: &str,
    expected_size: u64,
) -> Result<(), String> {
    state.set(UpdateState::Downloading { progress: 0.0 });

    let client = reqwest::Client::builder()
        .user_agent("win_aide-updater")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let response = client
        .get(download_url)
        .send()
        .await
        .map_err(|e| format!("下载请求失败: {e}"))?;

    let total_size = response.content_length().unwrap_or(expected_size);
    let temp_path = get_temp_exe_path();

    let mut file = tokio::fs::File::create(&temp_path)
        .await
        .map_err(|e| format!("创建临时文件失败: {e}"))?;

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    use tokio::io::AsyncWriteExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("下载数据失败: {e}"))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("写入文件失败: {e}"))?;
        downloaded += chunk.len() as u64;
        let progress = (downloaded as f64 / total_size as f64).min(1.0);
        state.set(UpdateState::Downloading { progress });
    }

    file.flush().await.map_err(|e| format!("刷新文件失败: {e}"))?;
    drop(file);

    // 校验文件大小
    let metadata = tokio::fs::metadata(&temp_path)
        .await
        .map_err(|e| format!("读取文件信息失败: {e}"))?;
    if metadata.len() != expected_size {
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(format!(
            "文件大小不匹配: 期望 {} 字节，实际 {} 字节",
            expected_size,
            metadata.len()
        ));
    }

    state.set(UpdateState::Ready);
    Ok(())
}

fn get_temp_exe_path() -> PathBuf {
    std::env::temp_dir().join("win_aide_update.exe")
}
```

**Step 2: 添加自替换函数**

继续在 `updater.rs` 中添加：

```rust
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 生成 bat 脚本执行替换并重启
pub fn apply_update() -> Result<(), String> {
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("获取当前 exe 路径失败: {e}"))?;
    let current_exe_str = current_exe.to_string_lossy();
    let temp_exe = get_temp_exe_path();
    let temp_exe_str = temp_exe.to_string_lossy();

    let bat_content = format!(
        "@echo off\r\n\
         timeout /t 2 /nobreak >nul\r\n\
         copy /y \"{}\" \"{}\"\r\n\
         start \"\" \"{}\"\r\n\
         del \"%~f0\"\r\n",
        temp_exe_str, current_exe_str, current_exe_str
    );

    let bat_path = std::env::temp_dir().join("win_aide_updater.bat");
    std::fs::write(&bat_path, &bat_content)
        .map_err(|e| format!("写入更新脚本失败: {e}"))?;

    std::process::Command::new("cmd")
        .args(["/C", &bat_path.to_string_lossy()])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| format!("启动更新脚本失败: {e}"))?;

    std::process::exit(0);
}
```

**Step 3: 验证编译**

Run: `cargo check -p desktop`
Expected: 编译通过

**Step 4: Commit**

```bash
git add packages/desktop/src/updater.rs
git commit -m "feat: 实现更新下载与自替换重启逻辑"
```

---

### Task 4: 实现更新弹窗 UI 组件

**Files:**
- Create: `packages/ui/src/update_dialog.rs`
- Modify: `packages/ui/src/lib.rs` (注册并导出组件)
- Modify: `packages/ui/Cargo.toml` (无需修改，已有 dioxus 依赖)

**Step 1: 创建 update_dialog.rs**

```rust
use dioxus::prelude::*;

/// 更新弹窗的显示数据
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateDialogState {
    /// 有新版本可用
    Available {
        version: String,
        name: String,
        body: String,
    },
    /// 下载中
    Downloading {
        progress: f64,
    },
    /// 下载完成，准备安装
    Ready,
    /// 出错
    Error {
        message: String,
    },
}

#[component]
pub fn UpdateDialog(
    state: UpdateDialogState,
    on_update: EventHandler<()>,
    on_dismiss: EventHandler<()>,
    on_retry: EventHandler<()>,
    on_install: EventHandler<()>,
) -> Element {
    rsx! {
        // 模态遮罩
        div {
            class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
            onclick: move |_| {
                // 下载中不允许关闭
                if !matches!(state, UpdateDialogState::Downloading { .. }) {
                    on_dismiss.call(());
                }
            },
            div {
                class: "bg-bg-card rounded-xl p-6 w-[420px] max-h-[80vh] shadow-2xl border border-border-subtle flex flex-col",
                onclick: move |e| e.stop_propagation(),

                match &state {
                    UpdateDialogState::Available { version, name, body } => rsx! {
                        // 更新图标
                        div { class: "w-10 h-10 rounded-full bg-accent/10 flex items-center justify-center mb-4",
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                width: "20",
                                height: "20",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                class: "text-accent",
                                path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }
                                polyline { points: "7 10 12 15 17 10" }
                                line { x1: "12", y1: "15", x2: "12", y2: "3" }
                            }
                        }
                        h3 { class: "text-lg font-semibold text-text-primary mb-1",
                            "发现新版本"
                        }
                        p { class: "text-sm text-text-muted mb-3",
                            "v{version}"
                            if !name.is_empty() {
                                " — {name}"
                            }
                        }
                        // 更新日志
                        if !body.is_empty() {
                            div { class: "mb-4 p-3 bg-bg-primary rounded-lg border border-border-subtle max-h-[200px] overflow-y-auto",
                                pre { class: "text-xs text-text-secondary whitespace-pre-wrap font-sans leading-relaxed",
                                    "{body}"
                                }
                            }
                        }
                        // 按钮
                        div { class: "flex justify-end gap-2",
                            button {
                                class: "px-4 py-2 text-sm text-text-secondary hover:text-text-primary hover:bg-bg-hover rounded-lg transition-colors cursor-pointer",
                                onclick: move |_| on_dismiss.call(()),
                                "稍后提醒"
                            }
                            button {
                                class: "px-4 py-2 text-sm bg-accent text-white rounded-lg hover:bg-accent-focus transition-colors cursor-pointer font-medium",
                                onclick: move |_| on_update.call(()),
                                "立即更新"
                            }
                        }
                    },
                    UpdateDialogState::Downloading { progress } => rsx! {
                        h3 { class: "text-lg font-semibold text-text-primary mb-4",
                            "正在下载更新..."
                        }
                        // 进度条
                        div { class: "w-full bg-bg-primary rounded-full h-2.5 mb-2",
                            div {
                                class: "bg-accent h-2.5 rounded-full transition-all duration-300",
                                style: "width: {progress * 100.0:.0}%",
                            }
                        }
                        p { class: "text-sm text-text-muted text-center",
                            "{progress * 100.0:.0}%"
                        }
                    },
                    UpdateDialogState::Ready => rsx! {
                        // 成功图标
                        div { class: "w-10 h-10 rounded-full bg-success-subtle flex items-center justify-center mb-4",
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                width: "20",
                                height: "20",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                class: "text-success",
                                path { d: "M20 6 9 17l-5-5" }
                            }
                        }
                        h3 { class: "text-lg font-semibold text-text-primary mb-2",
                            "下载完成"
                        }
                        p { class: "text-sm text-text-secondary mb-4",
                            "更新已下载完成，点击安装将关闭应用并自动完成更新。"
                        }
                        div { class: "flex justify-end gap-2",
                            button {
                                class: "px-4 py-2 text-sm text-text-secondary hover:text-text-primary hover:bg-bg-hover rounded-lg transition-colors cursor-pointer",
                                onclick: move |_| on_dismiss.call(()),
                                "稍后安装"
                            }
                            button {
                                class: "px-4 py-2 text-sm bg-accent text-white rounded-lg hover:bg-accent-focus transition-colors cursor-pointer font-medium",
                                onclick: move |_| on_install.call(()),
                                "安装并重启"
                            }
                        }
                    },
                    UpdateDialogState::Error { message } => rsx! {
                        // 错误图标
                        div { class: "w-10 h-10 rounded-full bg-danger-subtle flex items-center justify-center mb-4",
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                width: "20",
                                height: "20",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                class: "text-danger",
                                circle { cx: "12", cy: "12", r: "10" }
                                path { d: "m15 9-6 6" }
                                path { d: "m9 9 6 6" }
                            }
                        }
                        h3 { class: "text-lg font-semibold text-text-primary mb-2",
                            "更新失败"
                        }
                        p { class: "text-sm text-text-secondary mb-4",
                            "{message}"
                        }
                        div { class: "flex justify-end gap-2",
                            button {
                                class: "px-4 py-2 text-sm text-text-secondary hover:text-text-primary hover:bg-bg-hover rounded-lg transition-colors cursor-pointer",
                                onclick: move |_| on_dismiss.call(()),
                                "关闭"
                            }
                            button {
                                class: "px-4 py-2 text-sm bg-accent text-white rounded-lg hover:bg-accent-focus transition-colors cursor-pointer font-medium",
                                onclick: move |_| on_retry.call(()),
                                "重试"
                            }
                        }
                    },
                }
            }
        }
    }
}
```

**Step 2: 在 lib.rs 中注册并导出**

在 `packages/ui/src/lib.rs` 底部添加：

```rust
mod update_dialog;
pub use update_dialog::{UpdateDialog, UpdateDialogState};
```

**Step 3: 验证编译**

Run: `cargo check -p ui`
Expected: 编译通过

**Step 4: Commit**

```bash
git add packages/ui/src/update_dialog.rs packages/ui/src/lib.rs
git commit -m "feat: 实现更新弹窗 UI 组件"
```

---

### Task 5: 集成到主页面 — 定时检查与 UI 交互

**Files:**
- Modify: `packages/desktop/src/views/home.rs`

**Step 1: 添加更新状态与定时检查**

在 `home.rs` 的 `Home` 组件函数开头（现有 signal 声明区域之后）添加：

```rust
use crate::updater::{self, UpdateState};
use ui::{UpdateDialog, UpdateDialogState};
```

在 Home 组件中现有 signal 声明后添加：

```rust
let mut update_state = use_signal(|| UpdateState::Idle);

// 定时检查更新
use_future(move || async move {
    // 启动后延迟 30 秒
    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    loop {
        updater::check_update(&mut update_state).await;
        // 每 4 小时检查一次
        tokio::time::sleep(std::time::Duration::from_secs(4 * 3600)).await;
    }
});
```

**Step 2: 在底部状态栏添加版本号和手动检查按钮**

将 `home.rs` 中现有的底部状态栏替换为：

```rust
// ── 底部状态栏 ──
div { class: "px-5 py-2 bg-bg-card border-t border-border-default flex items-center justify-between text-xs text-text-muted",
    span { "共 {shortcut_count} 个快捷键，{enabled_count} 个已启用" }
    div { class: "flex items-center gap-3",
        if paused() {
            span { class: "text-warning-text font-medium", "已暂停" }
        }
        // 版本号 + 检查更新按钮
        match update_state() {
            UpdateState::Checking => rsx! {
                span { class: "text-text-muted animate-pulse", "检查更新中..." }
            },
            UpdateState::Available(_) => rsx! {
                button {
                    class: "text-accent hover:text-accent-focus cursor-pointer font-medium",
                    onclick: move |_| {
                        // 点击显示弹窗（状态已经是 Available，弹窗自动显示）
                    },
                    "有新版本可用"
                }
            },
            _ => rsx! {
                button {
                    class: "hover:text-text-primary cursor-pointer transition-colors",
                    title: "点击检查更新",
                    onclick: move |_| {
                        spawn(async move {
                            updater::check_update(&mut update_state).await;
                        });
                    },
                    "v{updater::current_version()}"
                }
            },
        }
    }
}
```

注意：需要在 `updater.rs` 中添加一个公开的辅助函数：

```rust
pub fn current_version() -> &'static str {
    CURRENT_VERSION
}
```

**Step 3: 在页面底部（关闭花括号之前）添加更新弹窗**

在 `home.rs` 的 rsx 中，进程选择弹窗之后添加：

```rust
// ── 更新弹窗 ──
match update_state() {
    UpdateState::Available(ref info) => {
        let info_clone = info.clone();
        rsx! {
            UpdateDialog {
                state: UpdateDialogState::Available {
                    version: info.version.clone(),
                    name: info.name.clone(),
                    body: info.body.clone(),
                },
                on_update: move |_| {
                    let url = info_clone.download_url.clone();
                    let size = info_clone.size;
                    spawn(async move {
                        if let Err(e) = updater::download_update(&mut update_state, &url, size).await {
                            update_state.set(UpdateState::Error(e));
                        }
                    });
                },
                on_dismiss: move |_| update_state.set(UpdateState::Idle),
                on_retry: move |_| {
                    spawn(async move {
                        updater::check_update(&mut update_state).await;
                    });
                },
                on_install: move |_| {},
            }
        }
    }
    UpdateState::Downloading { progress } => rsx! {
        UpdateDialog {
            state: UpdateDialogState::Downloading { progress },
            on_update: move |_| {},
            on_dismiss: move |_| {},
            on_retry: move |_| {},
            on_install: move |_| {},
        }
    },
    UpdateState::Ready => rsx! {
        UpdateDialog {
            state: UpdateDialogState::Ready,
            on_update: move |_| {},
            on_dismiss: move |_| update_state.set(UpdateState::Idle),
            on_retry: move |_| {},
            on_install: move |_| {
                if let Err(e) = updater::apply_update() {
                    update_state.set(UpdateState::Error(e));
                }
            },
        }
    },
    UpdateState::Error(ref msg) => rsx! {
        UpdateDialog {
            state: UpdateDialogState::Error { message: msg.clone() },
            on_update: move |_| {},
            on_dismiss: move |_| update_state.set(UpdateState::Idle),
            on_retry: move |_| {
                spawn(async move {
                    updater::check_update(&mut update_state).await;
                });
            },
            on_install: move |_| {},
        }
    },
    _ => rsx! {},
}
```

**Step 4: 验证编译**

Run: `cargo check -p desktop`
Expected: 编译通过

**Step 5: Commit**

```bash
git add packages/desktop/src/views/home.rs packages/desktop/src/updater.rs
git commit -m "feat: 集成更新检查到主页面（定时检查 + 手动触发 + 弹窗交互）"
```

---

### Task 6: 托盘菜单添加"检查更新"

**Files:**
- Modify: `packages/desktop/src/tray.rs`
- Modify: `packages/desktop/src/main.rs` (处理托盘事件)

**Step 1: 在 tray.rs 中添加菜单项**

在 `tray.rs` 中添加常量：

```rust
pub const MENU_CHECK_UPDATE: &str = "check_update";
```

在 `TrayEvent` 枚举中添加变体：

```rust
pub enum TrayEvent {
    Show,
    TogglePause,
    CheckUpdate,
    Quit,
}
```

在 `create_tray` 函数中，在 `pause_item` 和 `quit_item` 之间添加菜单项：

```rust
let check_update_item = MenuItem::with_id(MENU_CHECK_UPDATE, "检查更新", true, None);

// 在 menu.append 序列中，pause_item 后面添加：
let _ = menu.append(&PredefinedMenuItem::separator());
let _ = menu.append(&check_update_item);
```

在 `poll_tray_event` 的 match 中添加：

```rust
MENU_CHECK_UPDATE => Some(TrayEvent::CheckUpdate),
```

**Step 2: 在 main.rs 中处理 CheckUpdate 事件**

在 `main.rs` 的 App 组件中，需要让 `update_state` Signal 可被托盘事件访问。

首先，将 `update_state` 从 Home 提升到 App 组件中作为 context：

在 App 组件中添加：

```rust
let mut update_state = use_context_provider(|| Signal::new(updater::UpdateState::Idle));
```

然后在现有的托盘事件轮询 `use_future` 中添加 `CheckUpdate` 分支：

```rust
tray::TrayEvent::CheckUpdate => {
    let mut update_state = update_state.clone();
    spawn(async move {
        updater::check_update(&mut update_state).await;
    });
    // 显示主窗口以便用户看到结果
    let window = dioxus::desktop::window();
    window.set_visible(true);
    window.set_focus();
}
```

同时需要修改 Home 组件的 props，添加 `update_state: Signal<updater::UpdateState>`，并从 App 传递过来。在 Home 中不再自己创建 update_state signal，而是通过 props 接收或通过 `use_context` 获取。

**推荐做法：使用 use_context**

在 Home 组件中：

```rust
let mut update_state: Signal<updater::UpdateState> = use_context();
```

这样不需要修改 Home 的 props 签名。

**Step 3: 验证编译**

Run: `cargo check -p desktop`
Expected: 编译通过

**Step 4: Commit**

```bash
git add packages/desktop/src/tray.rs packages/desktop/src/main.rs packages/desktop/src/views/home.rs
git commit -m "feat: 托盘菜单添加检查更新入口"
```

---

### Task 7: 添加 tokio 完整特性支持

**Files:**
- Modify: `packages/desktop/Cargo.toml`

**Step 1: 检查并更新 tokio features**

当前 tokio features 为 `["rt", "time"]`。下载功能需要 `fs` 支持。更新为：

```toml
tokio = { version = "1.50.0", features = ["rt", "time", "fs"] }
```

**Step 2: 验证编译**

Run: `cargo check -p desktop`
Expected: 编译通过

**Step 3: Commit**

```bash
git add packages/desktop/Cargo.toml
git commit -m "feat: 为 tokio 添加 fs feature 以支持异步文件写入"
```

---

### Task 8: 最终验证与清理

**Step 1: 运行 clippy 检查**

Run: `cargo clippy --workspace -- -D warnings`
Expected: 无 warning 和 error

**Step 2: 修复所有 clippy 警告（如有）**

根据 clippy 输出修复问题。常见问题：
- 未使用的 imports
- 不必要的 clone
- match 分支可简化

**Step 3: 运行测试**

Run: `cargo test --workspace`
Expected: 所有测试通过

**Step 4: 构建 release 验证**

Run: `cd packages/desktop && dx build --release`
Expected: 构建成功

**Step 5: Commit（如有修复）**

```bash
git add -A
git commit -m "fix: 修复 clippy 警告和编译问题"
```
