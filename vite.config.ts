import { defineConfig } from "vite";

export default defineConfig({
  clearScreen: false,
  root: "./",
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
  server: {
    port: 5173,
    strictPort: true,
  },
});
