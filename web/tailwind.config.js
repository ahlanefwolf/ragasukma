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
        sans:    ["IBM Plex Sans", "-apple-system", "BlinkMacSystemFont", "system-ui", "sans-serif"],
        grotesk: ["IBM Plex Sans", "sans-serif"],
        heading: ["IBM Plex Sans", "sans-serif"],
        mono:    ["IBM Plex Mono", "Menlo", "Monaco", "Consolas", "monospace"],
        carter:  ["IBM Plex Sans", "sans-serif"],
        bebas:   ["IBM Plex Sans", "sans-serif"],
        bangers: ["IBM Plex Sans", "sans-serif"],
        open:    ["IBM Plex Sans", "sans-serif"],
      },
    },
  },
  plugins: [],
};
