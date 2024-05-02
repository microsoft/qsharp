// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { IOperationInfo, getCompilerWorker } from "qsharp-lang";
import { Uri } from "vscode";
import { sendMessageToPanel } from "./webviewPanel";

export async function showDocumentationCommand(
  extensionUri: Uri,
  operation: IOperationInfo | undefined,
) {
  // Reveal panel an show 'Loading...' for immediate feedback.
  sendMessageToPanel(
    "documentationPanelType", // This is needed to route the message to the proper panel
    true,
    null,
  );

  // Get std library documentation from compiler.
  const compilerWorkerScriptPath = Uri.joinPath(
    extensionUri,
    "./out/compilerWorker.js",
  ).toString();
  const worker = getCompilerWorker(compilerWorkerScriptPath);
  let content = await worker.getCombinedDocumentation();

  const message = {
    command: "showDocumentationCommand", // This is handled in webview.tsx onMessage
    contentToRender: content,
  };

  sendMessageToPanel(
    "documentationPanelType", // This is needed to route the message to the proper panel
    true,
    message,
  );
}
