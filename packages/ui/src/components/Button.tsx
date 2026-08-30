import type { ButtonHTMLAttributes, ReactNode } from 'react';

type Variant = 'primary' | 'secondary' | 'ghost' | 'danger';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  icon?: ReactNode;
}

const styles: Record<Variant, string> = {
  primary:
    'text-white bg-gradient-to-br from-accent/85 to-accent-soft/70 hover:brightness-110 shadow-glass-btn border border-white/25 backdrop-blur-md disabled:opacity-50',
  secondary:
    'glass text-text hover:bg-white/10 active:scale-[0.98] disabled:opacity-50',
  ghost:
    'text-muted hover:text-text hover:bg-white/8 active:scale-[0.98] disabled:opacity-40',
  danger:
    'bg-danger/90 text-white hover:bg-danger shadow-glass-soft border border-white/15 disabled:opacity-50',
};

/** 主操作按钮，保证对比度与可识别性（任务书 §28 约束） */
export function Button({ variant = 'primary', icon, children, className = '', ...rest }: ButtonProps) {
  return (
    <button
      className={`inline-flex items-center justify-center gap-2 rounded-2xl px-4 py-2.5 text-sm font-medium transition-all duration-180 active:scale-[0.97] hover:-translate-y-0.5 disabled:cursor-not-allowed disabled:hover:translate-y-0 disabled:active:scale-100 ${styles[variant]} ${className}`}
      {...rest}
    >
      {icon}
      {children}
    </button>
  );
}
