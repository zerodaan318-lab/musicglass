/**
 * SettingsPage —— MusicGlass 设置页面（任务书 §33-§37，§45）
 *
 * 左侧分组标签栏（常规 / 外观 / 转换 / 输出 / 元数据 / 性能 / 高级 / 关于）切换右侧内容面板，
 * 当前标签使用 accent 高亮，面板切换通过 Framer Motion 平滑过渡。
 *
 * 组件为受控组件：所有改动均通过 props.onChange 向上传递新 Settings 对象，
 * 绝不直接 mutate 传入的 props。字符串统一使用中文，TS strict 模式无未用变量。
 */

import { useState, type ReactNode } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { GlassCard } from '@/components/GlassCard';
import { Button } from '@/components/Button';
import { openFolderDialog } from '@/tauri';
import { IconSettings } from '@/components/Icon';
import {
  DEFAULT_SETTINGS,
  type Settings,
  type AppearanceMode,
  type OverwritePolicy,
  type QualityPreset,
} from '@musicglass/shared';

/** 分组标签定义（§33 至少包含以下分组） */
type TabId =
  | 'general'
  | 'appearance'
  | 'conversion'
  | 'output'
  | 'metadata'
  | 'performance'
  | 'advanced'
  | 'about';

const TABS: { id: TabId; label: string; icon: string }[] = [
  { id: 'general', label: '常规', icon: '⚙' },
  { id: 'appearance', label: '外观', icon: '◐' },
  { id: 'conversion', label: '转换', icon: '⇄' },
  { id: 'output', label: '输出', icon: '📁' },
  { id: 'metadata', label: '元数据', icon: '🏷' },
  { id: 'performance', label: '性能', icon: '⚡' },
  { id: 'advanced', label: '高级', icon: '🔧' },
  { id: 'about', label: '关于', icon: 'ℹ' },
];

/* ===================== 通用受控子控件 ===================== */

interface SegmentedProps<T extends string> {
  name: string;
  value: T;
  options: { value: T; label: string }[];
  onChange: (v: T) => void;
}

/** 三选/多选分段控件（外观模式、内存策略等） */
function Segmented<T extends string>({ name, value, options, onChange }: SegmentedProps<T>) {
  return (
    <div className="inline-flex rounded-2xl glass p-1 gap-1">
      {options.map((opt) => {
        const active = opt.value === value;
        return (
          <button
            key={opt.value}
            type="button"
            onClick={() => onChange(opt.value)}
            className={`relative px-4 py-1.5 text-sm rounded-xl transition-colors duration-180 ${
              active ? 'text-white' : 'text-muted hover:text-text'
            }`}
          >
            {active && (
              <motion.span
                layoutId={`seg-${name}`}
                className="absolute inset-0 rounded-xl bg-accent/25 ring-1 ring-accent/40"
                transition={{ type: 'spring', stiffness: 400, damping: 30 }}
              />
            )}
            <span className="relative z-10">{opt.label}</span>
          </button>
        );
      })}
    </div>
  );
}

/** 开关（元数据四个开关、确认有损等） */
function Toggle({
  checked,
  onChange,
  label,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
  label?: string;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      onClick={() => onChange(!checked)}
      className={`relative h-6 w-11 shrink-0 rounded-full transition-all duration-180 ${
        checked ? 'bg-accent shadow-glass-btn' : 'bg-white/10 border border-white/15'
      }`}
    >
      <motion.span
        animate={{ x: checked ? 22 : 2 }}
        transition={{ type: 'spring', stiffness: 500, damping: 32 }}
        className="absolute top-0.5 left-0 h-5 w-5 rounded-full bg-white shadow"
      />
    </button>
  );
}

/** 单行设置项：标题 + 描述 + 右侧控件 */
function Field({
  title,
  desc,
  children,
}: {
  title: string;
  desc?: string;
  children: ReactNode;
}) {
  return (
    <GlassCard className="flex items-center justify-between gap-4 rounded-2xl">
      <div className="min-w-0">
        <div className="font-medium text-text">{title}</div>
        {desc && <div className="mt-0.5 text-sm text-muted">{desc}</div>}
      </div>
      <div className="shrink-0">{children}</div>
    </GlassCard>
  );
}

/** 文本框 */
function TextInput({
  value,
  placeholder,
  onChange,
}: {
  value: string;
  placeholder?: string;
  onChange: (v: string) => void;
}) {
  return (
    <input
      type="text"
      value={value}
      placeholder={placeholder}
      onChange={(e) => onChange(e.target.value)}
      className="w-64 rounded-2xl border border-white/12 bg-white/5 px-3 py-2 text-sm text-text placeholder:text-muted/60 outline-none focus:border-accent/60"
    />
  );
}

/* ===================== 面板内容 ===================== */

function AppearancePanel({ settings, onChange }: PanelProps) {
  return (
    <div className="space-y-4">
      <Field title="外观模式" desc="深色 / 浅色 / 跟随系统">
        <Segmented<AppearanceMode>
          name="appearance"
          value={settings.appearance}
          options={[
            { value: 'dark', label: '深色' },
            { value: 'light', label: '浅色' },
            { value: 'system', label: '跟随系统' },
          ]}
          onChange={(v) => onChange({ ...settings, appearance: v })}
        />
      </Field>
    </div>
  );
}

function ConversionPanel({ settings, onChange }: PanelProps) {
  const qualityOptions: { value: QualityPreset; label: string }[] = [
    { value: 'lossless', label: '无损' },
    { value: 'high', label: '高' },
    { value: 'standard', label: '标准' },
    { value: 'custom', label: '自定义' },
  ];
  return (
    <div className="space-y-4">
      <Field title="默认质量预设" desc="新建转换任务的默认输出质量">
        <select
          value={settings.conversion.defaultQuality}
          onChange={(e) =>
            onChange({
              ...settings,
              conversion: {
                ...settings.conversion,
                defaultQuality: e.target.value as QualityPreset,
              },
            })
          }
          className="rounded-2xl border border-white/12 bg-white/5 px-3 py-2 text-sm text-text outline-none focus:border-accent/60"
        >
          {qualityOptions.map((o) => (
            <option key={o.value} value={o.value}>
              {o.label}
            </option>
          ))}
        </select>
      </Field>
      <Field title="有损转换需确认" desc="转换为有损格式前弹出确认（原则一）">
        <Toggle
          label="有损转换需确认"
          checked={settings.conversion.confirmLossy}
          onChange={(v) =>
            onChange({
              ...settings,
              conversion: { ...settings.conversion, confirmLossy: v },
            })
          }
        />
      </Field>
    </div>
  );
}

function OutputPanel({ settings, onChange }: PanelProps) {
  const overwriteOptions: { value: OverwritePolicy; label: string }[] = [
    { value: 'ask', label: '询问' },
    { value: 'always', label: '总是覆盖' },
    { value: 'skip', label: '跳过' },
  ];
  return (
    <div className="space-y-4">
      <Field title="默认输出目录" desc="转换结果的保存位置（留空则使用源文件同目录）">
        <div className="flex items-center gap-2">
          <TextInput
            value={settings.output.defaultDirectory}
            placeholder="例如：D:/Music/Converted"
            onChange={(v) =>
              onChange({
                ...settings,
                output: { ...settings.output, defaultDirectory: v },
              })
            }
          />
          <Button
            variant="ghost"
            onClick={async () => {
              const dir = await openFolderDialog();
              if (dir && dir[0]) {
                onChange({
                  ...settings,
                  output: { ...settings.output, defaultDirectory: dir[0] },
                });
              }
            }}
          >
            浏览
          </Button>
        </div>
      </Field>
      <Field title="文件夹结构" desc="按变量组织子目录，如 {album} 或 {artist}/{album}">
        <TextInput
          value={settings.output.folderStructure}
          placeholder="{album}"
          onChange={(v) =>
            onChange({
              ...settings,
              output: { ...settings.output, folderStructure: v },
            })
          }
        />
      </Field>
      <Field title="文件名模板" desc="输出文件名格式，如 {track} {title}">
        <TextInput
          value={settings.output.filenameTemplate}
          placeholder="{track} {title}"
          onChange={(v) =>
            onChange({
              ...settings,
              output: { ...settings.output, filenameTemplate: v },
            })
          }
        />
      </Field>
      <Field title="覆盖策略" desc="目标文件已存在时的处理方式">
        <select
          value={settings.output.overwrite}
          onChange={(e) =>
            onChange({
              ...settings,
              output: {
                ...settings.output,
                overwrite: e.target.value as OverwritePolicy,
              },
            })
          }
          className="rounded-2xl border border-white/12 bg-white/5 px-3 py-2 text-sm text-text outline-none focus:border-accent/60"
        >
          {overwriteOptions.map((o) => (
            <option key={o.value} value={o.value}>
              {o.label}
            </option>
          ))}
        </select>
      </Field>
    </div>
  );
}

function MetadataPanel({ settings, onChange }: PanelProps) {
  const { metadata } = settings;
  const setMeta = (patch: Partial<Settings['metadata']>) =>
    onChange({ ...settings, metadata: { ...metadata, ...patch } });
  return (
    <div className="space-y-4">
      <Field title="嵌入封面" desc="将专辑封面写入输出文件">
        <Toggle
          label="嵌入封面"
          checked={metadata.embedCover}
          onChange={(v) => setMeta({ embedCover: v })}
        />
      </Field>
      <Field title="嵌入歌词" desc="将歌词写入输出文件">
        <Toggle
          label="嵌入歌词"
          checked={metadata.embedLyrics}
          onChange={(v) => setMeta({ embedLyrics: v })}
        />
      </Field>
      <Field title="保留扩展标签" desc="保留原始文件中的扩展标签（如 ReplayGain）">
        <Toggle
          label="保留扩展标签"
          checked={metadata.preserveExtendedTags}
          onChange={(v) => setMeta({ preserveExtendedTags: v })}
        />
      </Field>
      <Field title="校验元数据" desc="转换完成后校验元数据完整性">
        <Toggle
          label="校验元数据"
          checked={metadata.verifyMetadata}
          onChange={(v) => setMeta({ verifyMetadata: v })}
        />
      </Field>
    </div>
  );
}

function PerformancePanel({ settings, onChange }: PanelProps) {
  const { performance } = settings;
  const setPerf = (patch: Partial<Settings['performance']>) =>
    onChange({ ...settings, performance: { ...performance, ...patch } });
  const onNum = (key: 'concurrency' | 'cpuThreads', raw: string) => {
    const n = Number(raw);
    setPerf({ [key]: Number.isFinite(n) && n >= 0 ? Math.floor(n) : 0 });
  };
  return (
    <div className="space-y-4">
      <Field title="并发数" desc="同时进行的转换任务数（建议为 CPU 核心数的一半）">
        <input
          type="number"
          min={1}
          value={performance.concurrency}
          onChange={(e) => onNum('concurrency', e.target.value)}
          className="w-24 rounded-2xl border border-white/12 bg-white/5 px-3 py-2 text-sm text-text outline-none focus:border-accent/60"
        />
      </Field>
      <Field title="CPU 线程数" desc="每个任务使用的线程数（0 = 自动）">
        <input
          type="number"
          min={0}
          value={performance.cpuThreads}
          onChange={(e) => onNum('cpuThreads', e.target.value)}
          className="w-24 rounded-2xl border border-white/12 bg-white/5 px-3 py-2 text-sm text-text outline-none focus:border-accent/60"
        />
      </Field>
      <Field title="临时目录" desc="转换中间文件存放路径（留空则使用系统临时目录）">
        <TextInput
          value={performance.tempDirectory}
          placeholder="例如：D:/Temp/MusicGlass"
          onChange={(v) => setPerf({ tempDirectory: v })}
        />
      </Field>
      <Field title="内存策略" desc="在内存占用与速度之间取舍">
        <Segmented<'balanced' | 'low' | 'performance'>
          name="memory"
          value={performance.memoryStrategy}
          options={[
            { value: 'low', label: '节省' },
            { value: 'balanced', label: '均衡' },
            { value: 'performance', label: '性能' },
          ]}
          onChange={(v) => setPerf({ memoryStrategy: v })}
        />
      </Field>
    </div>
  );
}

function GeneralPanel() {
  // 常规：合理的占位内容（本地状态，不绑定到 Settings）
  const [autoCheck, setAutoCheck] = useState(true);
  const [confirmClose, setConfirmClose] = useState(false);
  return (
    <div className="space-y-4">
      <GlassCard>
        <div className="font-medium text-text">常规设置</div>
        <div className="mt-1 text-sm text-muted">
          此处用于应用级通用偏好（占位）。后续版本将接入语言、启动行为与通知等选项。
        </div>
      </GlassCard>
      <Field title="启动时检查更新" desc="每次启动自动检查新版本（模拟项）">
        <Toggle label="启动时检查更新" checked={autoCheck} onChange={setAutoCheck} />
      </Field>
      <Field title="关闭前确认" desc="退出应用时弹出确认（模拟项）">
        <Toggle label="关闭前确认" checked={confirmClose} onChange={setConfirmClose} />
      </Field>
    </div>
  );
}

function AdvancedPanel({ onChange }: { onChange: (next: Settings) => void }) {
  const [logLevel, setLogLevel] = useState<'error' | 'warn' | 'info' | 'debug'>('info');
  return (
    <div className="space-y-4">
      <Field title="日志级别" desc="控制台与日志文件输出详细程度（模拟项）">
        <select
          value={logLevel}
          onChange={(e) => setLogLevel(e.target.value as typeof logLevel)}
          className="rounded-2xl border border-white/12 bg-white/5 px-3 py-2 text-sm text-text outline-none focus:border-accent/60"
        >
          <option value="error">错误</option>
          <option value="warn">警告</option>
          <option value="info">信息</option>
          <option value="debug">调试</option>
        </select>
      </Field>
      <Field title="开发者模式" desc="显示内部调试信息与性能面板（模拟项）">
        <Toggle label="开发者模式" checked={false} onChange={() => {}} />
      </Field>
      <GlassCard className="flex items-center justify-between gap-4 rounded-2xl">
        <div className="min-w-0">
          <div className="font-medium text-text">重置所有设置</div>
          <div className="mt-0.5 text-sm text-muted">恢复为默认值（占位，立即生效）</div>
        </div>
        <Button
          variant="danger"
          onClick={() => onChange({ ...DEFAULT_SETTINGS })}
        >
          重置设置
        </Button>
      </GlassCard>
    </div>
  );
}

function AboutPanel() {
  const APP_VERSION = '0.1.0';
  return (
    <div className="space-y-4">
      <GlassCard strong>
        <div className="flex items-center gap-3">
          <div className="glass-icon grid h-12 w-12 place-items-center rounded-2xl bg-accent/20 text-accent text-2xl">
            ♪
          </div>
          <div>
            <div className="text-lg font-semibold text-text">MusicGlass</div>
            <div className="text-sm text-muted">本地音乐格式转换器 · v{APP_VERSION}</div>
          </div>
        </div>
        <div className="mt-4 space-y-1 text-sm text-muted">
          <div>· 纯本地运行，文件不上传云端，保护隐私。</div>
          <div>· 支持常见与专有加密格式（NCM / QMC / MGG / MFLAC）解析转换。</div>
          <div>· 技术栈：React 18 + TypeScript + Vite + Tailwind CSS 3 + Framer Motion 11；核心由 Rust 实现。</div>
        </div>
      </GlassCard>
    </div>
  );
}

/* ===================== 主组件 ===================== */

interface PanelProps {
  settings: Settings;
  onChange: (next: Settings) => void;
}

export function SettingsPage({
  settings,
  onChange,
}: {
  settings: Settings;
  onChange: (next: Settings) => void;
}) {
  const [active, setActive] = useState<TabId>('general');

  const renderPanel = () => {
    switch (active) {
      case 'general':
        return <GeneralPanel />;
      case 'appearance':
        return <AppearancePanel settings={settings} onChange={onChange} />;
      case 'conversion':
        return <ConversionPanel settings={settings} onChange={onChange} />;
      case 'output':
        return <OutputPanel settings={settings} onChange={onChange} />;
      case 'metadata':
        return <MetadataPanel settings={settings} onChange={onChange} />;
      case 'performance':
        return <PerformancePanel settings={settings} onChange={onChange} />;
      case 'advanced':
        return <AdvancedPanel onChange={onChange} />;
      case 'about':
        return <AboutPanel />;
      default:
        return null;
    }
  };

  return (
    <div className="app-bg h-full overflow-hidden">
      <div className="mx-auto flex h-full max-w-5xl flex-col px-6 py-6">
        {/* 标题 */}
        <div className="mb-5 flex items-center gap-3">
          <IconSettings className="text-accent" width={26} height={26} />
          <h1 className="text-2xl font-semibold text-text">设置</h1>
        </div>

        <div className="flex min-h-0 flex-1 gap-6">
          {/* 左侧分组标签栏 */}
          <nav className="flex w-40 shrink-0 flex-col gap-1">
            {TABS.map((tab) => {
              const isActive = tab.id === active;
              return (
                <button
                  key={tab.id}
                  type="button"
                  onClick={() => setActive(tab.id)}
                  className={`relative flex items-center gap-2 rounded-2xl px-3 py-2.5 text-left text-sm transition-colors duration-180 ${
                    isActive ? 'text-white' : 'text-muted hover:text-text hover:bg-white/5'
                  }`}
                >
                  {isActive && (
                    <motion.span
                      layoutId="tab-highlight"
                      className="absolute inset-0 rounded-2xl bg-accent/15 ring-1 ring-accent/30 backdrop-blur-md"
                      transition={{ type: 'spring', stiffness: 400, damping: 32 }}
                    />
                  )}
                  <span className="relative z-10">{tab.icon}</span>
                  <span className="relative z-10">{tab.label}</span>
                </button>
              );
            })}
          </nav>

          {/* 右侧内容面板（Framer Motion 平滑切换） */}
          <section className="min-w-0 flex-1 overflow-y-auto pr-2">
            <AnimatePresence mode="wait">
              <motion.div
                key={active}
                initial={{ opacity: 1, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -8 }}
                transition={{ duration: 0.22, ease: 'easeOut' }}
              >
                {renderPanel()}
              </motion.div>
            </AnimatePresence>
          </section>
        </div>
      </div>
    </div>
  );
}

export default SettingsPage;
