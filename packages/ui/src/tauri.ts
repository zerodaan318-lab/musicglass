/**
 * Tauri 适配层
 * 在 Tauri 壳内调用真实 Rust IPC；在纯 Vite 预览下动态导入失败则降级为模拟。
 * 这样同一套 UI 既能 `cargo tauri dev` 真运行，也能 `pnpm dev` 单独预览。
 */
import type { DetectedFile, ConversionTask, Metadata } from '@musicglass/shared';

let tauri: typeof import('@tauri-apps/api') | null = null;
let invokeFn: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | null = null;

async function ensureTauri() {
  if (tauri || invokeFn === null) {
    // 已尝试过
  }
  try {
    const mod = await import('@tauri-apps/api');
    tauri = mod;
    // Tauri 2 推荐用 @tauri-apps/api/core 的 invoke
    const core = await import('@tauri-apps/api/core');
    invokeFn = core.invoke;
  } catch {
    invokeFn = null; // 纯前端环境，降级
  }
  return invokeFn;
}

export function isTauri(): boolean {
  return typeof (globalThis as any).__TAURI__ !== 'undefined' || invokeFn !== null;
}

// ── 降级模拟（仅在非 Tauri 环境使用）──
function mockDetect(name: string, sizeBytes: number): DetectedFile {
  const ext = (name.split('.').pop() || '').toLowerCase();
  const supported = ['mp3', 'flac', 'wav', 'm4a', 'aac', 'ogg', 'opus', 'ape', 'wma', 'ncm', 'qmc', 'qmc0', 'qmc2', 'qmc3', 'mgg', 'mgg0', 'mgg1', 'mflac', 'mflac0'].includes(ext);
  const isProprietary = ext.startsWith('ncm') || ext.startsWith('qmc') || ext.startsWith('mgg') || ext.startsWith('mflac');
  return {
    id: Math.random().toString(36).slice(2, 10),
    name,
    path: `C:\\Users\\User\\Music\\${name}`,
    sizeBytes,
    format: (supported ? ext : 'unknown') as DetectedFile['format'],
    confidence: supported ? 0.98 : 0,
    isProprietary,
    supported,
    durationSec: Math.floor(Math.random() * 240) + 30,
  };
}

export async function detectFiles(paths: string[]): Promise<DetectedFile[]> {
  const invoke = await ensureTauri();
  if (!invoke) {
    return paths.map((p) => mockDetect(p.split(/[\\/]/).pop() || p, 3_000_000));
  }
  const dtos = (await invoke('detect_files', { paths })) as any[];
  return dtos.map((d) => ({
    id: Math.random().toString(36).slice(2, 10),
    name: d.path.split(/[\\/]/).pop() || d.path,
    path: d.path,
    sizeBytes: d.size_bytes,
    format: d.format,
    confidence: d.confidence,
    isProprietary: d.is_proprietary,
    supported: d.supported,
    durationSec: d.duration_secs ?? undefined,
  }));
}

export async function extractMetadata(path: string): Promise<Metadata | null> {
  const invoke = await ensureTauri();
  if (!invoke) return null;
  try {
    return (await invoke('extract_metadata', { path })) as Metadata;
  } catch {
    return null;
  }
}

export async function convertAudio(
  input: string,
  output: string,
  targetFormat: string,
  bitrate?: number
): Promise<void> {
  const invoke = await ensureTauri();
  if (!invoke) return; // 模拟环境无操作
  await invoke('convert_audio', { input, output, targetFormat, bitrate });
}

/** 转换单个任务（供 TaskList 驱动；真实环境会调后端并回传进度事件） */
export async function runTask(task: ConversionTask, outputDir: string): Promise<void> {
  const invoke = await ensureTauri();
  if (!invoke) return;
  const outName = `${task.title}.${task.toFormat}`;
  const outPath = `${outputDir}/${outName}`;
  await convertAudio(task.path ?? '', outPath, task.toFormat);
}

export async function doctor(): Promise<{ ffmpegAvailable: boolean }> {
  const invoke = await ensureTauri();
  if (!invoke) return { ffmpegAvailable: false };
  const r = (await invoke('doctor')) as any;
  return { ffmpegAvailable: !!r?.ffmpeg_available };
}
