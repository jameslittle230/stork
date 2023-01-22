const { version } = require("./package.json");
const fs = require("fs");

const outfile = "js/dist/stork.js";

const esbuild = require("esbuild");

esbuild
  .build({
    entryPoints: ["js/index.ts"],
    bundle: true,
    minify: true,
    globalName: "stork",
    target: "ES2015",
    outfile,
    define: { __VERSION: `"${version}"`, import: null },
    logOverride: {
      "empty-import-meta": "debug",
    },
  })
  .then(() => {
    const { size } = fs.statSync(outfile);
    console.log(`${outfile}: ${size} bytes`);
  })
  .catch(() => process.exit(1));

esbuild.build({
  entryPoints: ["js/stork.css"],
  minify: true,
  outfile: "js/dist/stork.css",
});
