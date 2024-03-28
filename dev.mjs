// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/*
Watch mode for a fast developer inner loop. Usage: "node dev.mjs"

WARNING: This is largely heuristics to optimize common developer workflows.
Always use ./build.py to ensure that all projects are built correctly before check-in.
Also run ./build.py to do any initial repo setup (npm install, copying 3rd party libs, etc.)

Once running, any changes to the source code for Rust directories listed, or for
the npm, vscode, or playground projects, should automatically recompile. Just
reload the playground page or reload the VS Code window to see the changes.

Notes:

- This builds the wasm module, npm package, VS Code extension, and runs the playground.
- It does NOT build Python packages or native binaries (currently).
- It does NOT watch for docs, katas, or samples changes (currently).
- It does NOT build the Node.js wasm package (or run any of the node unit tests).
- It builds debug binaries (whereas ./build.py builds for release).
- Future updates could include watching for katas changes, and supporting '--release'

*/

// @ts-check

import { subscribe } from "@parcel/watcher";
import { spawn, spawnSync } from "node:child_process";
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

function onRustChange() {
  console.log("Compiling the .wasm module with wasm-pack");

  // This takes ~3-4 seconds on rebuild after some Rust changes. (Non-dev builds take ~15-20 seconds)
  // Build only web and not node targets to half time.
  const buildDir = join(thisDir, "target", "wasm32", "debug", "web");
  const result = spawnSync(
    "wasm-pack",
    ["build", "--dev", "--no-pack", "--target", "web", "--out-dir", buildDir],
    { cwd: wasmDir },
  );
  console.log("wasm-pack done! ", result.stderr.toString());

  console.log("Copying the wasm-pack ouput files to the npm package");
  const npmLibDir = join(npmDir, "lib", "web");

  ["qsc_wasm_bg.wasm", "qsc_wasm.d.ts", "qsc_wasm.js"].forEach((file) =>
    copyFileSync(join(buildDir, file), join(npmLibDir, file)),
  );

  // The below copies the .wasm file from the npm dir to VS Code and playground projects
  // They already watch the .d.ts file from the npm package, so will rebuild if it changes.
  copyWasmToVsCode();
  copyWasmToPlayground();
}

// Do an initial build
onRustChange();

// Then watch the Rust directories for code changes
[coreDir, libsDir, vslsDir, wasmDir].forEach((dir) =>
  subscribe(dir, onRustChange),
);

// Build/watch the npm project
const npmWatcher = spawn("npm", ["run", "tsc:watch"], { cwd: npmDir });
npmWatcher.stdout.on("data", (data) => console.log(`${data}`));
npmWatcher.stderr.on("data", (data) => console.error(`npm error: ${data}`));
npmWatcher.on("close", (code) =>
  console.log(`npm watcher exited with: `, code),
);

// Kick off VS Code watch mode (this will detect changes in the npm package it depends on)
watchVsCode();

// Start the playground server (will also detect changes in the npm packages it needs)
buildPlayground(true);
