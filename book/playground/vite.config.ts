import { defineConfig } from "vite";

export default defineConfig({
  base: "/playground/",
  build: {
    outDir: "../src/playground",
    emptyOutDir: true,
  },
});
