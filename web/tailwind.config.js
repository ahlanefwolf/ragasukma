/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}"
  ],
  theme: {
    extend: {
      colors: {
        theme: {
          bg:      "var(--bg)",
          surface: "var(--surface)",
          card:    "var(--card)",
          border:  "var(--border)",
          text:    "var(--text)",
          muted:   "var(--muted)",
          cyan:    "var(--cyan)",
          rose:    "var(--rose)",
        },
      },
      fontFamily: {
        sans:      ["Plus Jakarta Sans", "-apple-system", "BlinkMacSystemFont", "system-ui", "sans-serif"],
        heading:   ["Playwrite NG Modern", "cursive"],
        script:    ["Playwrite NG Modern", "cursive"],
        mono:      ["JetBrains Mono", "Menlo", "Monaco", "Consolas", "monospace"],
      },
    },
  },
  plugins: [],
};
