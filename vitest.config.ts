import { defineConfig } from "vitest/config";
import { svelte, vitePreprocess } from "@sveltejs/vite-plugin-svelte";
import { svelteTesting } from "@testing-library/svelte/vite";

export default defineConfig({
  plugins: [
    // Skip svelte.config.js (its vitePreprocess style step hits Vite 6's
    // preprocessCSS path, which is incompatible with the vitest transform) and
    // preprocess inline with style disabled — our <style> blocks are plain CSS.
    // TS script preprocessing stays on so `<script lang="ts">` still works.
    svelte({ configFile: false, preprocess: vitePreprocess({ style: false }) }),
    svelteTesting(),
  ],
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.ts"],
  },
});
