use dioxus::prelude::*;
use ui::{Navbar, ShortcutForm, ShortcutFormData, ShortcutList, ShortcutRow};

use crate::config::{self, AppConfig, Shortcut};

#[component]
pub fn Home(
    config: Signal<AppConfig>,
    on_config_changed: EventHandler<AppConfig>,
) -> Element {
    let mut show_form = use_signal(|| false);
    let mut editing_id = use_signal(|| None::<String>);
    let mut conflict_msg = use_signal(|| None::<String>);
    let mut form_data = use_signal(ShortcutFormData::default);
    let mut show_settings = use_signal(|| false);
    let mut delete_confirm = use_signal(|| None::<String>);

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
            enabled: s.enabled,
        })
        .collect();

    let save_and_notify = move |new_config: AppConfig| {
        config::save_config(&new_config);
        on_config_changed.call(new_config);
    };

    rsx! {
        div { class: "flex flex-col h-screen",
            // 工具栏
            Navbar {
                div { class: "flex items-center gap-2",
                    h1 { class: "text-lg font-semibold text-white", "Win Aide" }
                    span { class: "text-sm text-gray-400", "快捷键启动器" }
                }
                div { class: "flex gap-2",
                    button {
                        class: "px-3 py-1.5 bg-accent text-white rounded hover:bg-accent-focus transition-colors cursor-pointer text-sm",
                        onclick: move |_| {
                            form_data.set(ShortcutFormData::default());
                            editing_id.set(None);
                            conflict_msg.set(None);
                            show_form.set(true);
                        },
                        "+ 添加快捷键"
                    }
                    button {
                        class: "px-3 py-1.5 text-gray-300 hover:text-white border border-gray-700 rounded hover:bg-gray-700 transition-colors cursor-pointer text-sm",
                        onclick: move |_| show_settings.set(!show_settings()),
                        "设置"
                    }
                }
            }

            // 设置面板
            if show_settings() {
                div { class: "px-4 py-3 bg-bg-card border-b border-gray-700",
                    h3 { class: "text-sm font-semibold text-gray-300 mb-3", "设置" }
                    div { class: "flex gap-6",
                        label { class: "flex items-center gap-2 text-sm text-gray-300 cursor-pointer",
                            input {
                                r#type: "checkbox",
                                checked: config().settings.auto_start,
                                class: "w-4 h-4 accent-accent",
                                onchange: move |_| {
                                    let mut cfg = config();
                                    cfg.settings.auto_start = !cfg.settings.auto_start;
                                    save_and_notify(cfg);
                                },
                            }
                            "开机自启"
                        }
                        label { class: "flex items-center gap-2 text-sm text-gray-300 cursor-pointer",
                            input {
                                r#type: "checkbox",
                                checked: config().settings.start_minimized,
                                class: "w-4 h-4 accent-accent",
                                onchange: move |_| {
                                    let mut cfg = config();
                                    cfg.settings.start_minimized = !cfg.settings.start_minimized;
                                    save_and_notify(cfg);
                                },
                            }
                            "启动时最小化到托盘"
                        }
                    }
                }
            }

            // 快捷键列表
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

            // 删除确认对话框
            if let Some(del_id) = delete_confirm() {
                div {
                    class: "fixed inset-0 bg-black/60 flex items-center justify-center z-50",
                    div { class: "bg-bg-card rounded-lg p-6 w-[350px] shadow-xl",
                        h3 { class: "text-lg text-white mb-4", "确认删除" }
                        p { class: "text-gray-300 text-sm mb-6", "确定要删除这个快捷键配置吗？此操作不可撤销。" }
                        div { class: "flex justify-end gap-3",
                            button {
                                class: "px-4 py-2 text-gray-300 hover:text-white transition-colors cursor-pointer",
                                onclick: move |_| delete_confirm.set(None),
                                "取消"
                            }
                            button {
                                class: "px-4 py-2 bg-red-600 text-white rounded hover:bg-red-500 transition-colors cursor-pointer",
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

            // 新增/编辑弹窗
            if show_form() {
                ShortcutForm {
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
                }
            }
        }
    }
}
