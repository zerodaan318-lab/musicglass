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
import { detectFiles, runTask } from './tauri';

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

  /** 合并已检测文件并更新概览（按路径去重，避免拖放/对话框重复添加） */
  const addFiles = useCallback((next: DetectedFile[]) => {
    setFiles((prev) => {
      const seen = new Set(prev.map((f) => f.path));
      const filtered = next.filter((f) => !seen.has(f.path));
      const merged = [...prev, ...filtered];
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
      const outputDir = cfg.outputFolder || 'D:\\Music\\Converted';
      // 为每个文件生成一条转换任务，透传真实路径
      const newTasks = files.map<ConversionTask>((f) => ({
        id: f.id,
        title: f.name.replace(/\.[^.]+$/, ''),
        artist: undefined,
        path: f.path,
        fromFormat: f.format,
        toFormat: cfg.outputFormat,
        progress: 0,
        status: 'pending',
      }));
      setTasks((prev) => [...prev, ...newTasks]);
      setView('tasks');

      // 真实驱动：逐个调用后端 convert_audio（任务书 §6 Pipeline）
      (async () => {
        for (const t of newTasks) {
          setTasks((prev) =>
            prev.map((x) => (x.id === t.id ? { ...x, status: 'processing', progress: 5 } : x))
          );
          try {
            const res = await runTask(t, outputDir);
            setTasks((prev) =>
              prev.map((x) =>
                x.id === t.id
                  ? {
                      ...x,
                      status: 'completed',
                      progress: 100,
                      outputPath: res.outputPath,
                      verification: res.verification,
                    }
                  : x
              )
            );
          } catch (e: any) {
            setTasks((prev) =>
              prev.map((x) =>
                x.id === t.id
                  ? {
                      ...x,
                      status: 'failed',
                      progress: 0,
                      error: {
                        title: String(e?.message || e || '转换失败'),
                        reason: '后端转换命令返回错误',
                        suggestion: '检查输入文件是否存在、FFmpeg 是否可用，或查看控制台日志',
                      },
                    }
                  : x
              )
            );
          }
        }
      })();
    },
    [files]
  );

  const removeFile = useCallback((id: string) => {
    setFiles((prev) => {
      const merged = prev.filter((f) => f.id !== id);
      setSummary(merged.length > 0 ? computeSummary(merged) : null);
      return merged;
    });
  }, []);

  const clearFiles = useCallback(() => {
    setFiles([]);
    setSummary(null);
  }, []);

  const pauseTask = useCallback((id: string) => {
    setTasks((prev) => prev.map((t) => (t.id === id ? { ...t, status: 'cancelled' } : t)));
  }, []);

  const cancelTask = useCallback((id: string) => {
    setTasks((prev) => prev.map((t) => (t.id === id ? { ...t, status: 'cancelled' } : t)));
  }, []);

  const updateSettings = useCallback((next: Settings) => setSettings(next), []);

  const pushError = useCallback((e: AppErrorView) => setErrors((prev) => [...prev, e]), []);
  const clearErrors = useCallback(() => setErrors([]), []);

  const activeTaskCount = tasks.filter(
    (t) => t.status === 'pending' || t.status === 'processing'
  ).length;

  return {
    files, summary, tasks, config, settings, errors, view, activeTaskCount,
    addFiles, addFilesFromPaths, startConversion, pauseTask, cancelTask,
    removeFile, clearFiles,
    updateSettings, pushError, clearErrors, setView,
  };
}
