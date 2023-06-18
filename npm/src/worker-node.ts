// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// This module supports running the qsharp compiler in a Node.js worker thread. It should be
// used in a Node.js module in a manner similar to the below to create it with the right log level:
//
//     const worker = new Worker(join(thisDir,"worker-node.js"), {
//         workerData: {qscLogLevel: log.getLogLevel() }
//     });

import * as Comlink from "comlink";
import { parentPort, workerData } from "node:worker_threads";
import * as wasm from "../lib/node/qsc_wasm.cjs";
import { Compiler } from "./compiler.js";
import { log } from "./log.js";
import { eventTransferHandler } from "./worker-common.js";
import nodeEndpoint from "comlink/dist/umd/node-adapter.js";
import events from "events";
events.setMaxListeners(300);

// eslint-disable-next-line @typescript-eslint/no-explicit-any
Comlink.transferHandlers.set("EVENT", eventTransferHandler as any);

class NodeCompilerWorker extends Compiler {
  constructor() {
    super(wasm);
    if (workerData && typeof workerData.qscLogLevel === "number") {
      log.setLogLevel(workerData.qscLogLevel);
    }
  }
}

const compiler = new NodeCompilerWorker();

// @ts-expect-error TypeScript really doesn't believe that the default
// export from node-adapter is callable. But it is.
Comlink.expose(compiler, nodeEndpoint(parentPort));

export type INodeCompilerWorker = typeof compiler;
