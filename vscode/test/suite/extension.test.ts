// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { assert } from "chai";

export default function() {
  suite("Q# Extension Test Suite", () => {

    const workspaceFolder =
      vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders[0];
    assert(workspaceFolder, "Expecting an open folder");

    const workspaceFolderUri = workspaceFolder.uri;

    test("Q# language is registered", async () => {
      const uri = vscode.Uri.joinPath(workspaceFolderUri, "test.qs");
      const doc = await vscode.workspace.openTextDocument(uri);
      assert.equal(doc.languageId, "qsharp", "document language should be `qsharp`");
    });
  });
}
