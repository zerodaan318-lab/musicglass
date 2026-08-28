# MusicGlass 进度

> 本文件持续更新。每完成一个 Phase：运行测试 → 检查 diff → commit → push → 更新本文件。

## 当前状态

- **版本**：v0.0.3（开发初期）
- **当前 Phase**：Phase 3 ✅ 完成，准备进入 Phase 4（任务系统）
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
- `core`：AppError（含 code/suggestion 映射）、Metadata（Unified Model + lost_fields）、AudioInfo、路径安全工具、MusicContainer trait、Format 枚举、container 类型
- `detector`：magic bytes + 扩展名格式识别（FLAC/Ogg/Opus/M4A/WAV/MP3/NCM/QMC...）
- `audio`：FFmpeg 封装骨架（参数数组调用，杜绝 shell 注入）、ffmpeg 路径解析（env→exe同目录→PATH）、ConversionRequest 类型、build.rs 自动复制 ffmpeg 到 target
- `metadata` / `task-manager`：类型骨架（Phase 3/4 扩充）
- `plugins`：MusicContainer trait + `Plugin` 枚举注册表（ncm/qmc 静态分发，无 dyn 无循环依赖）
- 单元测试：core 4 + detector 4 + audio 1 = 9 个，全部通过
- commit `c2221f0`（含 Phase 1 文档补完）已 push

### Phase 3 — Metadata ✅ 完成
- `metadata` crate 引入 `lofty 0.25` 做真实标签读写（覆盖 MP3/FLAC/M4A/WAV/OGG/OPUS + cover/lyrics）
- `reader.rs`：用 `ItemKey` + `get_string` 把各格式读成 Unified `Metadata`
- `writer.rs`：用 `ItemKey` + `insert_text` 把 Unified `Metadata` 写回目标格式（打开已有文件→填 primary tag→save）
- `cover.rs`：封面读取/嵌入（JPEG/PNG/WEBP 经 `MimeType::from_str` 兜底），`Picture::unchecked().build()` 构造
- `mapper.rs`：格式能力检查 + 丢失字段记录（如 WAV 不支持 Lyrics/BPM/Disc）
- `Format` 枚举从 detector 上移到 core（消除跨 crate 依赖）
- 集成测试 `tests/roundtrip.rs`：ffmpeg 生成静音 FLAC → 写含中文歌名/歌词/track/disc/year 的 Metadata → 读回全字段校验 ✓
- 全量测试：core 4 + detector 4 + audio 1 + metadata 7 = **16 个全部通过**
- FFmpeg n9.0.1（GPL 全编码器）已下载解压至 `resources/ffmpeg/bin/`，build.rs 自动复制到 `target/debug/resources/`

## 已完成
- 项目脚手架、Git/GitHub 初始化（Phase 0）
- 架构文档全套（Phase 1）
- 音频核心骨架（Phase 2）：workspace、检测器、转换引擎接口、插件系统抽象
- Metadata 真实读写（Phase 3）：Unified Model 映射、封面、歌词、丢失记录、集成验证

## 待完成
- Phase 4 ~ Phase 11（见 `PROJECT_PLAN.md`）

## 已知问题
- `lofty` 的 `MimeType` 枚举无 Webp 变体，封面写 webp 时经 `MimeType::from_str` 落入 `Unknown` 分支（数据保留，mime 标记为 image/webp）；读取时也能正确取回 bytes。功能可用，仅类型枚举不显式标注 webp。
- `resources/ffmpeg/` 二进制已被 `.gitignore` 排除，不进 git（随包分发，符合任务书第 56 节）

## Git / GitHub 状态
- Remote：`https://github.com/zerodaan318-lab/musicglass.git`
- Branch：`main`（track origin/main）
- 同步状态：✅ 已 commit + push（Phase 3 commit 见下方）
