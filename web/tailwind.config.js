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
        grotesk: ["Space Grotesk", "sans-serif"],
        heading: ["Space Grotesk", "sans-serif"],
        carter:  ["Space Grotesk", "sans-serif"], // backward compatibility
        bebas:   ["Space Grotesk", "sans-serif"], // fallback alias
        bangers: ["Space Grotesk", "sans-serif"], // fallback alias
        open:    ["Open Sans", "sans-serif"],
      },
    },
  },
  plugins: [],
};
