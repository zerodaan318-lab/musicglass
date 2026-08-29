/**
 * HomePage —— 首页拖放区（任务书 §28）
 *
 * 职责：
 *  1. 呈现品牌标题 MusicGlass + 玻璃卡片拖放区（Glassmorphism / Minimal / Smooth）；
 *  2. 提供 Add Files / Add Folder 两个显式入口，保证按钮识别度与操作效率（§28 约束）；
 *  3. 列出支持格式，让用户导入前就知道能处理什么；
 *  4. 把"导入"抽象为 onFilesAdded 回调 —— 当前用前端模拟数据，接入 Tauri 后
 *     只需把 mockFilesImport / mockFolderImport 换成 invoke(IPC.detectFiles)。
 *
 * 本组件不做统计计算：统计由 store 汇总后交给 ImportSummary 展示。
 */
import { useCallback, useRef, useState } from 'react';
import type { DragEvent } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import type { AudioFormat, DetectedFile } from '@musicglass/shared';
import { INPUT_FORMATS, isProprietary } from '@musicglass/shared';
import { GlassCard } from '@/components/GlassCard';
import { Button } from '@/components/Button';
import { IconFolder, IconMusic, IconPlus } from '@/components/Icon';
import { formatLabel, uid } from '@/lib/format';

interface HomePageProps {
  /** 导入完成（拖放 / 选择文件 / 选择文件夹）后向上抛出检测结果 */
  onFilesAdded: (files: DetectedFile[]) => void;
}

/** 首页展示的支持格式（顺序对齐任务书 §28 文案，末尾省略号表示还有更多） */
const SUPPORTED_CHIPS: AudioFormat[] = [
  'mp3', 'flac', 'wav', 'm4a', 'aac', 'ogg', 'opus', 'ncm', 'qmc', 'mgg', 'mflac',
];

/** 演示用文件名清单：18 个可直接解码 + 5 个 NCM + 2 个 QMC，正好对齐 §29 的示例 */
const MOCK_NAMES = [
  '01 Nightfall.mp3',
  '02 City Lights.mp3',
  '03 Paper Plane.mp3',
  '04 Glass Ocean.flac',
  '05 Aurora.flac',
  '06 Winter Garden.flac',
  '07 Silent Harbor.wav',
  '08 Analog Dream.wav',
  '09 Midnight Drive.m4a',
  '10 Neon Rain.m4a',
  '11 Slow Motion.aac',
  '12 Blue Hour.ogg',
  '13 晴天.mp3',
  '14 稻香.mp3',
  '15 夜曲.flac',
  '16 海阔天空.flac',
  '17 光年之外.m4a',
  '18 起风了.opus',
  '19 告白气球.ncm',
  '20 说好不哭.ncm',
  '21 千里之外.ncm',
  '22 花海.ncm',
  '23 兰亭序.ncm',
  '24 青花瓷.qmc',
  '25 东风破.qmc',
];

/** 演示数据的总时长目标：01:43:28（任务书 §29） */
const MOCK_TOTAL_SEC = 6208;

/** 由扩展名推断格式；真实检测（magic bytes + 置信度）由 Rust core 负责 */
function detectFormat(name: string): AudioFormat {
  const ext = name.split('.').pop()?.toLowerCase() ?? '';
  return (INPUT_FORMATS as string[]).includes(ext) ? (ext as AudioFormat) : 'unknown';
}

/** 构造单个 DetectedFile（前端占位数据） */
function makeFile(name: string, index: number, dir: string): DetectedFile {
  const format = detectFormat(name);
  const proprietary = isProprietary(format);
  const seconds = 200 + ((index * 37) % 120);
  const bytesPerSec = format === 'flac' || format === 'wav' ? 900_000 : 40_000;
  return {
    id: uid(),
    name,
    path: `${dir}/${name}`,
    sizeBytes: Math.round(seconds * bytesPerSec),
    format,
    confidence: format === 'unknown' ? 0.35 : proprietary ? 0.92 : 0.99,
    isProprietary: proprietary,
    // 专有容器需插件解密后才能转换，导入阶段先标记为不可直接处理
    supported: format !== 'unknown' && !proprietary,
    durationSec: seconds,
  };
}

/** 把整批时长对齐到目标值，让概览页与任务书示例一致（仅演示数据使用） */
function alignDuration(files: DetectedFile[], targetSec: number): DetectedFile[] {
  if (files.length === 0) return files;
  const current = files.reduce((sum, f) => sum + (f.durationSec ?? 0), 0);
  const delta = targetSec - current;
  if (delta === 0) return files;
  const step = Math.trunc(delta / files.length);
  const rest = delta - step * files.length;
  return files.map((f, i) => ({
    ...f,
    durationSec: Math.max(30, (f.durationSec ?? 0) + step + (i === files.length - 1 ? rest : 0)),
  }));
}

/** 模拟"选择文件夹"：整库 25 个文件 */
function mockFolderImport(): DetectedFile[] {
  const files = MOCK_NAMES.map((name, i) => makeFile(name, i, 'D:/Music/Library'));
  return alignDuration(files, MOCK_TOTAL_SEC);
}

/** 模拟"选择文件"：混合挑选 8 个（含专有格式，便于验证不支持提示） */
function mockFilesImport(): DetectedFile[] {
  return [0, 3, 6, 9, 13, 17, 18, 23].map((idx, i) => makeFile(MOCK_NAMES[idx], i, 'D:/Music'));
}

/** 拖入真实文件时用浏览器 File 信息构造结果（时长需解码后才知道，留空） */
function fromDroppedFile(file: File): DetectedFile {
  const format = detectFormat(file.name);
  const proprietary = isProprietary(format);
  return {
    id: uid(),
    name: file.name,
    path: file.name,
    sizeBytes: file.size,
    format,
    confidence: format === 'unknown' ? 0.35 : proprietary ? 0.92 : 0.99,
    isProprietary: proprietary,
    supported: format !== 'unknown' && !proprietary,
    durationSec: undefined,
  };
}

export function HomePage({ onFilesAdded }: HomePageProps) {
  const [dragging, setDragging] = useState(false);
  // dragenter/dragleave 会在子元素之间冒泡，用深度计数避免高亮闪烁
  const dragDepth = useRef(0);

  const handleDragOver = useCallback((e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
  }, []);

  const handleDragEnter = useCallback((e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    dragDepth.current += 1;
    setDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    dragDepth.current = Math.max(0, dragDepth.current - 1);
    if (dragDepth.current === 0) setDragging(false);
  }, []);

  const handleDrop = useCallback(
    (e: DragEvent<HTMLDivElement>) => {
      e.preventDefault();
      dragDepth.current = 0;
      setDragging(false);
      const dropped = Array.from(e.dataTransfer?.files ?? []);
      // 浏览器环境能拿到真实文件；Tauri 之外拿不到路径时退回演示数据
      onFilesAdded(dropped.length > 0 ? dropped.map(fromDroppedFile) : mockFolderImport());
    },
    [onFilesAdded],
  );

  return (
    <motion.div
      className="mx-auto w-full max-w-3xl"
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.45, ease: 'easeOut' }}
    >
      {/* 品牌区 */}
      <motion.header
        className="mb-8 flex flex-col items-center text-center"
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.05, duration: 0.4, ease: 'easeOut' }}
      >
        <div className="glass-strong mb-4 flex h-12 w-12 items-center justify-center rounded-xl2 text-accent">
          <IconMusic width={24} height={24} />
        </div>
        <h1 className="text-3xl font-semibold tracking-tight text-text sm:text-4xl">MusicGlass</h1>
        <p className="mt-2 text-sm text-muted">本地音乐格式转换器 · 无损优先 · 元数据完整保留</p>
      </motion.header>

      {/* 拖放区：玻璃卡片 + 柔和阴影，拖拽时以强调色描边提示 */}
      <div
        onDragOver={handleDragOver}
        onDragEnter={handleDragEnter}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
      >
        <GlassCard
          strong
          className={`relative overflow-hidden transition-shadow duration-300 ${
            dragging ? 'shadow-glass-hover ring-2 ring-accent/70' : ''
          }`}
        >
          {/* 拖拽柔光层：只改背景，不降低文字对比度（§28 约束） */}
          <AnimatePresence>
            {dragging && (
              <motion.div
                className="pointer-events-none absolute inset-0 bg-accent/10"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                transition={{ duration: 0.2 }}
              />
            )}
          </AnimatePresence>

          <div className="relative flex flex-col items-center justify-center rounded-xl2 border border-dashed border-border/25 px-6 py-14">
            <motion.div
              className="mb-5 flex h-16 w-16 items-center justify-center rounded-full bg-accent/15 text-accent"
              animate={dragging ? { scale: 1.08 } : { scale: 1 }}
              transition={{ type: 'spring', stiffness: 260, damping: 18 }}
            >
              <IconPlus width={28} height={28} />
            </motion.div>

            <p className="text-lg font-medium text-text">Drop music files here</p>
            <p className="mt-1.5 text-xs text-muted">支持整个文件夹拖入，自动识别格式与置信度</p>

            <div className="mt-7 flex flex-wrap items-center justify-center gap-3">
              <Button
                variant="primary"
                icon={<IconPlus width={18} height={18} />}
                onClick={() => onFilesAdded(mockFilesImport())}
              >
                Add Files
              </Button>
              <Button
                variant="secondary"
                icon={<IconFolder width={18} height={18} />}
                onClick={() => onFilesAdded(mockFolderImport())}
              >
                Add Folder
              </Button>
            </div>
          </div>
        </GlassCard>
      </div>

      {/* 支持格式 */}
      <motion.section
        className="mt-8 flex flex-col items-center"
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.16, duration: 0.4, ease: 'easeOut' }}
      >
        <p className="mb-3 text-xs font-medium uppercase tracking-[0.18em] text-muted">Supported:</p>
        <ul className="flex flex-wrap items-center justify-center gap-2">
          {SUPPORTED_CHIPS.map((f) => (
            <li
              key={f}
              className={`glass rounded-xl px-3 py-1.5 text-xs font-medium tracking-wide ${
                isProprietary(f) ? 'text-accent' : 'text-text'
              }`}
              title={isProprietary(f) ? '专有加密格式，由插件解析' : '标准音频格式'}
            >
              {formatLabel(f)}
            </li>
          ))}
          <li className="px-2 py-1.5 text-xs text-muted">...</li>
        </ul>
      </motion.section>
    </motion.div>
  );
}

export default HomePage;
