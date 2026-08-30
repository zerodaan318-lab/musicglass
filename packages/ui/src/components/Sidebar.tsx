import { motion } from 'framer-motion';
import type { ReactNode } from 'react';
import { IconMusic, IconPlay, IconSettings } from '@/components/Icon';
import type { View } from '@/store';

interface NavItem {
  key: View;
  label: string;
  icon: ReactNode;
}

const NAV: NavItem[] = [
  { key: 'home', label: '首页', icon: <IconMusic width={18} height={18} /> },
  { key: 'convert', label: '转换', icon: <IconPlay width={18} height={18} /> },
  { key: 'tasks', label: '任务列表', icon: <IconPlay width={18} height={18} /> },
  { key: 'settings', label: '设置', icon: <IconSettings width={18} height={18} /> },
];

interface SidebarProps {
  view: View;
  onNav: (v: View) => void;
  taskCount: number;
}

/** 左侧玻璃导航栏 */
export function Sidebar({ view, onNav, taskCount }: SidebarProps) {
  return (
    <aside className="flex w-60 shrink-0 flex-col gap-2 p-4">
      <div className="glass mb-4 flex items-center gap-3 rounded-3xl px-3 py-3">
        <div className="glass-icon grid h-11 w-11 place-items-center rounded-2xl text-accent">
          <IconMusic width={24} height={24} />
        </div>
        <div>
          <h1 className="text-lg font-semibold leading-tight text-text">MusicGlass</h1>
          <p className="text-xs text-muted">本地音乐格式转换器</p>
        </div>
      </div>

      <nav className="flex flex-col gap-1">
        {NAV.map((item) => {
          const active = view === item.key;
          return (
            <button
              key={item.key}
              onClick={() => onNav(item.key)}
              className={`relative flex items-center gap-3 rounded-xl px-3 py-2.5 text-sm transition-colors duration-180 ${
                active ? 'text-text' : 'text-muted hover:text-text hover:bg-white/5'
              }`}
            >
              {active && (
                <motion.span
                  layoutId="nav-active"
                  className="absolute inset-0 rounded-xl bg-accent/15 ring-1 ring-accent/30 backdrop-blur-md"
                  transition={{ type: 'spring', stiffness: 400, damping: 32 }}
                />
              )}
              <span className={`relative z-10 ${active ? 'text-accent' : ''}`}>{item.icon}</span>
              <span className={`relative z-10 font-medium ${active ? 'text-text' : ''}`}>{item.label}</span>
              {item.key === 'tasks' && taskCount > 0 && (
                <span className="relative z-10 ml-auto grid h-5 min-w-5 place-items-center rounded-full bg-accent px-1.5 text-xs font-semibold text-white">
                  {taskCount}
                </span>
              )}
            </button>
          );
        })}
      </nav>

      <div className="glass-soft mt-auto rounded-2xl px-3 py-3 text-xs text-muted">
        <p>完全本地运行 · 不依赖云端</p>
      </div>
    </aside>
  );
}
