// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// This module is the main entry point for use in Node.js environments. For browser environments,
// the "./browser.js" file is the entry point module.

import { createRequire } from "node:module";
import { Worker } from "node:worker_threads";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { log } from "./log.js";
import { Compiler, ICompiler, ICompilerWorker } from "./compiler/compiler.js";
import {
  ResponseMsgType,
  createCompilerProxy,
} from "./compiler/worker-common.js";

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
  const worker = new Worker(join(thisDir, "./compiler/worker-node.js"), {
    workerData: { qscLogLevel: log.getLogLevel() },
  });

  // If you lose the 'this' binding, some environments have issues.
  const postMessage = worker.postMessage.bind(worker);
  const setMsgHandler = (handler: (e: ResponseMsgType) => void) =>
    worker.addListener("message", handler);
  const onTerminate = () => worker.terminate();

  return createCompilerProxy(postMessage, setMsgHandler, onTerminate);
}
