# MusicGlass 桌面壳 (apps/desktop)

Tauri 2 桌面应用壳，把 `packages/ui`（React 前端）与 `crates/*`（Rust 业务核心）桥接起来。

## 架构

```
packages/ui  (React + Vite)  ──IPC invoke──▶  src-tauri/src/main.rs  ──▶  crates/*
   Glassmorphism UI              #[tauri::command]          detect/audio/plugins/metadata
```

前端通过 `packages/ui/src/tauri.ts` 调用 Rust 命令；在纯 `pnpm dev` 预览下自动降级为模拟数据。

## IPC 命令（main.rs 已实现）

| 命令 | 参数 | 后端调用 | 说明 |
|------|------|----------|------|
| `detect_files` | `paths: Vec<String>` | `musicglass_detector::detect` | 批量格式识别（§10） |
| `inspect_audio` | `path` | `musicglass_audio::inspect` | ffprobe 音频信息（§6） |
| `extract_proprietary` | `path, out_dir` | `musicglass_plugins::Plugin::find().extract_audio` | NCM/QMC 解密（§7/§8） |
| `extract_metadata` | `path` | `plugin.extract_metadata` / `metadata::read_metadata` | 统一 Metadata（§20） |
| `convert_audio` | `input, output, target_format, bitrate?` | `musicglass_audio::convert` | FFmpeg 转换（§6） |
| `doctor` | — | `audio::ffmpeg_available` | 启动自检 |

## 构建

```bash
# 开发（前端热更新 + Rust 编译）
cd apps/desktop
cargo tauri dev

# 打包（Phase 8 EXE，生成 nsis 安装包 + zip 便携包）
cargo tauri build
```

## ⚠ 环境依赖（首次搭建必读）

本机当前**尚未满足** Tauri 2 编译/运行条件，需补齐以下三项后才能 `cargo tauri dev/build`：

1. **MSVC 构建工具（必装，需管理员）**
   - Tauri 的 `tauri-build` 链接 Windows 系统库需要 `cl.exe`（Visual Studio Build Tools / MSVC）。
   - 以**管理员** PowerShell 运行：
     ```powershell
     winget install Microsoft.VisualStudio.2022.BuildTools --silent --override "--wait --add Microsoft.VisualStudio.Workload.VCTools --add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 --add Microsoft.VisualStudio.Component.Windows11SDK.22621"
     ```
   - 验证：`cl.exe` 出现在 `C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\*\bin\Hostx64\x64\`

2. **WebView2 Runtime（必装，渲染前端）**
   - `winget install Microsoft.WebView2Runtime` 已下架，改用微软官方直链：
     https://go.microsoft.com/fwlink/?LinkId=2093437 （MicrosoftEdgeWebView2RuntimeInstallerX64.exe）
   - 下载后双击或管理员运行。

3. **cargo-tauri CLI（已在装）**
   - `cargo install tauri-cli --version "^2"`（编译较慢，约数分钟）

补齐后执行 `cargo tauri dev` 即可真机验证完整链路。

## 当前状态（诚实声明）

- ✅ 壳代码完整：Cargo.toml / tauri.conf.json / capabilities / build.rs / main.rs（7 个 IPC 命令接真实 crate）
- ✅ 前端适配层 `tauri.ts` 完成，store 已切到 `detectFiles`（真实+降级）
- ❌ 本机未验证编译/运行（缺 MSVC + WebView2，见上）
- 补齐环境前，仅能 `pnpm dev` 纯前端预览（降级模拟）
