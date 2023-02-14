import { version } from "./package.json";
import { defineConfig } from "tsup";

export default defineConfig([
  {
    clean: true,
    entry: { stork: "js/index.ts" },
    format: ["cjs", "esm", "iife"],
    minify: true,
    globalName: "stork",
    dts: {
      entry: { stork: "js/index.ts" },
      compilerOptions: { moduleResolution: "node" },
    },
    sourcemap: true,
    define: { __VERSION: `"${version}"` },
    outDir: "js/dist",
    outExtension({ format }) {
      if (format === "iife") return { js: ".js" };
      return {
        js: `.${format}.js`,
      };
    },
    external: ["stork-search"],
  },
  {
    clean: true,
    entry: ["js/stork.css"],
    minify: true,
    outDir: "js/dist",
  },
]);
