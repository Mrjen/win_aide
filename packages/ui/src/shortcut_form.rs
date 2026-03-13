use dioxus::prelude::*;

use crate::Modifier;

#[derive(Debug, Clone, PartialEq)]
pub struct ShortcutFormData {
    pub id: Option<String>,
    pub name: String,
    pub exe_name: String,
    pub exe_path: String,
    pub modifier: Modifier,
    pub hotkey: String,
}

impl Default for ShortcutFormData {
    fn default() -> Self {
        Self {
            id: None,
            name: String::new(),
            exe_name: String::new(),
            exe_path: String::new(),
            modifier: Modifier::Alt,
            hotkey: String::new(),
        }
    }
}

#[component]
pub fn ShortcutForm(
    initial: ShortcutFormData,
    conflict_message: Option<String>,
    on_save: EventHandler<ShortcutFormData>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut name = use_signal(|| initial.name.clone());
    let mut exe_name = use_signal(|| initial.exe_name.clone());
    let mut exe_path = use_signal(|| initial.exe_path.clone());
    let mut modifier = use_signal(|| initial.modifier.clone());
    let mut hotkey = use_signal(|| initial.hotkey.clone());

    let title = if initial.id.is_some() { "编辑快捷键" } else { "添加快捷键" };
    let initial_id = initial.id.clone();

    rsx! {
        // 遮罩层
        div {
            class: "fixed inset-0 bg-black/60 flex items-center justify-center z-50",
            onclick: move |_| on_cancel.call(()),

            // 弹窗
            div {
                class: "bg-bg-card rounded-lg p-6 w-[450px] shadow-xl",
                onclick: move |e| e.stop_propagation(),

                h2 { class: "text-xl font-semibold text-white mb-6", "{title}" }

                // 应用名称
                div { class: "mb-4",
                    label { class: "block text-sm text-gray-400 mb-1", "应用名称" }
                    input {
                        r#type: "text",
                        class: "w-full bg-bg-primary border border-gray-700 rounded px-3 py-2 text-white focus:border-accent-focus focus:outline-none",
                        placeholder: "例如：Chrome",
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                    }
                }

                // 进程名
                div { class: "mb-4",
                    label { class: "block text-sm text-gray-400 mb-1", "进程名" }
                    input {
                        r#type: "text",
                        class: "w-full bg-bg-primary border border-gray-700 rounded px-3 py-2 text-white focus:border-accent-focus focus:outline-none",
                        placeholder: "例如：chrome.exe",
                        value: "{exe_name}",
                        oninput: move |e| exe_name.set(e.value()),
                    }
                }

                // 路径
                div { class: "mb-4",
                    label { class: "block text-sm text-gray-400 mb-1", "可执行文件路径" }
                    div { class: "flex gap-2",
                        input {
                            r#type: "text",
                            class: "flex-1 bg-bg-primary border border-gray-700 rounded px-3 py-2 text-white focus:border-accent-focus focus:outline-none text-sm",
                            placeholder: "C:\\Program Files\\...",
                            value: "{exe_path}",
                            oninput: move |e| exe_path.set(e.value()),
                        }
                        button {
                            class: "px-3 py-2 bg-gray-700 text-white rounded hover:bg-gray-600 transition-colors cursor-pointer text-sm",
                            onclick: move |_| {
                                let file = rfd::FileDialog::new()
                                    .add_filter("可执行文件", &["exe"])
                                    .pick_file();
                                if let Some(path) = file {
                                    let path_str = path.to_string_lossy().to_string();
                                    let file_name = path.file_name()
                                        .map(|n| n.to_string_lossy().to_string())
                                        .unwrap_or_default();
                                    exe_path.set(path_str);
                                    if exe_name().is_empty() {
                                        exe_name.set(file_name.clone());
                                    }
                                    if name().is_empty() {
                                        name.set(file_name.trim_end_matches(".exe").to_string());
                                    }
                                }
                            },
                            "浏览..."
                        }
                    }
                }

                // 快捷键
                div { class: "mb-6",
                    label { class: "block text-sm text-gray-400 mb-1", "快捷键" }
                    div { class: "flex gap-2 items-center",
                        select {
                            class: "bg-bg-primary border border-gray-700 rounded px-3 py-2 text-white focus:border-accent-focus focus:outline-none cursor-pointer",
                            value: "{modifier().display_name()}",
                            onchange: move |e| {
                                let val = e.value();
                                modifier.set(match val.as_str() {
                                    "Ctrl" => Modifier::Ctrl,
                                    "Win" => Modifier::Win,
                                    _ => Modifier::Alt,
                                });
                            },
                            option { value: "Alt", "Alt" }
                            option { value: "Ctrl", "Ctrl" }
                            option { value: "Win", "Win" }
                        }
                        span { class: "text-gray-400 text-lg", "+" }
                        input {
                            r#type: "text",
                            class: "w-16 bg-bg-primary border border-gray-700 rounded px-3 py-2 text-white text-center uppercase focus:border-accent-focus focus:outline-none",
                            placeholder: "A",
                            maxlength: 1,
                            value: "{hotkey}",
                            oninput: move |e| {
                                let val = e.value();
                                if let Some(c) = val.chars().last() {
                                    if c.is_ascii_alphabetic() {
                                        hotkey.set(c.to_ascii_uppercase().to_string());
                                    }
                                } else {
                                    hotkey.set(String::new());
                                }
                            },
                        }
                    }
                }

                // 冲突提示
                if let Some(msg) = &conflict_message {
                    div { class: "mb-4 p-3 bg-red-900/30 border border-red-700 rounded text-red-300 text-sm",
                        "{msg}"
                    }
                }

                // 按钮
                div { class: "flex justify-end gap-3",
                    button {
                        class: "px-4 py-2 text-gray-300 hover:text-white transition-colors cursor-pointer",
                        onclick: move |_| on_cancel.call(()),
                        "取消"
                    }
                    button {
                        class: "px-4 py-2 bg-accent text-white rounded hover:bg-accent-focus transition-colors cursor-pointer",
                        onclick: {
                            let initial_id = initial_id.clone();
                            move |_| {
                                on_save.call(ShortcutFormData {
                                    id: initial_id.clone(),
                                    name: name(),
                                    exe_name: exe_name(),
                                    exe_path: exe_path(),
                                    modifier: modifier(),
                                    hotkey: hotkey(),
                                });
                            }
                        },
                        "保存"
                    }
                }
            }
        }
    }
}
