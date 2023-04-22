// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// Support running the compiler inside a browser WebWorker

import * as wasm from "../lib/web/qsc_wasm.js";
import { Compiler, CompilerEvents } from "./compiler.js";
import { getWorkerEventHandlers, handleMessageInWorker } from "./worker-common.js";

let eventHandlers: CompilerEvents = getWorkerEventHandlers(self.postMessage);

let compiler: Compiler | null = null;

// This export should be assigned to 'self.onmessage' in a WebWorker
export function messageHandler(e: MessageEvent) {
    const data = e.data;

    if (!data.type || typeof data.type !== "string") {
        console.error(`Unrecognized msg: ${data}`);
        return;
    }

    switch (data.type) {
        case "init":
            wasm.initSync(data.wasmModule);
            compiler = new Compiler(wasm, eventHandlers);
            break;
        default:
            if (!compiler) {
                console.error(`Received message ${data} before the compiler was initialized`);
            } else {
                handleMessageInWorker(data, compiler, self.postMessage);
            }
    }
}
