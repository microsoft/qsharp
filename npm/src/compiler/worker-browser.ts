// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as wasm from "../../lib/web/qsc_wasm.js";
import { log } from "../log.js";
import { invokeWorkerMethod } from "../worker-common.js";
import { Compiler } from "./compiler.js";
import {
  ICompilerMethodsOnly,
  getWorkerEventHandlers
} from "./worker-common.js";

// Used to sent messages back to the client when events occur during request processing
const evtTarget = getWorkerEventHandlers(self.postMessage);

let compiler: Compiler | null = null;

// This export should be assigned to 'self.onmessage' in a WebWorker
export function messageHandler(e: MessageEvent) {
  const data = e.data;

  if (!data.type || typeof data.type !== "string") {
    log.error(`Unrecognized msg: ${data}`);
    return;
  }

  switch (data.type) {
    case "init":
      log.setLogLevel(data.qscLogLevel);
      wasm.initSync(data.wasmModule);
      compiler = new Compiler(wasm);
      break;
    default:
      if (!compiler) {
        log.error(
          `Received message before the compiler was initialized: %o`,
          data
        );
      } else {
        invokeWorkerMethod(data, compiler as ICompilerMethodsOnly, self.postMessage, evtTarget);
      }
  }
}
