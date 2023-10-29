// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//@ts-check

import { copyFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { build, context } from "esbuild";

const thisDir = dirname(fileURLToPath(import.meta.url));
const libsDir = join(thisDir, "..", "node_modules");
const isWatch = process.argv.includes("--watch");

/** @type {import("esbuild").BuildOptions} */
const buildOptions = {
  entryPoints: [
    join(thisDir, "src", "extension.ts"),
    join(thisDir, "src", "compilerWorker.ts"),
    join(thisDir, "src", "debugger/debug-service-worker.ts"),
    join(thisDir, "src", "webview/webview.tsx"),
  ],
  outdir: join(thisDir, "out"),
  bundle: true,
  mainFields: ["browser", "module", "main"],
  external: ["vscode"],
  format: "cjs",
  platform: "browser",
  target: ["es2020"],
  sourcemap: "linked",
  //logLevel: "debug",
  define: { "import.meta.url": "undefined" },
};

function copyWasm() {
  // Copy the wasm module into the extension directory
  let qsharpWasm = join(thisDir, "..", "npm", "lib", "web", "qsc_wasm_bg.wasm");
  let qsharpDest = join(thisDir, `wasm`);

  console.log("Copying the qsharp wasm file over from: " + qsharpWasm);
  mkdirSync(qsharpDest, { recursive: true });
  copyFileSync(qsharpWasm, join(qsharpDest, "qsc_wasm_bg.wasm"));
}

function copyMathJax() {
  let mathjaxBase = join(libsDir, `mathjax/es5`);
  let mathjaxDest = join(thisDir, `out/mathjax`);

  // To minimize the size of the extension, we only copy over the files we need
  const toCopy = [
    "tex-chtml.js",
    "input/tex/extensions/physics.js",
    "output/chtml/fonts/woff-v2/MathJax_Main-Bold.woff",
    "output/chtml/fonts/woff-v2/MathJax_Main-Italic.woff",
    "output/chtml/fonts/woff-v2/MathJax_Main-Regular.woff",
    "output/chtml/fonts/woff-v2/MathJax_Math-BoldItalic.woff",
    "output/chtml/fonts/woff-v2/MathJax_Math-Italic.woff",
    "output/chtml/fonts/woff-v2/MathJax_Math-Regular.woff",
    "output/chtml/fonts/woff-v2/MathJax_Size1-Regular.woff",
    "output/chtml/fonts/woff-v2/MathJax_Size2-Regular.woff",
    "output/chtml/fonts/woff-v2/MathJax_Size3-Regular.woff",
    "output/chtml/fonts/woff-v2/MathJax_Size4-Regular.woff",
    "output/chtml/fonts/woff-v2/MathJax_Zero.woff",
  ];

  console.log("Copying the Mathjax files over from: " + mathjaxBase);
  mkdirSync(mathjaxDest, { recursive: true });
  toCopy.forEach((file) => {
    const src = join(mathjaxBase, file);
    const dest = join(mathjaxDest, file);

    mkdirSync(dirname(dest), { recursive: true });
    copyFileSync(src, dest);
  });
}

function buildBundle() {
  console.log("Running esbuild");

  build(buildOptions).then(() =>
    console.log(`Built bundle to ${join(thisDir, "out")}`)
  );
}

async function buildWatch() {
  console.log("Building vscode extension in watch mode");

  // Plugin to log start/end of build events (mostly to help VS Code problem matcher)
  /** @type {import("esbuild").Plugin} */
  const buildPlugin = {
    name: "Build Events",
    setup(build) {
      build.onStart(() => console.log("esbuild build started"));
      build.onEnd(() => console.log("esbuild build complete"));
    },
  };
  let ctx = await context({
    ...buildOptions,
    plugins: [buildPlugin],
    color: false,
  });

  ctx.watch();
}

if (isWatch) {
  buildWatch();
} else {
  copyWasm();
  copyMathJax();
  buildBundle();
}
