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

  context.subscriptions.push(
    vscode.commands.registerCommand(
      "qsharp-vscode.populateFilesList",
      async (qsharpJsonUri: vscode.Uri | undefined) => {
        // If called from the content menu qsharpJsonUri will be the full qsharp.json uri
        // If called from the command palette is will be undefined, so use the active editor
        log.info("populateQsharpFilesList called with", qsharpJsonUri);

        qsharpJsonUri =
          qsharpJsonUri ?? vscode.window.activeTextEditor?.document.uri;
        if (!qsharpJsonUri) {
          log.error(
            "populateFilesList called, but argument or active editor is not qsharp.json",
          );
          return;
        }

        log.debug("Populating qsharp.json files for: ", qsharpJsonUri.path);

        // First, verify the qsharp.json can be opened and is a valid json file
        const qsharpJsonDoc =
          await vscode.workspace.openTextDocument(qsharpJsonUri);
        if (!qsharpJsonDoc) {
          log.error("Unable to open the qsharp.json file at ", qsharpJsonDoc);
          return;
        }

        let manifestObj: any = {};
        try {
          manifestObj = JSON.parse(qsharpJsonDoc.getText());
        } catch (err: any) {
          await vscode.window.showErrorMessage(
            `Unable to parse the contents of ${qsharpJsonUri.path}`,
          );
          return;
        }

        // Recursively find all .qs documents under the ./src dir
        const files: string[] = [];
        const srcDir = vscode.Uri.joinPath(qsharpJsonUri, "..", "src");

        async function getQsFilesInDir(dir: vscode.Uri) {
          const dirFiles = (await vscode.workspace.fs.readDirectory(dir)).sort(
            (a, b) => {
              // To order the list, put files before directories, then sort alphabetically
              if (a[1] !== b[1]) return a[1] < b[1] ? -1 : 1;
              return a[0] < b[0] ? -1 : 1;
            },
          );
          for (const [name, type] of dirFiles) {
            if (type === vscode.FileType.File && name.endsWith(".qs")) {
              files.push(vscode.Uri.joinPath(dir, name).toString());
            } else if (type === vscode.FileType.Directory) {
              await getQsFilesInDir(vscode.Uri.joinPath(dir, name));
            }
          }
          return files;
        }
        await getQsFilesInDir(srcDir);

        // Update the files property of the qsharp.json and write back to the document
        const srcDirPrefix = srcDir.toString() + "";
        manifestObj["files"] = files.map((file) =>
          file.replace(srcDirPrefix, "src"),
        );

        // Apply the edits to the qsharp.json
        const edit = new vscode.WorkspaceEdit();
        edit.replace(
          qsharpJsonUri,
          new vscode.Range(0, 0, qsharpJsonDoc.lineCount, 0),
          JSON.stringify(manifestObj, null, 2),
        );
        if (!(await vscode.workspace.applyEdit(edit))) {
          vscode.window.showErrorMessage(
            "Unable to update the qsharp.json file. Check the file is writable",
          );
          return;
        }

        // Bring the qsharp.json to the front for the user to save
        await vscode.window.showTextDocument(qsharpJsonDoc);
      },
    ),
  );
}
