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
  root.setAttribute('data-theme', resolve());
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
