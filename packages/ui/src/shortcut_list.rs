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
    pub icon_data: Option<String>,
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
            div { class: "grid grid-cols-[44px_36px_140px_1fr_1fr_88px] gap-3 px-5 py-2.5 text-xs font-medium text-text-muted uppercase tracking-wider border-b border-border-subtle",
                span {}
                span {}
                span { "快捷键" }
                span { "应用名称" }
                span { "路径" }
                span { class: "text-right", "操作" }
            }

            // 数据行
            for shortcut in shortcuts.iter() {
                {
                    let id_toggle = shortcut.id.clone();
                    let id_edit = shortcut.id.clone();
                    let id_delete = shortcut.id.clone();
                    let is_enabled = shortcut.enabled;
                    rsx! {
                        div {
                            class: if is_enabled {
                                "grid grid-cols-[44px_36px_140px_1fr_1fr_88px] gap-3 px-5 py-3 items-center border-b border-border-subtle hover:bg-bg-hover transition-colors group"
                            } else {
                                "grid grid-cols-[44px_36px_140px_1fr_1fr_88px] gap-3 px-5 py-3 items-center border-b border-border-subtle hover:bg-bg-hover transition-colors group opacity-50"
                            },
                            // 启用复选框
                            div { class: "flex items-center justify-center",
                                input {
                                    r#type: "checkbox",
                                    checked: shortcut.enabled,
                                    onchange: move |_| on_toggle.call(id_toggle.clone()),
                                }
                            }
                            // 应用图标
                            div { class: "flex items-center justify-center",
                                if let Some(ref icon_uri) = shortcut.icon_data {
                                    img {
                                        src: "{icon_uri}",
                                        width: "24",
                                        height: "24",
                                        class: "shrink-0 rounded",
                                    }
                                } else {
                                    div { class: "w-6 h-6 shrink-0 rounded bg-accent/20 flex items-center justify-center",
                                        svg {
                                            xmlns: "http://www.w3.org/2000/svg",
                                            width: "14",
                                            height: "14",
                                            view_box: "0 0 24 24",
                                            fill: "none",
                                            stroke: "currentColor",
                                            stroke_width: "2",
                                            class: "text-accent",
                                            rect { x: "2", y: "3", width: "20", height: "14", rx: "2" }
                                            path { d: "M8 21h8" }
                                            path { d: "M12 17v4" }
                                        }
                                    }
                                }
                            }
                            // 快捷键徽章
                            div {
                                span { class: "inline-flex items-center gap-1 px-2 py-1 bg-accent-subtle text-accent text-xs font-mono font-medium rounded-md",
                                    "{shortcut.modifier}"
                                    span { class: "text-text-muted", "+" }
                                    "{shortcut.hotkey}"
                                }
                            }
                            // 应用名称
                            div { class: "min-w-0",
                                span { class: "text-sm text-text-primary truncate block font-medium", "{shortcut.name}" }
                                span { class: "text-xs text-text-muted truncate block", "{shortcut.exe_name}" }
                            }
                            // 路径
                            span { class: "text-text-muted text-xs truncate", "{shortcut.exe_path}" }
                            // 操作按钮
                            div { class: "flex gap-1 justify-end opacity-0 group-hover:opacity-100 transition-opacity",
                                button {
                                    class: "p-1.5 text-text-muted hover:text-accent hover:bg-accent-subtle rounded-md transition-colors cursor-pointer",
                                    title: "编辑",
                                    onclick: move |_| on_edit.call(id_edit.clone()),
                                    // edit/pencil icon
                                    svg {
                                        xmlns: "http://www.w3.org/2000/svg",
                                        width: "14",
                                        height: "14",
                                        view_box: "0 0 24 24",
                                        fill: "none",
                                        stroke: "currentColor",
                                        stroke_width: "2",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        path { d: "M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z" }
                                    }
                                }
                                button {
                                    class: "p-1.5 text-text-muted hover:text-danger hover:bg-danger-subtle rounded-md transition-colors cursor-pointer",
                                    title: "删除",
                                    onclick: move |_| on_delete.call(id_delete.clone()),
                                    // trash icon
                                    svg {
                                        xmlns: "http://www.w3.org/2000/svg",
                                        width: "14",
                                        height: "14",
                                        view_box: "0 0 24 24",
                                        fill: "none",
                                        stroke: "currentColor",
                                        stroke_width: "2",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        path { d: "M3 6h18" }
                                        path { d: "M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" }
                                        path { d: "M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" }
                                        path { d: "M10 11v6" }
                                        path { d: "M14 11v6" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 空状态
            if shortcuts.is_empty() {
                div { class: "flex flex-col items-center justify-center py-20 px-4",
                    div { class: "w-16 h-16 rounded-2xl bg-bg-hover flex items-center justify-center mb-5",
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "28",
                            height: "28",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "1.5",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            class: "text-text-muted",
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
                    p { class: "text-base text-text-secondary font-medium mb-1", "暂无快捷键配置" }
                    p { class: "text-sm text-text-muted", "点击上方「添加」按钮开始配置" }
                }
            }
        }
    }
}
