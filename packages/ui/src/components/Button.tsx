import type { ButtonHTMLAttributes, ReactNode } from 'react';

type Variant = 'primary' | 'secondary' | 'ghost' | 'danger';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  icon?: ReactNode;
}

const styles: Record<Variant, string> = {
  primary:
    'bg-accent text-white hover:brightness-110 shadow-glass disabled:opacity-50',
  secondary:
    'glass text-text hover:bg-surface-strong/60 disabled:opacity-50',
  ghost:
    'text-muted hover:text-text hover:bg-surface-strong/40 disabled:opacity-40',
  danger:
    'bg-danger/90 text-white hover:bg-danger disabled:opacity-50',
};

/** 主操作按钮，保证对比度与可识别性（任务书 §28 约束） */
export function Button({ variant = 'primary', icon, children, className = '', ...rest }: ButtonProps) {
  return (
    <button
      className={`inline-flex items-center justify-center gap-2 rounded-xl px-4 py-2.5 text-sm font-medium transition disabled:cursor-not-allowed ${styles[variant]} ${className}`}
      {...rest}
    >
      {icon}
      {children}
    </button>
  );
}
