// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { getCompilerWorker } from "qsharp-lang";
import { isQsharpDocument } from "./common";
import { getTarget } from "./config";
import { Uri, window } from "vscode";
import { loadProject } from "./projectSystem";
import { sendMessageToPanel } from "./webviewPanel";

export async function showDocumentationCommand(extensionUri: Uri) {
  // Reveal panel an show 'Loading...' for immediate feedback.
  sendMessageToPanel(
    "documentation", // This is needed to route the message to the proper panel
    true,
    null,
  );
  const editor = window.activeTextEditor;
  if (!editor || !isQsharpDocument(editor.document)) {
    throw new Error("The currently active window is not a Q# file");
  }

  const docUri = editor.document.uri;
  const program = await loadProject(docUri);
  const targetProfile = getTarget();

  // Get std library documentation from compiler.
  const compilerWorkerScriptPath = Uri.joinPath(
    extensionUri,
    "./out/compilerWorker.js",
  ).toString();
  const worker = getCompilerWorker(compilerWorkerScriptPath);
  const documentation = await worker.getDocumentationContent(
    program.sources,
    targetProfile,
    program.languageFeatures,
  );

  // Concatenate all documentation.
  // The following adds an empty line and a horizontal line
  // between documentation for different functions.
  const content = documentation.join("<br/>\n\n---\n\n");

  const message = {
    command: "showDocumentationCommand", // This is handled in webview.tsx onMessage
    contentToRender: content,
  };

  sendMessageToPanel(
    "documentation", // This is needed to route the message to the proper panel
    true,
    message,
  );
}
