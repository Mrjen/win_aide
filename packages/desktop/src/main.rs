use dioxus::prelude::*;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

mod autostart;
mod config;
mod hotkey;
mod launcher;
mod tray;
mod views;

use config::AppConfig;
use views::Home;

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

/// 通过 context 共享的后台通信通道
#[derive(Clone)]
pub struct BackendChannels {
    pub hotkey_tx: Arc<Mutex<Sender<hotkey::HotkeyCommand>>>,
    pub launcher_tx: Arc<Mutex<Sender<Vec<config::Shortcut>>>>,
}

static BACKEND_CHANNELS: std::sync::Mutex<Option<BackendChannels>> = std::sync::Mutex::new(None);

fn main() {
    let initial_config = config::load_config();

    // 同步开机自启状态
    autostart::set_autostart(initial_config.settings.auto_start);

    // 启动快捷键监听线程
    let (hotkey_cmd_tx, hotkey_event_rx) = hotkey::start_hotkey_listener();
    let _ = hotkey_cmd_tx.send(hotkey::HotkeyCommand::RegisterAll(
        initial_config.shortcuts.clone(),
    ));

    // 启动 launcher 处理线程
    let launcher_update_tx = launcher::start_launcher(
        hotkey_event_rx,
        initial_config.shortcuts.clone(),
    );

    // 创建系统托盘（必须在 main 线程中创建）
    let _tray = tray::create_tray();

    // 通过全局状态传递 channels 给 Dioxus 组件
    BACKEND_CHANNELS
        .lock()
        .unwrap()
        .replace(BackendChannels {
            hotkey_tx: Arc::new(Mutex::new(hotkey_cmd_tx)),
            launcher_tx: Arc::new(Mutex::new(launcher_update_tx)),
        });

    dioxus::LaunchBuilder::new()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_window(
                    dioxus::desktop::tao::window::WindowBuilder::new()
                        .with_title("Win Aide")
                        .with_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(800, 600)),
                )
                .with_close_behaviour(dioxus::desktop::WindowCloseBehaviour::WindowHides)
                .with_menu(None),
        )
        .launch(App);
}

#[component]
fn App() -> Element {
    let channels = use_context_provider(|| {
        BACKEND_CHANNELS.lock().unwrap().take().expect("BackendChannels 未初始化")
    });

    let mut config = use_signal(|| config::load_config());

    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        div { class: "bg-bg-primary text-white font-sans min-h-screen",
            Home {
                config: config,
                on_config_changed: move |new_config: AppConfig| {
                    config.set(new_config.clone());

                    // 同步开机自启状态
                    autostart::set_autostart(new_config.settings.auto_start);

                    // 通知快捷键线程更新注册
                    if let Ok(tx) = channels.hotkey_tx.lock() {
                        let _ = tx.send(hotkey::HotkeyCommand::RegisterAll(
                            new_config.shortcuts.clone(),
                        ));
                    }

                    // 通知 launcher 线程更新配置
                    if let Ok(tx) = channels.launcher_tx.lock() {
                        let _ = tx.send(new_config.shortcuts);
                    }
                },
            }
        }
    }
}
