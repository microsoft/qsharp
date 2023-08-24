// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//@ts-check

import { copyFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { build } from "esbuild";

const thisDir = dirname(fileURLToPath(import.meta.url));

/** @type {import("esbuild").BuildOptions} */
const buildOptions = {
  bundle: true,
  mainFields: ["browser", "module", "main"],
  external: ["vscode"],
  format: "cjs",
  platform: "browser",
  target: ["es2020"],
  sourcemap: "linked",
  define: { "import.meta.url": "undefined" },
};

/** @type {import("esbuild").BuildOptions} */
const extensionOptions = {
  entryPoints: [
    join(thisDir, "src", "extension.ts"),
    join(thisDir, "src", "compilerWorker.ts"),
    join(thisDir, "src", "debugger", "debug-service-worker.ts"),
  ],
  outdir: join(thisDir, "out"),
  ...buildOptions,
};

/** @type {import("esbuild").BuildOptions} */
const testOptions = {
  entryPoints: [join(thisDir, "test", "suite", "index.ts")],
  outdir: join(thisDir, "test", "out"),
  ...buildOptions,
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

  build(extensionOptions).then(() =>
    console.log(`Built bundle to ${extensionOptions.outdir}`)
  );
}

function buildTests() {
  console.log("Running esbuild");

  build(testOptions).then(() =>
    console.log(`Built bundle to ${testOptions.outdir}`)
  );
}

copyWasm();
buildBundle();
buildTests();
