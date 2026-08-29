# QMC 格式研究文档（FORMAT_RESEARCH_QMC.md）

> 任务书 §9 要求：研究 QMC 系列格式结构、算法、参考实现，并遵守对应项目 License。

## 1. 概述

QMC（QQ Music Container）是腾讯 QQ 音乐客户端下载的**专有加密容器**，覆盖扩展名：`.qmc` / `.qmc0` / `.qmc2` / `.qmc3` / `.qmcogg` / `.mgg` / `.mgg0` / `.mgg1` / `.mflac` / `.mflac0`。

**关键事实（任务书 §9 明确要求"不要假设所有专有格式内部结构完全一样"）：QMC 实际是两代混合格式，算法差异很大：**

- **v1 静态加密**：整个文件用固定查找表的 XOR 流密码，无需每文件密钥。用于早期 `.qmc0` / `.qmcflac`。
- **QMC2（v2）**：密钥由文件尾部存储的 `ekey`（Base64）派生，派生的 key 驱动两种 cipher 之一——RC4 变体（key > 300 字节）或 Map 查表变体（key ≤ 300 字节）。文件尾部带 `QTag` 魔数标记。

QMC **没有独立内嵌的 metadata/cover 段**——解密后的输出就是标准内嵌音频（MP3 / FLAC / OGG 等），其标签与封面存在于解密后的音频本身，按标准格式读取即可。

本模块**不破解任何版权保护机制**，仅对用户在 QQ 音乐客户端合法下载、本人拥有的文件做本地格式转换。所有密钥/算法常量来自公开逆向研究，开源解码器均使用同样的值。

## 2. v1 静态加密（legacy）

整文件逐字节与 keystream 异或：

```text
for i in 0..file_len:
    out[i] = in[i] ^ keystream.next()
```

keystream 由 `QmcSeed` 状态机产生（参考 `presburger/qmc-decoder`，MIT License，seed.hpp 1:1 翻译）：

```text
初始: x = -1, y = 8, dx = +1, index = -1
固定查找表 seedMap[8][7]:
  [0x4a,0xd6,0xca,0x90,0x67,0xf7,0x52]
  [0x5e,0x95,0x23,0x9f,0x13,0x11,0x7e]
  [0x47,0x74,0x3d,0x90,0xaa,0x3f,0x51]
  [0xc6,0x09,0xd5,0x9f,0xfa,0x66,0xf9]
  [0xf3,0xd6,0xa1,0x90,0xa0,0xf7,0xf0]
  [0x1d,0x95,0xde,0x9f,0x84,0x11,0xf4]
  [0x0e,0x74,0xbb,0x90,0xbc,0x3f,0x92]
  [0x00,0x09,0x5b,0x9f,0x62,0x66,0xa1]

next_mask():
    index += 1
    if x < 0:        ret = 0xc3; dx = 1;  y = (8 - y) % 8
    elif x > 6:      ret = 0xd8; dx = -1; y = 7 - y
    else:            ret = seedMap[y][x]
    x += dx
    if index == 0x8000 or (index > 0x8000 and (index+1) % 0x8000 == 0):
        return next_mask()   # 每 0x8000 字节跳过 1 个位置
    return ret
```

前 10 字节 keystream（单元测试验证）：`c3 4a d6 ca 90 67 f7 52 d8 a1`

## 3. QMC2（v2）格式

### 3.1 文件尾部布局

```text
[ 加密音频数据 | metadata/ekey 区 | QTag 魔数 ]
                                       ^^^^^^^^ 末尾 4 字节 LE u32 = 0x67615451
```

- **v2 检测**：末尾 4 字节 == `"QTag"`（LE 0x67615451）。
- 倒数 8 字节：前 4 字节 = metadata 大小（**大端** u32），后 4 字节 = `QTag`。
- ekey 区格式：`ekey,songid,version,`（逗号分隔），ekey 在前。

- **v1 检测**（无 QTag 时）：末尾 4 字节 == ekey 长度（LE u32），范围 `1..=0x400`。ekey 紧接在其前。

### 3.2 ekey 派生

ekey 是 Base64 字符串，经 `parse_ekey` 还原出 cipher key：

1. Base64 解码。
2. 若以 `"QQMusic EncV2,Key:"` 开头：走两段 TEA 解密（STAGE1_KEY / STAGE2_KEY），再次 Base64 解码得到 v1 形式 ekey。
3. 取前 8 字节为 header，剩余为 body。TEA key 由 `derive_tea_key(header)` 构造（固定 simple-key `69 56 46 38 2b 20 15 0b` 与 header 交错成 16 字节）。
4. `body` 用 TEA 解密，结果 = `[header || decrypted_body]` = 最终 cipher key。

TEA：32 轮，小端 u32，标准 TEA 加解密（delta = 0x9E3779B9）。

### 3.3 两种 cipher

cipher key 长度决定算法：

- **RC4 变体**（key > 300 字节）：标准 RC4 KSA 初始化 S 盒，额外维护 `hash`（由 key 字节乘积溢出推导）。解密分段：
  - 首段（offset < 0x80）：`out[i] = in[i] ^ rc4_key[calc_segment_key(offset, rc4_key[offset%n]) % n]`
  - 其余段（0x1400 字节一块）：每块先按 `calc_segment_key(seg_id, ...)` 丢弃若干 RC4 输出，再逐字节 `^ rc4_derive()`。
  - `calc_segment_key(id, seed) = floor( hash / ((id+1)*seed) * 100 )`
- **Map 变体**（key ≤ 300 字节）：`map_l(offset) = scramble_by_index( key[(offset²+71214) % len], idx )`，`scramble_by_index` 为按 `(idx+4)&7` 位的循环移位异或。逐字节 `^ map_l(offset+i)`。

两种 cipher 的核心算法向量已对照 `bczhc/qmc-decrypt`（MIT/Apache）的单元测试逐字节验证。

## 4. 内嵌格式判定

解密后按扩展名/魔数识别内嵌音频：

| 扩展名 | 内嵌格式 |
|--------|----------|
| `.qmcflac` / `.mflac` / `.mflac0` | FLAC |
| `.mgg` / `.mgg0` / `.mgg1` / `.qmcogg` | OGG / OGG-based |
| `.qmc` / `.qmc0` / `.qmc2` / `.qmc3` | MP3 |

魔数兜底：`fLaC`→FLAC，`ID3`/`0xFF0xFB`→MP3，`OggS`→OGG，`RIFF...WAVE`→WAV，`....ftyp`→M4A。

## 5. Metadata 与 Cover

QMC 容器内不单独存储 metadata/cover。解密得到标准音频后，本插件将其写入临时文件并调用 `musicglass-metadata` 的 `read_metadata` / `read_cover`（底层 lofty）读取标签与封面——与处理普通音频文件完全一致。解密后优先采用无损路径（FLAC→FLAC 不重编码），严格遵守任务书 §3 音质铁律。

## 6. 已验证样本状态

| 样本 | 来源 | 状态 | 验证结果 |
|------|------|------|----------|
| keystream 单元测试 | 对照 seed.hpp 手动推导 | 通过 | 前 10 字节与参考实现一致 |
| RC4 / Map cipher 参考向量 | 取自 `qmc2-rust` 公开单元测试 | 通过 | 首段/第二段解密输出逐字节一致 |
| v1 静态 round-trip | 自构造（200KB，>0x8000 触发跳位逻辑） | 通过 | 加密→解密逐字节还原 |
| 真实 `.qmc*/.mgg` 端到端解密 | **待补充** | 需要用户合法拥有的样本 | — |

> 诚实声明：截至本文撰写，尚未用真实 QMC 文件做端到端解密验证（受版权约束不主动下载受限音乐）。keystream、RC4、Map 三类核心算法已通过对照公开参考实现的单元测试，v1 静态路径已通过自包含 round-trip 验证。待用户提供合法样本后补充真实验证记录并更新本表。

## 7. 研究来源与 License 合规

本模块为**独立 clean-room 重新实现**，未直接复制任何上游源码。算法描述来自公开研究：

- `Presburger/qmc-decoder`（C++，MIT）—— v1 静态 seedMap keystream（seed.hpp）
- `bczhc/qmc-decrypt` → `third_party/qmc2-rust`（Rust，MIT/Apache）—— QMC2 的 RC4/Map cipher、ekey TEA 派生、QTag 尾部检测（作为算法权威参考；其 `src/` 与 `third_party/` 均为公开学习资源）
- `unlock-music` 项目（公开测试夹具 `testdata/*_raw.bin`/`*_target.bin`/`*_key.bin`，用于解码器验证）

实现中仅复用公开的固定常量（查找表、TEA key、魔数）与文件结构知识，不复制任何专有代码。
