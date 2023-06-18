// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as Comlink from "comlink";
import * as wasm from "../lib/web/qsc_wasm.js";
import { Compiler } from "./compiler.js";
import { log } from "./log.js";
import { eventTransferHandler } from "./worker-common.js";

// This module supports running the compiler inside a browser WebWorker. This is set as
// the "qsharp/worker" entry point using 'conditional exports' in package.json.

// eslint-disable-next-line @typescript-eslint/no-explicit-any
Comlink.transferHandlers.set("EVENT", eventTransferHandler as any);

class WebCompilerWorker extends Compiler {
  init(wasmModule: WebAssembly.Module, qscLogLevel: number) {
    log.setLogLevel(qscLogLevel);
    wasm.initSync(wasmModule);
    super.initWasm(wasm);
  }
}

const compiler = new WebCompilerWorker();

Comlink.expose(compiler);

export type IWebCompilerWorker = typeof compiler;
