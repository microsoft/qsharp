// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//@ts-check

import { copyFileSync, mkdirSync, cpSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { build, context } from "esbuild";

const thisDir = dirname(fileURLToPath(import.meta.url));
const libsDir = join(thisDir, "..", "node_modules");

// Use minified libraries
const isRelease = process.argv.includes("--release");
const outdir = join(thisDir, "public/libs");

/** @type {import("esbuild").BuildOptions} */
const buildOptions = {
  entryPoints: [
    join(thisDir, "src/main.tsx"),
    join(thisDir, "src/compiler-worker.ts"),
    join(thisDir, "src/language-service-worker.ts"),
  ],
  outdir,
  bundle: true,
  target: ["es2020", "chrome64", "edge79", "firefox62", "safari11.1"],
  define: { "import.meta.url": "document.URL" },
  sourcemap: "linked",
  minify: isRelease ? true : false,
};

// Copy the relevant external libraries from node_modules into the static site files
function copyLibs() {
  let monacoBase = join(
    libsDir,
    `monaco-editor/${isRelease ? "min" : "dev"}/vs`,
  );
  let monacoDest = join(thisDir, `public/libs/monaco/vs`);

  console.log("Copying the Monaco files over from: " + monacoBase);
  mkdirSync(monacoDest, { recursive: true });
  cpSync(monacoBase, monacoDest, { recursive: true });

  let mathjaxBase = join(libsDir, `mathjax/es5`);
  let mathjaxDest = join(thisDir, `public/libs/mathjax`);

  console.log("Copying the Mathjax files over from: " + mathjaxBase);
  mkdirSync(mathjaxDest, { recursive: true });
  cpSync(mathjaxBase, mathjaxDest, { recursive: true });

  let githubMarkdown = join(
    libsDir,
    "github-markdown-css/github-markdown-light.css",
  );
  let githubMarkdownDest = join(thisDir, "public/libs/github-markdown.css");
  copyFileSync(githubMarkdown, githubMarkdownDest);

  let qsharpWasm = join(thisDir, "..", "npm/lib/web/qsc_wasm_bg.wasm");
  let qsharpDest = join(thisDir, `public/libs/qsharp`);

  console.log("Copying the qsharp wasm file over from: " + qsharpWasm);
  mkdirSync(qsharpDest, { recursive: true });
  copyFileSync(qsharpWasm, join(qsharpDest, "qsc_wasm_bg.wasm"));
}

function buildBundle() {
  console.log("Running esbuild");

  build(buildOptions).then(() => console.log(`Built bundles to ${outdir}`));
}

// Serve the site or build it?
if (process.argv.includes("--serve")) {
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
  const servedir = join(thisDir, "public");

  // See https://esbuild.github.io/api/#serve
  console.log("Starting the playground on http://localhost:5555");
  await ctx.serve({
    port: 5555,
    servedir,
  });
} else {
  copyLibs();
  buildBundle();
}
