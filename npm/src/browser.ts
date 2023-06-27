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

// Used to track if an instance is already instantiated
let wasmPromise: Promise<wasm.InitOutput>;

export async function loadWasmModule(uriOrBuffer: string | ArrayBuffer) {
  if (typeof uriOrBuffer === "string") {
    const wasmRequst = await fetch(uriOrBuffer);
    const wasmBuffer = await wasmRequst.arrayBuffer();
    wasmModule = await WebAssembly.compile(wasmBuffer);
  } else {
    wasmModule = await WebAssembly.compile(uriOrBuffer);
  }
}

export async function getCompiler(): Promise<ICompiler> {
  if (!wasmModule) throw "Wasm module must be loaded first";
  if (!wasmPromise) wasmPromise = initWasm(wasmModule);
  await wasmPromise;

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
  if (!wasmPromise) wasmPromise = initWasm(wasmModule);
  await wasmPromise;

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
  getKata,
  type Example,
  type Exercise,
  type Kata,
  type KataItem,
} from "./katas.js";
export { default as samples } from "./samples.generated.js";
export { type VSDiagnostic } from "./vsdiagnostic.js";
export { log, type LogLevel };
export type { ICompilerWorker };
export type { ILanguageServiceWorker, ILanguageService };
export { type LanguageServiceEvent } from "./language-service/language-service.js";
