# TailwindCSS 全平台配置设计

## 目标

将项目从手写 CSS 全面迁移到 TailwindCSS v4，覆盖所有平台（web、desktop、mobile）和共享 UI 组件库。

## 架构

根目录统一配置 Tailwind，编译产物复制到各平台包的 `assets/` 目录。

```
win_aide/
├── package.json                  (npm 依赖: @tailwindcss/cli)
├── tailwind.css                  (Tailwind 入口: @import "tailwindcss")
├── build-css.sh                  (统一编译脚本)
├── packages/
│   ├── ui/src/                   (组件改用 Tailwind class，删除旧 CSS)
│   ├── web/assets/tailwind.css   (编译产物)
│   ├── desktop/assets/tailwind.css
│   └── mobile/assets/tailwind.css
```

## CSS 迁移映射

### main.css (全局样式)
| 原 CSS | Tailwind |
|--------|----------|
| `background-color: #0f1116` | `bg-[#0f1116]` |
| `color: #ffffff` | `text-white` |
| `font-family: 'Segoe UI'...` | `font-sans` |
| `margin: 20px` | `m-5` |

### hero.css
| 原 CSS | Tailwind |
|--------|----------|
| `#hero` flex column center | `flex flex-col justify-center items-center` |
| `#links` 400px column | `w-[400px] text-left text-xl text-white flex flex-col` |
| `#links a` 边框圆角 | `text-white no-underline my-2.5 border border-white rounded-[5px] p-2.5 hover:bg-[#1f1f1f] hover:cursor-pointer` |
| `#header` max-width | `max-w-[1200px]` |

### navbar.css
| 原 CSS | Tailwind |
|--------|----------|
| `#navbar` flex row | `flex flex-row` |
| `#navbar a` | `text-white mr-5 no-underline transition-colors duration-200 hover:cursor-pointer hover:text-[#91a4d2]` |

### echo.css
| 原 CSS | Tailwind |
|--------|----------|
| `#echo` 容器 | `w-[360px] mx-auto mt-[50px] bg-[#1e222d] p-5 rounded-[10px]` |
| `#echo h4` | `m-0 mb-[15px]` |
| `#echo input` | `border-0 border-b border-white bg-transparent text-white transition-colors duration-200 outline-none block pb-[5px] w-full focus:border-b-[#6d85c6]` |
| `#echo p` | `mt-5 ml-auto` |

### blog.css
| 原 CSS | Tailwind |
|--------|----------|
| `#blog` | `mt-[50px]` |
| `#blog a` | `text-white mt-[50px]` |

## 删除的文件

- `packages/ui/assets/styling/hero.css`
- `packages/ui/assets/styling/navbar.css`
- `packages/ui/assets/styling/echo.css`
- `packages/web/assets/main.css`
- `packages/web/assets/blog.css`
- `packages/desktop/assets/main.css`
- `packages/desktop/assets/blog.css`
- `packages/mobile/assets/main.css`
- `packages/mobile/assets/blog.css`

## 修改的文件

- `packages/ui/src/hero.rs` — 删除 CSS 引用，改用 Tailwind class
- `packages/ui/src/navbar.rs` — 同上
- `packages/ui/src/echo.rs` — 同上
- `packages/web/src/main.rs` — 引用 tailwind.css 替代 main.css
- `packages/web/src/views/blog.rs` — 删除 blog.css 引用，改用 Tailwind class
- `packages/desktop/src/main.rs` — 同 web
- `packages/desktop/src/views/blog.rs` — 同 web
- `packages/mobile/src/main.rs` — 同 web
- `packages/mobile/src/views/blog.rs` — 同 web
- `.gitignore` — 添加 tailwind 编译产物

## 新增的文件

- `package.json` — npm 依赖
- `tailwind.css` — Tailwind 入口
- `build-css.sh` — 编译脚本
