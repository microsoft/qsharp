// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { isMainThread, parentPort } from "node:worker_threads";

import * as wasm from "../lib/node/qsc_wasm.cjs";
import { Compiler, CompilerEvents } from "./compiler.js";
import { getWorkerEventHandlers, handleMessageInWorker } from "./worker-common.js";

if (isMainThread) throw "Worker script should be loaded in a Worker thread only";

const port = parentPort!;
const postMessage = (val: any) => port.postMessage(val);
let events: CompilerEvents = getWorkerEventHandlers(postMessage);
let compiler = new Compiler(wasm, events);

port.on("message", data => {
    if (!data.type || typeof data.type !== "string") {
        console.error(`Unrecognized msg: ${data}`);
        return;
    }

    handleMessageInWorker(data, compiler, postMessage);
});
