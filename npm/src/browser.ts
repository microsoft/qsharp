// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// This module is the entry point for browser environments. For Node.js environment,
// the "./main.js" module is the entry point.

import initWasm, * as wasm from "../lib/web/qsc_wasm.js";
import { LogLevel, log } from "./log.js";
import {
  Compiler,
  CompilerState,
  ICompiler,
  ICompilerWorker,
} from "./compiler.js";
import { ResponseMsgType, createWorkerProxy } from "./worker-common.js";
import * as Comlink from "comlink";
import { ICompletionList } from "../lib/node/qsc_wasm.cjs";
import { VSDiagnostic } from "./common.js";
import { IQscEventTarget, QscEvents } from "./events.js";

// Create once. A module is stateless and can be efficiently passed to WebWorkers.
let wasmModule: WebAssembly.Module | null = null;

// Used to track if an instance is already instantiated
let wasmInstance: wasm.InitOutput;

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
  const setMsgHandler = (handler: (e: ResponseMsgType) => void) =>
    (worker.onmessage = (ev) => handler(ev.data));
  const onTerminate = () => worker.terminate();

  return createWorkerProxy(postMessage, setMsgHandler, onTerminate);
}

type InitableCompiler = {
  init(w: WebAssembly.Module, qscLogLevel: number): void;
} & ICompiler;

Comlink.transferHandlers.set("EVENT", {
  canHandle: ((obj: unknown) => obj instanceof Event) as (
    obj: unknown
  ) => obj is Event,
  serialize: (ev: Event) => {
    return [
      {
        type: ev.type,
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        detail: (ev as any).detail,
      },
      [],
    ];
  },
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  deserialize: (obj: any) => {
    const ev = new Event(obj.type);
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (ev as any).detail = obj.detail;
    return ev;
  },
});

class ComlinkCompilerProxy implements ICompilerWorker {
  constructor(private compiler: ICompiler, private worker: Worker) {}
  checkCode(code: string): Promise<VSDiagnostic[]> {
    return this.compiler.checkCode(code);
  }
  getHir(code: string): Promise<string> {
    return this.compiler.getHir(code);
  }
  getCompletions(): Promise<ICompletionList> {
    return this.compiler.getCompletions();
  }
  run(
    code: string,
    expr: string,
    shots: number,
    eventHandler: IQscEventTarget
  ): Promise<void> {
    return this.compiler.run(code, expr, shots, Comlink.proxy(eventHandler));
  }
  runKata(
    user_code: string,
    verify_code: string,
    eventHandler: IQscEventTarget
  ): Promise<boolean> {
    return this.compiler.runKata(
      user_code,
      verify_code,
      Comlink.proxy(eventHandler)
    );
  }
  setStateHandler(
    onstatechange: (state: CompilerState) => void
  ): Promise<void> {
    return this.compiler.setStateHandler(Comlink.proxy(onstatechange));
  }
  terminate() {
    this.worker.terminate();
  }
}

export function getCompilerComlinkProxy(
  workerArg: string | Worker
): ICompilerWorker {
  if (!wasmModule) throw "Wasm module must be loaded first";

  // Create or use the WebWorker
  const worker =
    typeof workerArg === "string" ? new Worker(workerArg) : workerArg;
  const obj: Comlink.Remote<InitableCompiler> = Comlink.wrap(worker);
  obj.init(wasmModule, log.getLogLevel());

  return new ComlinkCompilerProxy(obj, worker);
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
