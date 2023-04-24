// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { isMainThread, parentPort, workerData } from "node:worker_threads";

import * as wasm from "../lib/node/qsc_wasm.cjs";
import { log } from "./log.js";
import { Compiler, CompilerEvents } from "./compiler.js";
import { getWorkerEventHandlers, handleMessageInWorker } from "./worker-common.js";

if (isMainThread) throw "Worker script should be loaded in a Worker thread only";
if (workerData && typeof workerData.qscLogLevel === 'number') {
    log.setLogLevel(workerData.qscLogLevel);
}

const port = parentPort!;
const postMessage = port.postMessage.bind(port);

const events: CompilerEvents = getWorkerEventHandlers(postMessage);
const compiler = new Compiler(wasm, events);

port.on("message", data => {
    if (!data.type || typeof data.type !== "string") {
        log.error(`Unrecognized msg: %O"`, data);
        return;
    }

    handleMessageInWorker(data, compiler, postMessage);
});
