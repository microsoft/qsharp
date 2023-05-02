import * as vscode from "vscode";

export function activate(context: vscode.ExtensionContext) {
    registerQSharpNotebookHandlers(context);
}

/**
 * Sets up handlers to detect Q# code cells in Jupyter notebooks and set the language to Q#.
 */
function registerQSharpNotebookHandlers(context: vscode.ExtensionContext) {
    const qsharpLanguageId = "qsharp";
    const qsharpCellMagic = "%%qsharp";
    const jupyterNotebookType = "jupyter-notebook";

    context.subscriptions.push(vscode.workspace.onDidOpenNotebookDocument((notebookDocument) => {
        if (notebookDocument.notebookType === jupyterNotebookType) {
            updateQSharpCellLanguages(notebookDocument.getCells());
        }
    }));

    context.subscriptions.push(vscode.workspace.onDidChangeNotebookDocument((event) => {
        if (event.notebook.notebookType === jupyterNotebookType) {
            // change.document will be undefined if the cell contents did not change -- filter those out.
            const changedCells = event.cellChanges.filter(change => change.document).map(change => change.cell);
            const addedCells = event.contentChanges.map(change => change.addedCells).flat();
            updateQSharpCellLanguages(changedCells.concat(addedCells));
        }
    }));

    function updateQSharpCellLanguages(cells: vscode.NotebookCell[]) {
        for (const cell of cells) {
            // If this is a code cell that starts with %%qsharp, and language isn't already set to Q#, set it.
            if (cell.kind === vscode.NotebookCellKind.Code) {
                const document = cell.document;
                if (document.languageId !== qsharpLanguageId &&
                    document.lineAt(0).text.startsWith(qsharpCellMagic)) {
                    vscode.languages.setTextDocumentLanguage(cell.document, qsharpLanguageId);
                }
            }
        }
    }
}
