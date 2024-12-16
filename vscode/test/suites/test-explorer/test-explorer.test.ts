// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { assert } from "chai";
import { activateExtension, waitForCondition } from "../extensionUtils";

/**
 * Set to true to log Debug Adapter Protocol messages to the console.
 * This is useful for debugging test failures.
 */
const logDebugAdapterActivity = false;

suite("Q# Test Explorer Tests", function suite() {
  const workspaceFolder =
    vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders[0];
  assert(workspaceFolder, "Expecting an open folder");
  const fooUri = vscode.Uri.joinPath(workspaceFolder.uri, "src", "foo.qs");
  const barUri = vscode.Uri.joinPath(workspaceFolder.uri, "src", "bar.qs");

  this.beforeAll(async () => {
    await activateExtension();
  });

  this.afterEach(async () => {
    vscode.commands.executeCommand("workbench.action.closeAllEditors");
  });

  test("Launch with debugEditorContents command", async () => {
    await vscode.window.showTextDocument(fooUri);

    // launch debugger
    await vscode.commands.executeCommand("qsharp-vscode.debugEditorContents");

    await waitUntilPaused([
      {
        id: 0,
        source: {
          name: "foo.qs",
          path: "vscode-test-web://mount/src/foo.qs",
          sourceReference: 0,
          adapterData: "qsharp-adapter-data",
        },
        line: 5,
        column: 9,
        name: "Foo ",
        endLine: 5,
        endColumn: 15,
      },
      { id: 0, line: 0, column: 0, name: "entry", source: undefined },
    ]);
  });
});