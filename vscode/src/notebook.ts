// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { qsharpLanguageId } from "./common.js";
import { EventType, sendTelemetryEvent } from "./telemetry.js";
import { WorkspaceTreeProvider } from "./azure/treeView.js";
import { getPythonCodeForWorkspace } from "./azure/workspaceActions.js";
import { notebookTemplate } from "./notebookTemplate.js";

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

// Yes, this function is long, but mostly to deal with multi-folder VS Code workspace or multi
// Azure Quantum workspace connection scenarios. The actual notebook creation is pretty simple.
export function registerCreateNotebookCommand(
  context: vscode.ExtensionContext
) {
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "qsharp-vscode.createNotebook",
      async () => {
        // Update the workspace connection info in the notebook if workspaces are already connected to
        const tree = WorkspaceTreeProvider.instance;
        let choice: string | undefined = undefined;
        if (tree) {
          const workspaces = tree.getWorkspaceIds();
          // Default to the first (and maybe only) workspace, else prompt the user to select one
          choice = workspaces[0] || undefined;
          if (workspaces.length > 1) {
            choice = (
              await vscode.window.showQuickPick(
                workspaces.map((workspace) => ({
                  label: tree.getWorkspace(workspace)?.name || workspace,
                  id: workspace,
                })),
                {
                  title: "Select a workspace to use in the notebook",
                }
              )
            )?.id;
          }
        }

        function getCodeForWorkspace(choice: string | undefined) {
          if (choice) {
            const workspace =
              WorkspaceTreeProvider.instance?.getWorkspace(choice);
            if (workspace) {
              return getPythonCodeForWorkspace(
                workspace.id,
                workspace.endpointUri,
                workspace.name
              );
            }
          }
          // Else use dummy values
          return getPythonCodeForWorkspace("", "", "");
        }

        // Simplest way to replace the connection is just to stringify and then convert back
        let content = JSON.stringify(notebookTemplate);
        content = content.replace(
          `"# WORKSPACE_CONNECTION_CODE"`,
          JSON.stringify(
            "# Connect to the Azure Quantum workspace\n\n" +
              getCodeForWorkspace(choice)
          )
        );

        const document = await vscode.workspace.openNotebookDocument(
          "jupyter-notebook",
          JSON.parse(content)
        );
        await vscode.window.showNotebookDocument(document);
      }
    )
  );
}
