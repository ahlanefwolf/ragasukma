/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}"
  ],
  theme: {
    extend: {
      colors: {
        anime: {
          crimson: "#D96868",
          light: "#F2F2F2",
          sage: "#91AE6E",
          forest: "#689D4B",
          dark: "#1E1E24",
          card: "#282832",
          border: "#3B3B48"
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
