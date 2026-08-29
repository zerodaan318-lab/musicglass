import { useEffect } from 'react';
import type { AppearanceMode } from '@musicglass/shared';

/**
 * 应用主题到 <html data-theme>，支持 dark/light/system（任务书 §34）。
 * system 跟随 prefers-color-scheme。
 */
export function applyTheme(mode: AppearanceMode) {
  const root = document.documentElement;
  const resolve = (): 'dark' | 'light' => {
    if (mode === 'system') {
      return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
    }
    return mode;
  };
  const next = resolve();
  if (root.getAttribute('data-theme') === next) return;
  root.setAttribute('data-theme', next);
  // 强制同步重绘：WebView2 在 data-theme 切换后偶发复合层/backdrop-filter 不刷新，
  // 导致浅色主题下玻璃卡片与背景同色而整页看似空白。触发一次 reflow 修正。
  void document.body.offsetHeight;
}

export function useTheme(mode: AppearanceMode) {
  useEffect(() => {
    applyTheme(mode);
    if (mode === 'system') {
      const mq = window.matchMedia('(prefers-color-scheme: light)');
      const handler = () => applyTheme('system');
      mq.addEventListener('change', handler);
      return () => mq.removeEventListener('change', handler);
    }
  }, [mode]);
}
