// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// This module is the entry point for browser environments. For Node.js environment,
// the "./main.js" module is the entry point.

import initWasm, * as qscWasm from "../lib/web/qsc_wasm.js";
import { LogLevel, log } from "./log.js";
import { Compiler, ICompiler, ICompilerWorker } from "./compiler.js";
import * as Comlink from "comlink";
import { CompilerProxy, eventTransferHandler } from "./worker-common.js";
import { IWebCompilerWorker } from "./worker-browser.js";

// Create once. A module is stateless and can be efficiently passed to WebWorkers.
let wasmModule: WebAssembly.Module | null = null;

// Used to track if an instance is already instantiated
let wasmInstance: qscWasm.InitOutput;

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
  if (!wasmInstance) wasmInstance = await initWasm(wasmModule);

  return new Compiler(qscWasm);
}

export function getCompilerWorker(workerArg: string | Worker): ICompilerWorker {
  if (!wasmModule) throw "Wasm module must be loaded first";

  // Create or use the WebWorker
  const worker =
    typeof workerArg === "string" ? new Worker(workerArg) : workerArg;

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  Comlink.transferHandlers.set("EVENT", eventTransferHandler as any);
  const compiler: Comlink.Remote<IWebCompilerWorker> = Comlink.wrap(worker);
  compiler.init(wasmModule, log.getLogLevel());
  return new CompilerProxy(compiler, worker);
}

export type { ICompilerWorker };
export { log, type LogLevel };
export { type Dump, type ShotResult, type VSDiagnostic } from "./common.js";
export { type CompilerState } from "./compiler.js";
export {
  getAllKatas,
  getKata,
  type Kata,
  type KataItem,
  type Example,
  type Exercise,
} from "./katas.js";
export { default as samples } from "./samples.generated.js";
export { QscEventTarget } from "./events.js";
