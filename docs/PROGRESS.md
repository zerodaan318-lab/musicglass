# MusicGlass 进度

> 本文件持续更新。每完成一个 Phase：运行测试 → 检查 diff → commit → push → 更新本文件。

## 当前状态

- **版本**：v0.0.3（开发初期）
- **当前 Phase**：Phase 6 ✅ 完成（QMC 插件 + 格式研究），准备进入 Phase 7（UI）
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

### Phase 4 — 任务系统 ✅ 完成
- `task-manager` crate：
  - `tasks.rs`：TaskStatus 六态状态机、TaskManager（并发 worker 池，默认 CPU/2）、submit/cancel/retry、run_all 批量执行、单任务失败隔离（不影响其他）、cancel 通过 AtomicBool 协作式信号
  - `history.rs`：rusqlite (bundled) SQLite 历史表，record/query_all/clear（输入/输出/格式/大小/状态/时间/时长/错误）
- `audio` crate 真实化：
  - `inspect()`：ffprobe JSON 解析出 AudioInfo（codec/采样率/声道/位深/码率/时长/大小/无损判断）
  - `convert()`：真实 ffmpeg 参数映射（无损 copy 优先；mp3/flac/wav/m4a/aac/ogg/opus 编码器选择；码率/采样率参数）
  - `verify.rs`：转换后重新 inspect + 元数据对账（采样率/声道/时长/标题-艺术家-专辑 保真），音频+元数据双校验
  - `Format::from_codec()` 映射 ffmpeg codec 名
- 集成测试 `audio/tests/e2e.rs`：生成 FLAC → 写 Metadata → 转 MP3 320k → verify → history 写入查询，全链路通过
- 全量测试：core 4 + detector 4 + audio 2(e2e+unit) + metadata 7 + task-manager 6 = **23 个全部通过**

### Phase 5 — NCM 插件 ✅ 完成
- `crates/plugins/ncm/src/decrypt.rs`：clean-room 重新实现的 NCM 解密
  - 二进制布局解析：`CTENFDAM` magic + gap → RC4 Key Segment(XOR 0x64 → AES-128-ECB CORE_KEY → 去"neteasecloudmusic") → Metadata Segment(XOR 0x63 → 去"163 key(Don't modify):" → Base64 → AES-128-ECB META_KEY → 去"music:" → JSON) → CRC32+gap → Cover Segment → 音频(自定义 RC4 流加密)
  - 自定义 RC4 PRGA：`keystream[t] = S[(S[t+1] + S[(S[t+1]+t+1)&0xff])&0xff]`，无状态可分块解密
  - 依赖：`aes` 0.8 + `cipher`(block-padding) + `base64` 0.22 + `block-padding` 0.3
- `crates/plugins/ncm/src/metadata.rs`：NCM JSON → 统一 Metadata 模型映射 + 内嵌音频格式嗅探(detect_by_magic)
- `crates/plugins/ncm/src/lib.rs`：实现 `MusicContainer` trait 全部 5 个方法（can_handle / inspect / extract_audio / extract_metadata / extract_cover）
- `docs/FORMAT_RESEARCH_NCM.md`：格式结构、解密算法、字段映射、研究来源与 License 合规（任务书 §8 强制要求）
- `crates/plugins/ncm/tests/metadata.rs`：6 个单元测试（格式嗅探 + metadata 映射）全过
- **诚实声明**：尚未用真实 `.ncm` 文件做端到端解密验证，待用户提供合法拥有的样本后补充（文档已记录）
- 全量构建通过，新增 6 测试，总计 **29 个测试全过**

### Phase 6 — QMC 插件 ✅ 完成
- `crates/plugins/qmc/src/decrypt.rs`：clean-room 重新实现 QMC 两代格式
  - **v1 静态**：整文件逐字节 XOR 固定 8×7 keystream（参考 `presburger/qmc-decoder` MIT，seed.hpp 1:1 翻译），每 0x8000 字节跳 1 位
  - **QMC2 (v2)**：尾部 `QTag` 检测 + ekey TEA 派生（含 `QQMusic EncV2,Key:` 两段 TEA）→ 选 RC4 变体(key>300B) 或 Map 变体(key≤300B)
  - RC4/Map 算法向量对照 `bczhc/qmc-decrypt`（`third_party/qmc2-rust`，MIT/Apache）公开单测逐字节验证
- `crates/plugins/qmc/src/metadata.rs`：扩展名→内嵌格式映射 + 魔数嗅探
- `crates/plugins/qmc/src/lib.rs`：实现 `MusicContainer` trait 全部 5 方法
  - QMC 无独立 metadata/cover 段：解密后落临时文件，调 `musicglass-metadata` 读标签/封面
- `docs/FORMAT_RESEARCH_QMC.md`：两代格式结构/算法/来源/License 合规（任务书 §9 强制要求）
- 测试：`tests/keystream.rs`（5 个 v1 keystream + 嗅探）+ `tests/e2e_roundtrip.rs`（RC4/Map 参考向量 + v1 round-trip）
- 全量构建通过，qmc 新增 9 测试，总计 **38 个测试全过**
- **诚实声明**：未用真实 QMC 文件端到端验证（版权约束不主动下载），核心算法已对照公开参考实现单测 + v1 自包含 round-trip；待用户提供合法样本补充

## 已完成
- 项目脚手架、Git/GitHub 初始化（Phase 0）
- 架构文档全套（Phase 1）
- 音频核心骨架（Phase 2）
- Metadata 真实读写（Phase 3）
- 任务系统：队列/并发/取消重试/SQLite 历史 + 端到端转换验证（Phase 4）

### Phase 7 — UI（玻璃拟态前端）✅ 完成
- 技术栈（任务书 §40/§44）：React 18 + TypeScript + Vite + Tailwind CSS 3 + Framer Motion 11；桌面壳后续 Tauri 2（`apps/desktop`，Phase 8）
- 目录：`packages/ui`（React 前端）、`packages/shared`（前后端共享 TS 类型，与 Rust `core` serde 结构对齐）
- 设计系统（`packages/ui/src/index.css` + `tailwind.config.js`）：
  - 玻璃拟态 `glass`/`glass-strong` 类（blur 18~24px + saturate），CSS 变量驱动，**保证对比度/按钮识别/文字可读性**（任务书 §28 约束）
  - 深色优先 + `dark/light/system` 三态主题（`theme.tsx` 监听 system 变化）
  - 主题色全部走 Tailwind 语义色（`bg/surface/text/accent/...`）
- 公共组件（`packages/ui/src/components/`）：`GlassCard`、`Button`、`ProgressBar`、`ErrorCard`（人类可读 + 可折叠 Technical Details，任务书 §32）、`Sidebar`、`Icon`（内联 SVG，零额外依赖）
- 页面（任务书 §27-§38，组件化拆分，禁止单文件塞全部，§45）：
  - `HomePage`（拖拽/Add Files/Folder + 支持格式列表，§28）
  - `ImportSummary`（文件数/Supported/NCM/QMC 计数/总时长，§29）
  - `ConvertPage`（Input/Output Format/Quality/Metadata/Cover/Lyrics/Output Folder/Template/Start；有损转换强制确认，原则一，§30）
  - `TaskList`（Cover/Title/Artist/格式箭头/进度/速度/剩余/Pause/Cancel，§31）
  - `SettingsPage`（General/Appearance/Conversion/Output/Metadata/Performance/Advanced/About，§33-§37）
- 状态管理：`store.tsx`（轻量 `useAppStore`，受控状态，无散落全局变量）；`App.tsx` 侧边栏导航 + 视图切换 + 全局错误层
- 验证：`pnpm build`（tsc -b && vite build）零错误通过

### Phase 8 — Tauri 桌面壳 ✅ 完成（本机真机验证通过）
- 建立 `apps/desktop/src-tauri/`（Tauri 2 桌面壳），把 `packages/ui` 与 `crates/*` 桥接
- 文件：`Cargo.toml`（脱离根 workspace，path 依赖 crates）、`tauri.conf.json`、`capabilities/default.json`、`build.rs`、`src/main.rs`
- `main.rs` 实现 IPC 命令接真实后端（任务书 §7 插件架构，无 if ncm/if qmc）：
  - `detect_files` → `detector::detect`；`inspect_audio` → `audio::inspect`
  - `extract_proprietary` → `Plugin::find().extract_audio`（NCM/QMC 解密）
  - `extract_metadata` → `plugin.extract_metadata` / `metadata::read_metadata`
  - `convert_audio` → 真实转码 + **Metadata 嵌入（铁律二）+ 转后验证（铁律三）**，返回 `ConvertResultDto { outputPath, verification }`
  - `reveal_file` → 打开输出所在文件夹；`doctor` → `audio::ffmpeg_available`
- 前端 `tauri.ts` 适配层：动态 `import('@tauri-apps/api')`，**Tauri 内真 invoke、纯 Vite 预览降级模拟**（同一套 UI 双模式）
- store 改造：`addFilesFromPaths` 走 `detectFiles`（真实+降级），`startConversion` 透传真实路径并回写验证结果
- TaskList 已完成项显示「已验证 ✓ / 验证警告 ⚠」徽章（不等价不谎称 Bit-perfect）
- **本机真机验证（诚实记录）**：
  - ✅ `cargo tauri dev` 已在本机真运行（MSVC + WebView2 实际可用，之前文档误判为缺环境）
  - ✅ 拖入 / 转换 / 浏览 / 打开 / 验证 全链路实测通过
  - ✅ NCM→MP3、FLAC→MP3 等路径验证 metadata 嵌入 + 验证徽章正常
- cargo-tauri CLI 已装（`v2.11.4`，装在 D:\Hermes\mg-rust）
- **遗留**：EXE 打包（nsis 安装包 + zip 便携包）属 Phase 8 末尾，尚未做，见下方待完成

## 待完成
- Phase 8 EXE 打包（nsis 安装包 + zip 便携包，目标 §8）
- Phase 9 ~ Phase 11（见 `PROJECT_PLAN.md`）
- Phase 9 ~ Phase 11（见 `PROJECT_PLAN.md`）

## 已知问题
- `lofty` 的 `MimeType` 枚举无 Webp 变体，封面写 webp 时经 `MimeType::from_str` 落入 `Unknown` 分支（数据保留，mime 标记为 image/webp）；读取时也能正确取回 bytes。功能可用，仅类型枚举不显式标注 webp。
- `resources/ffmpeg/` 二进制已被 `.gitignore` 排除，不进 git（随包分发，符合任务书第 56 节）

## Git / GitHub 状态
- Remote：`https://github.com/zerodaan318-lab/musicglass.git`
- Branch：`main`（track origin/main）
- 同步状态：✅ 已 commit + push（Phase 3 commit 见下方）
