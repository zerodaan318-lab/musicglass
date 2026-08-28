# MusicGlass 技术决策记录（DECISIONS）

> 记录重要技术选择及其理由，便于后续重构时追溯「当初为什么这么设计」（任务书第 78 节）。
> 任何选型变更都必须在此追加新条目，不要覆盖旧条目。

---

## D-001：选择 Tauri 2 作为桌面壳
- **时间**：Phase 0（2026-08-28）
- **决定**：使用 Tauri 2 + React/TS 前端，而非 Electron。
- **理由**：
  - 包体远小于 Electron（WebView2 系统级复用）。
  - Rust 后端与核心 crate 同一语言，IPC 类型可对齐。
  - 文件系统访问走 Tauri 权限/作用域，天然限制任意路径写入，契合安全边界。
- **代价**：Windows 需 WebView2 Runtime（安装包可引导）；Rust 编译链较重。

## D-002：核心逻辑全部用 Rust
- **决定**：检测/转换/Metadata/任务/插件均用 Rust crate，不用 Node 做重逻辑。
- **理由**：强类型保证音频参数（SampleRate/BitDepth/Channels）与 Metadata 字段不丢；FFmpeg 通过 `std::process` 参数数组调用，规避命令注入。

## D-003：FFmpeg 作为唯一编解码底层
- **决定**：不自己实现任何成熟编解码器，全部走 FFmpeg 进程调用。
- **理由**：成熟、跨格式、可 `-f crc` / PCM 提取做校验；随包分发静态构建，用户无需自装。

## D-004：SQLite 记录历史
- **决定**：用 `rusqlite` 维护单文件 `history.db`。
- **理由**：轻量、单文件、易备份；满足任务书第 38 节全部字段要求。

## D-005：专有格式用插件 trait，禁止 if/else 堆主程序
- **决定**：定义 `MusicContainer` trait，NCM/QMC 各自实现，主程序经 `PluginRegistry` 调用。
- **理由**：任务书第 7 节强制要求；不同专有格式内部结构差异大，必须可独立演进。

## D-006：Unified Metadata 中间层
- **决定**：所有格式先读入统一 `Metadata` 结构，输出时按目标格式能力回写。
- **理由**：不同格式 Metadata 标准不同（ID3 / Vorbis / MP4 atom / APE），统一中间层避免每对组合写映射。

## D-007：Rust 工具链安装到非 OneDrive 目录
- **决定**：`RUSTUP_HOME=D:\Hermes\mg-rust\rustup`，`CARGO_HOME=D:\Hermes\mg-rust\cargo`。
- **理由**：OneDrive 同步目录放 Rust target（数万小文件）会被同步折磨且易冲突；非 OneDrive 路径干净、可一键清理。
- **影响**：不修改系统 PATH，调用 cargo 用绝对路径 `D:\Hermes\mg-rust\cargo\bin\cargo`。

## D-008：FFmpeg 二进制随包分发、不进 Git
- **决定**：FFmpeg 下载到 `resources/ffmpeg/bin/`，由 `.gitignore` 排除，构建时打包进 EXE/Portable。
- **理由**：任务书第 42/50/57 节——用户不应自装 FFmpeg；大型二进制不进仓库。

## D-009：GitHub 仓库默认 Private
- **决定**：`zerodaan318-lab/musicglass` 为 Private，稳定后再考虑公开。
- **理由**：任务书第 67 节——当前为开发阶段，先保护未完成代码与测试资料。

## D-010：每功能独立 commit + 持续 push，但 EXE 构建前先询问
- **决定**：代码 commit/push 按任务书自动执行；生成安装包/Portable EXE 前先与用户确认。
- **理由**：兼顾任务书「持续同步」要求与用户「构建前先询问」的偏好。
