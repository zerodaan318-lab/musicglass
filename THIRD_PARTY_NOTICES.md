# 第三方依赖登记

> 所有引入的 Rust crate / npm 包 / 外部二进制 / 算法实现均须在此登记（任务书第 50 节）。
> 禁止引入 License 不明的代码。

## Rust Crates（Cargo 依赖）

| 名称 | 版本 | 来源 | License | 用途 |
|------|------|------|---------|------|
| `serde` | 1 | crates.io | MIT/Apache-2.0 | 序列化/反序列化（Metadata、AudioInfo） |
| `thiserror` | 1 | crates.io | MIT/Apache-2.0 | 错误类型派生宏 |
| `log` | 0.4 | crates.io | MIT/Apache-2.0 | 日志门面 |
| `anyhow` | 1 | crates.io | MIT/Apache-2.0 | 备用错误容器 |

> 更多 crate 将在 Phase 3+ 引入（如 `rusqlite`、`walkdir`、`rayon` 等）时补充。

## 外部二进制

| 名称 | 版本 | 来源 | License | 用途 |
|------|------|------|---------|------|
| FFmpeg | 待锁定（gyan.dev essentials 最新稳定） | https://www.gyan.dev/ffmpeg/builds/ | LGPL-2.1+ / GPL（按启用组件） | 音频编解码、封装、PCM 提取 |
| Rust toolchain | stable (1.98.0) | https://static.rust-lang.org | MIT/Apache-2.0 | 编译核心与 Tauri 后端 |

## npm 包
（Phase 7 引入前端依赖时补充：React、TypeScript、Vite、Tailwind CSS、Framer Motion 等）

## 专有格式研究来源
- NCM：参见 `docs/FORMAT_RESEARCH_NCM.md`（Phase 5 编写），参考开源实现须遵守其 License。
- QMC：参见 `docs/formats/qmc.md`（Phase 6 编写）。
