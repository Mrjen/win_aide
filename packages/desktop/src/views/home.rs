use dioxus::prelude::*;
use ui::{Navbar, ProcessInfo, ProcessPicker, ShortcutForm, ShortcutFormData, ShortcutList, ShortcutRow, rgba_to_bmp_data_uri};

use crate::config::{self, AppConfig, Shortcut};

#[component]
pub fn Home(
    config: Signal<AppConfig>,
    paused: Signal<bool>,
    dark_mode: Signal<bool>,
    on_config_changed: EventHandler<AppConfig>,
) -> Element {
    let mut update_state: Signal<crate::updater::UpdateState> = use_context();

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
    let mut window_cycle_conflict = use_signal(|| None::<String>);

    // 定时检查更新
    use_future(move || async move {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        loop {
            crate::updater::check_update(&mut update_state).await;
            tokio::time::sleep(std::time::Duration::from_secs(4 * 3600)).await;
        }
    });

    // 将 config shortcuts 转换为 UI 行
    let rows: Vec<ShortcutRow> = config()
        .shortcuts
        .iter()
        .map(|s| {
            let icon_data = crate::process::extract_icon_rgba(&s.exe_path)
                .map(|rgba| rgba_to_bmp_data_uri(&rgba));
            ShortcutRow {
                id: s.id.clone(),
                name: s.name.clone(),
                exe_name: s.exe_name.clone(),
                exe_path: s.exe_path.clone(),
                modifier: s.modifier.display_name().to_string(),
                hotkey: s.key,
                enabled: s.enabled && !paused(),
                icon_data,
            }
        })
        .collect();

    let shortcut_count = config().shortcuts.len();
    let enabled_count = config().shortcuts.iter().filter(|s| s.enabled).count();

    let save_and_notify = move |new_config: AppConfig| {
        config::save_config(&new_config);
        on_config_changed.call(new_config);
    };

    let mut is_maximized = use_signal(|| false);

    rsx! {
        div { class: "flex flex-col h-screen",
            // ── 自定义标题栏（拖拽区 + 窗口控制按钮）──
            div {
                class: "flex items-center justify-end bg-bg-card select-none shrink-0 pr-1",
                onmousedown: move |_| {
                    dioxus::desktop::window().drag();
                },
                // 窗口控制按钮
                div {
                    class: "flex items-center",
                    onmousedown: move |e| e.stop_propagation(),
                    // 最小化
                    button {
                        class: "inline-flex items-center justify-center text-text-muted hover:text-text-primary hover:bg-bg-hover transition-colors cursor-pointer",
                        style: "width:48px;height:36px",
                        onclick: move |_| {
                            dioxus::desktop::window().window.set_minimized(true);
                        },
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "14",
                            height: "14",
                            view_box: "0 0 12 12",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "1.5",
                            path { d: "M2 6h8" }
                        }
                    }
                    // 最大化 / 还原
                    button {
                        class: "inline-flex items-center justify-center text-text-muted hover:text-text-primary hover:bg-bg-hover transition-colors cursor-pointer",
                        style: "width:48px;height:36px",
                        onclick: move |_| {
                            let win = dioxus::desktop::window();
                            let maximized = win.window.is_maximized();
                            win.window.set_maximized(!maximized);
                            is_maximized.set(!maximized);
                        },
                        if is_maximized() {
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                width: "14",
                                height: "14",
                                view_box: "0 0 12 12",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "1.2",
                                path { d: "M3.5 1.5h7v7" }
                                rect { x: "1.5", y: "3.5", width: "7", height: "7" }
                            }
                        } else {
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                width: "14",
                                height: "14",
                                view_box: "0 0 12 12",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "1.2",
                                rect { x: "1.5", y: "1.5", width: "9", height: "9" }
                            }
                        }
                    }
                    // 关闭（隐藏到托盘）
                    button {
                        class: "inline-flex items-center justify-center text-text-muted hover:text-white hover:bg-[#e81123] transition-colors cursor-pointer",
                        style: "width:48px;height:36px",
                        onclick: move |_| {
                            dioxus::desktop::window().set_visible(false);
                        },
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "14",
                            height: "14",
                            view_box: "0 0 12 12",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "1.5",
                            stroke_linecap: "round",
                            path { d: "M2 2l8 8" }
                            path { d: "M10 2l-8 8" }
                        }
                    }
                }
            }

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
                    onmousedown: move |e| e.stop_propagation(),
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
                    // ── 窗口循环切换设置 ──
                    div { class: "mt-4 pt-4 border-t border-border-subtle",
                        div { class: "flex items-center gap-2 mb-3",
                            span { class: "text-xs font-medium text-text-muted uppercase tracking-wide", "同应用窗口循环切换" }
                        }
                        div { class: "flex flex-wrap items-center gap-x-6 gap-y-3",
                            // 启用开关
                            label { class: "inline-flex items-center gap-2.5 text-sm text-text-secondary cursor-pointer select-none",
                                input {
                                    r#type: "checkbox",
                                    checked: config().settings.window_cycle.enabled,
                                    onchange: move |_| {
                                        let mut cfg = config();
                                        cfg.settings.window_cycle.enabled = !cfg.settings.window_cycle.enabled;
                                        save_and_notify(cfg);
                                    },
                                }
                                "启用"
                            }
                            // 修饰键选择
                            label { class: "inline-flex items-center gap-2 text-sm text-text-secondary",
                                span { "修饰键" }
                                select {
                                    class: "px-2 py-1 bg-bg-input border border-border-default rounded text-sm text-text-primary cursor-pointer",
                                    value: config().settings.window_cycle.modifier.display_name(),
                                    onchange: move |e: Event<FormData>| {
                                        let new_modifier = match e.value().as_str() {
                                            "Ctrl" => ui::Modifier::Ctrl,
                                            "Win" => ui::Modifier::Win,
                                            _ => ui::Modifier::Alt,
                                        };
                                        let cfg = config();
                                        if let Some(name) = config::window_cycle_conflicts(&cfg.shortcuts, &new_modifier, cfg.settings.window_cycle.key) {
                                            window_cycle_conflict.set(Some(format!("与快捷键「{}」冲突", name)));
                                        } else {
                                            window_cycle_conflict.set(None);
                                            let mut cfg = config();
                                            cfg.settings.window_cycle.modifier = new_modifier;
                                            save_and_notify(cfg);
                                        }
                                    },
                                    option { value: "Alt", selected: config().settings.window_cycle.modifier == ui::Modifier::Alt, "Alt" }
                                    option { value: "Ctrl", selected: config().settings.window_cycle.modifier == ui::Modifier::Ctrl, "Ctrl" }
                                    option { value: "Win", selected: config().settings.window_cycle.modifier == ui::Modifier::Win, "Win" }
                                }
                            }
                            // 按键输入
                            label { class: "inline-flex items-center gap-2 text-sm text-text-secondary",
                                span { "按键" }
                                input {
                                    r#type: "text",
                                    class: "w-12 px-2 py-1 bg-bg-input border border-border-default rounded text-sm text-text-primary text-center",
                                    value: config().settings.window_cycle.key.to_string(),
                                    maxlength: 1,
                                    onchange: move |e: Event<FormData>| {
                                        if let Some(ch) = e.value().chars().next() {
                                            let cfg = config();
                                            if let Some(name) = config::window_cycle_conflicts(&cfg.shortcuts, &cfg.settings.window_cycle.modifier, ch) {
                                                window_cycle_conflict.set(Some(format!("与快捷键「{}」冲突", name)));
                                            } else {
                                                window_cycle_conflict.set(None);
                                                let mut cfg = config();
                                                cfg.settings.window_cycle.key = ch;
                                                save_and_notify(cfg);
                                            }
                                        }
                                    },
                                }
                            }
                        }
                        // 提示信息
                        p { class: "mt-2 text-xs text-text-muted",
                            {format!(
                                "{}+{} 下一个窗口 / {}+Shift+{} 上一个窗口",
                                config().settings.window_cycle.modifier.display_name(),
                                config().settings.window_cycle.key,
                                config().settings.window_cycle.modifier.display_name(),
                                config().settings.window_cycle.key,
                            )}
                        }
                        // 冲突提示
                        if let Some(msg) = window_cycle_conflict() {
                            p { class: "mt-1.5 text-xs text-danger font-medium",
                                {msg}
                            }
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
                div { class: "flex items-center gap-3",
                    if paused() {
                        span { class: "text-warning-text font-medium", "已暂停" }
                    }
                    match update_state() {
                        crate::updater::UpdateState::Checking => rsx! {
                            span { class: "text-text-muted animate-pulse", "检查更新中..." }
                        },
                        crate::updater::UpdateState::Available(_) => rsx! {
                            span { class: "text-accent font-medium", "有新版本可用" }
                        },
                        _ => rsx! {
                            button {
                                class: "hover:text-text-primary cursor-pointer transition-colors",
                                title: "点击检查更新",
                                onclick: move |_| {
                                    spawn(async move {
                                        crate::updater::check_update(&mut update_state).await;
                                    });
                                },
                                "v{crate::updater::current_version()}"
                            }
                        },
                    }
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

            // ── 新增/编辑弹窗（进程选择弹窗打开时隐藏）──
            if show_form() && !show_process_picker() {
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

                        // 检查是否与窗口循环热键冲突
                        let wc = &cfg.settings.window_cycle;
                        if wc.enabled && wc.modifier == data.modifier && wc.key.eq_ignore_ascii_case(&key_char) {
                            conflict_msg.set(Some(format!("快捷键 {} + {} 与窗口循环切换冲突", data.modifier.display_name(), key_char)));
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

            // ── 更新弹窗 ──
            {
                use ui::{UpdateDialog, UpdateDialogState};
                match update_state() {
                    crate::updater::UpdateState::Available(ref info) => {
                        let info_clone = info.clone();
                        rsx! {
                            UpdateDialog {
                                state: UpdateDialogState::Available {
                                    version: info.version.clone(),
                                    name: info.name.clone(),
                                    body: info.body.clone(),
                                },
                                on_update: move |_| {
                                    let url = info_clone.download_url.clone();
                                    let size = info_clone.size;
                                    spawn(async move {
                                        if let Err(e) = crate::updater::download_update(&mut update_state, &url, size).await {
                                            update_state.set(crate::updater::UpdateState::Error(e));
                                        }
                                    });
                                },
                                on_dismiss: move |_| update_state.set(crate::updater::UpdateState::Idle),
                                on_retry: move |_| {
                                    spawn(async move {
                                        crate::updater::check_update(&mut update_state).await;
                                    });
                                },
                                on_install: move |_| {},
                            }
                        }
                    }
                    crate::updater::UpdateState::Downloading { progress } => rsx! {
                        UpdateDialog {
                            state: UpdateDialogState::Downloading { progress },
                            on_update: move |_| {},
                            on_dismiss: move |_| {},
                            on_retry: move |_| {},
                            on_install: move |_| {},
                        }
                    },
                    crate::updater::UpdateState::Ready => rsx! {
                        UpdateDialog {
                            state: UpdateDialogState::Ready,
                            on_update: move |_| {},
                            on_dismiss: move |_| update_state.set(crate::updater::UpdateState::Idle),
                            on_retry: move |_| {},
                            on_install: move |_| {
                                if let Err(e) = crate::updater::apply_update() {
                                    update_state.set(crate::updater::UpdateState::Error(e));
                                }
                            },
                        }
                    },
                    crate::updater::UpdateState::Error(ref msg) => rsx! {
                        UpdateDialog {
                            state: UpdateDialogState::Error { message: msg.clone() },
                            on_update: move |_| {},
                            on_dismiss: move |_| update_state.set(crate::updater::UpdateState::Idle),
                            on_retry: move |_| {
                                spawn(async move {
                                    crate::updater::check_update(&mut update_state).await;
                                });
                            },
                            on_install: move |_| {},
                        }
                    },
                    _ => rsx! {},
                }
            }
        }
    }
}
