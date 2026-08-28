# MusicGlass 技术栈

> 选型以「稳定可维护」为第一优先级（任务书第 43 节）。以下为 Phase 0 确定的基线，后续如需变更须在 `DECISIONS.md` 记录。

## 1. 选型总览

| 层 | 技术 | 用途 |
|----|------|------|
| 前端 | React + TypeScript + Vite | 桌面 UI 渲染 |
| 桌面壳 | Tauri 2 | 原生窗口、系统 API、IPC 桥接 |
| 核心逻辑 | Rust (stable) | 检测/转换/Metadata/任务/插件 |
| 音频底层 | FFmpeg | 编解码、格式封装、PCM 提取 |
| 数据库 | SQLite | 转换历史记录 |
| 样式 | Tailwind CSS + CSS Variables | 玻璃拟态主题（深/浅色） |
| 动画 | Framer Motion（或同类） | 平滑过渡 |

## 2. 各组件说明

### Tauri 2
- 用系统 WebView（Windows 为 WebView2），包体远小于 Electron。
- 前端通过 `invoke` 调用 Rust 命令，通过事件流接收进度/日志。
- 文件系统访问走 Tauri 权限与作用域配置，避免任意路径写入。

### Rust
- 核心 crate 全部用 Rust，强类型保证音频参数、Metadata 映射不丢字段。
- 异步用 `tokio`；FFmpeg 通过 `std::process` 参数数组调用，捕获 stdout/stderr 解析进度。

### FFmpeg
- **随包分发静态构建**，用户不需自行安装。
- 版本固定并记录于 `THIRD_PARTY_NOTICES.md`。
- 用于：解码、重编码、封装、封面/歌词轨道处理、`-f crc` 或 PCM 提取做 Bit-perfect 校验。

### SQLite
- 单文件 `history.db`，记录输入输出路径、格式、大小、状态、耗时、错误。
- 通过 `rusqlite` 访问，启动时自动建表。

## 3. 环境依赖（开发者侧）

| 工具 | 版本要求 | 用途 |
|------|----------|------|
| Rust toolchain | stable 最新 | 编译核心与 Tauri 后端 |
| Node.js + pnpm | LTS | 前端构建 |
| Tauri CLI | v2 | 打包 EXE / Portable |
| FFmpeg | 固定 release | 随包分发，开发期用于联调 |
| Git + GitHub CLI | 已就绪 | 版本控制与持续同步 |

## 4. 已知约束

- Tauri 2 在 Windows 需要 **WebView2 Runtime**（安装包内可引导）。
- FFmpeg 动态链接需注意 MSVC/MinGW 运行时一致性；优先静态二进制。
- 大文件（>1GB）禁止整文件读入内存，统一用流式/分块 IO（任务书第 48 节）。
- 中文/Emoji 路径在 Windows 下需确保 Rust `std::fs` 与 FFmpeg 参数编码一致。

## 5. 第三方依赖记录

所有引入的 crate / npm 包 / 外部算法均在 `THIRD_PARTY_NOTICES.md` 登记：名称、版本、来源、License、用途。专有格式解析参考的开源实现（如 NCM/QMC）须遵守其 License，并在 `docs/formats/` 下记录研究来源，禁止复制 License 不明代码。
