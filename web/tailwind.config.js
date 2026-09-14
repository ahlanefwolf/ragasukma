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
          bg: "#161D24",
          surface: "#1D252E",
          card: "#232D36",
          border: "#2F3B46",
          canvas: "#EFF7F6",
          cyan: "#7BDFF2",
          rose: "#F2B5D4",
          dark: "#232D36",
          text: "#EFF7F6",
          muted: "#9AAAB7"
        },
        anime: {
          crimson: "#F2B5D4",
          light: "#EFF7F6",
          sage: "#7BDFF2",
          forest: "#7BDFF2",
          dark: "#161D24",
          card: "#232D36",
          border: "#2F3B46"
        }
      },
      fontFamily: {
        bangers: ["Bangers", "cursive"],
        open: ["Open Sans", "sans-serif"]
      }
    }
  },
  plugins: []
};
