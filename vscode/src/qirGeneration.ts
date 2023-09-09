// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { getCompilerWorker, log } from "qsharp-lang";

let compilerWorkerScriptPath: string;

export async function getQirForActiveWindow(): Promise<string> {
  let result = "";
  const editor = vscode.window.activeTextEditor;
  if (!editor) return result;
  const code = editor.document.getText();

  // Create a temporary worker just to get the QIR, as it may loop/panic during codegen.
  // TODO: Could also start a timer here and kill it if running for too long without result.
  const worker = getCompilerWorker(compilerWorkerScriptPath);
  try {
    result = await worker.getQir(code);
  } catch (e: any) {
    vscode.window.showErrorMessage(
      "Code generation failed. Please ensure the code is compatible with the QIR base profile"
    );
    log.error("Codegen error. ", e.toString());
  }
  worker.terminate();

  return result;
}

export function initCodegen(context: vscode.ExtensionContext) {
  compilerWorkerScriptPath = vscode.Uri.joinPath(
    context.extensionUri,
    "./out/compilerWorker.js"
  ).toString();

  context.subscriptions.push(
    vscode.commands.registerCommand("quantum-get-qir", async () => {
      const qir = await getQirForActiveWindow();
      if (qir) {
        const qirDoc = await vscode.workspace.openTextDocument({
          language: "llvm",
          content: qir,
        });
        await vscode.window.showTextDocument(qirDoc);
      }
    })
  );
}
