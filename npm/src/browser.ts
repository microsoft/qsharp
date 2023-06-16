// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// This module is the entry point for browser environments. For Node.js environment,
// the "./main.js" module is the entry point.

import initWasm, * as wasm from "../lib/web/qsc_wasm.js";
import { LogLevel, log } from "./log.js";
import { Compiler, ICompiler, ICompilerWorker } from "./compiler/compiler.js";
import {
  ResponseMsgType as CompilerResponseMsgType,
  createWorkerProxy as createCompilerWorkerProxy,
} from "./compiler/worker-common.js";
import {
  ResponseMsgType as LanguageServiceResponseMsgType,
  createWorkerProxy as createLanguageServiceWorkerProxy,
} from "./language-service/worker-common.js";
import { ILanguageServiceEventTarget } from "./language-service/events.js";
import {
  ILanguageService,
  QSharpLanguageService,
} from "./language-service/language-service.js";

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
  const setMsgHandler = (handler: (e: CompilerResponseMsgType) => void) =>
    (worker.onmessage = (ev) => handler(ev.data));
  const onTerminate = () => worker.terminate();

  return createCompilerWorkerProxy(postMessage, setMsgHandler, onTerminate);
}

export async function getLanguageService(
  eventTarget: ILanguageServiceEventTarget
): Promise<ILanguageService> {
  if (!wasmModule) throw "Wasm module must be loaded first";
  if (!wasmPromise) wasmPromise = initWasm(wasmModule);
  await wasmPromise;

  return new QSharpLanguageService(wasm, eventTarget);
}

// Create the language service inside a WebWorker and proxy requests.
// If the Worker was already created via other means and is ready to receive
// messages, then the worker may be passed in and it will be initialized.
export function getLanguageServiceWorker(
  workerArg: string | Worker,
  eventTarget: ILanguageServiceEventTarget
): ILanguageService {
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
  const setMsgHandler = (
    handler: (e: LanguageServiceResponseMsgType) => void
  ) => (worker.onmessage = (ev) => handler(ev.data));

  return createLanguageServiceWorkerProxy(
    postMessage,
    setMsgHandler,
    eventTarget
  );
}

export type { ICompilerWorker };
export { log, type LogLevel };
export { type Dump, type ShotResult } from "./compiler/common.js";
export { type VSDiagnostic } from "./vsdiagnostic.js";
export { type CompilerState } from "./compiler/compiler.js";
export {
  getAllKatas,
  getKata,
  type Kata,
  type KataItem,
  type Example,
  type Exercise,
} from "./katas.js";
export { default as samples } from "./samples.generated.js";
export { QscEventTarget } from "./compiler/events.js";
