// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { qsharpLanguageId } from "./common.js";
import { EventType, sendTelemetryEvent } from "./telemetry.js";
import { WorkspaceTreeProvider } from "./azure/treeView.js";
import { getPythonCodeForWorkspace } from "./azure/workspaceActions.js";

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
        if (!vscode.workspace.workspaceFolders?.length) {
          vscode.window.showErrorMessage(
            "You must have an open folder to create a notebook."
          );
          return;
        }
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

        let notebookName = await vscode.window.showInputBox({
          prompt: "Enter a name for the new notebook",
        });
        if (!notebookName) return;
        if (!notebookName.endsWith(".ipynb")) {
          notebookName += ".ipynb";
        }

        // Create the notebook full uri
        const notebookUri = vscode.Uri.joinPath(
          workspaceFolder.uri,
          notebookName
        );

        // Check and warn if the file already exists
        const existingFiles = await vscode.workspace.fs.readDirectory(
          workspaceFolder.uri
        );
        if (existingFiles.some(([name]) => name === notebookName)) {
          // Ask the user to overwrite or cancel
          const choice = await vscode.window.showWarningMessage(
            `The file ${notebookName} already exists. Do you want to overwrite it?`,
            { modal: true },
            "Overwrite",
            "Cancel"
          );
          if (choice !== "Overwrite") return;
        }

        const templatePath = vscode.Uri.joinPath(
          context.extensionUri,
          "resources",
          "notebookTemplate.ipynb"
        );
        const template = await vscode.workspace.fs.readFile(templatePath);
        let content = new TextDecoder().decode(template);

        // Update the workspace connection info in the notebook if workspaces are already connected to
        const workspaces =
          WorkspaceTreeProvider.instance?.getWorkspaceIds() || [];
        let choice = workspaces[0] || undefined;
        if (workspaces.length > 1) {
          choice = await vscode.window.showQuickPick(workspaces);
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

        content = content.replace(
          `"# TODO: Workspace connection\\n"`,
          JSON.stringify(getCodeForWorkspace(choice))
        );

        await vscode.workspace.fs.writeFile(
          notebookUri,
          new TextEncoder().encode(content)
        );
      }
    )
  );
}
