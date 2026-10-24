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
          canvas: '#0F1218',
          surface: {
            DEFAULT: '#181D26',
            elevated: '#222936',
          },
          border: {
            subtle: '#333C4D',
            gold: '#9E8046',
          },
          text: {
            primary: '#EDEAE1',
            secondary: '#9FA7B8',
            muted: '#636D80',
          },
          accent: {
            DEFAULT: '#4FC3F7',
            hover: '#7CD4FD',
            active: '#2BA6DE',
            subtle: 'rgba(79, 195, 247, 0.12)',
            border: 'rgba(79, 195, 247, 0.3)',
          },
          primary: {
            DEFAULT: '#E5B85C',
            hover: '#F3CE7A',
            active: '#C79A3E',
            foreground: '#111318',
          },
          status: {
            success: '#45B876',
            warning: '#E09F3E',
            danger: '#E05345',
            info: '#3E99E0',
          },
        },
        theme: {
          bg:      "var(--bg)",
          surface: "var(--surface)",
          card:    "var(--card)",
          border:  "var(--border)",
          text:    "var(--text)",
          cyan:    "var(--cyan)",
          magenta: "var(--magenta)",
          rose:    "var(--rose)",
        },
      },
      fontFamily: {
        sans:      ["Rajdhani", "-apple-system", "BlinkMacSystemFont", "system-ui", "sans-serif"],
        display:   ["Cinzel", "-apple-system", "BlinkMacSystemFont", "system-ui", "sans-serif"],
        script:    ['"Lora"', "serif"],
        mono:      ["JetBrains Mono", "Menlo", "Monaco", "Consolas", "monospace"],
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
    },
  },
  plugins: [],
};
