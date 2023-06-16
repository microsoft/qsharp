// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as wasm from "../../lib/web/qsc_wasm.js";
import { log } from "../log.js";
import { QSharpLanguageService } from "./language-service.js";
import {
  getWorkerEventHandlers,
  handleMessageInWorker,
} from "./worker-common.js";

// Used to sent messages back to the client when events occur during request processing
const evtTarget = getWorkerEventHandlers(self.postMessage);

let languageService: QSharpLanguageService | null = null;

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
      languageService = new QSharpLanguageService(wasm, evtTarget);
      break;
    default:
      if (!languageService) {
        log.error(
          `Received message before the compiler was initialized: %o`,
          data
        );
      } else {
        handleMessageInWorker(data, languageService, self.postMessage);
      }
  }
}
