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
    /// exe_path (lowercase) -> (name, exe_name, exe_path_original)
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
            biCompression: BI_RGB.0,
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
