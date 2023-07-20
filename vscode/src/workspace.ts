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

  vscode.commands.registerCommand(
    "extension.qsharp.listWorkspaces",
    queryWorkspaces
  );
}
