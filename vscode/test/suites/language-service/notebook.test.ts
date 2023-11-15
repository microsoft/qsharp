// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { assert } from "chai";

suite("Q# Notebook Tests", () => {
  const workspaceFolder =
    vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders[0];
  assert(workspaceFolder, "Expecting an open folder");

  const workspaceFolderUri = workspaceFolder.uri;

  test("Cell language is set to Q#", async () => {
    await activate();
    const notebook = await vscode.workspace.openNotebookDocument(
      vscode.Uri.joinPath(workspaceFolderUri, "test-no-lang-metadata.ipynb"),
    );

    // Test if the cell with the %%qsharp magic has been detected
    // and its language switched to qsharp. We can only expect this to happen
    // after the notebook has happened, and the document handlers
    // have been invoked, but of course there is no callback for that.
    // So we just verify the notebook has been updated with the qsharp
    // language id within 50ms (If we start exceeding this timeout for some
    // reason, it's enough of a user-perceptible delay that we're probably
    // better off disabling this behavior, rather than suddenly change the
    // cell language from under the user after a delay).
    await new Promise<void>((resolve, reject) => {
      let done = false;
      setTimeout(() => {
        if (!done) {
          reject(new Error("timed out waiting for a Q# code cell"));
        }
      }, 50);

      vscode.workspace.onDidChangeNotebookDocument((event) => {
        if (!done && hasQSharpCell(event.notebook)) {
          done = true;
          resolve();
        }
      });

      // in case the notebook updates have already occurred by the time we get here
      if (hasQSharpCell(notebook)) {
        done = true;
        resolve();
      }

      function hasQSharpCell(notebook) {
        return notebook
          .getCells()
          .find((cell) => cell.document.languageId === "qsharp");
      }
    });
  });

  test("Diagnostics", async () => {
    await activate();
    const notebook = await vscode.workspace.openNotebookDocument(
      vscode.Uri.joinPath(workspaceFolderUri, "test.ipynb"),
    );

    const thirdQSharpCellUri = notebook.cellAt(3).document.uri;

    // Verify diagnostics in Q# cell
    const diagnostics = vscode.languages.getDiagnostics(thirdQSharpCellUri);
    assert.lengthOf(diagnostics, 1);

    assert.include(diagnostics[0].message, "syntax error");
    assert.equal(diagnostics[0].range.start.line, 2);
  });

  test("Definition", async () => {
    await activate();
    const notebook = await vscode.workspace.openNotebookDocument(
      vscode.Uri.joinPath(workspaceFolderUri, "test.ipynb"),
    );

    const firstQSharpCellUri = notebook.cellAt(1).document.uri;
    const secondQSharpCellUri = notebook.cellAt(2).document.uri;

    const pos = new vscode.Position(2, 0); // cursor on the usage of Test()

    // Verify go to definition across cells
    const actualDefinition = (await vscode.commands.executeCommand(
      "vscode.executeDefinitionProvider",
      secondQSharpCellUri,
      pos,
    )) as vscode.Location[];

    const location = actualDefinition[0];
    assert.equal(location.uri.toString(), firstQSharpCellUri.toString());
    assert.equal(location.range.start.line, 2);
    assert.equal(location.range.start.character, 10);
  });
});

async function activate() {
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const ext = vscode.extensions.getExtension("quantum.qsharp-lang-vscode-dev")!;
  await ext.activate();
}
