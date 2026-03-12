# TailwindCSS 全平台配置 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将项目从手写 CSS 全面迁移到 TailwindCSS v4，覆盖 web/desktop/mobile 三个平台和共享 UI 组件库。

**Architecture:** 根目录统一配置 Tailwind，通过 `@tailwindcss/cli` 编译，产物复制到各平台包的 `assets/` 目录。各平台 `main.rs` 顶层引入编译后的 CSS，所有组件使用 Tailwind utility class 替代手写 CSS。

**Tech Stack:** TailwindCSS v4, @tailwindcss/cli, Dioxus 0.7.1, npm

**Design doc:** `docs/plans/2026-03-12-tailwindcss-design.md`

---

### Task 1: 初始化 npm 和 TailwindCSS

**Files:**
- Create: `package.json`
- Create: `tailwind.css`

**Step 1: 创建 package.json**

```json
{
  "private": true,
  "devDependencies": {
    "@tailwindcss/cli": "^4"
  }
}
```

**Step 2: 创建 Tailwind 入口文件**

`tailwind.css`:
```css
@import "tailwindcss";

@theme {
  --color-bg-primary: #0f1116;
  --color-bg-card: #1e222d;
  --color-accent: #91a4d2;
  --color-accent-focus: #6d85c6;
}
```

**Step 3: 安装依赖**

Run: `npm install`
Expected: 生成 `node_modules/` 和 `package-lock.json`

**Step 4: 更新 .gitignore**

在 `.gitignore` 末尾追加：
```
node_modules/
packages/*/assets/tailwind.css
```

**Step 5: Commit**

```bash
git add package.json tailwind.css .gitignore
git commit -m "feat: 初始化 TailwindCSS v4 配置"
```

---

### Task 2: 创建构建脚本并验证 Tailwind 编译

**Files:**
- Create: `build-css.sh`

**Step 1: 创建构建脚本**

`build-css.sh`:
```bash
#!/bin/bash
set -e

echo "Building TailwindCSS..."
npx @tailwindcss/cli -i tailwind.css -o packages/web/assets/tailwind.css --minify
cp packages/web/assets/tailwind.css packages/desktop/assets/tailwind.css
cp packages/web/assets/tailwind.css packages/mobile/assets/tailwind.css
echo "TailwindCSS build complete."
```

**Step 2: 运行构建验证**

Run: `bash build-css.sh`
Expected: 三个平台的 `assets/tailwind.css` 文件生成成功

**Step 3: Commit**

```bash
git add build-css.sh
git commit -m "feat: 添加 TailwindCSS 统一编译脚本"
```

---

### Task 3: 迁移 UI 组件 — Hero

**Files:**
- Modify: `packages/ui/src/hero.rs`
- Delete: `packages/ui/assets/styling/hero.css`

**Step 1: 修改 hero.rs**

删除 `HERO_CSS` 常量和 `document::Link`，将所有 `id` 替换为 Tailwind class：

```rust
use dioxus::prelude::*;

const HEADER_SVG: Asset = asset!("/assets/header.svg");

#[component]
pub fn Hero() -> Element {
    rsx! {
        div { class: "flex flex-col justify-center items-center",
            img { src: HEADER_SVG, class: "max-w-[1200px]" }
            div { class: "w-[400px] text-left text-xl text-white flex flex-col",
                a { href: "https://dioxuslabs.com/learn/0.7/", class: "text-white no-underline my-2.5 border border-white rounded-[5px] p-2.5 hover:bg-[#1f1f1f] hover:cursor-pointer",
                    "📚 Learn Dioxus"
                }
                a { href: "https://dioxuslabs.com/awesome", class: "text-white no-underline my-2.5 border border-white rounded-[5px] p-2.5 hover:bg-[#1f1f1f] hover:cursor-pointer",
                    "🚀 Awesome Dioxus"
                }
                a { href: "https://github.com/dioxus-community/", class: "text-white no-underline my-2.5 border border-white rounded-[5px] p-2.5 hover:bg-[#1f1f1f] hover:cursor-pointer",
                    "📡 Community Libraries"
                }
                a { href: "https://github.com/DioxusLabs/sdk", class: "text-white no-underline my-2.5 border border-white rounded-[5px] p-2.5 hover:bg-[#1f1f1f] hover:cursor-pointer",
                    "⚙️ Dioxus Development Kit"
                }
                a { href: "https://marketplace.visualstudio.com/items?itemName=DioxusLabs.dioxus", class: "text-white no-underline my-2.5 border border-white rounded-[5px] p-2.5 hover:bg-[#1f1f1f] hover:cursor-pointer",
                    "💫 VSCode Extension"
                }
                a { href: "https://discord.gg/XgGxMSkvUM", class: "text-white no-underline my-2.5 border border-white rounded-[5px] p-2.5 hover:bg-[#1f1f1f] hover:cursor-pointer",
                    "👋 Community Discord"
                }
            }
        }
    }
}
```

**Step 2: 删除旧 CSS**

Run: `rm packages/ui/assets/styling/hero.css`

**Step 3: 验证编译**

Run: `cargo check -p ui`
Expected: PASS，无编译错误

**Step 4: Commit**

```bash
git add packages/ui/src/hero.rs
git rm packages/ui/assets/styling/hero.css
git commit -m "refactor: Hero 组件迁移到 TailwindCSS"
```

---

### Task 4: 迁移 UI 组件 — Navbar

**Files:**
- Modify: `packages/ui/src/navbar.rs`
- Delete: `packages/ui/assets/styling/navbar.css`

**Step 1: 修改 navbar.rs**

```rust
use dioxus::prelude::*;

#[component]
pub fn Navbar(children: Element) -> Element {
    rsx! {
        div { class: "flex flex-row [&>a]:text-white [&>a]:mr-5 [&>a]:no-underline [&>a]:transition-colors [&>a]:duration-200 hover:[&>a]:cursor-pointer hover:[&>a]:text-accent",
            {children}
        }
    }
}
```

注意：Navbar 的 children 是 `Link` 组件（渲染为 `<a>`），Tailwind 的 arbitrary variant `[&>a]` 可以选中直接子 `<a>` 元素。`text-accent` 使用 Task 1 中定义的自定义颜色。

**Step 2: 删除旧 CSS**

Run: `rm packages/ui/assets/styling/navbar.css`

**Step 3: 验证编译**

Run: `cargo check -p ui`
Expected: PASS

**Step 4: Commit**

```bash
git add packages/ui/src/navbar.rs
git rm packages/ui/assets/styling/navbar.css
git commit -m "refactor: Navbar 组件迁移到 TailwindCSS"
```

---

### Task 5: 迁移 UI 组件 — Echo

**Files:**
- Modify: `packages/ui/src/echo.rs`
- Delete: `packages/ui/assets/styling/echo.css`

**Step 1: 修改 echo.rs**

```rust
use dioxus::prelude::*;

/// Echo component that demonstrates fullstack server functions.
#[component]
pub fn Echo() -> Element {
    let mut response = use_signal(|| String::new());

    rsx! {
        div { class: "w-[360px] mx-auto mt-[50px] bg-bg-card p-5 rounded-[10px]",
            h4 { class: "m-0 mb-[15px]", "ServerFn Echo" }
            input {
                class: "border-0 border-b border-white bg-transparent text-white transition-colors duration-200 outline-none block pb-[5px] w-full focus:border-b-accent-focus",
                placeholder: "Type here to echo...",
                oninput: move |event| async move {
                    let data = api::echo(event.value()).await.unwrap();
                    response.set(data);
                },
            }

            if !response().is_empty() {
                p { class: "mt-5 ml-auto",
                    "Server echoed: "
                    i { "{response}" }
                }
            }
        }
    }
}
```

`bg-bg-card` 和 `focus:border-b-accent-focus` 使用 Task 1 中定义的自定义颜色。

**Step 2: 删除旧 CSS**

Run: `rm packages/ui/assets/styling/echo.css`

**Step 3: 验证编译**

Run: `cargo check -p ui`
Expected: PASS

**Step 4: Commit**

```bash
git add packages/ui/src/echo.rs
git rm packages/ui/assets/styling/echo.css
git commit -m "refactor: Echo 组件迁移到 TailwindCSS"
```

---

### Task 6: 删除空的 styling 目录

**Step 1: 删除目录**

Run: `rmdir packages/ui/assets/styling` (Windows) 或 `rm -r packages/ui/assets/styling` (Unix)

**Step 2: Commit**

```bash
git add -A packages/ui/assets/styling
git commit -m "chore: 删除已废弃的 UI 组件 CSS 目录"
```

---

### Task 7: 迁移 Web 平台

**Files:**
- Modify: `packages/web/src/main.rs`
- Modify: `packages/web/src/views/blog.rs`
- Delete: `packages/web/assets/main.css`
- Delete: `packages/web/assets/blog.css`

**Step 1: 修改 main.rs — 引入 Tailwind CSS**

将 `MAIN_CSS` 替换为 `TAILWIND_CSS`：

```rust
use dioxus::prelude::*;

use ui::Navbar;
use views::{Blog, Home};

mod views;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(WebNavbar)]
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        div { class: "bg-bg-primary text-white font-sans m-5 min-h-screen",
            Router::<Route> {}
        }
    }
}

#[component]
fn WebNavbar() -> Element {
    rsx! {
        Navbar {
            Link {
                to: Route::Home {},
                "Home"
            }
            Link {
                to: Route::Blog { id: 1 },
                "Blog"
            }
        }

        Outlet::<Route> {}
    }
}
```

**Step 2: 修改 blog.rs — 删除 CSS 引用，使用 Tailwind**

```rust
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Blog(id: i32) -> Element {
    rsx! {
        div { class: "mt-[50px]",
            h1 { "This is blog #{id}!" }
            p { "In blog #{id}, we show how the Dioxus router works and how URL parameters can be passed as props to our route components." }

            Link {
                to: Route::Blog { id: id - 1 },
                class: "text-white",
                "Previous"
            }
            span { " <---> " }
            Link {
                to: Route::Blog { id: id + 1 },
                class: "text-white",
                "Next"
            }
        }
    }
}
```

**Step 3: 删除旧 CSS**

Run: `rm packages/web/assets/main.css packages/web/assets/blog.css`

**Step 4: 构建 Tailwind 并验证编译**

Run: `bash build-css.sh && cargo check -p web`
Expected: PASS

**Step 5: Commit**

```bash
git add packages/web/src/main.rs packages/web/src/views/blog.rs
git rm packages/web/assets/main.css packages/web/assets/blog.css
git commit -m "refactor: Web 平台迁移到 TailwindCSS"
```

---

### Task 8: 迁移 Desktop 平台

**Files:**
- Modify: `packages/desktop/src/main.rs`
- Modify: `packages/desktop/src/views/blog.rs`
- Delete: `packages/desktop/assets/main.css`
- Delete: `packages/desktop/assets/blog.css`

**Step 1: 修改 main.rs**

与 Web 平台结构相同，替换 `MAIN_CSS` → `TAILWIND_CSS`，添加全局 Tailwind class wrapper：

```rust
use dioxus::prelude::*;

use ui::Navbar;
use views::{Blog, Home};

mod views;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(DesktopNavbar)]
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
}

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        div { class: "bg-bg-primary text-white font-sans m-5 min-h-screen",
            Router::<Route> {}
        }
    }
}

#[component]
fn DesktopNavbar() -> Element {
    rsx! {
        Navbar {
            Link {
                to: Route::Home {},
                "Home"
            }
            Link {
                to: Route::Blog { id: 1 },
                "Blog"
            }
        }

        Outlet::<Route> {}
    }
}
```

**Step 2: 修改 blog.rs**

与 Web 平台的 blog.rs 完全相同（只是 Route 枚举来自不同 crate）：

```rust
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Blog(id: i32) -> Element {
    rsx! {
        div { class: "mt-[50px]",
            h1 { "This is blog #{id}!" }
            p { "In blog #{id}, we show how the Dioxus router works and how URL parameters can be passed as props to our route components." }

            Link {
                to: Route::Blog { id: id - 1 },
                class: "text-white",
                "Previous"
            }
            span { " <---> " }
            Link {
                to: Route::Blog { id: id + 1 },
                class: "text-white",
                "Next"
            }
        }
    }
}
```

**Step 3: 删除旧 CSS 并验证**

Run: `rm packages/desktop/assets/main.css packages/desktop/assets/blog.css`
Run: `cargo check -p desktop`
Expected: PASS

**Step 4: Commit**

```bash
git add packages/desktop/src/main.rs packages/desktop/src/views/blog.rs
git rm packages/desktop/assets/main.css packages/desktop/assets/blog.css
git commit -m "refactor: Desktop 平台迁移到 TailwindCSS"
```

---

### Task 9: 迁移 Mobile 平台

**Files:**
- Modify: `packages/mobile/src/main.rs`
- Modify: `packages/mobile/src/views/blog.rs`
- Delete: `packages/mobile/assets/main.css`
- Delete: `packages/mobile/assets/blog.css`

**Step 1: 修改 main.rs**

```rust
use dioxus::prelude::*;

use ui::Navbar;
use views::{Blog, Home};

mod views;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(MobileNavbar)]
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
}

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        div { class: "bg-bg-primary text-white font-sans m-5 min-h-screen",
            Router::<Route> {}
        }
    }
}

#[component]
fn MobileNavbar() -> Element {
    rsx! {
        Navbar {
            Link {
                to: Route::Home {},
                "Home"
            }
            Link {
                to: Route::Blog { id: 1 },
                "Blog"
            }
        }

        Outlet::<Route> {}
    }
}
```

**Step 2: 修改 blog.rs**

```rust
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Blog(id: i32) -> Element {
    rsx! {
        div { class: "mt-[50px]",
            h1 { "This is blog #{id}!" }
            p { "In blog #{id}, we show how the Dioxus router works and how URL parameters can be passed as props to our route components." }

            Link {
                to: Route::Blog { id: id - 1 },
                class: "text-white",
                "Previous"
            }
            span { " <---> " }
            Link {
                to: Route::Blog { id: id + 1 },
                class: "text-white",
                "Next"
            }
        }
    }
}
```

**Step 3: 删除旧 CSS 并验证**

Run: `rm packages/mobile/assets/main.css packages/mobile/assets/blog.css`
Run: `cargo check -p mobile`
Expected: PASS

**Step 4: Commit**

```bash
git add packages/mobile/src/main.rs packages/mobile/src/views/blog.rs
git rm packages/mobile/assets/main.css packages/mobile/assets/blog.css
git commit -m "refactor: Mobile 平台迁移到 TailwindCSS"
```

---

### Task 10: 全局验证

**Step 1: 重新构建 Tailwind CSS**

Run: `bash build-css.sh`
Expected: 编译成功，三个产物生成

**Step 2: 全工作区编译检查**

Run: `cargo check`
Expected: 全部 PASS

**Step 3: Clippy 检查**

Run: `cargo clippy`
Expected: 无 warning 或 error

**Step 4: 最终 Commit**

```bash
git add -A
git commit -m "chore: TailwindCSS 全平台迁移完成"
```
