# MusicGlass 进度

> 本文件持续更新。每完成一个 Phase：运行测试 → 检查 diff → commit → push → 更新本文件。

## 当前状态

- **版本**：v0.0.1（开发初期）
- **当前 Phase**：Phase 2 ✅ 完成，准备进入 Phase 3（Metadata）
- **最近 Commit**：见下方 Git 状态
- **GitHub**：✅ 已连接 `zerodaan318-lab/musicglass`（Private），main 已同步

## 阶段记录

### Phase 0 — 环境与 GitHub ✅ 完成
- Git 2.54.0 / gh 2.98.0 已就绪，已登录 `zerodaan318-lab`
- 本地 Git 初始化（main 分支）
- 创建 `.gitignore`（排除密钥/用户音乐/大型二进制/target/node_modules 等）
- 创建 `README.md` + `docs/README.md`
- 创建 GitHub 私有仓库 `musicglass`
- 初始 commit `23fe36c` 并 push 至 origin/main

### Phase 1 — 架构（文档族）✅ 完成
- `docs/ARCHITECTURE.md` — 分层/模块/Pipeline/插件/任务/安全边界
- `docs/TECH_STACK.md` — 选型总览与环境依赖
- `docs/PROJECT_PLAN.md` — 目标/铁律/Phase 计划/范围
- `docs/PROGRESS.md` — 本文件
- `docs/DECISIONS.md` — 技术决策记录（D-001~D-008）
- `docs/RECOVERY.md` — 从 GitHub 恢复开发环境手册
- `docs/formats/README.md` — 格式研究模板与索引
- `THIRD_PARTY_NOTICES.md` — 第三方依赖登记
- `scripts/git-sync.ps1` — 自动同步脚本（显式 add，禁 `git add .`）
- 工具链：Rust stable 1.98.0 装于 `D:\Hermes\mg-rust`（不污染全局）

### Phase 2 — 音频核心 ✅ 完成
- workspace `Cargo.toml`（7 个 member crate）
- `core`：AppError（含 code/suggestion 映射）、Metadata（Unified Model + lost_fields）、AudioInfo、路径安全工具、MusicContainer trait、container 类型
- `detector`：magic bytes + 扩展名格式识别（FLAC/Ogg/Opus/M4A/WAV/MP3/NCM/QMC...）
- `audio`：FFmpeg 封装骨架（参数数组调用，杜绝 shell 注入）、ffmpeg 路径解析、ConversionRequest 类型
- `metadata` / `task-manager`：类型骨架（Phase 3/4 扩充）
- `plugins`：MusicContainer trait + `Plugin` 枚举注册表（ncm/qmc 静态分发，无 dyn 无循环依赖）
- 单元测试：core 4 + detector 4 + audio 1 = 9 个，全部通过
- `cargo build` 与 `cargo test` 全绿
- commit `c2221f0`（含 Phase 1 文档补完）已 push

## 已完成
- 项目脚手架、Git/GitHub 初始化（Phase 0）
- 架构文档全套（Phase 1）：架构/技术栈/计划/进度/决策/恢复/格式研究框架/第三方登记/同步脚本
- 音频核心骨架（Phase 2）：workspace、检测器、转换引擎接口、插件系统抽象

## 正在进行
- Phase 3 前置：FFmpeg 二进制下载（后台进行，gyan.dev 源不稳定，正在重试）

## 待完成
- Phase 3 ~ Phase 11（见 `PROJECT_PLAN.md`）

## 已知问题
- FFmpeg 二进制仍未完整下载到 `resources/ffmpeg/`；Phase 2 的 `inspect()`/`convert()` 已实现骨架但未接真实 FFmpeg（需二进制就位后验证）
- `.toolchain/` 与 `resources/ffmpeg/` 已被 `.gitignore` 排除，不进 git

## Git / GitHub 状态
- Remote：`https://github.com/zerodaan318-lab/musicglass.git`
- Branch：`main`（track origin/main）
- 同步状态：✅ 已 commit + push 至 `c2221f0`
