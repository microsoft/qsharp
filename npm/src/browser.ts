// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// This module is the entry point for browser environments. For Node.js environment,
// the "./main.js" module is the entry point.

import initWasm, * as wasm from "../lib/web/qsc_wasm.js";
import { log } from "./log.js";
import { Compiler, ICompiler, ICompilerWorker } from "./compiler.js";
import { createWorkerProxy } from "./worker-common.js";

// Create once. A module is stateless and can be efficiently passed to WebWorkers.
let wasmModule: WebAssembly.Module | null = null;

// Used to track if an instance is already instantiated
let wasmInstance: any;

export async function loadWasmModule(uri: string) {
    const wasmRequst = await fetch(uri);
    const wasmBuffer = await wasmRequst.arrayBuffer();
    wasmModule = await WebAssembly.compile(wasmBuffer);
}

export async function getCompiler(): Promise<ICompiler> {
    if (!wasmModule) throw "Wasm module must be loaded first";
    if (!wasmInstance) wasmInstance = await initWasm(wasmModule);

    return new Compiler(wasm);
}

// Create the compiler inside a WebWorker and proxy requests
export function getCompilerWorker(script: string): ICompilerWorker {
    if (!wasmModule) throw "Wasm module must be loaded first";

    // Create a WebWorker
    const worker = new Worker(script);

    // Send it the Wasm module to instantiate
    worker.postMessage({ "type": "init", wasmModule, qscLogLevel: log.getLogLevel() });

    // If you lose the 'this' binding, some environments have issues
    const postMessage = worker.postMessage.bind(worker);
    const setMsgHandler = (handler: (e: any) => void) =>
        worker.onmessage = (ev) => handler(ev.data);
    const onTerminate = () => worker.terminate();

    return createWorkerProxy(postMessage, setMsgHandler, onTerminate);
}

export type { ICompilerWorker }
export { log }
export { renderDump, exampleDump } from "./state-table.js"
export { type Dump, type ShotResult, type VSDiagnostic } from "./common.js";
export { getAllKatas, getKata, type Kata, type KataItem, type Example, type Exercise } from "./katas.js";
export { QscEventTarget } from "./events.js";
