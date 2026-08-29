# QMC 格式研究文档（FORMAT_RESEARCH_QMC.md）

> 任务书 §9 要求：研究 QMC 系列格式结构、算法、参考实现，并遵守对应项目 License。

## 1. 概述

QMC（QQ Music Container）是腾讯 QQ 音乐客户端下载的**专有加密容器**，覆盖扩展名：`.qmc` / `.qmc0` / `.qmc2` / `.qmc3` / `.qmcogg` / `.mgg` / `.mgg0` / `.mgg1` / `.mflac` / `.mflac0`。与 NCM 不同，QMC **没有独立的内嵌 metadata/cover 段**——整个文件是一个统一的 XOR 流密码，解密后的输出就是标准内嵌音频（MP3 / FLAC / OGG 等），其标签与封面存在于解密后的音频本身，按标准格式读取即可。

本模块**不破解任何版权保护机制**，仅对用户在 QQ 音乐客户端合法下载、本人拥有的文件做本地格式转换。密钥流查找表是 QQ 音乐客户端公开内置的固定常量，所有开源解码器均使用同样的值。

## 2. 解密算法

整文件逐字节与 keystream 异或：

```text
for i in 0..file_len:
    out[i] = in[i] ^ keystream.next()
```

keystream 由 `seed` 状态机产生（参考 `presburger/qmc-decoder`，MIT License）。

### 状态机（1:1 对应 seed.hpp）

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

前 10 字节 keystream（单元验证通过）：`c3 4a d6 ca 90 67 f7 52 d8 a1`

## 3. 内嵌格式判定

解密后按扩展名/魔数识别内嵌音频：

| 扩展名 | 内嵌格式 |
|--------|----------|
| `.qmcflac` / `.mflac` / `.mflac0` | FLAC |
| `.mgg` / `.mgg0` / `.mgg1` / `.qmcogg` | OGG / OGG-based |
| `.qmc` / `.qmc0` / `.qmc2` / `.qmc3` | MP3 |

魔数兜底：`fLaC`→FLAC，`ID3`/`0xFF0xFB`→MP3，`OggS`→OGG，`RIFF...WAVE`→WAV，`....ftyp`→M4A。

## 4. Metadata 与 Cover

QMC **不在容器内单独存储** metadata/cover。解密得到标准音频后，本插件将其写入临时文件并调用 `musicglass-metadata` 的 `read_metadata` / `read_cover`（底层 lofty）读取标签与封面——与处理普通音频文件完全一致。解密后优先采用无损路径（FLAC→FLAC 不重编码），严格遵守任务书 §3 音质铁律。

## 5. 已验证样本状态

| 样本 | 来源 | 状态 |
|------|------|------|
| keystream 单元测试 | 对照 seed.hpp 手动推导 | 通过（前 10 字节与参考实现一致） |
| 真实 `.qmc*/.mgg` 端到端解密 | **待补充** | 需要用户提供合法拥有的 QMC 样本做实测 |

> 诚实声明：截至本文撰写，尚未用真实 QMC 文件做端到端验证。keystream 算法已通过对照参考实现的单元测试，解密逻辑与多个成熟开源解码器一致。待用户提供样本后补充真实验证记录并更新本表。

## 6. 研究来源与 License 合规

本模块为**独立 clean-room 重新实现**，未直接复制任何上游源码。算法描述来自公开研究：

- `Presburger/qmc-decoder`（C++，MIT License）—— 提供 keystream 状态机 `seed.hpp` 与全文件异或循环 `decoder.cpp` 的权威描述
- `QQMusicDecrypt`（Python，PyPI）—— 确认 QMC 解密算法源自 qmc-decoder
- 其他社区 qmc 解码项目（qmc2 / mgg 解析器）

实现中仅复用公开的固定查找表常量与文件结构知识，不复制任何专有代码。
