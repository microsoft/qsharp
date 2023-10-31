// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as wasm from "../../lib/web/qsc_wasm.js";
import { log } from "../log.js";
import { QSharpDebugService } from "./debug-service.js";
import { createDebugServiceDispatcher } from "./worker-proxy.js";

let invokeDebugger: ReturnType<typeof createDebugServiceDispatcher> | null =
  null;

// This export should be assigned to 'self.onmessage' in a WebWorker
export function messageHandler(e: MessageEvent) {
  const data = e.data;

  if (!data.type || typeof data.type !== "string") {
    log.error(`Unrecognized msg: ${data}`);
    return;
  }

  switch (data.type) {
    case "init":
      {
        log.setLogLevel(data.qscLogLevel);
        wasm.initSync(data.wasmModule);
        const debugService = new QSharpDebugService(wasm);
        invokeDebugger = createDebugServiceDispatcher(
          self.postMessage.bind(self),
          debugService,
        );
      }
      break;
    default:
      if (!invokeDebugger) {
        log.error(
          `Received message before the debugger was initialized: %o`,
          data,
        );
      } else {
        invokeDebugger(data);
      }
  }
}
