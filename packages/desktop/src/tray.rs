use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};

/// 托盘菜单项 ID
pub const MENU_SHOW: &str = "show";
pub const MENU_PAUSE: &str = "pause";
pub const MENU_CHECK_UPDATE: &str = "check_update";
pub const MENU_QUIT: &str = "quit";

/// 托盘事件
#[derive(Debug, Clone, PartialEq)]
pub enum TrayEvent {
    Show,
    TogglePause,
    CheckUpdate,
    Quit,
}

/// 创建默认图标（简单的彩色方块）
fn create_default_icon() -> Icon {
    // 16x16 RGBA 图标（蓝紫色，与主题 accent 色一致）
    let size = 16u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for _ in 0..size * size {
        rgba.push(0x91); // R
        rgba.push(0xa4); // G
        rgba.push(0xd2); // B
        rgba.push(0xFF); // A
    }
    Icon::from_rgba(rgba, size, size).expect("无法创建托盘图标")
}

/// 托盘实例，包含图标和暂停菜单项引用
pub struct Tray {
    pub _icon: TrayIcon,
    pub pause_item: MenuItem,
}

/// 初始化系统托盘
pub fn create_tray() -> Tray {
    let menu = Menu::new();

    let show_item = MenuItem::with_id(MENU_SHOW, "显示主窗口", true, None);
    let pause_item = MenuItem::with_id(MENU_PAUSE, "暂停所有快捷键", true, None);
    let quit_item = MenuItem::with_id(MENU_QUIT, "退出", true, None);

    let _ = menu.append(&show_item);
    let _ = menu.append(&PredefinedMenuItem::separator());
    let _ = menu.append(&pause_item);
    let check_update_item = MenuItem::with_id(MENU_CHECK_UPDATE, "检查更新", true, None);
    let _ = menu.append(&PredefinedMenuItem::separator());
    let _ = menu.append(&check_update_item);
    let _ = menu.append(&PredefinedMenuItem::separator());
    let _ = menu.append(&quit_item);

    let icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Win Aide - 快捷键启动器")
        .with_icon(create_default_icon())
        .build()
        .expect("无法创建系统托盘");

    Tray {
        _icon: icon,
        pause_item,
    }
}

/// 轮询托盘菜单事件（非阻塞）
pub fn poll_tray_event() -> Option<TrayEvent> {
    if let Ok(event) = MenuEvent::receiver().try_recv() {
        match event.id.0.as_str() {
            MENU_SHOW => Some(TrayEvent::Show),
            MENU_PAUSE => Some(TrayEvent::TogglePause),
            MENU_CHECK_UPDATE => Some(TrayEvent::CheckUpdate),
            MENU_QUIT => Some(TrayEvent::Quit),
            _ => None,
        }
    } else {
        None
    }
}
