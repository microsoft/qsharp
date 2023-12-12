// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as wasm from "../../lib/web/qsc_wasm.js";
import { log } from "../log.js";
import { QSharpLanguageService } from "./language-service.js";
import { createLanguageServiceDispatcher } from "./worker-proxy.js";

let invokeCompiler: ReturnType<typeof createLanguageServiceDispatcher> | null =
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
        const languageService = new QSharpLanguageService(wasm);
        invokeCompiler = createLanguageServiceDispatcher(
          self.postMessage.bind(self),
          languageService,
        );
      }
      break;
    default:
      if (!invokeCompiler) {
        log.error(
          `Received message before the compiler was initialized: %o`,
          data,
        );
      } else {
        invokeCompiler(data);
      }
  }
}
