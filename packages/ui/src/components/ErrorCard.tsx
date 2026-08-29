import { useState } from 'react';
import type { AppErrorView } from '@musicglass/shared';

/**
 * 错误 UI（任务书 §32）
 * 必须转成人类可读描述（原因 + 建议），
 * 同时保留可折叠的 Technical Details 供调试。
 */
export function ErrorCard({ error }: { error: AppErrorView }) {
  const [open, setOpen] = useState(false);
  return (
    <div className="glass rounded-2xl border-danger/40 p-5">
      <div className="flex items-start gap-3">
        <span className="mt-0.5 grid h-8 w-8 shrink-0 place-items-center rounded-full bg-danger/15 text-danger">
          !
        </span>
        <div className="min-w-0">
          <h3 className="text-base font-semibold text-text">{error.title}</h3>
          <p className="mt-1 text-sm text-muted">
            <span className="font-medium text-text">原因：</span>
            {error.reason}
          </p>
          <p className="mt-1 text-sm text-muted">
            <span className="font-medium text-text">建议：</span>
            {error.suggestion}
          </p>
          {error.technical && (
            <div className="mt-3">
              <button
                onClick={() => setOpen((o) => !o)}
                className="text-xs font-medium text-accent hover:underline"
              >
                {open ? '收起 Technical Details' : '查看 Technical Details'}
              </button>
              {open && (
                <pre className="mt-2 max-h-48 overflow-auto rounded-lg bg-black/40 p-3 text-xs text-muted">
                  {error.technical}
                </pre>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
