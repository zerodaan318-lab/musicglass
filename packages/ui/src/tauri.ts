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
  try {
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
  } catch (e) {
    // 单个文件检测失败（如路径不存在）不应让整页崩溃：降级为 unsupported 标记
    console.warn('[detectFiles] 部分文件检测失败，已降级:', e);
    return paths.map((p) => ({
      id: Math.random().toString(36).slice(2, 10),
      name: p.split(/[\\/]/).pop() || p,
      path: p,
      sizeBytes: 0,
      format: 'unknown' as DetectedFile['format'],
      confidence: 0,
      isProprietary: false,
      supported: false,
      durationSec: undefined,
    }));
  }
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

/** 转换单个任务（供 TaskList 驱动；真实环境会调后端并回传进度事件）
 *  返回输出文件的完整路径，供完成后「打开/定位」使用。 */
export async function runTask(task: ConversionTask, outputDir: string): Promise<string> {
  const invoke = await ensureTauri();
  const outName = `${task.title}.${task.toFormat}`;
  // 统一用反斜杠（Windows 路径规范），避免 explorer / select 定位失败
  const normDir = outputDir.replace(/\//g, '\\');
  const outPath = `${normDir}\\${outName}`;
  if (!invoke) return outPath; // 模拟环境无操作，仍返回预期路径
  await convertAudio(task.path ?? '', outPath, task.toFormat);
  return outPath;
}

export async function doctor(): Promise<{ ffmpegAvailable: boolean }> {
  const invoke = await ensureTauri();
  if (!invoke) return { ffmpegAvailable: false };
  const r = (await invoke('doctor')) as any;
  return { ffmpegAvailable: !!r?.ffmpeg_available };
}

/** 在文件资源管理器中定位输出文件（Windows 资源管理器选中 / macOS 打开所在文件夹） */
export async function revealFile(path: string): Promise<void> {
  const invoke = await ensureTauri();
  if (!invoke) return; // 纯前端环境无操作
  await invoke('reveal_file', { path });
}

// ── 文件选择对话框（Tauri dialog 插件）──
// 非 Tauri 环境返回 null，调用方应降级到模拟样本。
export async function openFilesDialog(multiple = true): Promise<string[] | null> {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({
      multiple,
      title: '选择音乐文件',
      filters: [
        { name: '音频', extensions: ['mp3', 'flac', 'wav', 'm4a', 'aac', 'ogg', 'opus', 'ape', 'wma', 'ncm', 'qmc', 'mgg', 'mflac'] },
        { name: '所有文件', extensions: ['*'] },
      ],
    });
    if (!selected) return null;
    return Array.isArray(selected) ? selected : [selected];
  } catch {
    return null; // 非 Tauri 环境降级
  }
}

export async function openFolderDialog(): Promise<string[] | null> {
  const { open } = await import('@tauri-apps/plugin-dialog');
  // directory:true 走 Win32 文件夹选择框；显式 defaultPath 避免某些环境下静默失败
  const selected = await open({
    directory: true,
    multiple: false,
    title: '选择音乐文件夹',
    defaultPath: 'D:\\Music\\Converted',
  });
  if (!selected) return null;
  return Array.isArray(selected) ? selected : [selected];
}
