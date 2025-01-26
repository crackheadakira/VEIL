import type { Config } from "tailwindcss";
import { iconsPlugin, getIconCollections } from '@egoist/tailwindcss-icons';

export default {
  content: ["./src/**/*.vue"],
  plugins: [
    iconsPlugin({
      collections: getIconCollections(["fluent"]),
      scale: 1.5,
    })],
} satisfies Config