// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import initWasm, * as wasm from "../lib/web/qsc_wasm.js";
import { log } from "./log.js";
import { Compiler, ICompiler, ICompilerWorker, CompilerEvents } from "./compiler.js";
import { createWorkerProxy } from "./worker-common.js";

// Instantiate once. A module is stateless and can be efficiently passed to WebWorkers.
let wasmModule: WebAssembly.Module | null = null;

// Used to track if already instantiated
let wasmInstance: any;

export async function loadWasmModule(uri: string) {
    const wasmRequst = await fetch(uri);
    const wasmBuffer = await wasmRequst.arrayBuffer();
    wasmModule = await WebAssembly.compile(wasmBuffer);
}

export async function getCompiler(callbacks: CompilerEvents): Promise<ICompiler> {
    if (!wasmModule) throw "Wasm module must be loaded first";
    if (!wasmInstance) wasmInstance = await initWasm(wasmModule);

    return new Compiler(wasm, callbacks);
}

// Create the compiler inside a WebWorker and proxy requests
export function getCompilerWorker(script: string, callbacks: CompilerEvents): ICompilerWorker {
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

    return createWorkerProxy(callbacks, postMessage, setMsgHandler, onTerminate);
}

export { renderDump, exampleDump } from "./state-table.js"
export { getResultsHandler, type Dump, type ShotResult, type VSDiagnostic } from "./common.js";
export { getAllKatas, getKata, type Kata, type KataItem, type Exercise } from "./katas.js";
