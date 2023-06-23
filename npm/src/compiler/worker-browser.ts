// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as wasm from "../../lib/web/qsc_wasm.js";
import { log } from "../log.js";
import { Compiler } from "./compiler.js";
import { createCompilerDispatcher } from "./worker-proxy.js";

let invokeCompiler: ReturnType<typeof createCompilerDispatcher> | null = null;

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
        const compiler = new Compiler(wasm);
        invokeCompiler = createCompilerDispatcher(
          self.postMessage.bind(self),
          compiler
        );
      }
      break;
    default:
      if (!invokeCompiler) {
        log.error(
          `Received message before the compiler was initialized: %o`,
          data
        );
      } else {
        invokeCompiler(data);
      }
  }
}
