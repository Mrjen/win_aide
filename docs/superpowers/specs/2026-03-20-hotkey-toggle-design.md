# 快捷键 Toggle 显示/最小化

## 概述

当前快捷键行为是单向的：按下快捷键总是激活目标窗口。本设计为快捷键增加 toggle 语义——如果目标窗口已在前台，再次按快捷键将其最小化到任务栏；如果不在前台或未运行，则激活或启动。

## 需求

1. **Toggle 行为**：快捷键在"激活"和"最小化到任务栏"之间切换
   - 窗口不存在 → 启动进程（不变）
   - 窗口存在但不在前台 → 激活窗口（不变，含托盘恢复兜底）
   - 窗口存在且是当前前台窗口 → **最小化到任务栏**（新增）

2. **多窗口场景**：如果应用有多个窗口，只最小化当前前台的那一个

3. **窗口循环增强**：`Alt + `` 窗口循环切换时，需要能发现并激活隐藏的窗口

## 改动范围

仅涉及 `packages/desktop/src/launcher.rs` 一个文件。

### 1. `launch_or_activate` 函数

在查找到窗口后、调用 `activate_window` 之前，新增前台窗口判断：

```
找到窗口 hwnd →
  hwnd == GetForegroundWindow()？
    是 → ShowWindow(hwnd, SW_MINIMIZE)
    否 → activate_window(hwnd)（现有逻辑不变，含托盘兜底恢复）
没找到 → launch_process（不变）
```

- 前台判断使用 `GetForegroundWindow()` 直接与 hwnd 比较
- 最小化使用 `ShowWindow(hwnd, SW_MINIMIZE)`
- 托盘恢复兜底逻辑保持不变（仅窗口不可见/零尺寸时触发，前台窗口不会命中）

### 2. `collect_windows_callback` 函数

当前：`IsWindowVisible` 过滤，只枚举可见窗口。

改为：移除 `IsWindowVisible` 过滤，改为检查 `GetWindowTextLengthW > 0`（有标题的窗口）。这样隐藏的窗口也能被窗口循环发现并激活，无标题的辅助窗口仍被排除。

### 3. imports

新增 `SW_MINIMIZE`。

## 不变的部分

- `activate_window` 函数 — 已处理隐藏/最小化/零尺寸窗口恢复，无需修改
- `find_window_by_exe` 函数 — 窗口查找优先级不变
- `HotkeyEvent` / `HotkeyCommand` — 无需修改
- 配置数据模型 — 无需修改
- 无新增文件、无新增数据结构、无新增线程通信
