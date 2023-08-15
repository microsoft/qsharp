// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {
  ILanguageService,
  getLanguageService,
  loadWasmModule,
  log,
  getLibrarySourceContent,
  qsharpLibraryUriScheme,
} from "qsharp";
import * as vscode from "vscode";
import { createCompletionItemProvider } from "./completion.js";
import { createDefinitionProvider } from "./definition.js";
import { startCheckingQSharp } from "./diagnostics.js";
import { createHoverProvider } from "./hover.js";
import { registerQSharpNotebookHandlers } from "./notebook.js";
import { activateDebugger } from "./debugger/activate.js";
import {
  qsharpDocumentFilter,
  qsharpNotebookCellDocumentFilter,
} from "./common.js";

export async function activate(context: vscode.ExtensionContext) {
  initializeLogger();
  log.info("Q# extension activating.");

  vscode.workspace.registerTextDocumentContentProvider(
    qsharpLibraryUriScheme,
    new QsTextDocumentContentProvider()
  );

  const languageService = await loadLanguageService(context.extensionUri);

  context.subscriptions.push(
    ...registerDocumentUpdateHandlers(languageService)
  );

  context.subscriptions.push(...registerQSharpNotebookHandlers());

  context.subscriptions.push(startCheckingQSharp(languageService));

  // completions
  context.subscriptions.push(
    vscode.languages.registerCompletionItemProvider(
      qsharpDocumentFilter,
      createCompletionItemProvider(languageService),
      "."
    )
  );

  // hover
  context.subscriptions.push(
    vscode.languages.registerHoverProvider(
      qsharpDocumentFilter,
      createHoverProvider(languageService)
    )
  );

  // go to def
  context.subscriptions.push(
    vscode.languages.registerDefinitionProvider(
      "qsharp",
      createDefinitionProvider(languageService)
    )
  );

  activateDebugger(context);

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

function registerDocumentUpdateHandlers(languageService: ILanguageService) {
  vscode.workspace.textDocuments.forEach((document) => {
    updateIfQsharpDocument(document);
  });

  const subscriptions = [];
  subscriptions.push(
    vscode.workspace.onDidOpenTextDocument((document) => {
      updateIfQsharpDocument(document);
    })
  );

  subscriptions.push(
    vscode.workspace.onDidChangeTextDocument((evt) => {
      updateIfQsharpDocument(evt.document);
    })
  );

  subscriptions.push(
    vscode.workspace.onDidCloseTextDocument((document) => {
      if (
        vscode.languages.match(qsharpDocumentFilter, document) &&
        !vscode.languages.match(qsharpNotebookCellDocumentFilter, document)
      ) {
        // Notebook cells don't currently support the language service.
        languageService.closeDocument(document.uri.toString());
      }
    })
  );

  function updateIfQsharpDocument(document: vscode.TextDocument) {
    if (
      vscode.languages.match(qsharpDocumentFilter, document) &&
      !vscode.languages.match(qsharpNotebookCellDocumentFilter, document)
    ) {
      // Notebook cells don't currently support the language service.
      languageService.updateDocument(
        document.uri.toString(),
        document.version,
        document.getText(),
        true // PackageType "exe"
      );
    }
  }

  return subscriptions;
}

/**
 * Loads the Q# compiler including the WASM module
 */
async function loadLanguageService(baseUri: vscode.Uri) {
  const wasmUri = vscode.Uri.joinPath(baseUri, "./wasm/qsc_wasm_bg.wasm");
  const wasmBytes = await vscode.workspace.fs.readFile(wasmUri);
  await loadWasmModule(wasmBytes);
  return await getLanguageService();
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
