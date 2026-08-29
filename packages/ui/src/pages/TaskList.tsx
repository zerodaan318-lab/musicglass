/**
 * 转换列表页面（任务书 §31）
 * 以玻璃卡片逐条展示转换任务：封面 / 标题 / 艺术家 / 格式转换标签 / 进度 /
 * 速度 / 剩余时间 / 状态徽章，以及进行中的 暂停、取消 操作。
 * 已完成显示绿色对勾徽章；失败显示红色警示 + 标题级原因（不展开 Technical Details，
 * 那是 ErrorCard 的职责）。列表项进出场使用 Framer Motion 的 AnimatePresence 动画。
 */
import { AnimatePresence, motion } from 'framer-motion';
import type { ConversionTask, TaskStatus } from '@musicglass/shared';
import { GlassCard } from '@/components/GlassCard';
import { Button } from '@/components/Button';
import { ProgressBar } from '@/components/ProgressBar';
import { revealFile } from '@/tauri';
import {
  IconAlert,
  IconCheck,
  IconMusic,
  IconPause,
  IconPlay,
  IconX,
  IconFolder,
} from '@/components/Icon';
import { formatDuration, formatLabel } from '@/lib/format';

interface TaskListProps {
  /** 待展示的转换任务列表 */
  tasks: ConversionTask[];
  /** 暂停某条进行中的任务 */
  onPause: (id: string) => void;
  /** 取消某条任务 */
  onCancel: (id: string) => void;
  /** 开始 / 恢复某条任务（可选，用于 pending 态） */
  onResume?: (id: string) => void;
}

/** 各状态的徽章样式（任务书 §31 状态展示） */
const STATUS_BADGE: Record<TaskStatus, { label: string; className: string }> = {
  pending: { label: '等待中', className: 'bg-warning/15 text-warning' },
  processing: { label: '转换中', className: 'bg-accent/15 text-accent' },
  completed: { label: '已完成', className: 'bg-success/15 text-success' },
  failed: { label: '失败', className: 'bg-danger/15 text-danger' },
  cancelled: { label: '已取消', className: 'bg-muted/15 text-muted' },
  skipped: { label: '已跳过', className: 'bg-muted/15 text-muted' },
};

/** 状态徽章：已完成带绿色对勾，失败带红色警示 */
function StatusBadge({ status }: { status: TaskStatus }) {
  const meta = STATUS_BADGE[status];
  const Icon = status === 'completed' ? IconCheck : status === 'failed' ? IconAlert : null;
  return (
    <span
      className={`inline-flex shrink-0 items-center gap-1 rounded-full px-2.5 py-1 text-xs font-medium ${meta.className}`}
    >
      {Icon && <Icon width={14} height={14} />}
      {meta.label}
    </span>
  );
}

export function TaskList({ tasks, onPause, onCancel, onResume }: TaskListProps) {
  if (tasks.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center gap-3 py-20 text-center">
        <span className="grid h-14 w-14 place-items-center rounded-2xl bg-surface-strong/50 text-muted">
          <IconMusic width={26} height={26} />
        </span>
        <p className="text-sm text-muted">暂无转换任务</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-3">
      <AnimatePresence initial={false}>
        {tasks.map((task) => {
          const isProcessing = task.status === 'processing';
          const isPending = task.status === 'pending';
          const isActive = isProcessing || isPending;

          return (
            <motion.div
              key={task.id}
              layout
              initial={{ opacity: 1, y: 8, scale: 0.98 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, scale: 0.96, transition: { duration: 0.15 } }}
              transition={{ type: 'spring', stiffness: 320, damping: 28 }}
            >
              <GlassCard className="flex gap-4">
                {/* 封面：无封面时用 IconMusic 占位渐变圆角方块 */}
                {task.coverPath ? (
                  <img
                    src={task.coverPath}
                    alt={task.title}
                    className="aspect-square w-16 shrink-0 rounded-xl object-cover"
                  />
                ) : (
                  <div className="grid aspect-square w-16 shrink-0 place-items-center rounded-xl bg-gradient-to-br from-accent/30 to-success/20 text-white/70">
                    <IconMusic width={26} height={26} />
                  </div>
                )}

                <div className="min-w-0 flex-1">
                  {/* 标题 / 艺术家 + 状态徽章 */}
                  <div className="flex items-start justify-between gap-3">
                    <div className="min-w-0">
                      <h3 className="truncate text-base font-semibold text-text">
                        {task.title}
                      </h3>
                      {task.artist && (
                        <p className="truncate text-sm text-muted">{task.artist}</p>
                      )}
                    </div>
                    <StatusBadge status={task.status} />
                  </div>

                  {/* 格式转换标签：FROM → TO（大写） */}
                  <div className="mt-2 inline-flex items-center gap-1.5 rounded-lg bg-surface-strong/50 px-2.5 py-1 text-xs font-medium text-muted">
                    <span className="text-text">{formatLabel(task.fromFormat)}</span>
                    <span aria-hidden>→</span>
                    <span className="text-accent">{formatLabel(task.toFormat)}</span>
                  </div>

                  {/* 进度条 + 百分比 */}
                  <div className="mt-3 flex items-center gap-3">
                    <div className="flex-1">
                      <ProgressBar value={task.progress} />
                    </div>
                    <span className="w-10 text-right text-xs tabular-nums text-muted">
                      {Math.round(task.progress)}%
                    </span>
                  </div>

                  {/* 失败原因（标题级，不展开 Technical Details） */}
                  {task.status === 'failed' && task.error ? (
                    <div className="mt-3 flex items-center gap-2 text-sm text-danger">
                      <IconAlert className="shrink-0" />
                      <span className="truncate">{task.error.title}</span>
                    </div>
                  ) : (
                    /* 速度 / 剩余时间 + 操作按钮 */
                    <div className="mt-3 flex items-center justify-between gap-3">
                      <div className="flex items-center gap-4 text-xs text-muted">
                        {task.status === 'completed' && task.outputPath ? (
                          <span className="truncate text-success">已保存到 {task.outputPath}</span>
                        ) : (
                          <>
                            <span>速度 {task.speed ?? '--'}</span>
                            <span>剩余 {formatDuration(task.timeRemainingSec)}</span>
                          </>
                        )}
                      </div>
                      <div className="flex shrink-0 items-center gap-2">
                        {task.status === 'completed' && task.outputPath && (
                          <Button
                            variant="ghost"
                            icon={<IconFolder />}
                            onClick={() => revealFile(task.outputPath!)}
                          >
                            打开
                          </Button>
                        )}
                        {isActive && (
                          <>
                            {isProcessing && (
                              <Button
                                variant="ghost"
                                icon={<IconPause />}
                                onClick={() => onPause(task.id)}
                              >
                                暂停
                              </Button>
                            )}
                            {isPending && onResume && (
                              <Button
                                variant="ghost"
                                icon={<IconPlay />}
                                onClick={() => onResume(task.id)}
                              >
                                开始
                              </Button>
                            )}
                            <Button
                              variant="ghost"
                              className="text-danger hover:bg-danger/10 hover:text-danger"
                              icon={<IconX />}
                              onClick={() => onCancel(task.id)}
                            >
                              取消
                            </Button>
                          </>
                        )}
                      </div>
                    </div>
                  )}
                </div>
              </GlassCard>
            </motion.div>
          );
        })}
      </AnimatePresence>
    </div>
  );
}

export default TaskList;
