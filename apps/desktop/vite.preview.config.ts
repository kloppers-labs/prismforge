import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";

export default defineConfig({
  root: "src/dev-preview",
  plugins: [svelte({ configFile: "../../svelte.config.js" })],
  clearScreen: false,
  server: {
    port: 1430,
    strictPort: true,
    host: "localhost",
  },
});
