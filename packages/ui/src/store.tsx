import { useState, useCallback } from 'react';
import type {
  DetectedFile,
  ImportSummary,
  ConversionTask,
  ConversionConfig,
  Settings,
  AppErrorView,
} from '@musicglass/shared';
import { DEFAULT_SETTINGS } from '@musicglass/shared';
import { detectFiles } from './tauri';

export type View = 'home' | 'convert' | 'tasks' | 'settings';

export interface AppState {
  files: DetectedFile[];
  summary: ImportSummary | null;
  tasks: ConversionTask[];
  config: ConversionConfig;
  settings: Settings;
  errors: AppErrorView[];
  view: View;
}

const initialConfig: ConversionConfig = {
  outputFormat: 'flac',
  quality: 'lossless',
  outputFolder: '',
  filenameTemplate: '{track} {title}',
  overwrite: 'ask',
  embedCover: true,
  embedLyrics: true,
  preserveExtendedTags: true,
  verifyMetadata: true,
};

/** 根据文件列表计算导入概览统计（纯函数，无副作用，任务书 §29） */
function computeSummary(files: DetectedFile[]): ImportSummary {
  const byFormat: Record<string, number> = {};
  let supported = 0;
  let totalDuration = 0;
  for (const f of files) {
    byFormat[f.format] = (byFormat[f.format] ?? 0) + 1;
    if (f.supported) supported += 1;
    totalDuration += f.durationSec ?? 0;
  }
  return {
    total: files.length,
    supported,
    unsupported: files.length - supported,
    byFormat,
    totalDurationSec: totalDuration,
  };
}

/**
 * 轻量应用状态（任务书 §45：避免大量全局变量）。
 * 真实接入 Tauri 时，detect 走 detectFiles（→ invoke detect_files），convert 走 tauri.runTask。
 */
export function useAppStore() {
  const [files, setFiles] = useState<DetectedFile[]>([]);
  const [summary, setSummary] = useState<ImportSummary | null>(null);
  const [tasks, setTasks] = useState<ConversionTask[]>([]);
  const [config, setConfig] = useState<ConversionConfig>(initialConfig);
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);
  const [errors, setErrors] = useState<AppErrorView[]>([]);
  const [view, setView] = useState<View>('home');

  /** 合并已检测文件并更新概览 */
  const addFiles = useCallback((next: DetectedFile[]) => {
    setFiles((prev) => {
      const merged = [...prev, ...next];
      setSummary(computeSummary(merged));
      return merged;
    });
    setView('home');
  }, []);

  /**
   * 从真实文件路径检测（Tauri 环境下走 Rust detect_files，纯前端降级为模拟）。
   * 首页 Add Files / 拖拽 / Add Folder 的统一入口。
   */
  const addFilesFromPaths = useCallback(
    async (paths: string[]) => {
      if (paths.length === 0) return;
      const detected = await detectFiles(paths);
      addFiles(detected);
    },
    [addFiles]
  );

  const startConversion = useCallback(
    (cfg: ConversionConfig) => {
      setConfig(cfg);
      // 为每个文件生成一条转换任务，透传真实路径
      setTasks((prev) => [
        ...prev,
        ...files.map<ConversionTask>((f) => ({
          id: f.id,
          title: f.name.replace(/\.[^.]+$/, ''),
          artist: undefined,
          path: f.path,
          fromFormat: f.format,
          toFormat: cfg.outputFormat,
          progress: 0,
          status: 'pending',
        })),
      ]);
      setView('tasks');
    },
    [files]
  );

  const pauseTask = useCallback((id: string) => {
    setTasks((prev) => prev.map((t) => (t.id === id ? { ...t, status: 'cancelled' } : t)));
  }, []);

  const cancelTask = useCallback((id: string) => {
    setTasks((prev) => prev.map((t) => (t.id === id ? { ...t, status: 'cancelled' } : t)));
  }, []);

  const updateSettings = useCallback((next: Settings) => setSettings(next), []);

  const pushError = useCallback((e: AppErrorView) => setErrors((prev) => [...prev, e]), []);
  const clearErrors = useCallback(() => setErrors([]), []);

  return {
    files, summary, tasks, config, settings, errors, view,
    addFiles, addFilesFromPaths, startConversion, pauseTask, cancelTask,
    updateSettings, pushError, clearErrors, setView,
  };
}
