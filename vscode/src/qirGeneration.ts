// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { getCompilerWorker, log } from "qsharp-lang";
import { isQsharpDocument } from "./common";

let compilerWorkerScriptPath: string;

export async function getQirForActiveWindow(): Promise<string | undefined> {
  let result = "";
  const editor = vscode.window.activeTextEditor;
  if (!editor || !isQsharpDocument(editor.document)) return;

  // Check that the current target is base profile, and current doc has no errors.
  const targetProfile = vscode.workspace
    .getConfiguration("Q#")
    .get<string>("targetProfile", "full");
  if (targetProfile !== "base") {
    vscode.window.showErrorMessage(
      "Submitting to Azure is only supported when targeting the QIR base profile. " +
        "Please update the QIR target via the status bar selector or extension settings."
    );
    return;
  }

  // Get the diagnostics for the current document.
  const diagnostics = await vscode.languages.getDiagnostics(
    editor.document.uri
  );
  if (diagnostics?.length > 0) {
    vscode.window.showErrorMessage(
      "The current program contains errors that must be fixed before submitting to Azure"
    );
    return;
  }

  const code = editor.document.getText();

  // Create a temporary worker just to get the QIR, as it may loop/panic during codegen.
  // TODO: Could also start a timer here and kill it if running for too long without result.
  const worker = getCompilerWorker(compilerWorkerScriptPath);
  try {
    result = await worker.getQir(code);
  } catch (e: any) {
    vscode.window.showErrorMessage(
      "Code generation failed. Please ensure the code is compatible with the QIR base profile " +
        "by setting the target QIR profile to 'base' and fixing any errors."
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
