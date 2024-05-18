import type { Config } from 'tailwindcss'


export default {
  content: ["./src/**/*.vue"],
  theme: {
    extend: {
      fontSize: { // Major Third scale
        supporting: "0.8rem",
        main: "1rem",
        h6: "1.25rem",
        h5: "1.563rem",
        h4: "1.953rem",
        h3: "2.441rem",
        h2: "3.052rem",
        h1: "3.815rem",
      },
      colors: {
        primary: "#CD1919",
        placeholder: "#B29E95",
        text: "#C5BBB7",
        supporting: "#6D6D6D",
        stroke: "#222222",
        background: "#0F0F0F",
        card: "#111111",
      }
    },
  },
  plugins: [],
} satisfies Config