use crate::config::{Modifier, Shortcut};
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT,
    MOD_SHIFT, MOD_WIN,
};
use windows::Win32::UI::WindowsAndMessaging::{PeekMessageW, MSG, PM_REMOVE, WM_HOTKEY};

/// 快捷键触发事件
#[derive(Debug, Clone)]
pub enum HotkeyEvent {
    /// 用户快捷键触发
    ShortcutTriggered { shortcut_id: String },
    /// 窗口循环：下一个
    WindowCycleNext,
    /// 窗口循环：上一个
    WindowCyclePrev,
}

/// 发送给快捷键线程的指令
pub enum HotkeyCommand {
    /// 注册一组快捷键（会先注销所有旧的）
    RegisterAll(Vec<Shortcut>),
    /// 注销所有快捷键（暂停）
    UnregisterAll,
    /// 注册窗口循环热键
    RegisterWindowCycle { modifier: Modifier, key: char },
    /// 注销窗口循环热键
    UnregisterWindowCycle,
    /// 停止监听并退出线程
    Shutdown,
}

fn modifier_to_win32(modifier: &Modifier) -> HOT_KEY_MODIFIERS {
    match modifier {
        Modifier::Alt => MOD_ALT | MOD_NOREPEAT,
        Modifier::Ctrl => MOD_CONTROL | MOD_NOREPEAT,
        Modifier::Win => MOD_WIN | MOD_NOREPEAT,
    }
}

const WINDOW_CYCLE_NEXT_ID: i32 = 10001;
const WINDOW_CYCLE_PREV_ID: i32 = 10002;

fn key_to_vk(key: char) -> u32 {
    match key {
        '`' => 0xC0, // VK_OEM_3 (backtick)
        _ => key.to_ascii_uppercase() as u32,
    }
}

/// 启动快捷键监听线程
/// 返回 (command_sender, event_receiver)
pub fn start_hotkey_listener() -> (mpsc::Sender<HotkeyCommand>, mpsc::Receiver<HotkeyEvent>) {
    let (cmd_tx, cmd_rx) = mpsc::channel::<HotkeyCommand>();
    let (event_tx, event_rx) = mpsc::channel::<HotkeyEvent>();

    thread::spawn(move || {
        let mut registered_ids: HashMap<i32, String> = HashMap::new();
        let mut window_cycle_registered = false;

        loop {
            // 检查是否有新指令（非阻塞）
            if let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    HotkeyCommand::RegisterAll(shortcuts) => {
                        // 先注销所有旧快捷键
                        for &id in registered_ids.keys() {
                            unsafe {
                                let _ = UnregisterHotKey(HWND::default(), id);
                            }
                        }
                        registered_ids.clear();

                        // 注册新快捷键
                        let mut next_id: i32 = 1;
                        for shortcut in &shortcuts {
                            if !shortcut.enabled {
                                continue;
                            }
                            let id = next_id;
                            next_id += 1;
                            let modifiers = modifier_to_win32(&shortcut.modifier);
                            let vk = key_to_vk(shortcut.key);

                            unsafe {
                                if RegisterHotKey(HWND::default(), id, modifiers, vk).is_ok() {
                                    registered_ids.insert(id, shortcut.id.clone());
                                }
                            }
                        }
                    }
                    HotkeyCommand::UnregisterAll => {
                        for &id in registered_ids.keys() {
                            unsafe {
                                let _ = UnregisterHotKey(HWND::default(), id);
                            }
                        }
                        registered_ids.clear();
                        if window_cycle_registered {
                            unsafe {
                                let _ = UnregisterHotKey(HWND::default(), WINDOW_CYCLE_NEXT_ID);
                                let _ = UnregisterHotKey(HWND::default(), WINDOW_CYCLE_PREV_ID);
                            }
                            window_cycle_registered = false;
                        }
                    }
                    HotkeyCommand::Shutdown => {
                        // 注销所有快捷键
                        for &id in registered_ids.keys() {
                            unsafe {
                                let _ = UnregisterHotKey(HWND::default(), id);
                            }
                        }
                        if window_cycle_registered {
                            unsafe {
                                let _ = UnregisterHotKey(HWND::default(), WINDOW_CYCLE_NEXT_ID);
                                let _ = UnregisterHotKey(HWND::default(), WINDOW_CYCLE_PREV_ID);
                            }
                        }
                        break;
                    }
                    HotkeyCommand::RegisterWindowCycle { modifier, key } => {
                        if window_cycle_registered {
                            unsafe {
                                let _ = UnregisterHotKey(HWND::default(), WINDOW_CYCLE_NEXT_ID);
                                let _ = UnregisterHotKey(HWND::default(), WINDOW_CYCLE_PREV_ID);
                            }
                        }
                        let mod_flags = modifier_to_win32(&modifier);
                        let vk = key_to_vk(key);
                        unsafe {
                            let next_ok = RegisterHotKey(
                                HWND::default(),
                                WINDOW_CYCLE_NEXT_ID,
                                mod_flags,
                                vk,
                            )
                            .is_ok();
                            let prev_ok = RegisterHotKey(
                                HWND::default(),
                                WINDOW_CYCLE_PREV_ID,
                                mod_flags | MOD_SHIFT,
                                vk,
                            )
                            .is_ok();
                            window_cycle_registered = next_ok || prev_ok;
                        }
                    }
                    HotkeyCommand::UnregisterWindowCycle => {
                        if window_cycle_registered {
                            unsafe {
                                let _ = UnregisterHotKey(HWND::default(), WINDOW_CYCLE_NEXT_ID);
                                let _ = UnregisterHotKey(HWND::default(), WINDOW_CYCLE_PREV_ID);
                            }
                            window_cycle_registered = false;
                        }
                    }
                }
            }

            // 检查快捷键消息（非阻塞）
            let mut msg = MSG::default();
            unsafe {
                if PeekMessageW(&mut msg, HWND::default(), WM_HOTKEY, WM_HOTKEY, PM_REMOVE)
                    .as_bool()
                {
                    if msg.message == WM_HOTKEY {
                        let hotkey_id = msg.wParam.0 as i32;
                        match hotkey_id {
                            WINDOW_CYCLE_NEXT_ID => {
                                let _ = event_tx.send(HotkeyEvent::WindowCycleNext);
                            }
                            WINDOW_CYCLE_PREV_ID => {
                                let _ = event_tx.send(HotkeyEvent::WindowCyclePrev);
                            }
                            _ => {
                                if let Some(shortcut_id) = registered_ids.get(&hotkey_id) {
                                    let _ = event_tx.send(HotkeyEvent::ShortcutTriggered {
                                        shortcut_id: shortcut_id.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
            }

            // 短暂休眠避免 CPU 空转
            thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    (cmd_tx, event_rx)
}
