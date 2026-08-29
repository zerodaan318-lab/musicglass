interface ProgressBarProps {
  value: number; // 0..100
  className?: string;
}

/** 进度条，带渐变与微光（任务书 §31 进度展示） */
export function ProgressBar({ value, className = '' }: ProgressBarProps) {
  const v = Math.max(0, Math.min(100, value));
  return (
    <div className={`h-2 w-full overflow-hidden rounded-full bg-surface-strong/60 ${className}`}>
      <div
        className="h-full rounded-full bg-gradient-to-r from-accent to-success transition-[width] duration-300 ease-out"
        style={{ width: `${v}%` }}
      />
    </div>
  );
}
