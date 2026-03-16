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

/// 从嵌入的 PNG 创建托盘图标
fn create_icon_from_png() -> Icon {
    let png_bytes = include_bytes!("../assets/logo.png");
    let img = image::load_from_memory_with_format(png_bytes, image::ImageFormat::Png)
        .expect("无法加载 logo.png")
        .into_rgba8();
    let (w, h) = img.dimensions();
    Icon::from_rgba(img.into_raw(), w, h).expect("无法创建托盘图标")
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
        .with_icon(create_icon_from_png())
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
