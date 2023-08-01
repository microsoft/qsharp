// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// This module is the entry point for browser environments. For Node.js environment,
// the "./main.js" module is the entry point.

import initWasm, * as wasm from "../lib/web/qsc_wasm.js";
import { Compiler, ICompiler, ICompilerWorker } from "./compiler/compiler.js";
import { createCompilerProxy } from "./compiler/worker-proxy.js";
import {
  ILanguageService,
  ILanguageServiceWorker,
  QSharpLanguageService,
} from "./language-service/language-service.js";
import { createLanguageServiceProxy } from "./language-service/worker-proxy.js";
import { LogLevel, log } from "./log.js";

// Create once. A module is stateless and can be efficiently passed to WebWorkers.
let wasmModule: WebAssembly.Module | null = null;
let wasmModulePromise: Promise<void> | null = null;

// Used to track if an instance is already instantiated
let wasmInstancePromise: Promise<wasm.InitOutput> | null = null;

async function wasmLoader(uriOrBuffer: string | ArrayBuffer) {
  if (typeof uriOrBuffer === "string") {
    log.info("Fetching wasm module from %s", uriOrBuffer);
    performance.mark("fetch-wasm-start");
    const wasmRequst = await fetch(uriOrBuffer);
    const wasmBuffer = await wasmRequst.arrayBuffer();
    const fetchTiming = performance.measure("fetch-wasm", "fetch-wasm-start");
    log.logTelemetry({
      id: "fetch-wasm",
      data: {
        duration: fetchTiming.duration,
        uri: uriOrBuffer,
      },
    });

    wasmModule = await WebAssembly.compile(wasmBuffer);
  } else {
    log.info("Compiling wasm module from provided buffer");
    wasmModule = await WebAssembly.compile(uriOrBuffer);
  }
}

export function loadWasmModule(
  uriOrBuffer: string | ArrayBuffer
): Promise<void> {
  // Only initiate if not already in flight, to avoid race conditions
  if (!wasmModulePromise) {
    wasmModulePromise = wasmLoader(uriOrBuffer);
  }
  return wasmModulePromise;
}

async function instantiateWasm() {
  // Ensure loading the module has been initiated, and wait for it.
  if (!wasmModulePromise) throw "Wasm module must be loaded first";
  await wasmModulePromise;
  if (!wasmModule) throw "Wasm module failed to load";

  if (wasmInstancePromise) {
    // Either in flight or already complete. The prior request will do the init,
    // so just wait on that.
    await wasmInstancePromise;
    return;
  }

  // Set the promise to signal this is in flight, then wait on the result.
  wasmInstancePromise = initWasm(wasmModule);
  await wasmInstancePromise;

  // Once ready, set up logging and telemetry as soon as possible after instantiating
  wasm.initLogging(log.logWithLevel, log.getLogLevel());
  log.onLevelChanged = (level) => wasm.setLogLevel(level);
}

export async function getCompiler(): Promise<ICompiler> {
  await instantiateWasm();
  return new Compiler(wasm);
}

// Create the compiler inside a WebWorker and proxy requests.
// If the Worker was already created via other means and is ready to receive
// messages, then the worker may be passed in and it will be initialized.
export function getCompilerWorker(workerArg: string | Worker): ICompilerWorker {
  if (!wasmModule) throw "Wasm module must be loaded first";

  // Create or use the WebWorker
  const worker =
    typeof workerArg === "string" ? new Worker(workerArg) : workerArg;

  // Send it the Wasm module to instantiate
  worker.postMessage({
    type: "init",
    wasmModule,
    qscLogLevel: log.getLogLevel(),
  });

  // If you lose the 'this' binding, some environments have issues
  const postMessage = worker.postMessage.bind(worker);
  const onTerminate = () => worker.terminate();

  // Create the proxy which will forward method calls to the worker
  const proxy = createCompilerProxy(postMessage, onTerminate);

  // Let proxy handle response and event messages from the worker
  worker.onmessage = (ev) => proxy.onMsgFromWorker(ev.data);
  return proxy;
}

export async function getLanguageService(): Promise<ILanguageService> {
  if (!wasmModule) throw "Wasm module must be loaded first";
  if (!wasmInstancePromise) wasmInstancePromise = initWasm(wasmModule);
  await wasmInstancePromise;

  return new QSharpLanguageService(wasm);
}

// Create the compiler inside a WebWorker and proxy requests.
// If the Worker was already created via other means and is ready to receive
// messages, then the worker may be passed in and it will be initialized.
export function getLanguageServiceWorker(
  workerArg: string | Worker
): ILanguageServiceWorker {
  if (!wasmModule) throw "Wasm module must be loaded first";

  // Create or use the WebWorker
  const worker =
    typeof workerArg === "string" ? new Worker(workerArg) : workerArg;

  // Send it the Wasm module to instantiate
  worker.postMessage({
    type: "init",
    wasmModule,
    qscLogLevel: log.getLogLevel(),
  });

  // If you lose the 'this' binding, some environments have issues
  const postMessage = worker.postMessage.bind(worker);
  const onTerminate = () => worker.terminate();

  // Create the proxy which will forward method calls to the worker
  const proxy = createLanguageServiceProxy(postMessage, onTerminate);

  // Let proxy handle response and event messages from the worker
  worker.onmessage = (ev) => proxy.onMsgFromWorker(ev.data);
  return proxy;
}

export { type Dump, type ShotResult } from "./compiler/common.js";
export { type CompilerState } from "./compiler/compiler.js";
export { QscEventTarget } from "./compiler/events.js";
export {
  getAllKatas,
  getExerciseSources,
  getKata,
  type ContentItem,
  type Example,
  type Exercise,
  type ExplainedSolution,
  type ExplainedSolutionItem,
  type Kata,
  type KataSection,
  type Lesson,
  type LessonItem,
  type Question,
} from "./katas.js";
export { default as samples } from "./samples.generated.js";
export { type VSDiagnostic } from "./vsdiagnostic.js";
export { log, type LogLevel };
export type { ICompilerWorker, ICompiler };
export type { ILanguageServiceWorker, ILanguageService };
export { type LanguageServiceEvent } from "./language-service/language-service.js";
