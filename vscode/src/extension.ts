// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService, getLanguageService, loadWasmModule } from "qsharp";
import * as vscode from "vscode";
import { createCompletionItemProvider } from "./completion.js";
import { createDefinitionProvider } from "./definition.js";
import { startCheckingQSharp } from "./diagnostics.js";
import { createHoverProvider } from "./hover.js";
import { registerQSharpNotebookHandlers } from "./notebook.js";

export async function activate(context: vscode.ExtensionContext) {
  const output = vscode.window.createOutputChannel("Q#");
  output.appendLine("Q# extension activated.");

  // Override the global logger with functions that write to the output channel
  global.qscLog.error = (...args: unknown[]) => {
    output.appendLine(
      args.map((a) => (typeof a === "string" ? a : JSON.stringify(a))).join(" ")
    );
  };
  global.qscLog.warn = (...args: unknown[]) => {
    output.appendLine(
      args.map((a) => (typeof a === "string" ? a : JSON.stringify(a))).join(" ")
    );
  };
  global.qscLog.info = (...args: unknown[]) => {
    output.appendLine(
      args.map((a) => (typeof a === "string" ? a : JSON.stringify(a))).join(" ")
    );
  };
  global.qscLog.debug = (...args: unknown[]) => {
    output.appendLine(
      args.map((a) => (typeof a === "string" ? a : JSON.stringify(a))).join(" ")
    );
  };
  global.qscLog.trace = (...args: unknown[]) => {
    output.appendLine(
      args.map((a) => (typeof a === "string" ? a : JSON.stringify(a))).join(" ")
    );
  };

  const languageService = await loadLanguageService(context.extensionUri);

  context.subscriptions.push(
    ...registerDocumentUpdateHandlers(languageService)
  );

  context.subscriptions.push(...registerQSharpNotebookHandlers());

  context.subscriptions.push(startCheckingQSharp(languageService));

  // completions
  context.subscriptions.push(
    vscode.languages.registerCompletionItemProvider(
      "qsharp",
      createCompletionItemProvider(languageService),
      "."
    )
  );

  // hover
  context.subscriptions.push(
    vscode.languages.registerHoverProvider(
      "qsharp",
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
      if (vscode.languages.match("qsharp", document)) {
        languageService.closeDocument(document.uri.toString());
      }
    })
  );

  function updateIfQsharpDocument(document: vscode.TextDocument) {
    if (vscode.languages.match("qsharp", document)) {
      languageService.updateDocument(
        document.uri.toString(),
        document.version,
        document.getText()
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
