import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  test: {
    environment: "happy-dom",
    globals: true,
    setupFiles: ["./src/__tests__/setup.js"],
    include: ["src/**/*.test.{js,jsx}"],
  },
  resolve: {
    alias: {
      "@tauri-apps/api/core": new URL("./src/__tests__/__mocks__/tauri.js", import.meta.url).pathname,
      "@tauri-apps/plugin-notification": new URL("./src/__tests__/__mocks__/notification.js", import.meta.url).pathname,
    },
  },
});
