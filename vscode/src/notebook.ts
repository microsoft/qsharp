// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { qsharpLanguageId } from "./common.js";
import { EventType, sendTelemetryEvent } from "./telemetry.js";

/**
 * Sets up handlers to detect Q# code cells in Jupyter notebooks and set the language to Q#.
 */
export function registerQSharpNotebookHandlers() {
  const qsharpCellMagic = "%%qsharp";
  const jupyterNotebookType = "jupyter-notebook";

  vscode.workspace.notebookDocuments.forEach((notebookDocument) => {
    if (notebookDocument.notebookType === jupyterNotebookType) {
      updateQSharpCellLanguages(notebookDocument.getCells());
    }
  });

  const subscriptions = [];
  subscriptions.push(
    vscode.workspace.onDidOpenNotebookDocument((notebookDocument) => {
      if (notebookDocument.notebookType === jupyterNotebookType) {
        updateQSharpCellLanguages(notebookDocument.getCells());
      }
    })
  );

  subscriptions.push(
    vscode.workspace.onDidChangeNotebookDocument((event) => {
      if (event.notebook.notebookType === jupyterNotebookType) {
        // change.document will be undefined if the cell contents did not change -- filter those out.
        const changedCells = event.cellChanges
          .filter((change) => change.document)
          .map((change) => change.cell);
        const addedCells = event.contentChanges
          .map((change) => change.addedCells)
          .flat();
        updateQSharpCellLanguages(changedCells.concat(addedCells));
      }
    })
  );

  function updateQSharpCellLanguages(cells: vscode.NotebookCell[]) {
    for (const cell of cells) {
      // If this is a code cell that starts with %%qsharp, and language isn't already set to Q#, set it.
      if (cell.kind === vscode.NotebookCellKind.Code) {
        const document = cell.document;
        if (
          document.languageId !== qsharpLanguageId &&
          document.lineAt(0).text.startsWith(qsharpCellMagic)
        ) {
          vscode.languages.setTextDocumentLanguage(
            cell.document,
            qsharpLanguageId
          );
          sendTelemetryEvent(EventType.QSharpJupyterCellInitialized);
        }
      }
    }
  }

  return subscriptions;
}

export function registerCreateNotebookCommand(
  context: vscode.ExtensionContext
) {
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "qsharp-vscode.createNotebook",
      async () => {
        if (!vscode.workspace.workspaceFolders) {
          vscode.window.showErrorMessage(
            "You must have an open folder to create a notebook."
          );
          return;
        }
        const notebookName = await vscode.window.showInputBox({
          prompt: "Enter a name for the new notebook",
        });
        if (!notebookName) return;

        let workspaceFolder = vscode.workspace.workspaceFolders[0];

        // Handle multi-workspace scenarios
        if (vscode.workspace.workspaceFolders.length > 1) {
          // Show a quickpick for the workspace to use
          const choice = await vscode.window.showQuickPick(
            vscode.workspace.workspaceFolders.map((folder) => ({
              label: folder.name,
              workspace: folder,
            })),
            {
              title: "Select a workspace to create the notebook in",
            }
          );
          if (!choice) {
            // User cancelled
            return;
          }
          workspaceFolder = choice.workspace;
        }

        // Create the notebook full uri
        const notebookUri = vscode.Uri.joinPath(
          workspaceFolder.uri,
          notebookName + ".ipynb"
        );

        // Construct a Uint8Array containing 'Hello, world'
        const templatePath = vscode.Uri.joinPath(
          context.extensionUri,
          "resources",
          "notebookTemplate.ipynb"
        );
        const template = await vscode.workspace.fs.readFile(templatePath);
        let content = new TextDecoder().decode(template);
        content = content.replace(
          "{{WORKSPACE}}",
          "TODO: Put workspace details here"
        );

        vscode.workspace.fs.writeFile(
          notebookUri,
          new TextEncoder().encode(content)
        );
      }
    )
  );
}
