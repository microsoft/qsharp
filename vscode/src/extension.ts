// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { getCompiler, loadWasmModule } from "qsharp";
import * as vscode from "vscode";
import { startCheckingQSharp } from "./diagnostics.js";
import { registerQSharpNotebookHandlers } from "./notebook.js";

export async function activate(context: vscode.ExtensionContext) {
  const subscriptions = [];

  const compiler = await loadCompiler(context.extensionUri);

  subscriptions.push(...(await startCheckingQSharp(compiler)));

  subscriptions.push(...registerQSharpNotebookHandlers());

  context.subscriptions.push(...subscriptions);
}

/**
 * Loads the Q# compiler including the WASM module
 */
async function loadCompiler(baseUri: vscode.Uri) {
  const wasmUri = vscode.Uri.joinPath(baseUri, "./wasm/qsc_wasm_bg.wasm");
  const wasmBytes = await vscode.workspace.fs.readFile(wasmUri);
  await loadWasmModule(wasmBytes);
  return await getCompiler();
}
