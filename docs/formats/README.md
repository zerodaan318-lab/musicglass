# 音频格式研究文档（formats/）

> 任务书第 79 节要求：每个支持格式单独建研究文档，记录结构、Metadata、Cover、音频、已知限制、支持操作、测试、License/研究来源。
> 本文档定义统一模板与索引；**详细内容在对应 Phase 实际研究后填充，禁止凭记忆编造。**

## 研究模板（每个格式一份：`formats/<fmt>.md`）

```markdown
# <格式名> 研究

## 1. 文件结构 / 容器
## 2. Metadata 标准（标签框、封面轨道、歌词轨道）
## 3. Cover Art（位置、格式、多封面）
## 4. 音频编码（Codec / 采样率 / 位深 / 声道）
## 5. 已知限制
## 6. MusicGlass 支持的操作（读/写/转换/验证）
## 7. 测试样本与预期结果
## 8. License / 研究来源（官方文档、RFC、开源实现链接）
```

## 索引

| 格式 | 研究文档 | 负责 Phase | 状态 |
|------|----------|-----------|------|
| MP3 | `formats/mp3.md` | Phase 2 | ⬜ 待研究 |
| FLAC | `formats/flac.md` | Phase 2 | ⬜ |
| WAV | `formats/wav.md` | Phase 2 | ⬜ |
| M4A (AAC/ALAC) | `formats/m4a.md` | Phase 2 | ⬜ |
| AAC | `formats/aac.md` | Phase 2 | ⬜ |
| OGG (Vorbis/Opus) | `formats/ogg.md` | Phase 2 | ⬜ |
| OPUS | `formats/opus.md` | Phase 2 | ⬜ |
| NCM | `formats/ncm.md` + `FORMAT_RESEARCH_NCM.md` | Phase 5 | ⬜ |
| QMC 系列 | `formats/qmc.md` | Phase 6 | ⬜ |

## 通用研究来源（优先查官方，不凭记忆）
- MP3/ID3：ID3.org、MP3 规范
- FLAC：xiph.org/flac 规范
- OGG/Vorbis/Opus：xiph.org
- M4A/MP4：ISO/IEC 14496-12、Apple 文档
- NCM/QMC：已验证的开源实现（须遵守其 License，记录于 `THIRD_PARTY_NOTICES.md`）
