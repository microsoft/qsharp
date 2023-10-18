// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService } from "qsharp-lang";
import * as vscode from "vscode";
import { qsharpDocumentFilter, qsharpLanguageId } from "./common.js";
import { EventType, sendTelemetryEvent } from "./telemetry.js";
import { WorkspaceTreeProvider } from "./azure/treeView.js";
import { getPythonCodeForWorkspace } from "./azure/workspaceActions.js";
import { notebookTemplate } from "./notebookTemplate.js";

const qsharpCellMagic = "%%qsharp";
const jupyterNotebookType = "jupyter-notebook";

/**
 * Sets up handlers to detect Q# code cells in Jupyter notebooks and set the language to Q#.
 */
export function registerQSharpNotebookHandlers() {
  vscode.workspace.notebookDocuments.forEach((notebookDocument) => {
    if (notebookDocument.notebookType === jupyterNotebookType) {
      updateQSharpCellLanguages(notebookDocument.getCells());
    }
  });

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

const openQSharpNotebooks = new Set<string>();

/**
 * This one is for syncing with the language service
 */
export function registerQSharpNotebookCellUpdateHandlers(
  languageService: ILanguageService
) {
  vscode.workspace.notebookDocuments.forEach((notebook) => {
    updateIfQsharpNotebook(notebook);
  });

  const subscriptions = [];
  subscriptions.push(
    vscode.workspace.onDidOpenNotebookDocument((notebook) => {
      updateIfQsharpNotebook(notebook);
    })
  );

  subscriptions.push(
    vscode.workspace.onDidChangeNotebookDocument((event) => {
      updateIfQsharpNotebook(event.notebook);
    })
  );

  subscriptions.push(
    vscode.workspace.onDidCloseNotebookDocument((notebook) => {
      closeIfKnownQsharpNotebook(notebook);
    })
  );

  function updateIfQsharpNotebook(notebook: vscode.NotebookDocument) {
    if (notebook.notebookType === jupyterNotebookType) {
      const qsharpCells = getQSharpCells(notebook);
      const notebookUri = notebook.uri.toString();
      if (qsharpCells.length > 0) {
        openQSharpNotebooks.add(notebookUri);
        languageService.updateNotebookDocument(
          notebookUri,
          notebook.version,
          qsharpCells.map((cell) => {
            return {
              uri: cell.document.uri.toString(),
              code: getQSharpText(cell.document),
            };
          })
        );
      } else {
        // All Q# cells could have been deleted, check if we know this doc from previous calls
        closeIfKnownQsharpNotebook(notebook);
      }
    }
  }

  function closeIfKnownQsharpNotebook(notebook: vscode.NotebookDocument) {
    const notebookUri = notebook.uri.toString();
    if (openQSharpNotebooks.has(notebookUri)) {
      languageService.closeNotebookDocument(
        notebookUri,
        getQSharpCells(notebook).map((cell) => cell.document.uri.toString())
      );
      openQSharpNotebooks.delete(notebook.uri.toString());
    }
  }

  function getQSharpCells(notebook: vscode.NotebookDocument) {
    return notebook
      .getCells()
      .filter((cell) =>
        vscode.languages.match(qsharpDocumentFilter, cell.document)
      );
  }

  function getQSharpText(document: vscode.TextDocument) {
    // Erase the %%qsharp magic line if it's there
    // Replace it with whitespace so that document offsets remain the same.
    // This will save us from having to map offsets later when
    // communicating with the language service.
    if (document.lineAt(0).text.startsWith(qsharpCellMagic)) {
      return (
        "".padStart(qsharpCellMagic.length) +
        document.getText().substring(qsharpCellMagic.length)
      );
    } else {
      return document.getText();
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
