const withOpacity = (variableName) => {
  return ({ opacityValue }) => {
    if (opacityValue !== undefined) {
      return `color-mix(in srgb, var(${variableName}) calc(${opacityValue} * 100%), transparent)`;
    }
    return `var(${variableName})`;
  };
};

/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}"
  ],
  theme: {
    extend: {
      colors: {
        gi: {
          canvas: withOpacity('--gi-bg-canvas'),
          surface: {
            DEFAULT: withOpacity('--gi-bg-surface'),
            elevated: withOpacity('--gi-bg-surface-elevated'),
          },
          border: {
            subtle: withOpacity('--gi-border-subtle'),
            gold: withOpacity('--gi-border-gold'),
          },
          text: {
            primary: withOpacity('--gi-text-primary'),
            secondary: withOpacity('--gi-text-secondary'),
            muted: withOpacity('--gi-text-muted'),
          },
          accent: {
            DEFAULT: withOpacity('--gi-accent'),
            hover: withOpacity('--gi-accent-hover'),
            active: withOpacity('--gi-accent-active'),
            subtle: 'rgba(79, 195, 247, 0.12)',
            border: 'rgba(79, 195, 247, 0.3)',
          },
          primary: {
            DEFAULT: withOpacity('--gi-primary'),
            hover: withOpacity('--gi-primary-hover'),
            active: withOpacity('--gi-primary-active'),
            foreground: withOpacity('--gi-primary-foreground'),
          },
          status: {
            success: withOpacity('--gi-status-success'),
            warning: withOpacity('--gi-status-warning'),
            danger: withOpacity('--gi-status-danger'),
            info: withOpacity('--gi-status-info'),
          },
        },
        theme: {
          bg:      withOpacity('--bg'),
          surface: withOpacity('--surface'),
          card:    withOpacity('--card'),
          border:  withOpacity('--border'),
          text:    withOpacity('--text'),
          muted:   withOpacity('--muted'),
          cyan:    withOpacity('--cyan'),
          magenta: withOpacity('--violet'),
          rose:    withOpacity('--rose'),
          gold:    withOpacity('--gold'),
          crimson: withOpacity('--crimson'),
          violet:  withOpacity('--violet'),
          lilac:   withOpacity('--lilac'),
        },
      },
      fontFamily: {
        sans: [
          'system-ui',
          '-apple-system',
          'BlinkMacSystemFont',
          '"Segoe UI"',
          'Roboto',
          'Helvetica',
          'Arial',
          'sans-serif',
        ],
        heading: [
          'system-ui',
          '-apple-system',
          'BlinkMacSystemFont',
          '"Segoe UI"',
          'Roboto',
          'Helvetica',
          'Arial',
          'sans-serif',
        ],
        body: [
          'system-ui',
          '-apple-system',
          'BlinkMacSystemFont',
          '"Segoe UI"',
          'Roboto',
          'Helvetica',
          'Arial',
          'sans-serif',
        ],
        mono: [
          'ui-monospace',
          'SFMono-Regular',
          'Menlo',
          'Monaco',
          'Consolas',
          '"Liberation Mono"',
          '"Courier New"',
          'monospace',
        ],
      },
      fontSize: {
        '2xs': ['0.75rem', { lineHeight: '1.05rem' }],
        'xs': ['0.8125rem', { lineHeight: '1.15rem' }],
        'sm': ['0.9375rem', { lineHeight: '1.35rem' }],
        'base': ['1.0625rem', { lineHeight: '1.6rem' }],
        'lg': ['1.1875rem', { lineHeight: '1.75rem' }],
        'xl': ['1.375rem', { lineHeight: '1.85rem' }],
        '2xl': ['1.85rem', { lineHeight: '2.25rem' }],
        '3xl': ['2.25rem', { lineHeight: '2.6rem' }],
        '4xl': ['2.75rem', { lineHeight: '3rem' }],
      },
      boxShadow: {
        'gi-card':     '0 4px 20px -2px rgba(0, 0, 0, 0.25)',
        'gi-elevated': '0 8px 30px -4px rgba(0, 0, 0, 0.35)',
        'gi-modal':    '0 25px 50px -12px rgba(0, 0, 0, 0.75), 0 0 0 1px rgba(255, 255, 255, 0.05)',
        'gi-glow':     '0 0 16px rgba(var(--gi-primary-rgb), 0.35)',
        'gi-glow-cyan': '0 0 16px rgba(var(--gi-accent-rgb), 0.35)',
      },
    },
  },
  plugins: [],
};
