// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";

export function initCodegen(context: vscode.ExtensionContext) {
  const statusBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    200
  );
  statusBarItem.command = "quantum-set-target";
  statusBarItem.text = "QIR:Full";

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
