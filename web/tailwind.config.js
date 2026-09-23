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
        sans:    ["Plus Jakarta Sans", "-apple-system", "BlinkMacSystemFont", "system-ui", "sans-serif"],
        grotesk: ["Syne", "sans-serif"],
        heading: ["Syne", "sans-serif"],
        mono:    ["JetBrains Mono", "Menlo", "Monaco", "Consolas", "monospace"],
        celtic:  ["Syne", "sans-serif"],
        carter:  ["Syne", "sans-serif"],
        bebas:   ["Syne", "sans-serif"],
        bangers: ["Syne", "sans-serif"],
        open:    ["Plus Jakarta Sans", "sans-serif"],
      },
    },
  },
  plugins: [],
};
