// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as wasm from "../../lib/web/qsc_wasm.js";
import { TelemetryEvent, log } from "../log.js";
import { Compiler } from "./compiler.js";
import { createCompilerDispatcher } from "./worker-proxy.js";

let invokeCompiler: ReturnType<typeof createCompilerDispatcher> | null = null;

function telemetryHandler(telemetry: TelemetryEvent) {
  self.postMessage({
    messageType: "event",
    type: "telemetry-event",
    detail: telemetry,
  });
}

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
        log.setTelemetryCollector(telemetryHandler);
        wasm.initSync(data.wasmModule);

        // Set up logging and telemetry as soon as possible after instantiating
        wasm.initLogging(log.logWithLevel, log.getLogLevel());
        log.onLevelChanged = (level) => wasm.setLogLevel(level);

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
