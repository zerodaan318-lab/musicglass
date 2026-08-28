# 第三方依赖与开源声明（THIRD_PARTY_NOTICES）

> 任务书第 50/57 节强制要求：任何引入的 GitHub 项目 / Rust crate / npm 包 / FFmpeg / 算法实现，都必须记录名称、版本、来源、License、用途。
> 禁止直接复制 License 不明的项目代码。

## 格式（每新增一项追加）

| 名称 | 版本 | 来源 | License | 用途 |
|------|------|------|---------|------|
| FFmpeg | (固定 release，待填) | https://ffmpeg.org | GPL-2.0+ (部分 LGPL) | 编解码 / 格式封装 / PCM 提取 |
| rusqlite | (待填) | https://github.com/rusqlite/rusqlite | MIT | SQLite 访问 |
| tokio | (待填) | https://github.com/tokio-rs/tokio | MIT | 异步运行时 |
| Tauri 2 | (待填) | https://github.com/tauri-apps/tauri | MIT/Apache-2.0 | 桌面壳 / IPC |
| React | (待填) | https://github.com/facebook/react | MIT | 前端 UI |
| Vite | (待填) | https://github.com/vitejs/vite | MIT | 前端构建 |
| Tailwind CSS | (待填) | https://github.com/tailwindlabs/tailwindcss | MIT | 样式 |
| Framer Motion | (待填) | https://github.com/framer/motion | MIT | 动画 |
| serde | (待填) | https://github.com/serde-rs/serde | MIT/Apache-2.0 | 序列化 |

## 专有格式研究来源（NCM / QMC）
- 具体开源实现链接、对应 License、采用的研究结论，在 `docs/formats/ncm.md`、`docs/formats/qmc.md` 与 `docs/FORMAT_RESEARCH_NCM.md` 中记录。
- **重要**：仅参考其解析思路与密钥处理方案，遵守 License；不直接复制不明 License 的代码。
