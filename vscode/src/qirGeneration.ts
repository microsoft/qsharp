// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { getCompilerWorker, log } from "qsharp";

export function initCodegen(context: vscode.ExtensionContext) {
  const statusBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    200
  );
  statusBarItem.command = "quantum-set-target";
  statusBarItem.text = "QIR:Full";

  context.subscriptions.push(
    vscode.commands.registerCommand("quantum-get-qir", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) return;
      const code = editor.document.getText();

      // Create a temporary worker just to get the QIR, as it may loop/panic during codegen.
      const compilerWorkerScriptPath = vscode.Uri.joinPath(
        context.extensionUri,
        "./out/compilerWorker.js"
      );
      const worker = getCompilerWorker(compilerWorkerScriptPath.toString());
      try {
        const qir = await worker.getQir(code);

        const qirDoc = await vscode.workspace.openTextDocument({
          language: "llvm",
          content: qir,
        });
        await vscode.window.showTextDocument(qirDoc);
      } catch (e: any) {
        vscode.window.showErrorMessage(
          "Code generation failed. Please ensure the code is compatible with the QIR base profile"
        );
        log.error("Codegen error. ", e.toString());
      }
      worker.terminate();
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("quantum-set-target", async () => {
      const target = await vscode.window.showQuickPick(
        ["QIR:Base", "QIR:Adaptive", "QIR:Full"],
        { placeHolder: "Select the QIR target profile" }
      );
      if (target) statusBarItem.text = target;
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
