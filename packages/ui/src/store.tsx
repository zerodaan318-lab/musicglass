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

/**
 * 轻量应用状态（任务书 §45：避免大量全局变量）。
 * 真实接入 Tauri 时，detect/convert 等副作用改为调用 invoke(IPC.*)。
 */
export function useAppStore() {
  const [files, setFiles] = useState<DetectedFile[]>([]);
  const [summary, setSummary] = useState<ImportSummary | null>(null);
  const [tasks, setTasks] = useState<ConversionTask[]>([]);
  const [config, setConfig] = useState<ConversionConfig>(initialConfig);
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);
  const [errors, setErrors] = useState<AppErrorView[]>([]);
  const [view, setView] = useState<View>('home');

  const addFiles = useCallback((next: DetectedFile[]) => {
    setFiles((prev) => {
      const merged = [...prev, ...next];
      const byFormat: Record<string, number> = {};
      let supported = 0;
      let totalDuration = 0;
      for (const f of merged) {
        byFormat[f.format] = (byFormat[f.format] ?? 0) + 1;
        if (f.supported) supported += 1;
        totalDuration += f.durationSec ?? 0;
      }
      setSummary({
        total: merged.length,
        supported,
        unsupported: merged.length - supported,
        byFormat,
        totalDurationSec: totalDuration,
      });
      return merged;
    });
    setView('home');
  }, []);

  const startConversion = useCallback((cfg: ConversionConfig) => {
    setConfig(cfg);
    // 模拟：为每个文件生成一条转换任务
    setTasks((prev) => [
      ...prev,
      ...files.map<ConversionTask>((f) => ({
        id: f.id,
        title: f.name.replace(/\.[^.]+$/, ''),
        artist: undefined,
        fromFormat: f.format,
        toFormat: cfg.outputFormat,
        progress: 0,
        status: 'pending',
      })),
    ]);
    setView('tasks');
  }, [files]);

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
    addFiles, startConversion, pauseTask, cancelTask,
    updateSettings, pushError, clearErrors, setView,
  };
}
