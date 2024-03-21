// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { log, samples } from "qsharp-lang";
import { EventType, sendTelemetryEvent } from "./telemetry";

export async function initProjectCreator(context: vscode.ExtensionContext) {
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "qsharp-vscode.createProject",
      async (folderUri: vscode.Uri | undefined) => {
        sendTelemetryEvent(EventType.CreateProject, {}, {});

        if (!folderUri) {
          // This was run from the command palette, not the context menu, so create the project
          // in the root of an open workspace folder.
          const workspaceCount = vscode.workspace.workspaceFolders?.length || 0;
          if (workspaceCount === 0) {
            // No workspaces open
            vscode.window.showErrorMessage(
              "You must have a folder open to create a Q# project in",
            );
            return;
          } else if (workspaceCount === 1) {
            folderUri = vscode.workspace.workspaceFolders![0].uri;
          } else {
            const workspaceChoice = await vscode.window.showWorkspaceFolderPick(
              { placeHolder: "Pick the workspace folder for the project" },
            );
            if (!workspaceChoice) return;
            folderUri = workspaceChoice.uri;
          }
        }

        const edit = new vscode.WorkspaceEdit();
        const projUri = vscode.Uri.joinPath(folderUri, "qsharp.json");
        const mainUri = vscode.Uri.joinPath(folderUri, "src", "Main.qs");

        const sample = samples.find((elem) => elem.title === "Minimal");
        if (!sample) {
          // Should never happen.
          log.error("Unable to find the Minimal sample");
          return;
        }

        edit.createFile(projUri);
        edit.createFile(mainUri);

        edit.set(projUri, [
          new vscode.TextEdit(new vscode.Range(0, 0, 0, 0), "{}"),
        ]);
        edit.set(mainUri, [
          new vscode.TextEdit(new vscode.Range(0, 0, 0, 0), sample.code),
        ]);

        // This doesn't throw on failure, it just returns false
        if (!(await vscode.workspace.applyEdit(edit))) {
          vscode.window.showErrorMessage(
            "Unable to create the project. Check the project files don't already exist and that the file system is writable",
          );
        }
      },
    ),
  );
}
