use dioxus::prelude::*;

/// 更新弹窗的显示数据
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateDialogState {
    Available {
        version: String,
        name: String,
        body: String,
    },
    Downloading {
        progress: f64,
    },
    Ready,
    Error {
        message: String,
    },
}

#[component]
pub fn UpdateDialog(
    state: UpdateDialogState,
    on_update: EventHandler<()>,
    on_dismiss: EventHandler<()>,
    on_retry: EventHandler<()>,
    on_install: EventHandler<()>,
) -> Element {
    rsx! {
        // 模态遮罩
        div {
            class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
            onclick: move |_| {
                if !matches!(state, UpdateDialogState::Downloading { .. }) {
                    on_dismiss.call(());
                }
            },
            div {
                class: "bg-bg-card rounded-xl p-6 w-[420px] max-h-[80vh] shadow-2xl border border-border-subtle flex flex-col",
                onclick: move |e| e.stop_propagation(),

                match &state {
                    UpdateDialogState::Available { version, name, body } => rsx! {
                        // 下载图标
                        div { class: "w-10 h-10 rounded-full bg-accent/10 flex items-center justify-center mb-4",
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
                                class: "text-accent",
                                path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }
                                polyline { points: "7 10 12 15 17 10" }
                                line { x1: "12", y1: "15", x2: "12", y2: "3" }
                            }
                        }
                        h3 { class: "text-lg font-semibold text-text-primary mb-1",
                            "发现新版本"
                        }
                        p { class: "text-sm text-text-muted mb-3",
                            "v{version}"
                            if !name.is_empty() {
                                " — {name}"
                            }
                        }
                        if !body.is_empty() {
                            div { class: "mb-4 p-3 bg-bg-primary rounded-lg border border-border-subtle max-h-[200px] overflow-y-auto",
                                pre { class: "text-xs text-text-secondary whitespace-pre-wrap font-sans leading-relaxed",
                                    "{body}"
                                }
                            }
                        }
                        div { class: "flex justify-end gap-2",
                            button {
                                class: "px-4 py-2 text-sm text-text-secondary hover:text-text-primary hover:bg-bg-hover rounded-lg transition-colors cursor-pointer",
                                onclick: move |_| on_dismiss.call(()),
                                "稍后提醒"
                            }
                            button {
                                class: "px-4 py-2 text-sm bg-accent text-white rounded-lg hover:bg-accent-focus transition-colors cursor-pointer font-medium",
                                onclick: move |_| on_update.call(()),
                                "立即更新"
                            }
                        }
                    },
                    UpdateDialogState::Downloading { progress } => rsx! {
                        h3 { class: "text-lg font-semibold text-text-primary mb-4",
                            "正在下载更新..."
                        }
                        div { class: "w-full bg-bg-primary rounded-full h-2.5 mb-2",
                            div {
                                class: "bg-accent h-2.5 rounded-full transition-all duration-300",
                                style: "width: {progress * 100.0:.0}%",
                            }
                        }
                        p { class: "text-sm text-text-muted text-center",
                            "{progress * 100.0:.0}%"
                        }
                    },
                    UpdateDialogState::Ready => rsx! {
                        div { class: "w-10 h-10 rounded-full bg-success-subtle flex items-center justify-center mb-4",
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
                                class: "text-success",
                                path { d: "M20 6 9 17l-5-5" }
                            }
                        }
                        h3 { class: "text-lg font-semibold text-text-primary mb-2",
                            "下载完成"
                        }
                        p { class: "text-sm text-text-secondary mb-4",
                            "更新已下载完成，点击安装将关闭应用并自动完成更新。"
                        }
                        div { class: "flex justify-end gap-2",
                            button {
                                class: "px-4 py-2 text-sm text-text-secondary hover:text-text-primary hover:bg-bg-hover rounded-lg transition-colors cursor-pointer",
                                onclick: move |_| on_dismiss.call(()),
                                "稍后安装"
                            }
                            button {
                                class: "px-4 py-2 text-sm bg-accent text-white rounded-lg hover:bg-accent-focus transition-colors cursor-pointer font-medium",
                                onclick: move |_| on_install.call(()),
                                "安装并重启"
                            }
                        }
                    },
                    UpdateDialogState::Error { message } => rsx! {
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
                                circle { cx: "12", cy: "12", r: "10" }
                                path { d: "m15 9-6 6" }
                                path { d: "m9 9 6 6" }
                            }
                        }
                        h3 { class: "text-lg font-semibold text-text-primary mb-2",
                            "更新失败"
                        }
                        p { class: "text-sm text-text-secondary mb-4",
                            "{message}"
                        }
                        div { class: "flex justify-end gap-2",
                            button {
                                class: "px-4 py-2 text-sm text-text-secondary hover:text-text-primary hover:bg-bg-hover rounded-lg transition-colors cursor-pointer",
                                onclick: move |_| on_dismiss.call(()),
                                "关闭"
                            }
                            button {
                                class: "px-4 py-2 text-sm bg-accent text-white rounded-lg hover:bg-accent-focus transition-colors cursor-pointer font-medium",
                                onclick: move |_| on_retry.call(()),
                                "重试"
                            }
                        }
                    },
                }
            }
        }
    }
}
