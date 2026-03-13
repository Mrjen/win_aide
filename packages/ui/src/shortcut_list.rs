use dioxus::prelude::*;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct ShortcutRow {
    pub id: String,
    pub name: String,
    pub exe_name: String,
    pub exe_path: String,
    pub modifier: String,
    pub hotkey: char,
    pub enabled: bool,
}

#[component]
pub fn ShortcutList(
    shortcuts: Vec<ShortcutRow>,
    on_toggle: EventHandler<String>,
    on_edit: EventHandler<String>,
    on_delete: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "w-full",
            // 表头
            div { class: "grid grid-cols-[50px_120px_1fr_1fr_100px] gap-2 px-4 py-2 text-sm text-text-secondary border-b border-border-default",
                span { "启用" }
                span { "快捷键" }
                span { "应用名称" }
                span { "路径" }
                span { "操作" }
            }

            // 数据行
            for shortcut in shortcuts.iter() {
                {
                    let id_toggle = shortcut.id.clone();
                    let id_edit = shortcut.id.clone();
                    let id_delete = shortcut.id.clone();
                    rsx! {
                        div {
                            class: "grid grid-cols-[50px_120px_1fr_1fr_100px] gap-2 px-4 py-3 items-center border-b border-border-subtle hover:bg-bg-card transition-colors",
                            // 启用复选框
                            div {
                                input {
                                    r#type: "checkbox",
                                    checked: shortcut.enabled,
                                    class: "w-4 h-4 cursor-pointer accent-accent",
                                    onchange: move |_| on_toggle.call(id_toggle.clone()),
                                }
                            }
                            // 快捷键
                            span { class: "text-accent font-mono text-sm",
                                "{shortcut.modifier} + {shortcut.hotkey}"
                            }
                            // 应用名称
                            span { class: "text-text-primary truncate", "{shortcut.name}" }
                            // 路径
                            span { class: "text-text-secondary text-sm truncate", "{shortcut.exe_path}" }
                            // 操作按钮
                            div { class: "flex gap-2",
                                button {
                                    class: "px-2 py-1 text-sm text-text-secondary hover:text-text-primary hover:bg-border-default rounded transition-colors cursor-pointer",
                                    onclick: move |_| on_edit.call(id_edit.clone()),
                                    "编辑"
                                }
                                button {
                                    class: "px-2 py-1 text-sm text-red-400 hover:text-red-300 hover:bg-red-900/30 rounded transition-colors cursor-pointer",
                                    onclick: move |_| on_delete.call(id_delete.clone()),
                                    "删除"
                                }
                            }
                        }
                    }
                }
            }

            // 空状态
            if shortcuts.is_empty() {
                div { class: "text-center text-text-muted py-12",
                    p { class: "text-lg mb-2", "暂无快捷键配置" }
                    p { class: "text-sm", "点击上方「添加快捷键」开始配置" }
                }
            }
        }
    }
}
