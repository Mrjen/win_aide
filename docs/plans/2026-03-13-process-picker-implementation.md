# 从运行程序选择 — 实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在快捷键表单中新增"从运行程序选择"功能，让用户从当前有窗口的运行进程中选择目标程序。

**Architecture:** 在 `packages/ui` 中定义平台无关的 `ProcessInfo` 数据结构和 `ProcessPicker` 弹窗组件；在 `packages/desktop` 中通过 Win32 API 实现进程枚举和图标提取；`home.rs` 负责协调数据获取和组件通信。

**Tech Stack:** Rust, Dioxus 0.7.1, windows crate 0.58 (Win32 API), base64 crate

---

## Task 1: 定义 ProcessInfo 数据结构并添加 base64 依赖

**Files:**
- Modify: `packages/ui/Cargo.toml`
- Modify: `packages/ui/src/lib.rs`

**Step 1: 在 `packages/ui/Cargo.toml` 添加 base64 依赖**

```toml
[dependencies]
dioxus = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
rfd = "0.15"
base64 = "0.22"
```

**Step 2: 在 `packages/ui/src/lib.rs` 添加 ProcessInfo 结构体**

在 `pub enum Modifier` 块之后、`mod navbar;` 之前添加：

```rust
/// 运行中的进程信息（平台无关数据结构）
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessInfo {
    /// 显示名（去掉 .exe 后缀）
    pub name: String,
    /// 进程文件名（如 chrome.exe）
    pub exe_name: String,
    /// 完整可执行文件路径
    pub exe_path: String,
    /// 32x32 RGBA 图标原始字节（4096 字节），None 表示无图标
    pub icon_rgba: Option<Vec<u8>>,
}
```

同时在文件底部添加模块声明和导出（先声明，Task 3 再创建文件）：

```rust
mod process_picker;
pub use process_picker::ProcessPicker;
```

**Step 3: 运行 cargo check 验证**

Run: `cargo check -p ui`
Expected: 编译错误（process_picker 模块文件不存在），这是预期的，会在 Task 3 创建。

**Step 4: Commit**

```bash
git add packages/ui/Cargo.toml packages/ui/src/lib.rs
git commit -m "feat: 添加 ProcessInfo 数据结构和 base64 依赖"
```

---

## Task 2: 实现进程枚举函数 (Win32 API)

**Files:**
- Modify: `packages/desktop/Cargo.toml` (添加 Win32 features)
- Create: `packages/desktop/src/process.rs`
- Modify: `packages/desktop/src/main.rs` (添加 `mod process;`)

**Step 1: 在 `packages/desktop/Cargo.toml` 添加图标提取所需的 Win32 features**

在 `[dependencies.windows]` 的 features 列表中追加：

```toml
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
    "Win32_UI_Shell",
    "Win32_Graphics_Gdi",
]
```

**Step 2: 创建 `packages/desktop/src/process.rs`**

```rust
use std::collections::HashMap;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use ui::ProcessInfo;
use windows::Win32::Foundation::{CloseHandle, BOOL, HWND, LPARAM, TRUE};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, SelectObject, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Shell::ExtractIconExW;
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyIcon, EnumWindows, GetIconInfo, GetWindowThreadProcessId, IsWindowVisible,
};

struct EnumData {
    /// exe_path (lowercase) → (name, exe_name, exe_path_original)
    processes: HashMap<String, (String, String, String)>,
}

unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data = &mut *(lparam.0 as *mut EnumData);

    if !IsWindowVisible(hwnd).as_bool() {
        return TRUE;
    }

    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid == 0 {
        return TRUE;
    }

    if let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
        let mut buf = [0u16; 1024];
        let mut size = buf.len() as u32;
        if QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_FORMAT(0),
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut size,
        )
        .is_ok()
        {
            let path = OsString::from_wide(&buf[..size as usize]);
            let path_str = path.to_string_lossy().to_string();
            let key = path_str.to_lowercase();

            if !data.processes.contains_key(&key) {
                let file_name = path_str
                    .rsplit('\\')
                    .next()
                    .unwrap_or(&path_str)
                    .to_string();
                let display_name = file_name.trim_end_matches(".exe").to_string();
                data.processes
                    .insert(key, (display_name, file_name, path_str));
            }
        }
        let _ = CloseHandle(handle);
    }

    TRUE
}

/// 从 exe 文件路径提取 32x32 图标的 RGBA 字节
fn extract_icon_rgba(exe_path: &str) -> Option<Vec<u8>> {
    unsafe {
        let wide_path: Vec<u16> = exe_path.encode_utf16().chain(std::iter::once(0)).collect();
        let mut large_icon = [windows::Win32::UI::WindowsAndMessaging::HICON::default(); 1];

        let count = ExtractIconExW(
            windows::core::PCWSTR(wide_path.as_ptr()),
            0,
            Some(large_icon.as_mut_ptr()),
            None,
            1,
        );

        if count == 0 || large_icon[0].is_invalid() {
            return None;
        }

        let hicon = large_icon[0];
        let result = icon_to_rgba(hicon);
        let _ = DestroyIcon(hicon);
        result
    }
}

/// 将 HICON 转换为 32x32 RGBA 字节
unsafe fn icon_to_rgba(
    hicon: windows::Win32::UI::WindowsAndMessaging::HICON,
) -> Option<Vec<u8>> {
    let mut icon_info = windows::Win32::UI::WindowsAndMessaging::ICONINFO::default();
    if GetIconInfo(hicon, &mut icon_info).is_err() {
        return None;
    }

    let hbm_color = icon_info.hbmColor;
    if hbm_color.is_invalid() {
        if !icon_info.hbmMask.is_invalid() {
            DeleteObject(icon_info.hbmMask);
        }
        return None;
    }

    let size: i32 = 32;
    let hdc = CreateCompatibleDC(None);
    let old = SelectObject(hdc, hbm_color);

    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: size,
            biHeight: -size, // top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0 as u32,
            ..Default::default()
        },
        ..Default::default()
    };

    let mut pixels = vec![0u8; (size * size * 4) as usize];

    let lines = GetDIBits(
        hdc,
        hbm_color,
        0,
        size as u32,
        Some(pixels.as_mut_ptr() as *mut _),
        &mut bmi,
        DIB_RGB_COLORS,
    );

    SelectObject(hdc, old);
    DeleteDC(hdc);
    DeleteObject(hbm_color);
    if !icon_info.hbmMask.is_invalid() {
        DeleteObject(icon_info.hbmMask);
    }

    if lines == 0 {
        return None;
    }

    // Windows 返回 BGRA，转为 RGBA
    for chunk in pixels.chunks_exact_mut(4) {
        chunk.swap(0, 2); // B <-> R
    }

    Some(pixels)
}

/// 枚举所有有可见窗口的进程，按 exe_path 去重，按 name 排序
pub fn list_windowed_processes() -> Vec<ProcessInfo> {
    let mut data = EnumData {
        processes: HashMap::new(),
    };

    unsafe {
        let _ = EnumWindows(
            Some(enum_callback),
            LPARAM(&mut data as *mut EnumData as isize),
        );
    }

    let mut result: Vec<ProcessInfo> = data
        .processes
        .into_values()
        .map(|(name, exe_name, exe_path)| {
            let icon_rgba = extract_icon_rgba(&exe_path);
            ProcessInfo {
                name,
                exe_name,
                exe_path,
                icon_rgba,
            }
        })
        .collect();

    result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    result
}
```

**Step 3: 在 `packages/desktop/src/main.rs` 添加模块声明**

在 `mod launcher;` 后添加：

```rust
mod process;
```

**Step 4: 运行 cargo check 验证**

Run: `cargo check -p desktop`
Expected: 可能因 ui crate 的 process_picker 模块不存在而失败。先创建一个空的占位文件：

创建 `packages/ui/src/process_picker.rs`（空文件，后续 Task 3 填充）：

```rust
// 占位，Task 3 实现
```

然后再次：
Run: `cargo check -p desktop`
Expected: PASS

**Step 5: Commit**

```bash
git add packages/desktop/Cargo.toml packages/desktop/src/process.rs packages/desktop/src/main.rs packages/ui/src/process_picker.rs
git commit -m "feat: 实现 Win32 进程枚举和图标提取"
```

---

## Task 3: 创建 ProcessPicker 弹窗组件

**Files:**
- Modify: `packages/ui/src/process_picker.rs`（替换占位内容）

**Step 1: 实现 ProcessPicker 组件**

```rust
use base64::Engine;
use dioxus::prelude::*;

use crate::ProcessInfo;

/// 将 32x32 RGBA 像素数据编码为 BMP 格式的 base64 data URI
fn rgba_to_bmp_data_uri(rgba: &[u8]) -> String {
    let size: u32 = 32;
    let pixel_data_size = (size * size * 4) as u32;
    let file_size = 14 + 40 + pixel_data_size; // BITMAPFILEHEADER + BITMAPINFOHEADER + pixels

    let mut bmp = Vec::with_capacity(file_size as usize);

    // BITMAPFILEHEADER (14 bytes)
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&file_size.to_le_bytes());
    bmp.extend_from_slice(&[0u8; 4]); // reserved
    bmp.extend_from_slice(&(14u32 + 40).to_le_bytes()); // pixel data offset

    // BITMAPINFOHEADER (40 bytes)
    bmp.extend_from_slice(&40u32.to_le_bytes()); // header size
    bmp.extend_from_slice(&size.to_le_bytes()); // width
    bmp.extend_from_slice(&(-(size as i32)).to_le_bytes()); // height (negative = top-down)
    bmp.extend_from_slice(&1u16.to_le_bytes()); // planes
    bmp.extend_from_slice(&32u16.to_le_bytes()); // bits per pixel
    bmp.extend_from_slice(&[0u8; 24]); // compression + rest (all zeros for BI_RGB)

    // Pixel data: convert RGBA → BGRA for BMP
    for chunk in rgba.chunks_exact(4) {
        bmp.push(chunk[2]); // B
        bmp.push(chunk[1]); // G
        bmp.push(chunk[0]); // R
        bmp.push(chunk[3]); // A
    }

    let b64 = base64::engine::general_purpose::STANDARD.encode(&bmp);
    format!("data:image/bmp;base64,{b64}")
}

#[component]
pub fn ProcessPicker(
    processes: Vec<ProcessInfo>,
    loading: bool,
    on_select: EventHandler<ProcessInfo>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut search = use_signal(String::new);

    let filtered: Vec<&ProcessInfo> = processes
        .iter()
        .filter(|p| {
            let q = search().to_lowercase();
            if q.is_empty() {
                return true;
            }
            p.name.to_lowercase().contains(&q) || p.exe_path.to_lowercase().contains(&q)
        })
        .collect();

    rsx! {
        // 遮罩层
        div {
            class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
            onclick: move |_| on_cancel.call(()),

            // 弹窗
            div {
                class: "bg-bg-card rounded-xl shadow-2xl border border-border-subtle w-[520px] max-h-[70vh] flex flex-col",
                onclick: move |e| e.stop_propagation(),

                // 标题栏
                div { class: "flex items-center justify-between px-5 pt-5 pb-3",
                    h2 { class: "text-lg font-semibold text-text-primary", "从运行中的程序选择" }
                    button {
                        class: "p-1 text-text-muted hover:text-text-primary hover:bg-bg-hover rounded-md transition-colors cursor-pointer",
                        onclick: move |_| on_cancel.call(()),
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "18",
                            height: "18",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            path { d: "M18 6 6 18" }
                            path { d: "m6 6 12 12" }
                        }
                    }
                }

                // 搜索框
                div { class: "px-5 pb-3",
                    div { class: "relative",
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "16",
                            height: "16",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            class: "absolute left-3 top-1/2 -translate-y-1/2 text-text-muted pointer-events-none",
                            circle { cx: "11", cy: "11", r: "8" }
                            path { d: "m21 21-4.3-4.3" }
                        }
                        input {
                            r#type: "text",
                            class: "w-full bg-bg-input border border-border-default rounded-lg pl-9 pr-3 py-2 text-sm text-text-primary placeholder-text-muted focus:border-accent focus:outline-none transition-colors",
                            placeholder: "搜索程序...",
                            value: "{search}",
                            oninput: move |e| search.set(e.value()),
                        }
                    }
                }

                // 进程列表
                div { class: "flex-1 overflow-y-auto px-2 pb-4",
                    if loading {
                        div { class: "flex items-center justify-center py-12 text-text-muted text-sm",
                            "正在获取运行中的程序..."
                        }
                    } else if filtered.is_empty() {
                        div { class: "flex items-center justify-center py-12 text-text-muted text-sm",
                            if search().is_empty() {
                                "未找到运行中的程序"
                            } else {
                                "没有匹配的程序"
                            }
                        }
                    } else {
                        for process in filtered {
                            {
                                let p = process.clone();
                                rsx! {
                                    button {
                                        key: "{p.exe_path}",
                                        class: "w-full flex items-center gap-3 px-3 py-2.5 rounded-lg hover:bg-bg-hover transition-colors cursor-pointer text-left",
                                        onclick: move |_| on_select.call(p.clone()),

                                        // 图标
                                        if let Some(ref icon) = process.icon_rgba {
                                            img {
                                                src: "{rgba_to_bmp_data_uri(icon)}",
                                                width: "24",
                                                height: "24",
                                                class: "shrink-0 rounded",
                                            }
                                        } else {
                                            // 默认占位图标
                                            div { class: "w-6 h-6 shrink-0 rounded bg-accent/20 flex items-center justify-center",
                                                svg {
                                                    xmlns: "http://www.w3.org/2000/svg",
                                                    width: "14",
                                                    height: "14",
                                                    view_box: "0 0 24 24",
                                                    fill: "none",
                                                    stroke: "currentColor",
                                                    stroke_width: "2",
                                                    class: "text-accent",
                                                    rect { x: "2", y: "3", width: "20", height: "14", rx: "2" }
                                                    path { d: "M8 21h8" }
                                                    path { d: "M12 17v4" }
                                                }
                                            }
                                        }

                                        // 名称和路径
                                        div { class: "flex-1 min-w-0",
                                            div { class: "text-sm font-medium text-text-primary truncate", "{process.name}" }
                                            div { class: "text-xs text-text-muted truncate", "{process.exe_path}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

**Step 2: 运行 cargo check 验证**

Run: `cargo check -p ui`
Expected: PASS

**Step 3: Commit**

```bash
git add packages/ui/src/process_picker.rs
git commit -m "feat: 实现 ProcessPicker 进程选择弹窗组件"
```

---

## Task 4: ShortcutForm 添加"从运行程序选择"按钮

**Files:**
- Modify: `packages/ui/src/shortcut_form.rs`

**Step 1: 为 ShortcutForm 添加 on_pick_process 回调 prop**

在 `on_cancel: EventHandler<()>,` 后添加新 prop：

```rust
#[component]
pub fn ShortcutForm(
    initial: ShortcutFormData,
    conflict_message: Option<String>,
    on_save: EventHandler<ShortcutFormData>,
    on_cancel: EventHandler<()>,
    on_pick_process: EventHandler<()>,
) -> Element {
```

**Step 2: 在"浏览..."按钮之后添加"从运行程序选择"按钮**

在 `shortcut_form.rs` 中"浏览..."按钮的 `}` 之后，`}` (关闭 `div { class: "flex gap-2"`) 之前添加：

```rust
                        button {
                            class: "px-3 py-2 bg-bg-hover border border-border-default text-text-secondary rounded-lg hover:bg-border-default hover:text-text-primary transition-colors cursor-pointer text-sm font-medium shrink-0",
                            onclick: move |_| on_pick_process.call(()),
                            "选择运行中的程序"
                        }
```

**Step 3: 运行 cargo check 验证**

Run: `cargo check -p ui`
Expected: PASS

**Step 4: Commit**

```bash
git add packages/ui/src/shortcut_form.rs
git commit -m "feat: ShortcutForm 添加'选择运行中的程序'按钮"
```

---

## Task 5: 在 home.rs 中集成 ProcessPicker

**Files:**
- Modify: `packages/desktop/src/views/home.rs`

**Step 1: 添加导入**

在文件顶部的 import 区域添加：

```rust
use ui::{Navbar, ProcessPicker, ShortcutForm, ShortcutFormData, ShortcutList, ShortcutRow, ProcessInfo};
```

（替换原来的 `use ui::{Navbar, ShortcutForm, ShortcutFormData, ShortcutList, ShortcutRow};`）

**Step 2: 添加 ProcessPicker 相关状态**

在 `let mut delete_confirm = use_signal(...)` 之后添加：

```rust
    let mut show_process_picker = use_signal(|| false);
    let mut process_list = use_signal(Vec::<ProcessInfo>::new);
    let mut process_loading = use_signal(|| false);
```

**Step 3: 为 ShortcutForm 添加 on_pick_process 回调**

在现有 `ShortcutForm` 调用中添加新的 prop：

```rust
                ShortcutForm {
                    initial: form_data(),
                    conflict_message: conflict_msg(),
                    on_save: move |data: ShortcutFormData| {
                        // ... 现有保存逻辑不变 ...
                    },
                    on_cancel: move |_| show_form.set(false),
                    on_pick_process: move |_| {
                        show_process_picker.set(true);
                        process_loading.set(true);
                        spawn(async move {
                            let processes = tokio::task::spawn_blocking(|| {
                                crate::process::list_windowed_processes()
                            }).await.unwrap_or_default();
                            process_list.set(processes);
                            process_loading.set(false);
                        });
                    },
                }
```

**Step 4: 在 ShortcutForm 弹窗之后添加 ProcessPicker 弹窗**

在 `if show_form()` 块之后（文件末尾 `}` 之前）添加：

```rust
            // ── 进程选择弹窗 ──
            if show_process_picker() {
                ProcessPicker {
                    processes: process_list(),
                    loading: process_loading(),
                    on_select: move |info: ProcessInfo| {
                        exe_path 无法直接访问（在 ShortcutForm 内部），
                        所以需要通过 form_data signal 更新：
                        form_data.set(ShortcutFormData {
                            id: form_data().id,
                            name: info.name.clone(),
                            exe_name: info.exe_name.clone(),
                            exe_path: info.exe_path.clone(),
                            modifier: form_data().modifier,
                            hotkey: form_data().hotkey,
                        });
                        show_process_picker.set(false);
                        // 重新打开表单以反映更新（如果表单已关闭）
                        show_form.set(true);
                    },
                    on_cancel: move |_| {
                        show_process_picker.set(false);
                    },
                }
            }
```

**注意**：由于 ShortcutForm 内部的 `name`/`exe_name`/`exe_path` 是组件内 signal，外部无法直接修改。正确做法是更新 `form_data` 后重建 ShortcutForm。但 ShortcutForm 使用 `use_signal(|| initial.xxx.clone())` 初始化，只在首次渲染时读取 initial。

**因此需要调整策略**：当用户从 ProcessPicker 选择后，先关闭 ShortcutForm，更新 form_data，再重新打开 ShortcutForm。

修改 on_select 回调为：

```rust
                    on_select: move |info: ProcessInfo| {
                        // 关闭表单再重新打开，使 ShortcutForm 重新初始化
                        show_form.set(false);
                        form_data.set(ShortcutFormData {
                            id: form_data().id,
                            name: info.name.clone(),
                            exe_name: info.exe_name.clone(),
                            exe_path: info.exe_path.clone(),
                            modifier: form_data().modifier,
                            hotkey: form_data().hotkey,
                        });
                        show_process_picker.set(false);
                        // 延迟重新打开表单
                        spawn(async move {
                            show_form.set(true);
                        });
                    },
```

**Step 5: 运行 cargo check 验证**

Run: `cargo check -p desktop`
Expected: PASS

**Step 6: 运行 cargo clippy 验证**

Run: `cargo clippy -p desktop`
Expected: 无 warning

**Step 7: Commit**

```bash
git add packages/desktop/src/views/home.rs
git commit -m "feat: Home 集成 ProcessPicker，完成进程选择功能闭环"
```

---

## Task 6: 端到端手动验证

**Step 1: 启动应用**

Run: `cd packages/desktop && dx serve`

**Step 2: 验证功能流程**

1. 点击"添加"打开快捷键表单
2. 确认"浏览..."按钮旁有"选择运行中的程序"按钮
3. 点击"选择运行中的程序"按钮
4. 确认弹窗出现，显示当前运行的有窗口程序
5. 确认每个程序显示：图标、名称、exe 路径
6. 在搜索框输入关键字，确认列表实时过滤
7. 点击某个程序，确认表单自动填充 name、exe_name、exe_path
8. 确认可以继续编辑快捷键并保存

**Step 3: 验证边缘情况**

1. 点击弹窗外部区域或 ✕ 按钮，确认可以关闭
2. 搜索不存在的程序名，确认显示"没有匹配的程序"
3. 确认暗色和亮色主题下弹窗样式正确

**Step 4: Final commit**

```bash
git commit -m "feat: 完成'从运行程序选择'功能"
```
