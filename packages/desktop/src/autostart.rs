use std::env;
use windows::core::HSTRING;
use windows::Win32::System::Registry::{
    RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE,
    REG_SZ,
};

const APP_NAME: &str = "WinAide";
const REG_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

/// 设置开机自启
pub fn set_autostart(enabled: bool) {
    if enabled {
        enable_autostart();
    } else {
        disable_autostart();
    }
}

fn enable_autostart() {
    unsafe {
        let mut hkey = HKEY::default();
        let path = HSTRING::from(REG_PATH);

        if RegOpenKeyExW(HKEY_CURRENT_USER, &path, 0, KEY_SET_VALUE, &mut hkey).is_ok() {
            let exe_path = env::current_exe()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let name = HSTRING::from(APP_NAME);
            let value = HSTRING::from(&exe_path);
            let bytes: &[u8] = std::slice::from_raw_parts(
                value.as_ptr() as *const u8,
                (value.len() + 1) * 2, // 包含 null terminator
            );

            let _ = RegSetValueExW(hkey, &name, 0, REG_SZ, Some(bytes));
        }
    }
}

fn disable_autostart() {
    unsafe {
        let mut hkey = HKEY::default();
        let path = HSTRING::from(REG_PATH);

        if RegOpenKeyExW(HKEY_CURRENT_USER, &path, 0, KEY_SET_VALUE, &mut hkey).is_ok() {
            let name = HSTRING::from(APP_NAME);
            let _ = RegDeleteValueW(hkey, &name);
        }
    }
}
