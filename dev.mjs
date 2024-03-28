// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/*
Watch mode for a fast developer inner loop. Usage: "node dev.mjs"

WARNING: This is largely heuristics to optimize common developer workflows.
Always use ./build.py to ensure that all projects are built correctly before check-in.
Also run ./build.py to do any initial repo setup (npm install, copying 3rd party libs, etc.)

Usage:

- Save any code changes to Rust or TypeScript
- Rebuilds for updates should be auto-detected and take just a couple of seconds
- Simply refresh the browser to see playground changes
- Choose 'Developer: Reload Window' from the command palette for any running VS Code dev instances

Notes:

- This builds the wasm module, npm package, VS Code extension, and runs the playground.
- It does NOT build Python packages or native binaries (currently).
- It does NOT watch for docs, katas, or samples changes (currently).
- It does NOT build the Node.js wasm package (or run any of the node unit tests).
- It builds debug binaries (whereas ./build.py builds for release).

Future updates:

- Add a '--release' switch to build optimized binaries
- Watch for Katas changes and regenerate (maybe also docs & samples)
*/

// @ts-check

import { subscribe } from "@parcel/watcher";
import { spawnSync } from "node:child_process";
import { copyFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { copyWasmToVsCode, watchVsCode } from "./vscode/build.mjs";
import { buildPlayground, copyWasmToPlayground } from "./playground/build.js";

const thisDir = dirname(fileURLToPath(import.meta.url));

// Watch the source directories directly to avoid notification noise from .git, __pycache__, node_modules, target, etc.
const coreDir = join(thisDir, "compiler");
const libsDir = join(thisDir, "library");
const vslsDir = join(thisDir, "language_service");
const wasmDir = join(thisDir, "wasm");
const npmDir = join(thisDir, "npm");

const bldType = "debug";

async function onRustChange() {
  console.log("Compiling the .wasm module via wasm-pack");

  // This takes ~3-4 seconds on rebuild after some Rust changes. (Non-dev builds take ~15-20 seconds)
  // cd wasm && wasm-pack build --dev --no-pack --target web --out-dir "../target/wasm32/debug/web"
  const buildDir = join(thisDir, "target", "wasm32", bldType, "web");
  // TODO: Run wasm-pack (and stop if failed)
  const result = spawnSync(
    "wasm-pack",
    ["--dev", "--no-pack", "--target", "web", "--out-dir", buildDir],
    { cwd: wasmDir },
  );
  console.log("wasm-pack result: ", result.status);

  console.log("Copying the wasm-pack ouput files to the npm package");
  const npmLibDir = join(npmDir, "lib", "web");
  console.log("Copying the qsharp wasm files to npm from: " + buildDir);
  ["qsc_wasm_bg.wasm", "qsc_wasm.d.ts", "qsc_wasm.js"].forEach((file) =>
    copyFileSync(join(buildDir, file), join(npmLibDir, file)),
  );

  // This copies from the npm dir to VS Code and playground
  // Changing the wasm module in these doesn't require any rebuild, just a refresh.
  copyWasmToVsCode();
  copyWasmToPlayground();
}

subscribe(coreDir, onRustChange);
subscribe(libsDir, onRustChange);
subscribe(vslsDir, onRustChange);
subscribe(wasmDir, onRustChange);

// TODO: Kick off: cd npm; npm run tsc:watch

// Kick off VS Code watch mode (this will detect changes in the npm package it depends on)
watchVsCode();

// Start the playground server (will also detect changes in the npm packages it needs)
buildPlayground(true);
