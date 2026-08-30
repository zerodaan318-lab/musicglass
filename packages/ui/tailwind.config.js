/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        // 通过 CSS 变量驱动，支持深浅色（任务书 §34）
        bg: 'rgb(var(--mg-bg) / <alpha-value>)',
        surface: 'rgb(var(--mg-surface) / <alpha-value>)',
        'surface-strong': 'rgb(var(--mg-surface-strong) / <alpha-value>)',
        border: 'rgb(var(--mg-border) / <alpha-value>)',
        text: 'rgb(var(--mg-text) / <alpha-value>)',
        muted: 'rgb(var(--mg-muted) / <alpha-value>)',
        // 低饱和蓝紫品牌色（Liquid Glass 重做后更克制）
        accent: 'rgb(var(--mg-accent) / <alpha-value>)',
        'accent-soft': 'rgb(var(--mg-accent-soft) / <alpha-value>)',
        danger: 'rgb(var(--mg-danger) / <alpha-value>)',
        success: 'rgb(var(--mg-success) / <alpha-value>)',
        warning: 'rgb(var(--mg-warning) / <alpha-value>)',
      },
      backdropBlur: {
        glass: '18px',
        'glass-lg': '28px',
        'glass-xl': '40px',
      },
      boxShadow: {
        // 大范围、低透明度、柔 blur + 顶部 inset 高光（液态玻璃）
        glass:
          '0 20px 60px -24px rgba(8, 10, 26, 0.55), inset 0 1px 0 rgba(255,255,255,0.22), inset 0 -1px 2px rgba(8,10,26,0.30)',
        'glass-hover':
          '0 28px 80px -28px rgba(8, 10, 26, 0.65), inset 0 1px 0 rgba(255,255,255,0.30), inset 0 -1px 3px rgba(8,10,26,0.35)',
        'glass-soft':
          '0 12px 40px -28px rgba(8, 10, 26, 0.50), inset 0 1px 0 rgba(255,255,255,0.16)',
        'glass-btn':
          '0 8px 24px -10px rgba(20, 60, 160, 0.45), inset 0 1px 0 rgba(255,255,255,0.35)',
        // 激活态（拖入文件）环境辉光
        'glass-active':
          '0 24px 90px -22px rgba(40, 120, 230, 0.55), inset 0 1px 0 rgba(255,255,255,0.40), 0 0 0 1px rgba(140,180,255,0.45)',
      },
      borderRadius: {
        xl2: '1.25rem',
        '3xl': '1.5rem',
        '4xl': '2rem',
      },
      transitionDuration: {
        180: '180ms',
        220: '220ms',
      },
      keyframes: {
        'fade-in': {
          '0%': { opacity: '0', transform: 'translateY(6px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        'view-in': {
          '0%': { opacity: '0', transform: 'translateY(10px) scale(0.995)' },
          '100%': { opacity: '1', transform: 'translateY(0) scale(1)' },
        },
      },
      animation: {
        'fade-in': 'fade-in 0.25s ease-out',
        'view-in': 'view-in 0.32s cubic-bezier(0.22, 1, 0.36, 1) both',
      },
    },
  },
  plugins: [],
};
