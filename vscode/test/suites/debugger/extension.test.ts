// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { assert } from "chai";

suite("Q# Debugger Tests", () => {
  const workspaceFolder =
    vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders[0];
  assert(workspaceFolder, "Expecting an open folder");

  test("Placeholder - success", async () => {
    assert.equal(1, 1);
  });
});
