use dioxus::prelude::*;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

mod autostart;
mod config;
mod hotkey;
mod launcher;
mod process;
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

thread_local! {
    static TRAY_INSTANCE: std::cell::RefCell<Option<tray::Tray>> = const { std::cell::RefCell::new(None) };
}

fn main() {
    let initial_config = config::load_config();

    // 同步开机自启状态
    autostart::set_autostart(initial_config.settings.auto_start);

    // 启动快捷键监听线程
    let (hotkey_cmd_tx, hotkey_event_rx) = hotkey::start_hotkey_listener();
    let _ = hotkey_cmd_tx.send(hotkey::HotkeyCommand::RegisterAll(
        initial_config.shortcuts.clone(),
    ));

    // 注册窗口循环热键
    if initial_config.settings.window_cycle.enabled {
        let wc = &initial_config.settings.window_cycle;
        let _ = hotkey_cmd_tx.send(hotkey::HotkeyCommand::RegisterWindowCycle {
            modifier: wc.modifier.clone(),
            key: wc.key,
        });
    }

    // 启动 launcher 处理线程
    let launcher_update_tx = launcher::start_launcher(
        hotkey_event_rx,
        initial_config.shortcuts.clone(),
    );

    // 创建系统托盘（必须在 main 线程中创建）
    TRAY_INSTANCE.with(|t| {
        *t.borrow_mut() = Some(tray::create_tray());
    });

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
                        .with_decorations(false)
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

    let mut config = use_signal(config::load_config);
    let mut paused = use_signal(|| false);
    let mut dark_mode = use_signal(|| config().settings.dark_mode);

    // 轮询托盘菜单事件
    let tray_channels = channels.clone();
    use_future(move || {
        let channels = tray_channels.clone();
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                if let Some(event) = tray::poll_tray_event() {
                    match event {
                        tray::TrayEvent::Show => {
                            let window = dioxus::desktop::window();
                            window.set_visible(true);
                            window.set_focus();
                        }
                        tray::TrayEvent::TogglePause => {
                            paused.set(!paused());
                            if paused() {
                                if let Ok(tx) = channels.hotkey_tx.lock() {
                                    let _ = tx.send(hotkey::HotkeyCommand::UnregisterAll);
                                }
                                TRAY_INSTANCE.with(|t| {
                                    if let Some(tray) = t.borrow().as_ref() {
                                        tray.pause_item.set_text("恢复所有快捷键");
                                    }
                                });
                            } else {
                                let cfg = config::load_config();
                                if let Ok(tx) = channels.hotkey_tx.lock() {
                                    let _ = tx.send(hotkey::HotkeyCommand::RegisterAll(cfg.shortcuts));
                                    // 恢复窗口循环热键
                                    if cfg.settings.window_cycle.enabled {
                                        let wc = &cfg.settings.window_cycle;
                                        let _ = tx.send(hotkey::HotkeyCommand::RegisterWindowCycle {
                                            modifier: wc.modifier.clone(),
                                            key: wc.key,
                                        });
                                    }
                                }
                                TRAY_INSTANCE.with(|t| {
                                    if let Some(tray) = t.borrow().as_ref() {
                                        tray.pause_item.set_text("暂停所有快捷键");
                                    }
                                });
                            }
                        }
                        tray::TrayEvent::Quit => {
                            std::process::exit(0);
                        }
                    }
                }
            }
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        div { class: if dark_mode() { "dark bg-bg-primary text-text-primary font-sans min-h-screen" } else { "bg-bg-primary text-text-primary font-sans min-h-screen" },
            Home {
                config: config,
                paused: paused,
                dark_mode: dark_mode,
                on_config_changed: move |new_config: AppConfig| {
                    dark_mode.set(new_config.settings.dark_mode);
                    config.set(new_config.clone());

                    // 同步开机自启状态
                    autostart::set_autostart(new_config.settings.auto_start);

                    // 通知快捷键线程更新注册
                    if let Ok(tx) = channels.hotkey_tx.lock() {
                        let _ = tx.send(hotkey::HotkeyCommand::RegisterAll(
                            new_config.shortcuts.clone(),
                        ));
                        // 同步窗口循环热键
                        let _ = tx.send(hotkey::HotkeyCommand::UnregisterWindowCycle);
                        if new_config.settings.window_cycle.enabled {
                            let wc = &new_config.settings.window_cycle;
                            let _ = tx.send(hotkey::HotkeyCommand::RegisterWindowCycle {
                                modifier: wc.modifier.clone(),
                                key: wc.key,
                            });
                        }
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
