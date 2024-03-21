// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { samples } from "qsharp-lang";

export async function initProjectCreator(context: vscode.ExtensionContext) {
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "qsharp-vscode.createProject",
      async (folderUri: vscode.Uri | undefined) => {
        if (!folderUri) {
          // Was run from the command palette. So use the currently open root folder.
          if (!vscode.workspace.workspaceFolders?.length) return; // No folders open

          // TODO: Could show the workspace folder quick-pick if more than 1 root folder open.
          folderUri = vscode.workspace.workspaceFolders[0].uri;
        }

        const edit = new vscode.WorkspaceEdit();
        const projUri = vscode.Uri.joinPath(folderUri, "qsharp.json");
        const mainUri = vscode.Uri.joinPath(folderUri, "src", "main.qs");
        edit.createFile(projUri);
        edit.createFile(mainUri);

        edit.set(projUri, [
          new vscode.TextEdit(new vscode.Range(0, 0, 0, 0), "{}"),
        ]);
        edit.set(mainUri, [
          // Assumes the 'minimal' sample is at index 0. May want to do something
          // more foolproof (or even have a dedicate code sample for projects)
          new vscode.TextEdit(new vscode.Range(0, 0, 0, 0), samples[0].code),
        ]);
        await vscode.workspace.applyEdit(edit);
      },
    ),
  );
}
