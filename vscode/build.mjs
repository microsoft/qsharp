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
  entryPoints: [join(thisDir, "src", "extension.ts")],
  outfile: join(thisDir, "out", "extension.js"),
  bundle: true,
  external: ["vscode"],
  format: "cjs",
  platform: "browser",
  target: ["es2020"],
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

copyWasm();
buildBundle();
