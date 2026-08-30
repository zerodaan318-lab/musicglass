/**
 * ConvertPage —— 转换配置页面（任务书 §30）
 *
 * 职责：在用户从首页导入文件后，集中配置本次批量转换的全部参数，
 * 包括输入文件清单、输出格式、质量预设、元数据/封面/歌词开关、
 * 输出目录与文件命名模板，并最终通过 [Start Conversion] 触发转换。
 *
 * 关键约束（原则一）：当输出选择有损格式（mp3/aac/ogg/opus/m4a）
 * 且导入列表中包含无损音源时，必须内联明确警示并等待用户勾选确认，
 * 否则禁用「开始转换」按钮，避免不可逆的音频信息损失。
 */

import { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { GlassCard } from '@/components/GlassCard';
import { Button } from '@/components/Button';
import { IconFolder, IconPlay, IconCheck, IconAlert } from '@/components/Icon';
import {
  OUTPUT_FORMATS,
  isLossless,
  isProprietary,
  type DetectedFile,
  type ConversionConfig,
  type AudioFormat,
  type QualityPreset,
  type OverwritePolicy,
} from '@musicglass/shared';
import { formatBytes, formatDuration, formatLabel } from '@/lib/format';
import { openFolderDialog } from '@/tauri';

/** 有损输出格式（任务书 原则一） */
const LOSSY_OUTPUT_FORMATS: AudioFormat[] = ['mp3', 'aac', 'ogg', 'opus', 'm4a'];

const QUALITY_OPTIONS: { value: QualityPreset; label: string; hint: string }[] = [
  { value: 'lossless', label: '无损', hint: '保持原始采样，文件体积较大' },
  { value: 'high', label: '高品质', hint: '高压缩有损，听感接近无损' },
  { value: 'standard', label: '标准', hint: '在体积与音质间取得平衡' },
  { value: 'custom', label: '自定义', hint: '手动设定编码参数' },
];

const OVERWRITE_OPTIONS: { value: OverwritePolicy; label: string }[] = [
  { value: 'ask', label: '每次询问' },
  { value: 'always', label: '总是覆盖' },
  { value: 'skip', label: '跳过已存在' },
];

interface ConvertPageProps {
  files: DetectedFile[];
  onStart: (config: ConversionConfig) => void;
}

interface ToggleProps {
  label: string;
  description?: string;
  checked: boolean;
  onChange: (value: boolean) => void;
}

/** 通用开关（metadata / cover / lyrics 等） */
function Toggle({ label, description, checked, onChange }: ToggleProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      onClick={() => onChange(!checked)}
      className="flex w-full items-center justify-between gap-4 rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-left transition-colors hover:bg-white/10"
    >
      <span>
        <span className="block font-medium text-text">{label}</span>
        {description && <span className="mt-0.5 block text-xs text-muted">{description}</span>}
      </span>
      <span
        className={`relative h-6 w-11 shrink-0 rounded-full transition-all duration-180 ${
          checked ? 'bg-accent shadow-glass-btn' : 'bg-white/10 border border-white/15'
        }`}
      >
        <span
          className={`absolute top-0.5 h-5 w-5 rounded-full bg-white shadow transition-all ${
            checked ? 'left-[22px]' : 'left-0.5'
          }`}
        />
      </span>
    </button>
  );
}

const container = {
  hidden: {},
  show: { transition: { staggerChildren: 0.06 } },
};

const item = {
  hidden: { opacity: 0, y: 12 },
  show: { opacity: 1, y: 0, transition: { type: 'spring' as const, stiffness: 260, damping: 26 } },
};

export function ConvertPage({ files, onStart }: ConvertPageProps) {
  const [outputFormat, setOutputFormat] = useState<AudioFormat>('flac');
  const [quality, setQuality] = useState<QualityPreset>('lossless');
  const [outputFolder, setOutputFolder] = useState<string>('');
  const [filenameTemplate, setFilenameTemplate] = useState<string>('{track} {title}');
  const [overwrite, setOverwrite] = useState<OverwritePolicy>('ask');
  const [embedCover, setEmbedCover] = useState<boolean>(true);
  const [embedLyrics, setEmbedLyrics] = useState<boolean>(true);
  const [preserveExtendedTags, setPreserveExtendedTags] = useState<boolean>(true);
  const [verifyMetadata, setVerifyMetadata] = useState<boolean>(true);
  const [confirmLossy, setConfirmLossy] = useState<boolean>(false);

  // 原则一：无损音源 + 有损输出 => 必须确认
  const hasLosslessSource = files.some((f) => isLossless(f.format));
  const isLossyOutput = LOSSY_OUTPUT_FORMATS.includes(outputFormat);

  // 质量选项与输出格式联动：无损格式（FLAC/WAV）只能选「无损」，有损格式禁用「无损」
  const isLosslessOutput = !isLossyOutput;
  // 若当前质量与格式不匹配，自动纠正（如选 FLAC 却是有损质量 → 改无损）
  useEffect(() => {
    if (isLosslessOutput && quality !== 'lossless') setQuality('lossless');
    if (!isLosslessOutput && quality === 'lossless') setQuality('high');
  }, [isLosslessOutput, quality]);
  const showLossyWarning = hasLosslessSource && isLossyOutput;
  const canStart = !showLossyWarning || confirmLossy;

  const handleBrowse = async () => {
    try {
      const dirs = await openFolderDialog();
      if (dirs && dirs.length > 0) setOutputFolder(dirs[0]);
    } catch (e: any) {
      console.error('[handleBrowse] 打开文件夹对话框失败:', e);
      alert('无法打开文件夹选择框：' + (e?.message || e));
    }
  };

  const handleStart = () => {
    if (!canStart) return;
    const config: ConversionConfig = {
      outputFormat,
      quality,
      outputFolder,
      filenameTemplate,
      overwrite,
      embedCover,
      embedLyrics,
      preserveExtendedTags,
      verifyMetadata,
    };
    onStart(config);
  };

  return (
    <motion.div
      variants={container}
      initial="hidden"
      animate="show"
      className="mx-auto flex max-w-4xl flex-col gap-5 pb-10"
    >
      <motion.div variants={item} className="px-1">
        <h1 className="text-2xl font-semibold text-text">转换配置</h1>
        <p className="mt-1 text-sm text-muted">
          共 {files.length} 个文件待转换，请在开始前确认下列参数。
        </p>
      </motion.div>

      {/* Input Files */}
      <motion.div variants={item}>
        <GlassCard strong>
          <h2 className="mb-3 text-lg font-medium text-text">输入文件</h2>
          {files.length === 0 ? (
            <p className="text-sm text-muted">尚未导入任何文件。</p>
          ) : (
            <ul className="flex flex-col gap-2">
              {files.map((f) => (
                <li
                  key={f.id}
                  className="flex items-center justify-between gap-3 rounded-2xl border border-white/10 bg-white/5 px-4 py-2.5"
                >
                  <div className="min-w-0">
                    <div className="truncate font-medium text-text">{f.name}</div>
                    <div className="mt-0.5 text-xs text-muted">
                      {formatBytes(f.sizeBytes)}
                      {f.durationSec != null && ` · ${formatDuration(f.durationSec)}`}
                    </div>
                  </div>
                  <div className="flex shrink-0 items-center gap-2">
                    <span className="rounded-md bg-surface-strong px-2 py-1 text-xs font-semibold text-accent">
                      {formatLabel(f.format)}
                    </span>
                    {isProprietary(f.format) && (
                      <span className="rounded-md bg-warning/20 px-2 py-1 text-xs font-semibold text-warning">
                        需解析
                      </span>
                    )}
                  </div>
                </li>
              ))}
            </ul>
          )}
        </GlassCard>
      </motion.div>

      {/* Output Format */}
      <motion.div variants={item}>
        <GlassCard>
          <h2 className="mb-3 text-lg font-medium text-text">输出格式</h2>
          <div className="grid grid-cols-3 gap-2 sm:grid-cols-4 md:grid-cols-7">
            {OUTPUT_FORMATS.map((fmt) => {
              const active = fmt === outputFormat;
              return (
                <button
                  key={fmt}
                  type="button"
                  onClick={() => setOutputFormat(fmt)}
                  className={`flex items-center justify-center gap-1 rounded-2xl border px-3 py-2.5 text-sm font-medium transition-all duration-180 ${
                    active
                      ? 'border-accent/60 bg-accent/15 text-accent shadow-glass-btn'
                      : 'border-white/10 bg-white/5 text-muted hover:bg-white/10'
                  }`}
                >
                  {active && <IconCheck width={16} height={16} />}
                  {formatLabel(fmt)}
                </button>
              );
            })}
          </div>
        </GlassCard>
      </motion.div>

      {/* Quality */}
      <motion.div variants={item}>
        <GlassCard>
          <h2 className="mb-3 text-lg font-medium text-text">质量</h2>
          <div className="grid grid-cols-2 gap-2 md:grid-cols-4">
            {QUALITY_OPTIONS.map((opt) => {
              const active = opt.value === quality;
              // 无损格式禁用有损质量；有损格式禁用「无损」
              const disabled =
                (isLosslessOutput && opt.value !== 'lossless') ||
                (!isLosslessOutput && opt.value === 'lossless');
              return (
                <button
                  key={opt.value}
                  type="button"
                  disabled={disabled}
                  onClick={() => setQuality(opt.value)}
                  className={`rounded-2xl border px-3 py-3 text-left transition-all duration-180 ${
                    disabled
                      ? 'cursor-not-allowed border-white/10 bg-white/5 opacity-40'
                      : active
                      ? 'border-accent/60 bg-accent/15'
                      : 'border-white/10 bg-white/5 hover:bg-white/10'
                  }`}
                >
                  <div className={`text-sm font-semibold ${active ? 'text-accent' : 'text-text'}`}>
                    {opt.label}
                  </div>
                  <div className="mt-1 text-xs text-muted">{opt.hint}</div>
                </button>
              );
            })}
          </div>
        </GlassCard>
      </motion.div>

      {/* 原则一：有损警告 */}
      {showLossyWarning && (
        <motion.div variants={item}>
          <GlassCard className="border-warning/40">
            <div className="flex items-start gap-3">
              <IconAlert className="mt-0.5 shrink-0 text-warning" />
              <div>
                <p className="font-medium text-warning">有损转换警示</p>
                <p className="mt-1 text-sm text-muted">
                  该操作会将无损音频编码为有损格式，可能产生不可逆的音频信息损失。
                </p>
                <label className="mt-3 flex cursor-pointer items-center gap-2 text-sm text-text">
                  <input
                    type="checkbox"
                    checked={confirmLossy}
                    onChange={(e) => setConfirmLossy(e.target.checked)}
                    className="h-4 w-4 accent-accent"
                  />
                  我已了解风险，确认继续转换
                </label>
              </div>
            </div>
          </GlassCard>
        </motion.div>
      )}

      {/* Metadata */}
      <motion.div variants={item}>
        <GlassCard>
          <h2 className="mb-3 text-lg font-medium text-text">元数据</h2>
          <div className="flex flex-col gap-2">
            <Toggle
              label="保留扩展标签"
              description="写入 TXXX / 自定义字段等扩展元数据"
              checked={preserveExtendedTags}
              onChange={setPreserveExtendedTags}
            />
            <Toggle
              label="校验元数据"
              description="转换完成后核对标签完整性"
              checked={verifyMetadata}
              onChange={setVerifyMetadata}
            />
          </div>
        </GlassCard>
      </motion.div>

      {/* Cover */}
      <motion.div variants={item}>
        <GlassCard>
          <h2 className="mb-3 text-lg font-medium text-text">封面</h2>
          <Toggle
            label="嵌入封面"
            description="将专辑封面写入输出音频文件"
            checked={embedCover}
            onChange={setEmbedCover}
          />
        </GlassCard>
      </motion.div>

      {/* Lyrics */}
      <motion.div variants={item}>
        <GlassCard>
          <h2 className="mb-3 text-lg font-medium text-text">歌词</h2>
          <Toggle
            label="嵌入歌词"
            description="将内嵌歌词（LRC 格式 / 原文）写入输出音频文件"
            checked={embedLyrics}
            onChange={setEmbedLyrics}
          />
        </GlassCard>
      </motion.div>

      {/* Output Folder */}
      <motion.div variants={item}>
        <GlassCard>
          <h2 className="mb-3 text-lg font-medium text-text">输出目录</h2>
          <div className="flex gap-2">
            <input
              type="text"
              value={outputFolder}
              onChange={(e) => setOutputFolder(e.target.value)}
              placeholder="选择或输入输出目录"
              className="min-w-0 flex-1 rounded-2xl border border-white/12 bg-white/5 px-4 py-2.5 text-sm text-text outline-none placeholder:text-muted focus:border-accent/60"
            />
            <Button variant="secondary" icon={<IconFolder />} onClick={handleBrowse}>
              浏览
            </Button>
          </div>
          <div className="mt-4">
            <span className="text-sm text-muted">同名文件策略</span>
            <div className="mt-2 flex flex-wrap gap-2">
              {OVERWRITE_OPTIONS.map((opt) => {
                const active = opt.value === overwrite;
                return (
                  <button
                    key={opt.value}
                    type="button"
                    onClick={() => setOverwrite(opt.value)}
                    className={`rounded-xl border px-3 py-1.5 text-sm transition-all duration-180 ${
                      active
                        ? 'border-accent/60 bg-accent/15 text-accent'
                        : 'border-white/10 bg-white/5 text-muted hover:bg-white/10'
                    }`}
                  >
                    {opt.label}
                  </button>
                );
              })}
            </div>
          </div>
        </GlassCard>
      </motion.div>

      {/* Filename Template */}
      <motion.div variants={item}>
        <GlassCard>
          <h2 className="mb-3 text-lg font-medium text-text">文件名模板</h2>
          <input
            type="text"
            value={filenameTemplate}
            onChange={(e) => setFilenameTemplate(e.target.value)}
            placeholder="{track} {title}"
            className="w-full rounded-2xl border border-white/12 bg-white/5 px-4 py-2.5 text-sm text-text outline-none placeholder:text-muted focus:border-accent/60"
          />
          <p className="mt-2 text-xs text-muted">
            可用变量：{'{track}'} {'{title}'} {'{artist}'} {'{album}'} {'{year}'}
          </p>
        </GlassCard>
      </motion.div>

      {/* Start Conversion */}
      <motion.div variants={item} className="flex justify-end pt-2">
        <Button
          variant="primary"
          icon={<IconPlay />}
          onClick={handleStart}
          disabled={!canStart}
        >
          Start Conversion
        </Button>
      </motion.div>
    </motion.div>
  );
}
