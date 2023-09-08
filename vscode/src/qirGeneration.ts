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

  const statusBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    200
  );

  // TODO: Fetch from the setting and update the language service on change with Mine's PR.
  statusBarItem.command = "quantum-set-target";
  statusBarItem.text = "QIR:Full";

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

  context.subscriptions.push(
    vscode.commands.registerCommand("quantum-set-target", async () => {
      const target = await vscode.window.showQuickPick(
        ["QIR:Base", "QIR:Adaptive", "QIR:Full"],
        { placeHolder: "Select the QIR target profile" }
      );
      if (target) statusBarItem.text = target;
      // TODO: Update the language service
    })
  );

  if (vscode.window.activeTextEditor?.document.languageId === "qsharp") {
    statusBarItem.show();
  }
  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor((editor) => {
      if (editor?.document.languageId === "qsharp") {
        statusBarItem.show();
      } else {
        statusBarItem.hide();
      }
    })
  );
}
