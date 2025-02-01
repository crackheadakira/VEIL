import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import Unimport from "unimport/unplugin";

const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [
    tailwindcss(),
    vue(),
    Unimport.vite({
      presets: ["vue"],
      addons: { vueTemplate: true },
      dts: true,
      dirs: ["src/composables/*"],
    }),
  ],

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
