# MusicGlass 恢复手册

> 目标：即使本地项目目录完全丢失，也能从 GitHub 恢复完整开发环境（任务书第 72 节）。

## 1. 从 GitHub 克隆

```powershell
git clone https://github.com/zerodaan318-lab/musicglass.git
cd musicglass
```

如果仓库是 Private，先确保已登录 GitHub（`gh auth login` 或配置 SSH key）。

## 2. 恢复开发环境

### 2.1 Rust 工具链
本项目工具链装在 `D:\Hermes\mg-rust`（非默认路径）。恢复方式：

```powershell
$env:RUSTUP_HOME = "D:\Hermes\mg-rust\rustup"
$env:CARGO_HOME  = "D:\Hermes\mg-rust\cargo"
# 若目录不存在，先下载 rustup-init.exe 并安装：
#   D:\Hermes\rustup-init.exe -y --no-modify-path --default-toolchain stable --profile minimal
# 然后设默认：
rustup default stable-x86_64-pc-windows-msvc
```

调用 cargo 时用绝对路径：`D:\Hermes\mg-rust\cargo\bin\cargo`。

### 2.2 Node.js
前端用 Node 22+（npm 10+）。从 nodejs.org 安装 LTS 即可。

### 2.3 Tauri CLI
```powershell
npm install -g @tauri-apps/cli@latest
```

### 2.4 FFmpeg（随包分发，不进 git）
下载 static build 解压到 `resources/ffmpeg/`（结构见 `.gitignore` 排除项）：
- 下载 `ffmpeg-release-essentials.zip`（gyan.dev 或 BtbN）
- 解压后确保 `resources/ffmpeg/bin/ffmpeg.exe` 与 `ffprobe.exe` 存在

## 3. 构建

```powershell
# 后端
cargo build

# 前端 + 桌面（需先装 Tauri CLI 与 WebView2）
cd apps/desktop
npm install
npm run tauri dev      # 开发模式
npm run tauri build    # 产出 EXE / Portable
```

## 4. 运行测试

```powershell
cargo test
```

## 5. 恢复数据库
无需恢复——`history.db` 是运行时在用户机器生成的本地文件，不进 git，首次运行自动建表。

## 6. 生成 EXE
见 Phase 8 产物：`MusicGlass-Setup.exe` 与 `MusicGlass-Portable.zip`（或 Portable exe），FFmpeg 一并打包。

## 7. 快速核对清单
- [ ] `git clone` 成功
- [ ] `cargo --version` 可用
- [ ] `resources/ffmpeg/bin/ffmpeg.exe` 存在
- [ ] `cargo build` 通过
- [ ] `cargo test` 全绿
