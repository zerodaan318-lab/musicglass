import { motion } from 'framer-motion';
import type { ReactNode } from 'react';

interface GlassCardProps {
  children: ReactNode;
  className?: string;
  strong?: boolean;
  hover?: boolean;
}

/** 玻璃拟态卡片（任务书 §28：玻璃卡片 + 柔和阴影 + 适度背景模糊） */
export function GlassCard({ children, className = '', strong, hover }: GlassCardProps) {
  return (
    <motion.div
      className={`rounded-2xl p-5 ${strong ? 'glass-strong' : 'glass'} ${className}`}
      whileHover={hover ? { y: -2, boxShadow: '0 12px 40px rgba(0,0,0,0.24)' } : undefined}
      transition={{ type: 'spring', stiffness: 300, damping: 24 }}
    >
      {children}
    </motion.div>
  );
}
