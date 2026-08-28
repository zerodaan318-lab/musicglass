# MusicGlass 技术决策记录

> 记录每个重要技术选择及其理由，方便后续重构时不丢失上下文（任务书第 78 节）。

## D-001 为什么选 Tauri 2 而不是 Electron
- **决策**：桌面壳用 Tauri 2。
- **理由**：包体小（WebView2 复用系统浏览器）、Rust 后端天然类型安全、内存占用低、默认安全模型（Capability 作用域）避免任意文件写入。Electron 包体大、需 bundled Chromium，对"本地音乐转换器"这类轻量工具过度。

## D-002 为什么用 Rust 做核心
- **决策**：所有业务逻辑（检测/转换/Metadata/任务/插件）用 Rust crate。
- **理由**：音频参数、Metadata 映射要求零丢失字段，强类型 `Metadata` 模型能编译期防止字段遗漏；并发与流式 IO 安全性优于脚本语言；FFmpeg 通过参数数组调用，避免 shell 注入。

## D-003 为什么用 FFmpeg 作为编解码底层
- **决策**：不自研编解码器，统一走 FFmpeg。
- **理由**：成熟、覆盖所有目标格式、PCM 提取可做 Bit-perfect 验证。重造轮子既不稳定也不符合任务书第 6 节"不要重复实现成熟编解码器"。

## D-004 为什么用 SQLite
- **决策**：转换历史用 SQLite（单文件 `history.db`）。
- **理由**：零配置、随程序本地存储、支持结构化查询（按状态/格式/时间检索历史），无需独立服务进程。

## D-005 为什么用插件 trait 而非 if/else
- **决策**：专有格式（NCM/QMC）实现 `MusicContainer` trait，通过 `Plugin` 枚举注册分发。
- **理由**：任务书第 7 节明确禁止主程序写 `if ncm / if qmc`。但 trait 方法无 `self`（遵循任务书签名），故**不可 `dyn`**，改用枚举匹配分发，既保留静态接口又支持动态选择，且无循环依赖（枚举在 `plugins` crate，依赖 core+ncm+qmc）。

## D-006 为什么 Unified Metadata 放在 core
- **决策**：`Metadata` 模型定义在 `musicglass-core`，各格式 reader/writer 都映射至此。
- **理由**：单一真相源，避免各 crate 各自定义结构体导致字段漂移；`lost_fields()` 方法集中实现"丢失字段提示"逻辑（任务书第 21 节）。

## D-007 为什么 FFmpeg 通过参数数组调用
- **决策**：`std::process::Command` 传 `&[&str]`，绝不字符串拼接 shell。
- **理由**：用户文件名含空格/特殊字符/Emoji 时，shell 拼接会产生命令注入或解析错误（任务书第 49 节安全要求）。

## D-008 为什么 Rust 工具链装在 D:\Hermes\mg-rust（而非默认 C 盘）
- **决策**：`RUSTUP_HOME`/`CARGO_HOME` 指向 `D:\Hermes\mg-rust`，不修改全局 PATH。
- **理由**：项目目录在 OneDrive 同步区，避免几万小文件被同步；集中管理便于清理；不污染用户全局环境（用户偏好：非必须不塞 C 盘）。
