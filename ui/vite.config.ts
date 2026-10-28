import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Port is fixed: the Tauri devUrl points at 1420 and a stray port silently
// breaks `tauri dev`.
export default defineConfig({
  plugins: [svelte()],
  // Relative asset URLs: keeps `tauri://localhost` happy and lets the built
  // `ui/dist` be opened directly for a visual review without a dev server.
  base: "./",
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: "127.0.0.1",
  },
  build: {
    target: "chrome105",
    minify: "esbuild",
    sourcemap: false,
    // Keep the bundle small: the webview is only for settings, not typing.
    chunkSizeWarningLimit: 600,
  },
});
