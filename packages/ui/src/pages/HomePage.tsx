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
import { openFilesDialog, openFolderDialog, isTauri } from '@/tauri';

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
  // Tauri 环境的原生拖放由 App 层的 onDragDropEvent 统一处理，这里禁用 HTML 拖放避免重复添加
  const htmlDndEnabled = !isTauri();

  const handleDrop = useCallback(
    (e: DragEvent<HTMLDivElement>) => {
      e.preventDefault();
      setDragOver(false);
      const paths = Array.from(e.dataTransfer.files).map(
        (f) => (f as any).path || samplePath(f.name)
      ); // 浏览器降级用 name 拼
      if (paths.length === 0) return;
      onAddPaths(paths);
    },
    [onAddPaths]
  );

  // Tauri 模式下打开真实文件对话框；纯前端预览下降级为模拟样本
  const handleAddFiles = async () => {
    const paths = await openFilesDialog(true);
    if (paths && paths.length > 0) {
      onAddPaths(paths);
      return;
    }
    if (paths === null) simulateAdd(6, true); // 非 Tauri 环境降级
  };

  const handleAddFolder = async () => {
    const dirs = await openFolderDialog();
    if (dirs && dirs.length > 0) {
      // 真实环境：把目录路径交给检测逻辑（detector 会扫描目录内文件）
      onAddPaths(dirs);
      return;
    }
    if (dirs === null) simulateAdd(12, true);
  };

  // 仅在非 Tauri（纯前端预览）下降级使用，生成示例路径供 mock 检测
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
      {/* Hero：标题与副标题自然悬浮在背景中，不做卡片 */}
      <div className="pt-4 text-center">
        <motion.h2
          initial={{ opacity: 1, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          className="text-4xl font-semibold tracking-tight text-text"
        >
          MusicGlass
        </motion.h2>
        <p className="mt-2 text-muted">把已有音乐文件拖进来，自动识别格式、解析并转换</p>
      </div>

      {/* Drop Zone：全页面视觉中心，大型 Liquid Glass 区域 */}
      <motion.div
        onDragOver={htmlDndEnabled ? (e) => {
          e.preventDefault();
          setDragOver(true);
        } : undefined}
        onDragLeave={htmlDndEnabled ? () => setDragOver(false) : undefined}
        onDrop={htmlDndEnabled ? handleDrop : undefined}
        animate={{ scale: dragOver ? 1.012 : 1 }}
        transition={{ type: 'spring', stiffness: 260, damping: 24 }}
        className={`relative grid min-h-[360px] place-items-center rounded-3xl p-12 transition-all duration-220 ${
          dragOver
            ? 'border-accent/60 bg-accent/10 shadow-glass-active'
            : 'glass border-white/15'
        }`}
      >
        {/* 环境光：中心微辉，让玻璃有厚度 */}
        <div className="pointer-events-none absolute inset-0 rounded-3xl bg-[radial-gradient(circle_at_50%_38%,rgba(130,145,232,0.16),transparent_60%)]" />
        <div className="relative z-10 text-center">
          <motion.div
            animate={{ scale: dragOver ? 1.06 : 1 }}
            transition={{ type: 'spring', stiffness: 260, damping: 22 }}
            className="glass-icon mx-auto mb-5 grid h-20 w-20 place-items-center rounded-3xl text-accent"
          >
            <IconMusic width={38} height={38} />
          </motion.div>
          <p className="text-lg font-medium text-text">Drop music files here</p>
          <p className="mt-1 text-sm text-muted">支持拖拽多个文件，或点击下方按钮</p>
          <div className="mt-6 flex items-center justify-center gap-3">
            <Button icon={<IconPlus />} onClick={handleAddFiles}>
              Add Files
            </Button>
            <Button variant="secondary" icon={<IconFolder />} onClick={handleAddFolder}>
              Add Folder
            </Button>
          </div>
        </div>
      </motion.div>

      {/* 支持格式：轻量玻璃面板 + 玻璃药丸 */}
      <GlassCard className="text-center">
        <p className="mb-3 text-sm font-medium text-muted">支持的格式</p>
        <div className="flex flex-wrap justify-center gap-2">
          {SUPPORTED_LABELS.map((f) => (
            <span
              key={f}
              className="glass-pill rounded-full px-3.5 py-1.5 text-xs font-medium text-text"
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
