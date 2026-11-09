import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite";

// Tauri expects a fixed port and must not have Vite clear the terminal.
export default defineConfig({
  plugins: [vue()],
  resolve: { alias: { "@": "/src" } },
  clearScreen: false,
  server: { port: 1430, strictPort: true, watch: { ignored: ["**/src-tauri/**", "**/target/**"] } },
  build: { target: "es2022", chunkSizeWarningLimit: 1500 },
  test: { globals: true, environment: "jsdom", include: ["src/**/*.test.ts", "tests/**/*.test.ts"] },
});
