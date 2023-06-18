// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// This module is the main entry point for use in Node.js environments. For browser environments,
// the "./browser.js" file is the entry point module.

import * as Comlink from "comlink";
import nodeEndpoint from "comlink/dist/umd/node-adapter.js";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { Worker } from "node:worker_threads";
import { Compiler, ICompiler, ICompilerWorker } from "./compiler.js";
import { log } from "./log.js";
import { CompilerProxy, eventTransferHandler } from "./worker-common.js";
import { INodeCompilerWorker } from "./worker-node.js";

// Only load the Wasm module when first needed, as it may only be used in a Worker,
// and not in the main thread.
type Wasm = typeof import("../lib/node/qsc_wasm.cjs");
let wasm: Wasm | null = null;
const require = createRequire(import.meta.url);

export function getCompiler(): ICompiler {
  if (!wasm) wasm = require("../lib/node/qsc_wasm.cjs") as Wasm;
  return new Compiler(wasm);
}

export function getCompilerWorker(): ICompilerWorker {
  const thisDir = dirname(fileURLToPath(import.meta.url));
  const nodeWorker = new Worker(join(thisDir, "worker-node.js"), {
    workerData: { qscLogLevel: log.getLogLevel() },
  });
  // @ts-expect-error TypeScript really doesn't believe that the default
  // export from node-adapter is callable. But it is.
  const worker = nodeEndpoint(nodeWorker);

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  Comlink.transferHandlers.set("EVENT", eventTransferHandler as any);
  const compilerProxy: Comlink.Remote<INodeCompilerWorker> =
    Comlink.wrap(worker);

  return new CompilerProxy(compilerProxy, nodeWorker);
}
