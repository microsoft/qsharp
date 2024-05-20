// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { getCompilerWorker } from "qsharp-lang";
import { Uri } from "vscode";
import { sendMessageToPanel } from "./webviewPanel";
import { getActiveProgram } from "./programConfig";

export async function showDocumentationCommand(extensionUri: Uri) {
  const program = await getActiveProgram();
  if (!program.success) {
    throw new Error(program.errorMsg);
  }

  const { sources, profile, languageFeatures } = program.programConfig;

  // Reveal panel and show 'Loading...' for immediate feedback.
  sendMessageToPanel(
    "documentation", // This is needed to route the message to the proper panel
    true, // Reveal panel
    null, // With no message
  );

  // Get API documentation from compiler.
  const compilerWorkerScriptPath = Uri.joinPath(
    extensionUri,
    "./out/compilerWorker.js",
  ).toString();
  const worker = getCompilerWorker(compilerWorkerScriptPath);
  const docFiles = await worker.getDocumentation(
    sources,
    profile,
    languageFeatures,
  );

  const documentation: string[] = [];
  for (const file of docFiles) {
    // Some files may contain information other than documentation
    // For example, table of content is a separate file in a special format
    // We check presence of qsharp.name in metadata to make sure we take
    // only files that contain documentation from some qsharp object.
    if (file.metadata.indexOf("qsharp.name:") >= 0) {
      documentation.push(file.contents);
    }
  }

  const message = {
    command: "showDocumentationCommand", // This is handled in webview.tsx onMessage
    fragmentsToRender: documentation,
  };

  sendMessageToPanel(
    "documentation", // This is needed to route the message to the proper panel
    true, // Reveal panel
    message, // And ask it to display documentation
  );
}
