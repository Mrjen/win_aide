use crate::config::Shortcut;
use crate::hotkey::HotkeyEvent;
use std::collections::HashMap;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::sync::mpsc;
use std::thread;


use windows::Win32::Foundation::{CloseHandle, BOOL, HWND, LPARAM, TRUE};
use windows::Win32::System::Threading::{
    AttachThreadInput, GetCurrentThreadId, OpenProcess, QueryFullProcessImageNameW,
    PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, EnumWindows, GetForegroundWindow, GetWindowTextLengthW,
    GetWindowThreadProcessId, IsIconic, IsWindowVisible, SetForegroundWindow, ShowWindow,
    SW_MINIMIZE, SW_RESTORE, SW_SHOW,
};

/// 窗口循环方向
pub enum Direction {
    Next,
    Prev,
}

struct FindWindowData {
    exe_name: String,
    /// 可见且有标题的窗口（最优：正常显示的主窗口）
    visible_titled: Option<HWND>,
    /// 不可见但有标题的窗口（次优：最小化到托盘的主窗口）
    hidden_titled: Option<HWND>,
    /// 任意匹配的窗口（兜底：辅助窗口）
    any_match: Option<HWND>,
}

/// EnumWindows 回调：查找匹配 exe_name 的窗口（优先可见窗口，备选不可见窗口）
unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data = &mut *(lparam.0 as *mut FindWindowData);

    let mut process_id: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut process_id));

    if process_id == 0 {
        return TRUE;
    }

    // 打开进程获取可执行文件路径
    if let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) {
        let mut buffer = [0u16; 1024];
        let mut size = buffer.len() as u32;
        if QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_FORMAT(0),
            windows::core::PWSTR(buffer.as_mut_ptr()),
            &mut size,
        )
        .is_ok()
        {
            let path = OsString::from_wide(&buffer[..size as usize]);
            let path_str = path.to_string_lossy().to_lowercase();
            if path_str.ends_with(&data.exe_name.to_lowercase()) {
                let visible = IsWindowVisible(hwnd).as_bool();
                let has_title = GetWindowTextLengthW(hwnd) > 0;

                if visible && has_title {
                    // 可见且有标题 — 最优选择（正常显示的主窗口），立即返回
                    data.visible_titled = Some(hwnd);
                    let _ = CloseHandle(process);
                    return BOOL(0); // 停止枚举
                } else if !visible && has_title {
                    // 不可见但有标题 — 最小化到托盘的主窗口
                    if data.hidden_titled.is_none() {
                        data.hidden_titled = Some(hwnd);
                    }
                } else if data.any_match.is_none() {
                    // 无标题的辅助窗口 — 仅作兜底
                    data.any_match = Some(hwnd);
                }
            }
        }
        let _ = CloseHandle(process);
    }

    TRUE
}

/// 查找运行中的窗口
fn find_window_by_exe(exe_name: &str) -> Option<HWND> {
    let mut data = FindWindowData {
        exe_name: exe_name.to_string(),
        visible_titled: None,
        hidden_titled: None,
        any_match: None,
    };

    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut data as *mut FindWindowData as isize),
        );
    }

    // 优先级：可见有标题 > 隐藏有标题（托盘） > 任意匹配（辅助窗口兜底）
    data.visible_titled.or(data.hidden_titled).or(data.any_match)
}

struct CollectWindowsData {
    target_pid: u32,
    windows: Vec<HWND>,
}

/// EnumWindows 回调：收集指定进程的所有有标题窗口（含隐藏窗口，以支持循环切换到托盘窗口）
unsafe extern "system" fn collect_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data = &mut *(lparam.0 as *mut CollectWindowsData);

    // 跳过无标题的辅助窗口，但保留隐藏窗口（如最小化到托盘的窗口）
    if GetWindowTextLengthW(hwnd) == 0 {
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

        let mut windows = find_all_windows_by_pid(pid);
        if windows.len() <= 1 {
            return;
        }

        // 按 HWND 值排序，确保循环顺序不受 Z-order 影响
        windows.sort_by_key(|w| w.0 as usize);

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

/// 激活已有窗口（支持不可见窗口、最小化窗口、零尺寸窗口）
fn activate_window(hwnd: HWND) {
    unsafe {
        // 处理不可见窗口（如最小化到托盘的窗口）
        if !IsWindowVisible(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_SHOW);
        }

        // 处理最小化窗口或零尺寸窗口（某些应用如飞书通过缩为零尺寸实现托盘最小化）
        let mut rect = windows::Win32::Foundation::RECT::default();
        let _ = windows::Win32::UI::WindowsAndMessaging::GetWindowRect(hwnd, &mut rect);
        let is_zero_size = rect.left == rect.right && rect.top == rect.bottom;
        if IsIconic(hwnd).as_bool() || is_zero_size {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }

        // 使用 AttachThreadInput 绕过 Windows 前台窗口锁定限制
        // 当 win_aide 最小化到托盘时，它不是前台进程，
        // 直接调用 SetForegroundWindow 会被系统拒绝
        let foreground_hwnd = GetForegroundWindow();
        let foreground_tid = GetWindowThreadProcessId(foreground_hwnd, None);
        let current_tid = GetCurrentThreadId();

        let attached = if foreground_tid != 0 && foreground_tid != current_tid {
            AttachThreadInput(current_tid, foreground_tid, true).as_bool()
        } else {
            false
        };

        let _ = SetForegroundWindow(hwnd);
        let _ = BringWindowToTop(hwnd);

        if attached {
            let _ = AttachThreadInput(current_tid, foreground_tid, false);
        }
    }
}

/// 使用 ShellExecuteW 启动指定路径的程序
fn shell_execute(exe_path: &std::path::Path) {
    use windows::core::HSTRING;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let file = HSTRING::from(exe_path.as_os_str());
    let dir = exe_path
        .parent()
        .map(|p| HSTRING::from(p.as_os_str()))
        .unwrap_or_default();

    unsafe {
        ShellExecuteW(
            HWND::default(),
            &HSTRING::from("open"),
            &file,
            None,
            &dir,
            SW_SHOWNORMAL,
        );
    }
}

/// 启动新进程
fn launch_process(exe_path: &str) {
    shell_execute(std::path::Path::new(exe_path));
}

/// 查找启动器路径（处理 Electron/SquirrelSetup 等安装结构）
/// 例如：D:\software\Feishu\app\Feishu.exe → D:\software\Feishu\Feishu.exe
fn find_launcher_path(exe_path: &str) -> Option<std::path::PathBuf> {
    let path = std::path::Path::new(exe_path);
    let exe_name = path.file_name()?;
    let launcher = path.parent()?.parent()?.join(exe_name);
    if launcher.exists() && launcher != *path {
        Some(launcher)
    } else {
        None
    }
}

/// LaunchOrActivate：如果应用已运行则 toggle 窗口（前台→最小化，非前台→激活），否则启动
pub fn launch_or_activate(shortcut: &Shortcut) {
    if let Some(hwnd) = find_window_by_exe(&shortcut.exe_name) {
        // Toggle：如果目标窗口已在前台，最小化到任务栏
        let is_foreground = unsafe { GetForegroundWindow() == hwnd };
        if is_foreground {
            unsafe {
                let _ = ShowWindow(hwnd, SW_MINIMIZE);
            }
            return;
        }

        let was_hidden = unsafe { !IsWindowVisible(hwnd).as_bool() };
        let mut rect = windows::Win32::Foundation::RECT::default();
        unsafe { windows::Win32::UI::WindowsAndMessaging::GetWindowRect(hwnd, &mut rect).ok() };
        let is_zero_size = rect.left == rect.right && rect.top == rect.bottom;

        activate_window(hwnd);

        // 窗口不可见（隐藏或零尺寸）时触发兜底恢复
        // 某些应用（如飞书）通过缩为零尺寸实现托盘最小化，
        // 直接 ShowWindow 可能无法正确恢复，需要通过启动器触发单实例机制
        if was_hidden || is_zero_size {
            if let Some(launcher) = find_launcher_path(&shortcut.exe_path) {
                shell_execute(&launcher);
            } else {
                launch_process(&shortcut.exe_path);
            }
        }
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
        }
    });

    update_tx
}
