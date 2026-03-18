//! 跨平台共享 UI 组件库

pub use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Modifier {
    Alt,
    Ctrl,
    Win,
}

impl Modifier {
    pub fn display_name(&self) -> &str {
        match self {
            Modifier::Alt => "Alt",
            Modifier::Ctrl => "Ctrl",
            Modifier::Win => "Win",
        }
    }
}

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

mod navbar;
pub use navbar::Navbar;

mod shortcut_list;
pub use shortcut_list::{ShortcutList, ShortcutRow};

mod shortcut_form;
pub use shortcut_form::{ShortcutForm, ShortcutFormData};

mod process_picker;
pub use process_picker::{ProcessPicker, rgba_to_png_data_uri};

mod update_dialog;
pub use update_dialog::{UpdateDialog, UpdateDialogState};
