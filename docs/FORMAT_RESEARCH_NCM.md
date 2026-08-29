# NCM 格式研究文档（FORMAT_RESEARCH_NCM.md）

> 任务书 §8 要求：研究 NCM 文件结构、关键区域、Metadata、Cover、原始音频，记录已验证的样本与研究来源，并遵守对应项目 License。

## 1. 概述

NCM（NetEase Cloud Music）是网易云音乐客户端下载的**专有加密容器**。内部封装了一段标准音频（通常为 MP3 或 FLAC，极少数为 M4A/WAV/Ogg）以及歌曲元数据、封面图。文件本身不是音频容器格式，需解密后才能被标准播放器读取。

本实现**不破解任何版权保护机制**，仅对用户在网易云音乐客户端合法下载、本人拥有的文件进行本地格式转换。解密密钥（CORE_KEY / META_KEY）是网易云客户端公开内置的固定值，所有开源 ncmdump 工具均使用同样的值。

## 2. 二进制布局（按字节顺序）

```text
偏移        区域
0x00   Magic "CTENFDAM"          (8 字节固定标识)
0x08   Gap                       (2 字节，固定跳过)
0x0A   RC4 Key Segment:
         - 长度 (4 字节 LE)
         - 数据 (长度字节，每个字节 XOR 0x64)
0x0E+  Metadata Segment:
         - 长度 (4 字节 LE)
         - 数据 (长度字节，每个字节 XOR 0x63)
         ... (Gap 之后)
       CRC32 (4 字节) + Gap (5 字节)
       Cover Segment:
         - 长度 (4 字节 LE)
         - 数据 (原始 JPEG 或 PNG 字节)
       音频数据 (至文件末尾，自定义 RC4 流加密)
```

> 注：RC4 Key Segment 与 Metadata Segment 之后、Cover 之前存在一段固定 9 字节区域（CRC32 4 字节 + Gap 5 字节），解密时需 `seek` 跳过。

## 3. 解密算法

### 3.1 RC4 Key 解密

1. 读取 RC4 Key Segment 数据（已 XOR 0x64）。
2. AES-128-ECB 解密，密钥 `CORE_KEY = "hzHRAmso5kInbaxW"`（ASCII 16 字节）。
3. 解密结果应以 `"neteasecloudmusic"`（17 字节）开头，其后为真正的 RC4 密钥种子。

### 3.2 Metadata 解密

1. 读取 Metadata Segment 数据（已 XOR 0x63）。
2. 应以 `"163 key(Don't modify):"`（22 字节）开头，去除前缀。
3. 剩余部分为 Base64 编码，解码后得到 AES-128-ECB 密文。
4. AES-128-ECB 解密，密钥 `META_KEY = "#14ljk_!\]&0U<'("`（ASCII 16 字节）。
5. 解密结果应以 `"music:"`（6 字节）开头，去除前缀后为 JSON 文本。

### 3.3 音频流解密（自定义 RC4）

NCM 的流加密**不是标准 RC4**：

- **KSA（密钥调度）**：标准 RC4，用 RC4 Key 种子初始化 256 字节 S 盒。
- **PRGA（伪随机生成）**：定制公式，无状态、可按偏移直接计算：

```text
keystream[t] = S[ (S[t+1] + S[ (S[t+1] + t+1) & 0xff ]) & 0xff ]
```

其中 `t` 为 0 基字节偏移。由于公式无状态，可对任意偏移处的音频字节独立解密，便于分块流式处理（无需一次性载入整首歌曲到内存）。

音频数据逐字节与 keystream 异或即得到原始内嵌音频。

## 4. Metadata JSON 字段

解密后的 JSON 典型字段（来自 `music:` 之后）：

| 字段 | 含义 | 映射到 Unified Metadata |
|------|------|------------------------|
| `musicName` | 歌曲名 | `title` |
| `artist` | 数组，每项 `[name, id]` | `artist`（多艺术家用 ` / ` 连接） |
| `album` | 专辑名 | `album` |
| `albumId` | 专辑 ID | `extra["album_id"]` |
| `albumPic` | 封面 URL | `extra["cover_url"]` |
| `bitrate` | 码率（bps） | `extra["bitrate"]`（格式化为 kbps） |
| `duration` | 时长（毫秒） | `extra["duration_ms"]` |
| `format` | 内嵌音频格式字符串 | `extra["inner_format"]` |
| `lyric` | 歌词（部分样本有） | `lyrics` |
| `comment` | 注释（部分样本有） | `comment` |

> 封面二进制另存于 Cover Segment，与 `albumPic` URL 互补——优先使用 Cover Segment 的二进制图（避免联网下载）。

## 5. 内嵌音频格式嗅探

解密后的音频首字节决定真实格式：

| 魔数 / 特征 | 格式 |
|------------|------|
| `fLaC` | FLAC |
| `ID3` 或 `0xFF 0xFB`/`0xFF 0xFA` | MP3 |
| `....ftyp`（偏移 4） | M4A / AAC |
| `OggS` | Ogg Vorbis |
| `RIFF....WAVE` | WAV |

任务书 §8 强调：若内嵌为 FLAC，应**直接提取原始 FLAC**而非重新编码；若为 MP3，应 `MP3 → MP3` 尽量免重编码。当前 `extract_audio` 返回原始字节，由上层 `audio` 模块按需决定转码路径。

## 6. 已验证样本状态

| 样本 | 来源 | 状态 |
|------|------|------|
| 合成结构测试（仅验证 magic / 段解析路径） | 本地构造 | 单元测试通过 |
| 真实 `.ncm` 端到端解密 | **待补充** | 需要用户提供一个合法拥有的 `.ncm` 文件做实测验证 |

> 诚实声明：截至本文撰写，尚未用真实网易云 `.ncm` 文件做过端到端解密验证。解密算法本身来自多个成熟开源实现的公开文档，逻辑已通过单元级测试（RC4 公式、AES 密钥处理、metadata 映射、格式嗅探）。待用户提供样本后补充真实验证记录，届时更新本表。

## 7. 研究来源与 License 合规

本模块为**独立 clean-room 重新实现**，未直接复制任何上游源码。算法描述来自以下公开研究（均遵循各自 License，仅用于个人合法音乐存档）：

- `anonymous5l/ncmdump`（C++，MIT）
- `taurusxin/ncmdump`（Go，MIT）
- `nondanee/ncmdump`（C，MIT）
- `iqiziqi/ncmdump.rs`（Rust）
- `Johnserf-Seed/ncm2mp3`（Rust，Apache-2.0）—— 提供 NCM 自定义 RC4 PRGA 公式的权威描述
- `ncm_parser`（Rust crate，参考 AES/Base64 处理细节）

实现中不使用任何上游项目的专有代码，仅复用公开的固定密钥常量与文件结构知识。
