# MusicGlass

> 完全本地运行的 Windows 音乐格式转换器 —— 拖入音乐，智能识别，无损优先，歌曲信息完整迁移，输出必验证。

MusicGlass 是一个处理**用户已有本地音乐文件**的格式转换器，不是音乐下载器、搜索器或平台客户端。

## 核心特性

- **拖入即用**：拖入文件/文件夹，自动识别格式（含 NCM、QMC 等专有格式）
- **无损优先**：不偷偷降音质，有损转换明确提示并需用户确认
- **Metadata 完整迁移**：标题/歌手/专辑/封面/歌词等能保尽保，丢失字段必有提示
- **输出验证**：转换后重新读取输出文件，验证音频参数、Metadata、封面、歌词
- **批量任务**：队列、暂停、取消、重试、历史记录
- **Glassmorphism UI**：现代、极简、深色/浅色主题

## 支持格式

- **输入**：MP3 / FLAC / WAV / M4A / AAC / OGG / OPUS / APE / WMA + 专有格式 NCM / QMC 系列（插件化）
- **输出**：MP3 / FLAC / WAV / M4A / AAC / OGG / OPUS

## 技术栈

Tauri 2 · Rust · React + TypeScript · FFmpeg · SQLite · Tailwind CSS

## 文档

- [架构](docs/ARCHITECTURE.md) · [技术栈](docs/TECH_STACK.md) · [项目计划](docs/PROJECT_PLAN.md)
- [进度](docs/PROGRESS.md) · [决策记录](docs/DECISIONS.md) · [恢复手册](docs/RECOVERY.md)
- [格式研究](docs/formats/) · [第三方依赖](THIRD_PARTY_NOTICES.md)

## 构建

（Phase 8 提供 Installer + Portable 产物，参见 docs/RECOVERY.md）

## License

见仓库根目录 LICENSE 文件。

> 本项目为开发阶段，仓库默认 Private。
