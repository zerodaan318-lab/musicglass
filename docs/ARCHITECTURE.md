# MusicGlass 架构文档

> 本文档定义 MusicGlass 的软件架构、模块职责与核心数据流。所有开发必须遵循本文档的边界划分，禁止把多模块逻辑塞进单一文件。

## 1. 设计哲学

- **简单使用，复杂内部实现**：用户只看到「拖入 → 选格式 → 转换」三步；内部由多个独立 crate 协作完成识别、解析、转换、迁移、验证、记录。
- **三条铁律不可违反**（见 `PROJECT_PLAN.md`）：
  1. 不偷偷降音质，有损转换必须用户确认。
  2. Metadata / 封面 / 歌词能保尽保，丢失字段必须提示。
  3. 转换后必须重新读取输出文件并验证，Bit-perfect 不得虚标。

## 2. 总体分层

```text
┌─────────────────────────────────────────────┐
│  React + TypeScript UI  (packages/ui)        │  ← 玻璃拟态界面、拖拽、任务列表、设置
│  Tailwind CSS + Framer Motion               │
└───────────────┬─────────────────────────────┘
                │ Tauri IPC (invoke / events)
┌───────────────┴─────────────────────────────┐
│  apps/desktop/src-tauri  (Tauri 2 命令层)    │  ← Rust 命令桥接、窗口、文件系统权限
└───────────────┬─────────────────────────────┘
                │ 直接调用
┌───────────────┴─────────────────────────────┐
│  Rust Core Crates  (crates/*)                │  ← 业务逻辑：检测/转换/Metadata/任务/插件
└───────────────┬─────────────────────────────┘
                │ 进程调用
┌───────────────┴─────────────────────────────┐
│  FFmpeg (外部二进制，随包分发)                │  ← 编解码、格式封装、PCM 提取
│  SQLite (history.db)                         │  ← 转换历史记录
└─────────────────────────────────────────────┘
```

## 3. 模块划分（crates）

| Crate | 职责 | 关键产出 |
|-------|------|----------|
| `core` | 全局类型、错误定义、配置、路径安全处理、Unified Metadata 模型、日志初始化 | `Metadata`, `AudioInfo`, `ConversionConfig`, `AppError`, 路径校验工具 |
| `detector` | 格式识别：扩展名 + magic bytes + 容器结构；输出 `{format, confidence}` | `FormatDetector`, `DetectedFormat` |
| `audio` | FFmpeg 封装、Conversion Engine、AudioInfo 提取、QualityAnalyzer、输出验证 | `ConversionEngine`, `QualityAnalyzer`, `Verifier` |
| `metadata` | Unified Metadata 读写、Metadata Mapper、Cover 嵌入/提取、Lyrics 读写 | `MetadataMapper`, `CoverStore`, `LyricsIO` |
| `task-manager` | 任务队列、并发控制器、状态机、暂停/取消/重试、SQLite 历史读写 | `TaskQueue`, `ConcurrencyLimiter`, `HistoryStore` |
| `plugins` | 插件 trait + 注册器；专有格式实现（ncm, qmc） | `MusicContainer` trait, `PluginRegistry` |
| `plugins/ncm` | NCM 容器解析：解密、提取原始音频、Metadata、Cover、Lyrics | `NcmPlugin` |
| `plugins/qmc` | QMC 系列解析（按版本分处理），密钥缺失时明确报错 | `QmcPlugin` |

前端与后端共享数据结构放在 `packages/shared`（TypeScript 类型 ↔ Rust serde 结构对齐），保证 IPC 两侧字段一致。

## 4. 转换 Pipeline

统一入口，禁止为每种组合写 if/else（任务书第 6 节）：

```text
Input Path
  → Format Detection        (detector)
  → Source Inspection       (AudioInfo + Metadata + Cover + Lyrics)
  → [Plugin extraction if proprietary]   (plugins/*)
  → Decode to PCM/bytes     (FFmpeg)
  → Audio Processing        (sample rate / bit depth / channels 决策)
  → Encode to target        (FFmpeg, 按质量策略)
  → Metadata Mapping        (metadata)
  → Cover Embedding         (metadata)
  → Lyrics Embedding        (metadata)
  → Output File
  → Verification            (重新读取输出，逐项校验)
  → History Record          (task-manager → SQLite)
```

## 5. 插件系统

所有专有格式实现统一 trait（任务书第 7 节），主程序只通过 `PluginRegistry` 调用，禁止在 main 中写 `if ncm / if qmc`：

```rust
trait MusicContainer {
    fn can_handle(path: &Path) -> bool;
    fn inspect(path: &Path) -> Result<FileInfo, AppError>;
    fn extract_audio(path: &Path) -> Result<AudioStream, AppError>;
    fn extract_metadata(path: &Path) -> Result<Metadata, AppError>;
    fn extract_cover(path: &Path) -> Result<Option<CoverArt>, AppError>;
}
```

注册器在启动时加载所有内置插件；检测阶段按 `can_handle` 命中后交由对应插件处理。

## 6. 任务与并发模型

- `TaskQueue` 持有所有任务，状态：`Pending / Processing / Completed / Failed / Cancelled / Skipped`。
- `ConcurrencyLimiter` 默认 `CPU 逻辑核心 / 2`，用户可在设置中调整。
- 单任务失败不中断队列，记录错误并继续后续任务。
- 暂停 / 继续 / 取消 / 重试 / 跳过 通过任务级消息通道控制。
- 历史写入 `history.db`（SQLite），支持重开文件/文件夹/重试/清空。

## 7. 数据模型

`core::Metadata`（Unified Metadata Model，任务书第 20 节）为唯一中间表示：

```rust
struct Metadata {
    title, artist, album, album_artist,
    composer, lyricist, arranger, genre, year,
    track_number, track_total, disc_number, disc_total,
    comment, lyrics, bpm,
    // 扩展标签保留为 (key, value) 列表，尽量迁移
}
```

各格式读取后统一映射到此结构，输出时按目标格式能力回写；无对应字段则记录 `LossyMapping` 提示，绝不静默丢弃。

## 8. 验证系统

转换完成后必须重新读取输出文件并校验（任务书第 26 节）：
- 文件存在、可读取
- Codec / Sample Rate / Bit Depth / Channels / Duration
- Metadata 关键字段
- Cover 存在性
- Lyrics 存在性

无损且可严格等价的路径（如 FLAC→FLAC 无重编码）额外做 PCM 级比对，成立才显示 `Bit-perfect`。

## 9. 安全边界

- 路径遍历、特殊字符、超长路径、Emoji/中日文件名均需在 `core` 路径工具中处理。
- FFmpeg 通过参数数组调用（绝不字符串拼接 shell），杜绝命令注入。
- 日志禁止写入 Token / Key / Secret / 账号凭据。
- 不窃取任何第三方客户端凭据、Cookie、Token（任务书第 9 节）。

## 10. 错误处理

- 统一 `AppError` 枚举，前端映射为人类可读描述 + 可折叠 Technical Details。
- 插件解析失败返回明确原因与建议（如「需要额外密钥」而非泛泛 Error）。
- 禁止无错误处理直接 `unwrap` 上线。

## 11. 目录结构

```text
MusicGlass/
├── apps/desktop/        # Tauri 2 应用壳 (src-tauri + src)
├── crates/
│   ├── core/            # 类型/错误/配置/路径/Metadata 模型
│   ├── detector/        # 格式识别
│   ├── audio/           # FFmpeg 封装/转换引擎/验证
│   ├── metadata/        # Metadata 映射/Cover/Lyrics
│   ├── task-manager/    # 队列/并发/历史
│   └── plugins/
│       ├── ncm/
│       └── qmc/
├── packages/
│   ├── ui/              # React 前端
│   └── shared/          # 前后端共享类型
├── tests/               # 集成/回归测试
├── fixtures/            # 测试样本（小型，受 .gitignore 管控）
├── docs/                # 本文档及研究资料
├── scripts/             # git-sync.ps1 等
├── resources/           # 静态资源/图标
└── third_party/         # 第三方代码（带 License 记录）
```
