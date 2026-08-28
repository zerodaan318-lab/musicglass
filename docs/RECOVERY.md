# MusicGlass 恢复手册（RECOVERY）

> 目标：即使本地项目目录完全丢失，也能从 GitHub 恢复开发（任务书第 72 节）。
> 本文档随 Phase 进展持续补全命令与版本号。

---

## 1. 从 GitHub 克隆
```powershell
# 任意干净目录（建议非 OneDrive，例如 D:\Hermes）
git clone https://github.com/zerodaan318-lab/musicglass.git D:\Hermes\MusicGlass
cd D:\Hermes\MusicGlass
```

## 2. 恢复开发环境

### 2.1 Rust 工具链
本项目 Rust 装在非 OneDrive 目录，重装命令：
```powershell
$env:RUSTUP_HOME = "D:\Hermes\mg-rust\rustup"
$env:CARGO_HOME  = "D:\Hermes\mg-rust\cargo"
# rustup-init.exe 可从 https://rustup.rs 下载（Windows x64）
D:\Hermes\rustup-init.exe -y --no-modify-path --default-toolchain stable --profile minimal
```
验证：`D:\Hermes\mg-rust\cargo\bin\cargo --version`

### 2.2 Node.js + pnpm
- 安装 Node.js LTS（官网或 nvm）。
- `npm i -g pnpm`
- 项目前端依赖：`pnpm install`（在仓库根或 `apps/desktop` 视脚手架而定）

### 2.3 FFmpeg（随包分发，不进 Git）
开发期联调需要本地 FFmpeg：
- 自动：运行 `D:\Hermes\dl_ffmpeg.ps1`（下载并解压到 `D:\Hermes\mg-ffmpeg`，再拷入 `resources/ffmpeg/bin`）。
- 手动：从 https://www.gyan.dev/ffmpeg/builds 取 release-essentials，解压后把 `bin/ffmpeg.exe`、`bin/ffprobe.exe` 放到 `resources/ffmpeg/bin/`。

## 3. 构建
```powershell
# 核心 crate 检查/测试
$env:CARGO_HOME = "D:\Hermes\mg-rust\cargo"; $env:RUSTUP_HOME = "D:\Hermes\mg-rust\rustup"
D:\Hermes\mg-rust\cargo\bin\cargo test --workspace

# Tauri 桌面应用（开发模式）
pnpm tauri dev

# 打包 EXE / Portable（Phase 8）
pnpm tauri build
```

## 4. 运行
- 开发：`pnpm tauri dev`
- 生产：构建后的 `MusicGlass.exe`（Portable）或安装 `MusicGlass-Setup.exe`

## 5. 恢复历史数据库
- `history.db`（SQLite）位于用户数据目录（具体路径 Phase 4 确定）。
- 若文件丢失，程序启动时应自动重建空表（见 `task-manager` 的 `HistoryStore::ensure_schema`）。
- 重要历史建议定期从用户数据目录另存备份。

## 6. 生成 EXE（Phase 8 补全）
- Installer：`pnpm tauri build` 产出 `MusicGlass-Setup.exe`
- Portable：Tauri 产出 portable 包（具体形态 Phase 8 确定）
- FFmpeg 与 SQLite 随包内嵌，用户无需额外安装。

## 7. 常见问题
- **WebView2 缺失**：Windows 运行 `pnpm tauri dev`/`build` 前需安装 WebView2 Runtime。
- **中文/Emoji 路径乱码**：确保 Rust `std::fs` 与 FFmpeg 参数均走 UTF-8；Windows 下用 `std::os::windows::ffi::OsStrExt` 正确传参。
- **OneDrive 同步冲突**：Rust target / node_modules 必须放在非 OneDrive 目录，勿纳入同步。
