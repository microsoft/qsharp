// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { type ExtensionApi } from "../../src/extension";

// Q# extension log level. Increase this for debugging.
const extensionLogLevel = "warn";

export async function activateExtension() {
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const ext = vscode.extensions.getExtension("quantum.qsharp-lang-vscode-dev")!;
  if (ext.isActive) {
    return;
  }

  const start = performance.now();
  const extensionApi: ExtensionApi = await ext.activate();
  const logForwarder = extensionApi.logging;
  if (!logForwarder) {
    throw new Error(`qsharp-tests: extension did not return a log forwarder`);
  }

  logForwarder.setLevel(extensionLogLevel);
  logForwarder.setListener((level, ...args) => {
    // Write extension logs to the console.
    console.log(`qsharp: [${level}] ${args.join(" ")}`);
  });

  console.log(
    `qsharp-tests: activate() completed in ${performance.now() - start}ms`,
  );
}
