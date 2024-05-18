import type { Config } from 'tailwindcss'


export default {
  content: ["./src/**/*.vue"],
  theme: {
    extend: {
      colors: {
        primary: "#CD1919",
        placeholder: "#B29E95",
        text: "#C5BBB7",
        supporting: "#6D6D6D",
        stroke: {
          "100": "#222222",
          "200": "#303030",
        },
        background: "#0F0F0F",
        card: "#111111",
      },
      aspectRatio: {
        card: "3.9 / 1",
        secondaryCard: "0.75 / 1",
      }
    },
  },
  plugins: [],
} satisfies Config