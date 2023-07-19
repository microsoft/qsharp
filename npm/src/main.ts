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
import { createCompilerProxy } from "./compiler/worker-proxy.js";
import {
  ILanguageService,
  ILanguageServiceWorker,
  QSharpLanguageService,
} from "./language-service/language-service.js";
import { createLanguageServiceProxy } from "./language-service/worker-proxy.js";

// Only load the Wasm module when first needed, as it may only be used in a Worker,
// and not in the main thread.
type Wasm = typeof import("../lib/node/qsc_wasm.cjs");
let wasm: Wasm | null = null;
const require = createRequire(import.meta.url);

export function getCompiler(): ICompiler {
  if (!wasm) {
    wasm = require("../lib/node/qsc_wasm.cjs") as Wasm;
    // Set up logging and telemetry as soon as possible after instantiating
    wasm.initLogging(log.logWithLevel, log.getLogLevel());
    log.onLevelChanged = (level) => wasm?.setLogLevel(level);
  }
  return new Compiler(wasm);
}

export function getCompilerWorker(): ICompilerWorker {
  const thisDir = dirname(fileURLToPath(import.meta.url));
  const worker = new Worker(join(thisDir, "./compiler/worker-node.js"), {
    workerData: { qscLogLevel: log.getLogLevel() },
  });

  // Create the proxy which will forward method calls to the worker
  const proxy = createCompilerProxy(
    // If you lose the 'this' binding, some environments have issues.
    worker.postMessage.bind(worker),
    () => worker.terminate()
  );

  // Let proxy handle response and event messages from the worker
  worker.addListener("message", proxy.onMsgFromWorker);

  return proxy;
}

export function getLanguageService(): ILanguageService {
  if (!wasm) wasm = require("../lib/node/qsc_wasm.cjs") as Wasm;
  return new QSharpLanguageService(wasm);
}

export function getLanguageServiceWorker(): ILanguageServiceWorker {
  const thisDir = dirname(fileURLToPath(import.meta.url));
  const worker = new Worker(
    join(thisDir, "./language-service/worker-node.js"),
    {
      workerData: { qscLogLevel: log.getLogLevel() },
    }
  );

  // Create the proxy which will forward method calls to the worker
  const proxy = createLanguageServiceProxy(
    // If you lose the 'this' binding, some environments have issues.
    worker.postMessage.bind(worker),
    () => worker.terminate()
  );

  // Let proxy handle response and event messages from the worker
  worker.addListener("message", proxy.onMsgFromWorker);

  return proxy;
}
