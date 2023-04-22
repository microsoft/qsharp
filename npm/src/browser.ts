// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as wasm from "../lib/web/qsc_wasm.js";
import { Compiler, ICompiler, ICompilerWorker, CompilerEvents } from "./compiler.js";
import { createWorkerProxy } from "./worker-common.js";

// Instantiate once. A module is stateless and can be efficiently passed to WebWorkers.
let wasmModule: WebAssembly.Module | null = null;

// Used to track if already instantiated
let wasmInstance: any;

export async function loadWasmModule(uri: string) {
    const wasmRequst = await fetch(uri);
    const wasmBuffer = await wasmRequst.arrayBuffer();
    wasmModule = new WebAssembly.Module(wasmBuffer);
}

export function getCompiler(callbacks: CompilerEvents): ICompiler {
    if (!wasmModule) throw "Wasm module must be loaded first";
    if (!wasmInstance) wasmInstance = wasm.initSync(wasmModule);

    return new Compiler(wasm, callbacks);
}

// Create the compiler inside a WebWorker and proxy requests
export function getCompilerWorker(script: string, callbacks: CompilerEvents): ICompilerWorker {
    if (!wasmModule) throw "Wasm module must be loaded first";

    // Create a WebWorker
    const worker = new Worker(script);

    // Send it the Wasm module to instantiate
    worker.postMessage({ "type": "init", wasmModule });

    const postMessage = (val: any) => worker.postMessage(val);
    const setMsgHandler = (handler: (e: any) => void) => 
            worker.onmessage = (ev) => handler(ev.data);
    const onTerminate = () => worker.terminate();

    return createWorkerProxy(callbacks, postMessage, setMsgHandler, onTerminate);
}

export { renderDump, exampleDump } from "./state-table.js"
export { type Dump, type ShotResult, type VSDiagnostic } from "./common.js";
export { getAllKatas, getKata, type Kata, type KataItem, type Exercise } from "./katas.js";
