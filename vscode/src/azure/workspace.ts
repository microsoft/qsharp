// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { log } from "qsharp";

import { WorkspaceTreeProvider } from "./workspaceTree";
import { queryWorkspaces } from "./workspaceQuery";

let workspaceTreeProvider: WorkspaceTreeProvider;

export function setupWorkspaces(context: vscode.ExtensionContext) {
  workspaceTreeProvider = new WorkspaceTreeProvider(context);
  const workspaceTree = vscode.window.createTreeView("quantum-workspaces", {
    treeDataProvider: workspaceTreeProvider,
  });

  workspaceTree.onDidChangeSelection((evt) => {
    if (evt.selection.length) {
      log.debug("TreeView selection changed to ", evt.selection[0].label);
      evt.selection[0];
    }
  });

  vscode.commands.registerCommand("quantum-workspaces-refresh", () => {
    workspaceTreeProvider.refresh();
  });

  vscode.commands.registerCommand("quantum-workspaces-add", async () => {
    const accountPrompt = "Sign-in with a Microsoft account";
    const tokenPrompt = "Connect using an access token";
    const method = await vscode.window.showQuickPick(
      [accountPrompt, tokenPrompt],
      { title: "Select authentication method" }
    );
    if (method === tokenPrompt) {
      const _token = await vscode.window.showInputBox({
        title: "Enter the workspace access token",
      });
    } else {
      // TODO: Sign-in, select tenant, etc.
      const sub = await vscode.window.showQuickPick(
        ["MSDN Subscription", "Production"],
        { title: "Select the Azure subscription" }
      );
    }
  });

  vscode.commands.registerCommand("quantum-result-view", async () => {
    const doc = await vscode.workspace.openTextDocument({
      content: `# Results
ABC, [Zero, One]
DEF, 3.14159
`,
      language: "plaintext",
    });
    vscode.window.showTextDocument(doc);
  });

  vscode.commands.registerCommand(
    "extension.qsharp.listWorkspaces",
    queryWorkspaces
  );
}
