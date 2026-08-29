import { useEffect } from 'react';
import { useAppStore, type View } from './store';
import { useTheme } from './theme';
import { Sidebar } from './components/Sidebar';
import { ErrorCard } from './components/ErrorCard';
import { IconAlert } from './components/Icon';
import type { AppErrorView } from '@musicglass/shared';

// 页面组件（由子代理/后续步骤填充实现）
import { HomePage } from './pages/HomePage';
import { ImportSummary } from './pages/ImportSummary';
import { ConvertPage } from './pages/ConvertPage';
import { TaskList } from './pages/TaskList';
import { SettingsPage } from './pages/SettingsPage';

function GlobalErrors({ errors, onClose }: { errors: AppErrorView[]; onClose: () => void }) {
  if (errors.length === 0) return null;
  return (
    <div className="pointer-events-none fixed inset-x-0 bottom-6 z-50 flex flex-col items-center gap-2 px-4">
      {errors.slice(-3).map((e, i) => (
        <div key={i} className="pointer-events-auto w-full max-w-xl">
          <ErrorCard error={e} />
        </div>
      ))}
      <button
        onClick={onClose}
        className="pointer-events-auto flex items-center gap-1 text-xs text-muted hover:text-text"
      >
        <IconAlert width={14} height={14} /> 清除错误提示
      </button>
    </div>
  );
}

export default function App() {
  const store = useAppStore();
  useTheme(store.settings.appearance);

  const view: View = store.view;
  const hasFiles = store.files.length > 0;

  // 原生文件拖放（Tauri WebView2 不会把系统文件拖放传给 HTML onDrop，
  // 必须通过 getCurrentWebview().onDragDropEvent 接收，否则拖入无反应）
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      try {
        const { getCurrentWebview } = await import('@tauri-apps/api/webview');
        const wv = getCurrentWebview();
        unlisten = await wv.onDragDropEvent((event) => {
          if (event.payload.type !== 'drop') return; // 仅处理文件落下的瞬间
          const paths = event.payload.paths;
          if (paths && paths.length > 0) {
            store.addFilesFromPaths(paths);
          }
        });
      } catch {
        // 非 Tauri 环境（纯预览）忽略，走 HomePage 的 HTML onDrop 降级
      }
    })();
    return () => {
      if (unlisten) unlisten();
    };
  }, [store]);

  return (
    <div className="app-bg flex h-full w-full text-text">
      <Sidebar view={view} onNav={store.setView} taskCount={store.activeTaskCount} />

      <main className="flex-1 overflow-hidden">
        <div
          key={view}
          className="h-full overflow-y-auto px-8 py-6"
        >
          {view === 'home' && (
            <div className="mx-auto max-w-4xl space-y-6">
              <HomePage onAddPaths={store.addFilesFromPaths} />
              {store.summary && (
                <ImportSummary
                  summary={store.summary}
                  files={store.files}
                  onRemove={store.removeFile}
                  onClear={store.clearFiles}
                />
              )}
            </div>
          )}

          {view === 'convert' && hasFiles && (
            <div className="mx-auto max-w-4xl">
              <ConvertPage files={store.files} onStart={store.startConversion} />
            </div>
          )}

          {view === 'convert' && !hasFiles && (
            <EmptyHint onGo={() => store.setView('home')} />
          )}

          {view === 'tasks' && (
            <div className="mx-auto max-w-4xl">
              <TaskList
                tasks={store.tasks}
                onPause={store.pauseTask}
                onCancel={store.cancelTask}
              />
            </div>
          )}

          {view === 'settings' && (
            <div className="mx-auto max-w-4xl">
              <SettingsPage settings={store.settings} onChange={store.updateSettings} />
            </div>
          )}
        </div>
      </main>

      <GlobalErrors errors={store.errors} onClose={store.clearErrors} />
    </div>
  );
}

function EmptyHint({ onGo }: { onGo: () => void }) {
  return (
    <div className="grid h-full place-items-center text-center">
      <div className="glass max-w-sm rounded-2xl p-8">
        <p className="text-lg font-medium text-text">还没有导入文件</p>
        <p className="mt-2 text-sm text-muted">请先回到首页拖入音乐文件。</p>
        <button
          onClick={onGo}
          className="mt-4 rounded-xl bg-accent px-4 py-2 text-sm font-medium text-white hover:brightness-110"
        >
          去首页导入
        </button>
      </div>
    </div>
  );
}
