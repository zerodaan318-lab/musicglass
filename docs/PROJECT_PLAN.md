# MusicGlass 项目计划

## 1. 目标与范围（第一阶段）

开发完全本地运行的 **Windows 音乐格式转换器**：用户拖入已有音乐文件 → 自动识别（含 NCM/QMC 等专有格式）→ 转换 → 完整迁移 Metadata/封面/歌词 → 验证输出 → 记录历史。

**用户只关心结果**：原音乐 → 高质量转换 → 歌曲信息完整 → 可靠新文件。

## 2. 三条最高优先级原则（铁律）

1. **尽可能不降低音质**：禁止后台偷偷降 Bitrate / Sample Rate / Bit Depth / Channels；有损输出（如 FLAC→MP3）必须明确提示并获用户确认；有损源转无损（MP3→FLAC）须明确告知「不会恢复已损失信息」。
2. **Metadata 尽可能完整保留**：标题/歌手/专辑/编号/封面/歌词等能保尽保；目标格式无对应字段时记录并提示，不得静默丢弃。
3. **转换完成必须验证**：重新读取输出文件，校验音频参数、Metadata、封面、歌词；严格等价路径才标 `Bit-perfect`，禁止虚假显示。

## 3. 支持格式

- **输入**：MP3 / FLAC / WAV / M4A / AAC / OGG / OPUS / APE / WMA + 专有 NCM / QMC 系列（插件化）
- **输出**：MP3 / FLAC / WAV / M4A / AAC / OGG / OPUS

## 4. Phase 计划（状态实时见 `PROGRESS.md`）

| Phase | 内容 | 状态 |
|-------|------|------|
| 0 | 环境 + GitHub 初始化 | ✅ 完成 |
| 1 | 架构文档（本文档族） | 🔄 进行中 |
| 2 | 音频核心：FormatDetector / AudioInfo / Conversion Engine / FFmpeg | ⬜ 待开始 |
| 3 | Metadata：Unified Model / Mapper / Cover / Lyrics / Verification | ⬜ |
| 4 | 任务系统：Batch / Queue / Pause / Cancel / Retry / History | ⬜ |
| 5 | NCM 插件 + 测试 + 研究文档 | ⬜ |
| 6 | QMC 插件 + 测试 + 研究文档 | ⬜ |
| 7 | UI：玻璃拟态 / 深浅色 / 拖拽 / 任务列表 / 设置 | ⬜ |
| 8 | EXE：Installer + Portable | ⬜ |
| 9 | 压力测试：100/500/1000 文件、大文件、损坏文件 | ⬜ |
| 10 | 稳定性：错误处理 / 日志 / 恢复 / 防崩溃 | ⬜ |
| 11 | v1.0.0 验收发布 | ⬜ |

## 5. 不在第一阶段范围

Web 在线版、服务器、云端转换、登录、音乐下载/搜索、平台 API、云数据库、账号系统、Docker/网站部署（任务书第 84 节）。

## 6. 工程纪律（强制）

- 每完成一个功能独立 `commit`，持续 `push`（任务书第 52-53 节）。
- 单文件过大必须拆分；禁止全局变量硬编码、路径硬编码、无测试交付。
- 测试至少覆盖：格式检测 / Metadata / Cover / Lyrics / 文件名模板 / 任务管理 / 路径处理 / NCM / QMC / FFmpeg / 验证。
- 三态齐全（Local + Commit + Push）才算完成（任务书第 71 节）。
- `.gitignore` 排除密钥、用户音乐、大型二进制；GitHub 仓库默认 Private。

## 7. 验收标准索引

最终 v1.0.0 验收项见任务书第 83 节，覆盖：文件导入、普通格式、专有格式、音质、Metadata、任务、UI、EXE、工程。
