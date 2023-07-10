// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//@ts-check

import { copyFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { build, context } from "esbuild";

const thisDir = dirname(fileURLToPath(import.meta.url));
const isWatch = process.argv.includes("--watch");

/** @type {import("esbuild").BuildOptions} */
const buildOptions = {
  entryPoints: [
    join(thisDir, "src", "extension.ts"),
    join(thisDir, "src", "simulatorWorker.ts"),
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

function buildBundle() {
  console.log("Running esbuild");

  build(buildOptions).then(() =>
    console.log(`Built bundle to ${join(thisDir, "out")}`)
  );
}

async function buildWatch() {
  console.log("Building vscode extension in watch mode");
  let ctx = await context(buildOptions);
  ctx.watch();
}

if (isWatch) {
  buildWatch();
} else {
  copyWasm();
  buildBundle();
}
