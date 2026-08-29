// MusicGlass 前后端共享类型定义
// 与 Rust core crate 中的 serde 结构对齐（字段语义一致，命名采用 TS 习惯）

/** 所有支持的音频格式（输入 + 专有 + 输出） */
export type AudioFormat =
  | 'mp3' | 'flac' | 'wav' | 'm4a' | 'aac' | 'ogg' | 'opus'
  | 'ape' | 'wma'
  | 'ncm' | 'qmc' | 'qmc0' | 'qmc2' | 'qmc3'
  | 'mgg' | 'mgg0' | 'mgg1'
  | 'mflac' | 'mflac0'
  | 'unknown';

/** 阶段一支持的输入格式（任务书 §4） */
export const INPUT_FORMATS: AudioFormat[] = [
  'mp3', 'flac', 'wav', 'm4a', 'aac', 'ogg', 'opus', 'ape', 'wma',
  'ncm', 'qmc', 'qmc0', 'qmc2', 'qmc3', 'mgg', 'mgg0', 'mgg1', 'mflac', 'mflac0',
];

/** 阶段一支持的输出格式（任务书 §5） */
export const OUTPUT_FORMATS: AudioFormat[] = [
  'mp3', 'flac', 'wav', 'm4a', 'aac', 'ogg', 'opus',
];

/** 专有/加密容器格式（需要插件解析） */
export const PROPRIETARY_FORMATS: AudioFormat[] = [
  'ncm', 'qmc', 'qmc0', 'qmc2', 'qmc3', 'mgg', 'mgg0', 'mgg1', 'mflac', 'mflac0',
];

export function isProprietary(f: AudioFormat): boolean {
  return PROPRIETARY_FORMATS.includes(f);
}

export function isLossless(f: AudioFormat): boolean {
  return ['flac', 'wav', 'mflac', 'mflac0', 'ape', 'alac'].includes(f);
}

/** 拖入/导入后单个文件检测结果 */
export interface DetectedFile {
  id: string;
  name: string;
  path: string;
  sizeBytes: number;
  format: AudioFormat;
  confidence: number; // 0..1
  isProprietary: boolean;
  supported: boolean;
  durationSec?: number;
}

/** 导入概览统计（任务书 §29） */
export interface ImportSummary {
  total: number;
  supported: number;
  unsupported: number;
  byFormat: Record<string, number>;
  totalDurationSec: number;
}

/** Unified Metadata（任务书 §20，与 core::Metadata 对齐） */
export interface Metadata {
  title?: string;
  artist?: string;
  album?: string;
  albumArtist?: string;
  composer?: string;
  lyricist?: string;
  arranger?: string;
  genre?: string;
  year?: number;
  trackNumber?: number;
  trackTotal?: number;
  discNumber?: number;
  discTotal?: number;
  comment?: string;
  lyrics?: string;
  bpm?: number;
  cover?: boolean;
  extended?: Record<string, string>;
}

export type TaskStatus =
  | 'pending'
  | 'processing'
  | 'completed'
  | 'failed'
  | 'cancelled'
  | 'skipped';

/** 转换列表中的单条任务（任务书 §31） */
export interface ConversionTask {
  id: string;
  title: string;
  artist?: string;
  coverPath?: string;
  fromFormat: AudioFormat;
  toFormat: AudioFormat;
  progress: number; // 0..100
  speed?: string; // 例如 "12.4×" 或 "3.2 MB/s"
  timeRemainingSec?: number;
  status: TaskStatus;
  error?: AppErrorView;
}

/** 质量预设（任务书 原则一） */
export type QualityPreset = 'lossless' | 'high' | 'standard' | 'custom';

export type OverwritePolicy = 'ask' | 'always' | 'skip';

/** 转换配置（任务书 §30） */
export interface ConversionConfig {
  outputFormat: AudioFormat;
  quality: QualityPreset;
  outputFolder: string;
  filenameTemplate: string;
  overwrite: OverwritePolicy;
  embedCover: boolean;
  embedLyrics: boolean;
  preserveExtendedTags: boolean;
  verifyMetadata: boolean;
}

export type AppearanceMode = 'dark' | 'light' | 'system';

/** 设置（任务书 §33-§37） */
export interface Settings {
  appearance: AppearanceMode;
  conversion: {
    defaultQuality: QualityPreset;
    confirmLossy: boolean; // 有损转换需确认（原则一）
  };
  output: {
    defaultDirectory: string;
    folderStructure: string;
    filenameTemplate: string;
    overwrite: OverwritePolicy;
  };
  metadata: {
    embedCover: boolean;
    embedLyrics: boolean;
    preserveExtendedTags: boolean;
    verifyMetadata: boolean;
  };
  performance: {
    concurrency: number; // CPU/2 默认
    cpuThreads: number;
    tempDirectory: string;
    memoryStrategy: 'balanced' | 'low' | 'performance';
  };
}

export const DEFAULT_SETTINGS: Settings = {
  appearance: 'system',
  conversion: { defaultQuality: 'lossless', confirmLossy: true },
  output: {
    defaultDirectory: '',
    folderStructure: '{album}',
    filenameTemplate: '{track} {title}',
    overwrite: 'ask',
  },
  metadata: {
    embedCover: true,
    embedLyrics: true,
    preserveExtendedTags: true,
    verifyMetadata: true,
  },
  performance: {
    concurrency: Math.max(1, Math.floor((navigator.hardwareConcurrency || 4) / 2)),
    cpuThreads: 0,
    tempDirectory: '',
    memoryStrategy: 'balanced',
  },
};

/**
 * 错误展示模型（任务书 §32）
 * 必须转为人类可读描述，同时保留 Technical Details 供调试。
 */
export interface AppErrorView {
  title: string;
  reason: string;
  suggestion: string;
  technical?: string;
}

/** IPC 命令/事件名（前后端对齐，实际 Tauri 集成时启用） */
export const IPC = {
  detectFiles: 'detect_files',
  startConversion: 'start_conversion',
  pauseTask: 'pause_task',
  cancelTask: 'cancel_task',
  getSettings: 'get_settings',
  saveSettings: 'save_settings',
  getHistory: 'get_history',
} as const;
