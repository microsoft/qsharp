// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { QscEventTarget, getCompiler, loadWasmModule } from "qsharp";
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

  const { compiler, evtTarget } = await loadCompiler(context.extensionUri);

  context.subscriptions.push(...registerDocumentUpdateHandlers(compiler));

  context.subscriptions.push(...registerQSharpNotebookHandlers());

  context.subscriptions.push(startCheckingQSharp(evtTarget));

  // completions
  context.subscriptions.push(
    vscode.languages.registerCompletionItemProvider(
      "qsharp",
      createCompletionItemProvider(compiler),
      "."
    )
  );

  // hover
  context.subscriptions.push(
    vscode.languages.registerHoverProvider(
      "qsharp",
      createHoverProvider(compiler)
    )
  );

  // go to def
  context.subscriptions.push(
    vscode.languages.registerDefinitionProvider(
      "qsharp",
      createDefinitionProvider(compiler)
    )
  );
}

function registerDocumentUpdateHandlers(
  compiler: Awaited<ReturnType<typeof getCompiler>>
) {
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
        compiler.closeDocument(document.uri.toString());
      }
    })
  );

  function updateIfQsharpDocument(document: vscode.TextDocument) {
    if (vscode.languages.match("qsharp", document)) {
      compiler.updateDocument(
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
async function loadCompiler(baseUri: vscode.Uri) {
  const wasmUri = vscode.Uri.joinPath(baseUri, "./wasm/qsc_wasm_bg.wasm");
  const wasmBytes = await vscode.workspace.fs.readFile(wasmUri);
  await loadWasmModule(wasmBytes);
  const evtTarget = new QscEventTarget(false);
  return { compiler: await getCompiler(evtTarget), evtTarget };
}
