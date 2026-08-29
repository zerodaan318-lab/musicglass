/**
 * ImportSummary —— 导入后概览（任务书 §29）
 *
 * 职责：
 *  1. 用玻璃卡片展示导入统计：总文件数 / 可直接处理数 / 各专有格式计数（NCM、QMC…）/ 总时长；
 *  2. 给出可读的格式分布与文件预览列表，让用户在进入转换页前确认导入是否正确；
 *  3. 不支持的文件用 warning 色提示，但不阻断流程（错误信息人类可读，§32 精神）。
 *
 * 只做展示：统计数据由 store 计算并通过 props 传入，本组件不修改任何状态。
 */
import { useMemo } from 'react';
import { motion } from 'framer-motion';
import type { DetectedFile, ImportSummary as ImportSummaryData } from '@musicglass/shared';
import { GlassCard } from '@/components/GlassCard';
import { IconAlert, IconCheck, IconMusic } from '@/components/Icon';
import { formatBytes, formatDuration, formatLabel } from '@/lib/format';

interface ImportSummaryProps {
  /** store 汇总出的统计结果 */
  summary: ImportSummaryData;
  /** 已导入的文件明细，用于专有格式归类与预览列表 */
  files: DetectedFile[];
}

/** 预览列表最多显示的条数，避免长列表拖慢首页 */
const PREVIEW_LIMIT = 6;

/** 把 qmc0/qmc2/mgg1/mflac0 等变体归到同一个家族标签（NCM / QMC / MGG / MFLAC） */
function proprietaryFamily(format: string): string | null {
  if (format.startsWith('ncm')) return 'NCM';
  if (format.startsWith('qmc')) return 'QMC';
  if (format.startsWith('mgg')) return 'MGG';
  if (format.startsWith('mflac')) return 'MFLAC';
  return null;
}

/** 总时长按 §29 示例补零显示为 01:43:28（复用 formatDuration，不重复实现时间逻辑） */
function formatTotalDuration(sec: number): string {
  const text = formatDuration(sec);
  const parts = text.split(':');
  return parts.length === 3 ? `${parts[0].padStart(2, '0')}:${parts[1]}:${parts[2]}` : text;
}

/** 统计数字块：数值大、标签小，保证毛玻璃背景下仍有足够对比度（§28） */
function Stat({
  value,
  label,
  tone = 'text',
  delay = 0,
}: {
  value: string;
  label: string;
  tone?: 'text' | 'accent' | 'warning';
  delay?: number;
}) {
  const toneClass =
    tone === 'accent' ? 'text-accent' : tone === 'warning' ? 'text-warning' : 'text-text';
  return (
    <motion.div
      className="rounded-xl2 bg-surface-strong/40 px-4 py-4 text-center"
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay, duration: 0.3, ease: 'easeOut' }}
    >
      <div className={`text-2xl font-semibold tabular-nums ${toneClass}`}>{value}</div>
      <div className="mt-1 text-xs tracking-wide text-muted">{label}</div>
    </motion.div>
  );
}

export function ImportSummary({ summary, files }: ImportSummaryProps) {
  const { proprietaryStats, standardStats, totalBytes, preview, restCount } = useMemo(() => {
    const proprietary = new Map<string, number>();
    const standard = new Map<string, number>();

    for (const [format, count] of Object.entries(summary.byFormat)) {
      const family = proprietaryFamily(format);
      if (family) {
        proprietary.set(family, (proprietary.get(family) ?? 0) + count);
      } else if (format !== 'unknown') {
        standard.set(formatLabel(format), (standard.get(formatLabel(format)) ?? 0) + count);
      }
    }

    const sortDesc = (entries: [string, number][]) => entries.sort((a, b) => b[1] - a[1]);

    return {
      proprietaryStats: sortDesc([...proprietary.entries()]),
      standardStats: sortDesc([...standard.entries()]),
      totalBytes: files.reduce((sum, f) => sum + f.sizeBytes, 0),
      preview: files.slice(0, PREVIEW_LIMIT),
      restCount: Math.max(0, files.length - PREVIEW_LIMIT),
    };
  }, [summary.byFormat, files]);

  return (
    <motion.div
      className="mx-auto w-full max-w-3xl"
      initial={{ opacity: 0, y: 12 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.35, ease: 'easeOut' }}
    >
      <GlassCard className="space-y-5">
        {/* 标题行 */}
        <div className="flex items-center justify-between gap-3">
          <div className="flex items-center gap-2 text-sm font-medium text-text">
            <IconMusic width={18} height={18} className="text-accent" />
            导入概览
          </div>
          <span className="text-xs text-muted">{formatBytes(totalBytes)}</span>
        </div>

        {/* 核心统计（§29：Files / Supported / NCM / QMC） */}
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
          <Stat value={`${summary.total}`} label="Files" delay={0.02} />
          <Stat value={`${summary.supported}`} label="Supported" tone="accent" delay={0.06} />
          {proprietaryStats.slice(0, 2).map(([family, count], i) => (
            <Stat
              key={family}
              value={`${count}`}
              label={family}
              tone="warning"
              delay={0.1 + i * 0.04}
            />
          ))}
          {/* 没有专有格式时补上 0 值占位，保持四宫格视觉稳定 */}
          {proprietaryStats.length === 0 && (
            <>
              <Stat value="0" label="NCM" delay={0.1} />
              <Stat value="0" label="QMC" delay={0.14} />
            </>
          )}
        </div>

        {/* 其余专有格式（MGG / MFLAC 等）与标准格式分布 */}
        {(proprietaryStats.length > 2 || standardStats.length > 0) && (
          <div className="flex flex-wrap gap-2">
            {proprietaryStats.slice(2).map(([family, count]) => (
              <span
                key={family}
                className="rounded-xl bg-warning/10 px-2.5 py-1 text-xs font-medium text-warning"
              >
                {family} × {count}
              </span>
            ))}
            {standardStats.map(([label, count]) => (
              <span
                key={label}
                className="rounded-xl bg-surface-strong/50 px-2.5 py-1 text-xs font-medium text-muted"
              >
                {label} × {count}
              </span>
            ))}
          </div>
        )}

        {/* 总时长（§29：Total Duration / 01:43:28） */}
        <motion.div
          className="flex items-center justify-between rounded-xl2 bg-surface-strong/40 px-4 py-3.5"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.18, duration: 0.3 }}
        >
          <span className="text-xs uppercase tracking-[0.18em] text-muted">Total Duration</span>
          <span className="text-xl font-semibold tabular-nums text-text">
            {formatTotalDuration(summary.totalDurationSec)}
          </span>
        </motion.div>

        {/* 文件预览：格式徽标 + 名称 + 大小 + 时长 */}
        <ul className="divide-y divide-border/10">
          {preview.map((f) => (
            <li key={f.id} className="flex items-center gap-3 py-2.5">
              <span
                className={`w-16 shrink-0 rounded-lg px-2 py-1 text-center text-[11px] font-semibold tracking-wide ${
                  f.supported ? 'bg-accent/15 text-accent' : 'bg-warning/10 text-warning'
                }`}
              >
                {formatLabel(f.format)}
              </span>
              <span className="min-w-0 flex-1 truncate text-sm text-text" title={f.path}>
                {f.name}
              </span>
              <span className="shrink-0 text-xs tabular-nums text-muted">
                {formatBytes(f.sizeBytes)}
              </span>
              <span className="w-14 shrink-0 text-right text-xs tabular-nums text-muted">
                {formatDuration(f.durationSec)}
              </span>
            </li>
          ))}
        </ul>
        {restCount > 0 && (
          <p className="text-xs text-muted">…… 还有 {restCount} 个文件未展示</p>
        )}

        {/* 结论提示：可读的人类语言，不暴露技术细节（§32） */}
        {summary.unsupported > 0 ? (
          <p className="flex items-start gap-2 text-xs text-warning">
            <IconAlert width={14} height={14} className="mt-0.5 shrink-0" />
            <span>
              {summary.unsupported} 个文件是加密/专有容器或无法识别，需要解密插件处理后才能转换；
              其余 {summary.supported} 个文件可以直接开始转换。
            </span>
          </p>
        ) : (
          <p className="flex items-center gap-2 text-xs text-success">
            <IconCheck width={14} height={14} className="shrink-0" />
            全部文件均可直接转换，去「转换」页选择输出格式即可。
          </p>
        )}
      </GlassCard>
    </motion.div>
  );
}

export default ImportSummary;
