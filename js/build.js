import { defineConfig } from "tsup";

import { version } from "./package.json";

export default defineConfig([
  {
    clean: true,
    entry: { stork: "src/index.ts" },
    format: ["cjs", "esm", "iife"],
    minify: true,
    globalName: "stork",
    dts: {
      entry: { stork: "src/index.ts" },
      compilerOptions: { moduleResolution: "node" }
    },
    sourcemap: true,
    define: { __VERSION: `"${version}"` },
    outDir: "dist",
    outExtension({ format }) {
      if (format === "iife") return { js: ".js" };
      return {
        js: `.${format}.js`
      };
    },
    external: ["stork-search"]
  },
  {
    clean: true,
    entry: ["src/stork.css"],
    minify: true,
    outDir: "dist"
  }
]);
