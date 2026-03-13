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
            class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
            onclick: move |_| on_cancel.call(()),

            // 弹窗
            div {
                class: "bg-bg-card rounded-xl p-6 w-[460px] shadow-2xl border border-border-subtle",
                onclick: move |e| e.stop_propagation(),

                // 标题栏
                div { class: "flex items-center justify-between mb-6",
                    h2 { class: "text-lg font-semibold text-text-primary", "{title}" }
                    button {
                        class: "p-1 text-text-muted hover:text-text-primary hover:bg-bg-hover rounded-md transition-colors cursor-pointer",
                        onclick: move |_| on_cancel.call(()),
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
                            path { d: "M18 6 6 18" }
                            path { d: "m6 6 12 12" }
                        }
                    }
                }

                // 应用名称
                div { class: "mb-4",
                    label { class: "block text-sm font-medium text-text-secondary mb-1.5", "应用名称" }
                    input {
                        r#type: "text",
                        class: "w-full bg-bg-input border border-border-default rounded-lg px-3 py-2 text-sm text-text-primary placeholder-text-muted focus:border-accent focus:outline-none transition-colors",
                        placeholder: "例如：Chrome",
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                    }
                }

                // 进程名
                div { class: "mb-4",
                    label { class: "block text-sm font-medium text-text-secondary mb-1.5", "进程名" }
                    input {
                        r#type: "text",
                        class: "w-full bg-bg-input border border-border-default rounded-lg px-3 py-2 text-sm text-text-primary placeholder-text-muted focus:border-accent focus:outline-none transition-colors",
                        placeholder: "例如：chrome.exe",
                        value: "{exe_name}",
                        oninput: move |e| exe_name.set(e.value()),
                    }
                }

                // 路径
                div { class: "mb-4",
                    label { class: "block text-sm font-medium text-text-secondary mb-1.5", "可执行文件路径" }
                    div { class: "flex gap-2",
                        input {
                            r#type: "text",
                            class: "flex-1 bg-bg-input border border-border-default rounded-lg px-3 py-2 text-sm text-text-primary placeholder-text-muted focus:border-accent focus:outline-none transition-colors",
                            placeholder: "C:\\Program Files\\...",
                            value: "{exe_path}",
                            oninput: move |e| exe_path.set(e.value()),
                        }
                        button {
                            class: "px-3 py-2 bg-bg-hover border border-border-default text-text-secondary rounded-lg hover:bg-border-default hover:text-text-primary transition-colors cursor-pointer text-sm font-medium shrink-0",
                            onclick: move |_| {
                                spawn(async move {
                                    let file = rfd::AsyncFileDialog::new()
                                        .add_filter("可执行文件", &["exe"])
                                        .pick_file()
                                        .await;
                                    if let Some(file) = file {
                                        let path_str = file.path().to_string_lossy().to_string();
                                        let file_name = file.file_name();
                                        exe_path.set(path_str);
                                        if exe_name().is_empty() {
                                            exe_name.set(file_name.clone());
                                        }
                                        if name().is_empty() {
                                            name.set(file_name.trim_end_matches(".exe").to_string());
                                        }
                                    }
                                });
                            },
                            "浏览..."
                        }
                    }
                }

                // 快捷键
                div { class: "mb-6",
                    label { class: "block text-sm font-medium text-text-secondary mb-1.5", "快捷键" }
                    div { class: "flex gap-2 items-center",
                        select {
                            class: "bg-bg-input border border-border-default rounded-lg px-3 py-2 text-sm text-text-primary focus:border-accent focus:outline-none cursor-pointer transition-colors",
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
                        span { class: "text-text-muted text-base font-light", "+" }
                        input {
                            r#type: "text",
                            class: "w-14 bg-bg-input border border-border-default rounded-lg px-3 py-2 text-sm text-text-primary text-center uppercase font-mono font-medium focus:border-accent focus:outline-none transition-colors",
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
                    div { class: "mb-4 p-3 bg-danger-subtle border border-danger/20 rounded-lg text-danger text-sm flex items-center gap-2",
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
                            path { d: "M12 8v4" }
                            path { d: "M12 16h.01" }
                        }
                        span { "{msg}" }
                    }
                }

                // 按钮
                div { class: "flex justify-end gap-2 pt-2 border-t border-border-subtle",
                    button {
                        class: "px-4 py-2 text-sm text-text-secondary hover:text-text-primary hover:bg-bg-hover rounded-lg transition-colors cursor-pointer",
                        onclick: move |_| on_cancel.call(()),
                        "取消"
                    }
                    button {
                        class: "px-5 py-2 text-sm bg-accent text-white rounded-lg hover:bg-accent-focus transition-colors cursor-pointer font-medium",
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
