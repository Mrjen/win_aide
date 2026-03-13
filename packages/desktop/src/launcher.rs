use crate::config::Shortcut;
use crate::hotkey::HotkeyEvent;
use std::collections::HashMap;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::sync::mpsc;
use std::thread;
use windows::Win32::Foundation::{CloseHandle, BOOL, HWND, LPARAM, TRUE};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AllowSetForegroundWindow, EnumWindows, GetWindowThreadProcessId, IsIconic, IsWindowVisible,
    SetForegroundWindow, ShowWindow, ASFW_ANY, SW_RESTORE,
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
                data.found_hwnd = Some(hwnd);
                let _ = CloseHandle(process);
                return BOOL(0); // 停止枚举
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
        let _ = AllowSetForegroundWindow(ASFW_ANY);
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
