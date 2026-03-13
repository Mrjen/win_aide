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

mod navbar;
pub use navbar::Navbar;

mod shortcut_list;
pub use shortcut_list::{ShortcutList, ShortcutRow};
