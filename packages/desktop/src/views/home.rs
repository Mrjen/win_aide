use dioxus::prelude::*;
use ui::{Navbar, ProcessInfo, ProcessPicker, ShortcutForm, ShortcutFormData, ShortcutList, ShortcutRow};

use crate::config::{self, AppConfig, Shortcut};

#[component]
pub fn Home(
    config: Signal<AppConfig>,
    paused: Signal<bool>,
    dark_mode: Signal<bool>,
    on_config_changed: EventHandler<AppConfig>,
) -> Element {
    let mut show_form = use_signal(|| false);
    let mut editing_id = use_signal(|| None::<String>);
    let mut conflict_msg = use_signal(|| None::<String>);
    let mut form_data = use_signal(ShortcutFormData::default);
    let mut show_settings = use_signal(|| false);
    let mut delete_confirm = use_signal(|| None::<String>);
    let mut show_process_picker = use_signal(|| false);
    let mut process_list = use_signal(Vec::<ProcessInfo>::new);
    let mut process_loading = use_signal(|| false);
    let mut form_key = use_signal(|| 0u32);

    // 将 config shortcuts 转换为 UI 行
    let rows: Vec<ShortcutRow> = config()
        .shortcuts
        .iter()
        .map(|s| ShortcutRow {
            id: s.id.clone(),
            name: s.name.clone(),
            exe_name: s.exe_name.clone(),
            exe_path: s.exe_path.clone(),
            modifier: s.modifier.display_name().to_string(),
            hotkey: s.key,
            enabled: s.enabled && !paused(),
        })
        .collect();

    let shortcut_count = config().shortcuts.len();
    let enabled_count = config().shortcuts.iter().filter(|s| s.enabled).count();

    let save_and_notify = move |new_config: AppConfig| {
        config::save_config(&new_config);
        on_config_changed.call(new_config);
    };

    rsx! {
        div { class: "flex flex-col h-screen",
            // ── 顶部工具栏 ──
            Navbar {
                div { class: "flex items-center gap-3",
                    // App icon
                    div { class: "w-8 h-8 bg-accent/10 rounded-lg flex items-center justify-center",
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "18",
                            height: "18",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            class: "text-accent",
                            // keyboard icon
                            rect { x: "2", y: "4", width: "20", height: "16", rx: "2" }
                            path { d: "M6 8h.001" }
                            path { d: "M10 8h.001" }
                            path { d: "M14 8h.001" }
                            path { d: "M18 8h.001" }
                            path { d: "M6 12h.001" }
                            path { d: "M10 12h.001" }
                            path { d: "M14 12h.001" }
                            path { d: "M18 12h.001" }
                            path { d: "M8 16h8" }
                        }
                    }
                    div {
                        h1 { class: "text-base font-semibold text-text-primary leading-tight", "Win Aide" }
                        span { class: "text-xs text-text-muted", "快捷键启动器" }
                    }
                }
                div { class: "flex items-center gap-1.5",
                    // 添加快捷键按钮
                    button {
                        class: "inline-flex items-center gap-1.5 px-3 py-1.5 bg-accent text-white rounded-md hover:bg-accent-focus transition-colors cursor-pointer text-sm font-medium",
                        onclick: move |_| {
                            form_data.set(ShortcutFormData::default());
                            editing_id.set(None);
                            conflict_msg.set(None);
                            show_form.set(true);
                        },
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "14",
                            height: "14",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2.5",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            path { d: "M12 5v14" }
                            path { d: "M5 12h14" }
                        }
                        "添加"
                    }
                    // 分隔线
                    div { class: "w-px h-5 bg-border-default mx-1" }
                    // 主题切换按钮
                    button {
                        class: "p-1.5 text-text-muted hover:text-text-primary hover:bg-bg-hover rounded-md transition-colors cursor-pointer",
                        title: if dark_mode() { "切换到亮色模式" } else { "切换到暗色模式" },
                        onclick: move |_| {
                            let mut cfg = config();
                            cfg.settings.dark_mode = !cfg.settings.dark_mode;
                            save_and_notify(cfg);
                        },
                        if dark_mode() {
                            // sun icon
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                width: "16",
                                height: "16",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                circle { cx: "12", cy: "12", r: "4" }
                                path { d: "M12 2v2" }
                                path { d: "M12 20v2" }
                                path { d: "m4.93 4.93 1.41 1.41" }
                                path { d: "m17.66 17.66 1.41 1.41" }
                                path { d: "M2 12h2" }
                                path { d: "M20 12h2" }
                                path { d: "m6.34 17.66-1.41 1.41" }
                                path { d: "m19.07 4.93-1.41 1.41" }
                            }
                        } else {
                            // moon icon
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                width: "16",
                                height: "16",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                path { d: "M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" }
                            }
                        }
                    }
                    // 设置按钮
                    button {
                        class: if show_settings() {
                            "p-1.5 text-accent bg-accent-subtle rounded-md transition-colors cursor-pointer"
                        } else {
                            "p-1.5 text-text-muted hover:text-text-primary hover:bg-bg-hover rounded-md transition-colors cursor-pointer"
                        },
                        title: "设置",
                        onclick: move |_| show_settings.set(!show_settings()),
                        // settings/gear icon
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "16",
                            height: "16",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            path { d: "M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" }
                            circle { cx: "12", cy: "12", r: "3" }
                        }
                    }
                }
            }

            // ── 设置面板 ──
            if show_settings() {
                div { class: "px-5 py-4 bg-bg-card border-b border-border-default",
                    div { class: "flex flex-wrap gap-x-8 gap-y-3",
                        label { class: "inline-flex items-center gap-2.5 text-sm text-text-secondary cursor-pointer select-none",
                            input {
                                r#type: "checkbox",
                                checked: config().settings.auto_start,
                                onchange: move |_| {
                                    let mut cfg = config();
                                    cfg.settings.auto_start = !cfg.settings.auto_start;
                                    save_and_notify(cfg);
                                },
                            }
                            "开机自启"
                        }
                        label { class: "inline-flex items-center gap-2.5 text-sm text-text-secondary cursor-pointer select-none",
                            input {
                                r#type: "checkbox",
                                checked: config().settings.start_minimized,
                                onchange: move |_| {
                                    let mut cfg = config();
                                    cfg.settings.start_minimized = !cfg.settings.start_minimized;
                                    save_and_notify(cfg);
                                },
                            }
                            "启动时最小化到托盘"
                        }
                        label { class: "inline-flex items-center gap-2.5 text-sm text-text-secondary cursor-pointer select-none",
                            input {
                                r#type: "checkbox",
                                checked: config().settings.dark_mode,
                                onchange: move |_| {
                                    let mut cfg = config();
                                    cfg.settings.dark_mode = !cfg.settings.dark_mode;
                                    save_and_notify(cfg);
                                },
                            }
                            "暗色模式"
                        }
                    }
                }
            }

            // ── 暂停状态横幅 ──
            if paused() {
                div { class: "px-5 py-2.5 bg-warning-bg border-b border-warning-border text-warning-text text-sm flex items-center gap-2",
                    svg {
                        xmlns: "http://www.w3.org/2000/svg",
                        width: "16",
                        height: "16",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        class: "shrink-0",
                        circle { cx: "12", cy: "12", r: "10" }
                        path { d: "M10 15V9" }
                        path { d: "M14 15V9" }
                    }
                    span { "所有快捷键已暂停 — 通过托盘菜单恢复" }
                }
            }

            // ── 快捷键列表 ──
            div { class: "flex-1 overflow-y-auto",
                ShortcutList {
                    shortcuts: rows,
                    on_toggle: move |id: String| {
                        let mut cfg = config();
                        if let Some(s) = cfg.shortcuts.iter_mut().find(|s| s.id == id) {
                            s.enabled = !s.enabled;
                        }
                        save_and_notify(cfg);
                    },
                    on_edit: move |id: String| {
                        let cfg = config();
                        if let Some(s) = cfg.shortcuts.iter().find(|s| s.id == id) {
                            form_data.set(ShortcutFormData {
                                id: Some(s.id.clone()),
                                name: s.name.clone(),
                                exe_name: s.exe_name.clone(),
                                exe_path: s.exe_path.clone(),
                                modifier: s.modifier.clone(),
                                hotkey: s.key.to_string(),
                            });
                            editing_id.set(Some(s.id.clone()));
                            conflict_msg.set(None);
                            show_form.set(true);
                        }
                    },
                    on_delete: move |id: String| {
                        delete_confirm.set(Some(id));
                    },
                }
            }

            // ── 底部状态栏 ──
            div { class: "px-5 py-2 bg-bg-card border-t border-border-default flex items-center justify-between text-xs text-text-muted",
                span { "共 {shortcut_count} 个快捷键，{enabled_count} 个已启用" }
                if paused() {
                    span { class: "text-warning-text font-medium", "已暂停" }
                }
            }

            // ── 删除确认对话框 ──
            if let Some(del_id) = delete_confirm() {
                div {
                    class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
                    onclick: move |_| delete_confirm.set(None),
                    div {
                        class: "bg-bg-card rounded-xl p-6 w-[380px] shadow-2xl border border-border-subtle",
                        onclick: move |e| e.stop_propagation(),
                        // 危险图标
                        div { class: "w-10 h-10 rounded-full bg-danger-subtle flex items-center justify-center mb-4",
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                width: "20",
                                height: "20",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                class: "text-danger",
                                path { d: "m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z" }
                                path { d: "M12 9v4" }
                                path { d: "M12 17h.01" }
                            }
                        }
                        h3 { class: "text-lg font-semibold text-text-primary mb-2", "确认删除" }
                        p { class: "text-text-secondary text-sm mb-6 leading-relaxed", "确定要删除这个快捷键配置吗？此操作不可撤销。" }
                        div { class: "flex justify-end gap-2",
                            button {
                                class: "px-4 py-2 text-sm text-text-secondary hover:text-text-primary hover:bg-bg-hover rounded-lg transition-colors cursor-pointer",
                                onclick: move |_| delete_confirm.set(None),
                                "取消"
                            }
                            button {
                                class: "px-4 py-2 text-sm bg-danger text-white rounded-lg hover:bg-danger-hover transition-colors cursor-pointer font-medium",
                                onclick: move |_| {
                                    let mut cfg = config();
                                    cfg.shortcuts.retain(|s| s.id != del_id);
                                    save_and_notify(cfg);
                                    delete_confirm.set(None);
                                },
                                "删除"
                            }
                        }
                    }
                }
            }

            // ── 新增/编辑弹窗 ──
            if show_form() {
                ShortcutForm {
                    key: "{form_key}",
                    initial: form_data(),
                    conflict_message: conflict_msg(),
                    on_save: move |data: ShortcutFormData| {
                        if data.name.is_empty() || data.exe_path.is_empty() || data.hotkey.is_empty() {
                            conflict_msg.set(Some("请填写所有必填字段".to_string()));
                            return;
                        }
                        let key_char = data.hotkey.chars().next().unwrap();

                        let cfg = config();
                        if config::has_conflict(&cfg.shortcuts, &data.modifier, key_char, data.id.as_deref()) {
                            conflict_msg.set(Some(format!("快捷键 {} + {} 已被占用", data.modifier.display_name(), key_char)));
                            return;
                        }

                        let mut cfg = config();
                        if let Some(edit_id) = &data.id {
                            if let Some(s) = cfg.shortcuts.iter_mut().find(|s| &s.id == edit_id) {
                                s.name = data.name;
                                s.exe_name = data.exe_name;
                                s.exe_path = data.exe_path;
                                s.modifier = data.modifier;
                                s.key = key_char;
                            }
                        } else {
                            cfg.shortcuts.push(Shortcut {
                                id: uuid::Uuid::new_v4().to_string(),
                                name: data.name,
                                exe_name: data.exe_name,
                                exe_path: data.exe_path,
                                modifier: data.modifier,
                                key: key_char,
                                enabled: true,
                            });
                        }
                        save_and_notify(cfg);
                        show_form.set(false);
                    },
                    on_cancel: move |_| show_form.set(false),
                    on_pick_process: move |_| {
                        show_process_picker.set(true);
                        process_loading.set(true);
                        spawn(async move {
                            let processes = tokio::task::spawn_blocking(|| {
                                crate::process::list_windowed_processes()
                            }).await.unwrap_or_default();
                            process_list.set(processes);
                            process_loading.set(false);
                        });
                    },
                }
            }

            // ── 进程选择弹窗 ──
            if show_process_picker() {
                ProcessPicker {
                    processes: process_list(),
                    loading: process_loading(),
                    on_select: move |info: ProcessInfo| {
                        form_data.set(ShortcutFormData {
                            id: form_data().id,
                            name: info.name.clone(),
                            exe_name: info.exe_name.clone(),
                            exe_path: info.exe_path.clone(),
                            modifier: form_data().modifier,
                            hotkey: form_data().hotkey,
                        });
                        show_process_picker.set(false);
                        // 递增 key 强制 ShortcutForm 重建，重新读取 initial 值
                        form_key.set(form_key() + 1);
                    },
                    on_cancel: move |_| {
                        show_process_picker.set(false);
                    },
                }
            }
        }
    }
}
