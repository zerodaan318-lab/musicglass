/** 格式化时长（秒 -> mm:ss 或 hh:mm:ss） */
export function formatDuration(sec?: number): string {
  if (sec == null || !isFinite(sec)) return '--:--';
  const s = Math.floor(sec % 60);
  const m = Math.floor((sec / 60) % 60);
  const h = Math.floor(sec / 3600);
  const pad = (n: number) => n.toString().padStart(2, '0');
  return h > 0 ? `${h}:${pad(m)}:${pad(s)}` : `${pad(m)}:${pad(s)}`;
}

/** 格式化文件大小（字节 -> 人类可读） */
export function formatBytes(bytes: number): string {
  if (!bytes) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

/** 格式中文/英文标签 */
export function formatLabel(f: string): string {
  return f.toUpperCase();
}

/** 生成稳定 id（前端模拟用，真实环境由后端返回） */
export function uid(): string {
  return Math.random().toString(36).slice(2, 10);
}
