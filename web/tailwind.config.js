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
            muted: '#5F6675',
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
          muted:   "var(--muted)",
          cyan:    "var(--cyan)",
          rose:    "var(--rose)",
          crimson: "var(--crimson)",
          gold:    "var(--gold)",
          violet:  "var(--violet)",
          lilac:   "var(--lilac)",
        },
      },
      fontFamily: {
        heading:   ['"Lora"', "serif"],
        body:      ['"Plus Jakarta Sans"', "sans-serif"],
        sans:      ["Plus Jakarta Sans", "-apple-system", "BlinkMacSystemFont", "system-ui", "sans-serif"],
        script:    ['"Lora"', "serif"],
        mono:      ["JetBrains Mono", "Menlo", "Monaco", "Consolas", "monospace"],
      },
      fontSize: {
        xs:   ['0.75rem', { lineHeight: '1rem' }],
        sm:   ['0.875rem', { lineHeight: '1.25rem' }],
        base: ['1rem', { lineHeight: '1.5rem' }],
        lg:   ['1.25rem', { lineHeight: '1.75rem' }],
        xl:   ['1.5rem', { lineHeight: '2rem' }],
        '2xl': ['1.85rem', { lineHeight: '2.25rem' }],
        '3xl': ['2.25rem', { lineHeight: '2.6rem' }],
        '4xl': ['2.75rem', { lineHeight: '3rem' }],
      },
    },
  },
  plugins: [],
};
