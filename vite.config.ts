import tailwindcss from "@tailwindcss/vite";
import vue from "@vitejs/plugin-vue";
import path from "path";
import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [tailwindcss(), vue()],

  clearScreen: false,
  server: {
    strictPort: true,
    host: host || false,
    watch: {
      port: 5173,
      ignored: ["**/src-tauri/**"],
    },
    envPrefix: ["VITE_", "TAURI_ENV_*"],
  },

  resolve: {
    alias: {
      "@": path.resolve(__dirname, "src"),
      "@/components": path.resolve(__dirname, "src/components"),
      "@/composables": path.resolve(__dirname, "src/composables"),
      "@/pages": path.resolve(__dirname, "src/pages"),
    },
  },

  build: {
    // Tauri uses Chromium on Windows and WebKit on macOS and Linux
    target:
      process.env.TAURI_ENV_PLATFORM == "windows" ? "chrome105" : "safari13",
    // don't minify for debug builds
    minify: !!process.env.TAURY_ENV_DEBUG,
    // produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
}));
