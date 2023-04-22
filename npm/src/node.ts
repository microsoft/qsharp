// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { createRequire } from "node:module";
import { Worker } from "node:worker_threads";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { Compiler, CompilerEvents, ICompiler, ICompilerWorker } from "./compiler.js";
import { createWorkerProxy } from "./worker-common.js";

// Only load the Wasm module when first needed, as it may only be used in a Worker,
// and not in the main thread.
type Wasm = typeof import("../lib/node/qsc_wasm.cjs");
let wasm: Wasm | null = null;
const require = createRequire(import.meta.url);

export function getCompiler(callbacks: CompilerEvents) : ICompiler {
    if (!wasm) wasm = require("../lib/node/qsc_wasm.cjs") as Wasm;
    return new Compiler(wasm, callbacks);
}

export function getCompilerWorker(callbacks: CompilerEvents) : ICompilerWorker {
    const thisDir = dirname(fileURLToPath(import.meta.url));
    const worker = new Worker(join(thisDir,"node-worker.js"));

    const postMessage = (val: any) => worker.postMessage(val);
    const setMsgHandler = (handler: (e: any) => void) => worker.on("message", handler);
    const onTerminate = () => worker.terminate();

    return createWorkerProxy(callbacks, postMessage, setMsgHandler, onTerminate);
}
