use base64::Engine;
use dioxus::prelude::*;

use crate::ProcessInfo;

/// 将 32x32 RGBA 像素数据编码为 BMP 格式的 base64 data URI
pub fn rgba_to_bmp_data_uri(rgba: &[u8]) -> String {
    let size: u32 = 32;
    let pixel_data_size = size * size * 4;
    let file_size = 14 + 40 + pixel_data_size;

    let mut bmp = Vec::with_capacity(file_size as usize);

    // BITMAPFILEHEADER (14 bytes)
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&file_size.to_le_bytes());
    bmp.extend_from_slice(&[0u8; 4]); // reserved
    bmp.extend_from_slice(&(14u32 + 40).to_le_bytes()); // pixel data offset

    // BITMAPINFOHEADER (40 bytes)
    bmp.extend_from_slice(&40u32.to_le_bytes()); // header size
    bmp.extend_from_slice(&size.to_le_bytes()); // width
    bmp.extend_from_slice(&(-(size as i32)).to_le_bytes()); // height (negative = top-down)
    bmp.extend_from_slice(&1u16.to_le_bytes()); // planes
    bmp.extend_from_slice(&32u16.to_le_bytes()); // bits per pixel
    bmp.extend_from_slice(&[0u8; 24]); // compression + rest (all zeros for BI_RGB)

    // Pixel data: convert RGBA → BGRA for BMP
    for chunk in rgba.chunks_exact(4) {
        bmp.push(chunk[2]); // B
        bmp.push(chunk[1]); // G
        bmp.push(chunk[0]); // R
        bmp.push(chunk[3]); // A
    }

    let b64 = base64::engine::general_purpose::STANDARD.encode(&bmp);
    format!("data:image/bmp;base64,{b64}")
}

#[component]
pub fn ProcessPicker(
    processes: Vec<ProcessInfo>,
    loading: bool,
    on_select: EventHandler<ProcessInfo>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut search = use_signal(String::new);

    let filtered: Vec<&ProcessInfo> = processes
        .iter()
        .filter(|p| {
            let q = search().to_lowercase();
            if q.is_empty() {
                return true;
            }
            p.name.to_lowercase().contains(&q) || p.exe_path.to_lowercase().contains(&q)
        })
        .collect();

    rsx! {
        // 遮罩层
        div {
            class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
            onclick: move |_| on_cancel.call(()),

            // 弹窗
            div {
                class: "bg-bg-card rounded-xl shadow-2xl border border-border-default flex flex-col overflow-hidden",
                    style: "width: 520px; max-height: 70vh;",
                onclick: move |e| e.stop_propagation(),

                // 标题栏
                div { class: "flex items-center justify-between px-5 pt-5 pb-3",
                    h2 { class: "text-lg font-semibold text-text-primary", "从运行中的程序选择" }
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

                // 搜索框
                div { class: "px-5 pb-3",
                    div { class: "relative",
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
                            class: "absolute left-3 top-1/2 -translate-y-1/2 text-text-muted pointer-events-none",
                            circle { cx: "11", cy: "11", r: "8" }
                            path { d: "m21 21-4.3-4.3" }
                        }
                        input {
                            r#type: "text",
                            class: "w-full bg-bg-input border border-border-default rounded-lg pl-9 pr-3 py-2 text-sm text-text-primary placeholder-text-muted focus:border-accent focus:outline-none transition-colors",
                            placeholder: "搜索程序...",
                            value: "{search}",
                            oninput: move |e| search.set(e.value()),
                        }
                    }
                }

                // 进程列表
                div { class: "overflow-y-auto px-2 pb-4",
                    style: "max-height: calc(70vh - 120px);",
                    if loading {
                        div { class: "flex items-center justify-center py-12 text-text-muted text-sm",
                            "正在获取运行中的程序..."
                        }
                    } else if filtered.is_empty() {
                        div { class: "flex items-center justify-center py-12 text-text-muted text-sm",
                            if search().is_empty() {
                                "未找到运行中的程序"
                            } else {
                                "没有匹配的程序"
                            }
                        }
                    } else {
                        for process in filtered {
                            {
                                let p = process.clone();
                                rsx! {
                                    button {
                                        key: "{p.exe_path}",
                                        class: "w-full flex items-center gap-3 px-3 py-2.5 rounded-lg hover:bg-bg-hover transition-colors cursor-pointer text-left",
                                        onclick: move |_| on_select.call(p.clone()),

                                        // 图标
                                        if let Some(ref icon) = process.icon_rgba {
                                            img {
                                                src: "{rgba_to_bmp_data_uri(icon)}",
                                                width: "24",
                                                height: "24",
                                                class: "shrink-0 rounded",
                                            }
                                        } else {
                                            // 默认占位图标
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

                                        // 名称和路径
                                        div { class: "flex-1 min-w-0",
                                            div { class: "text-sm font-medium text-text-primary truncate", "{process.name}" }
                                            div { class: "text-xs text-text-muted truncate", "{process.exe_path}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
