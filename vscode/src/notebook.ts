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
