// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log, getLibrarySourceContent, qsharpLibraryUriScheme } from "qsharp";
import * as vscode from "vscode";
import { registerQSharpNotebookHandlers } from "./notebook.js";
import { activateDebugger } from "./debugger/activate.js";
import { activateLanguageService } from "./language_service/activate.js";

export async function activate(context: vscode.ExtensionContext) {
  initializeLogger();
  log.info("Q# extension activating.");

  vscode.workspace.registerTextDocumentContentProvider(
    qsharpLibraryUriScheme,
    new QsTextDocumentContentProvider()
  );

  context.subscriptions.push(
    ...(await activateLanguageService(context.extensionUri))
  );

  context.subscriptions.push(...registerQSharpNotebookHandlers());

  await activateDebugger(context);

  log.info("Q# extension activated.");
}

function initializeLogger() {
  const output = vscode.window.createOutputChannel("Q#", { log: true });

  // Override the global logger with functions that write to the output channel
  log.error = output.error;
  log.warn = output.warn;
  log.info = output.info;
  log.debug = output.debug;
  log.trace = output.trace;

  // The numerical log levels for VS Code and qsharp don't match.
  function mapLogLevel(logLevel: vscode.LogLevel) {
    switch (logLevel) {
      case vscode.LogLevel.Off:
        return "off";
      case vscode.LogLevel.Trace:
        return "trace";
      case vscode.LogLevel.Debug:
        return "debug";
      case vscode.LogLevel.Info:
        return "info";
      case vscode.LogLevel.Warning:
        return "warn";
      case vscode.LogLevel.Error:
        return "error";
    }
  }

  log.setLogLevel(mapLogLevel(output.logLevel));
  output.onDidChangeLogLevel((level) => {
    log.setLogLevel(mapLogLevel(level));
  });
}

export class QsTextDocumentContentProvider
  implements vscode.TextDocumentContentProvider
{
  onDidChange?: vscode.Event<vscode.Uri> | undefined;
  provideTextDocumentContent(
    uri: vscode.Uri,
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    token: vscode.CancellationToken
  ): vscode.ProviderResult<string> {
    return getLibrarySourceContent(uri.path);
  }
}
