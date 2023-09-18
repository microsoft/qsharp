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

/** @type {import("esbuild").BuildOptions} */
const extensionOptions = {
  ...buildOptions,
  entryPoints: [
    join(thisDir, "src", "extension.ts"),
    join(thisDir, "src", "compilerWorker.ts"),
    join(thisDir, "src", "debugger", "debug-service-worker.ts"),
  ],
  outdir: join(thisDir, "out"),
};

/** @type {import("esbuild").BuildOptions} */
const testOptions = {
  ...buildOptions,
  entryPoints: [join(thisDir, "test", "suite", "index.ts")],
  outdir: join(thisDir, "test", "out"),
};

function copyWasm() {
  // Copy the wasm module into the extension directory
  let qsharpWasm = join(thisDir, "..", "npm", "lib", "web", "qsc_wasm_bg.wasm");
  let qsharpDest = join(thisDir, `wasm`);

  console.log("Copying the qsharp wasm file over from: " + qsharpWasm);
  mkdirSync(qsharpDest, { recursive: true });
  copyFileSync(qsharpWasm, join(qsharpDest, "qsc_wasm_bg.wasm"));
}

function buildProduct() {
  console.log("Running esbuild");

  build(extensionOptions).then(() =>
    console.log(`Built product to ${extensionOptions.outdir}`)
  );
}

function buildTests() {
  console.log("Running esbuild");

  build(testOptions).then(() =>
    console.log(`Built tests to ${testOptions.outdir}`)
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
    ...extensionOptions,
    plugins: [buildPlugin],
    color: false,
  });

  ctx.watch();
}

if (isWatch) {
  buildWatch();
} else {
  copyWasm();
  buildProduct();
  buildTests();
}
