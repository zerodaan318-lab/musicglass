/**
 * 首页 — 拖拽导入 + 添加文件/文件夹（任务书 §28）
 * 视觉：玻璃卡片 + 适度背景模糊；毛玻璃不影响可读性/按钮识别/对比度（§28 约束）
 * 真实接入 Tauri 时，拖拽/选择拿到真实路径 → onAddPaths(paths) → store.detectFiles → 后端 detect_files
 * 纯前端预览下，mock paths 经 tauri.ts 降级为模拟检测
 */
import { useState, useCallback, type DragEvent } from 'react';
import { motion } from 'framer-motion';
import { GlassCard } from '@/components/GlassCard';
import { Button } from '@/components/Button';
import { IconPlus, IconFolder, IconMusic } from '@/components/Icon';

const SUPPORTED_LABELS = ['MP3', 'FLAC', 'WAV', 'M4A', 'NCM', 'QMC', '...'];

// 示例文件名（仅用于生成演示路径，检测在后端/降级层完成）
const SAMPLE_NAMES = [
  'Artist - Song A.flac', 'Artist - Song B.mp3', 'Album - Track 03.m4a',
  'Live Recording.wav', 'Podcast.ogg', 'Demo.opus',
  '网易云下载.ncm', 'QQ音乐缓存.qmc', '加密歌曲.mgg',
];
const samplePath = (name: string) => `C:\\Users\\User\\Music\\${name}`;

export function HomePage({ onAddPaths }: { onAddPaths: (paths: string[]) => void }) {
  const [dragOver, setDragOver] = useState(false);

  const handleDrop = useCallback(
    (e: DragEvent<HTMLDivElement>) => {
      e.preventDefault();
      setDragOver(false);
      const paths = Array.from(e.dataTransfer.files).map(
        (f) => (f as any).path || samplePath(f.name)
      ); // Tauri 下为真实路径；浏览器降级用 name 拼
      if (paths.length === 0) return;
      onAddPaths(paths);
    },
    [onAddPaths]
  );

  // 演示：生成示例路径（真实环境由文件对话框/拖拽提供）
  const simulateAdd = (count: number, withProprietary = false) => {
    const paths: string[] = [];
    for (let i = 0; i < count; i++) {
      const name = SAMPLE_NAMES[Math.floor(Math.random() * SAMPLE_NAMES.length)];
      paths.push(samplePath(name));
    }
    if (withProprietary) {
      paths.push(samplePath('示例.ncm'));
      paths.push(samplePath('缓存.qmc'));
    }
    onAddPaths(paths);
  };

  return (
    <div className="space-y-6">
      <div className="text-center">
        <motion.h2
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          className="text-3xl font-semibold tracking-tight text-text"
        >
          MusicGlass
        </motion.h2>
        <p className="mt-2 text-muted">把已有音乐文件拖进来，自动识别格式、解析并转换</p>
      </div>

      <motion.div
        onDragOver={(e) => {
          e.preventDefault();
          setDragOver(true);
        }}
        onDragLeave={() => setDragOver(false)}
        onDrop={handleDrop}
        animate={{ scale: dragOver ? 1.01 : 1 }}
        className={`relative grid place-items-center rounded-2xl border-2 border-dashed p-12 transition-colors ${
          dragOver ? 'border-accent bg-accent/10' : 'border-border/20 bg-surface/30'
        }`}
      >
        <div className="text-center">
          <div className="mx-auto mb-4 grid h-16 w-16 place-items-center rounded-2xl bg-accent/15 text-accent">
            <IconMusic width={32} height={32} />
          </div>
          <p className="text-lg font-medium text-text">Drop music files here</p>
          <p className="mt-1 text-sm text-muted">支持拖拽多个文件，或点击下方按钮</p>
          <div className="mt-6 flex items-center justify-center gap-3">
            <Button icon={<IconPlus />} onClick={() => simulateAdd(6, true)}>
              Add Files
            </Button>
            <Button variant="secondary" icon={<IconFolder />} onClick={() => simulateAdd(12, true)}>
              Add Folder
            </Button>
          </div>
        </div>
      </motion.div>

      <GlassCard className="text-center">
        <p className="mb-3 text-sm font-medium text-muted">Supported</p>
        <div className="flex flex-wrap justify-center gap-2">
          {SUPPORTED_LABELS.map((f) => (
            <span
              key={f}
              className="rounded-lg bg-surface-strong/60 px-3 py-1 text-xs font-medium text-text"
            >
              {f}
            </span>
          ))}
        </div>
        <p className="mt-4 text-xs text-muted">
          提示：专有格式（NCM / QMC / MGG 等）会被自动识别并通过插件解析
        </p>
      </GlassCard>
    </div>
  );
}
