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
        accent: 'rgb(var(--mg-accent) / <alpha-value>)',
        'accent-soft': 'rgb(var(--mg-accent-soft) / <alpha-value>)',
        danger: 'rgb(var(--mg-danger) / <alpha-value>)',
        success: 'rgb(var(--mg-success) / <alpha-value>)',
        warning: 'rgb(var(--mg-warning) / <alpha-value>)',
      },
      backdropBlur: {
        glass: '18px',
      },
      boxShadow: {
        glass: '0 8px 32px rgba(0, 0, 0, 0.18), inset 0 1px 0 rgba(255,255,255,0.08)',
        'glass-hover': '0 12px 40px rgba(0, 0, 0, 0.24), inset 0 1px 0 rgba(255,255,255,0.12)',
      },
      borderRadius: {
        xl2: '1.25rem',
      },
      keyframes: {
        'fade-in': {
          '0%': { opacity: '0', transform: 'translateY(6px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
      },
      animation: {
        'fade-in': 'fade-in 0.25s ease-out',
      },
    },
  },
  plugins: [],
};
