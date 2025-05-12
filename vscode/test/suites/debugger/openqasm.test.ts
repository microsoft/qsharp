// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { assert } from "chai";
import { activateExtension, waitForCondition } from "../extensionUtils";
import { DebugProtocol } from "@vscode/debugprotocol";
import { qsharpExtensionId } from "../../../src/common";
import { Tracker } from "./tracker";

suite("OpenQASM Debugger Tests", function suite() {
  const workspaceFolder =
    vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders[0];
  assert(workspaceFolder, "Expecting an open folder");

  const selfContainedName = "self-contained.qasm";

  const selfContainedUri = vscode.Uri.joinPath(
    workspaceFolder.uri,
    selfContainedName,
  );
  let tracker: Tracker | undefined;
  let disposable;

  this.beforeAll(async () => {
    await activateExtension();
  });

  this.beforeEach(async () => {
    tracker = new Tracker("openqasm");
    disposable = vscode.debug.registerDebugAdapterTrackerFactory("qsharp", {
      createDebugAdapterTracker(): vscode.ProviderResult<vscode.DebugAdapterTracker> {
        return tracker;
      },
    });
  });

  this.afterEach(async () => {
    disposable.dispose();
    tracker = undefined;
    await terminateSession();
    vscode.commands.executeCommand("workbench.action.closeAllEditors");
    vscode.debug.removeBreakpoints(vscode.debug.breakpoints);
  });

  test("Launch with debugEditorContents command", async () => {
    await vscode.window.showTextDocument(selfContainedUri);

    // launch debugger
    await vscode.commands.executeCommand(
      `${qsharpExtensionId}.debugEditorContents`,
    );

    await waitUntilPaused([
      {
        id: 0,
        source: {
          name: selfContainedName,
          path: `vscode-test-web://mount/${selfContainedName}`,
          sourceReference: 0,
          adapterData: "qsharp-adapter-data",
        },
        line: 5,
        column: 1,
        name: "program ",
        endLine: 5,
        endColumn: 9,
      },
      { id: 0, line: 0, column: 0, name: "entry", source: undefined },
    ]);
  });

  test("Launch with launch.json configuration - workspaceFolder substitution", async () => {
    // The DebugConfiguration object is what would go in launch.json,
    // pass it in directly here
    await vscode.debug.startDebugging(workspaceFolder, {
      name: `Launch ${selfContainedName}`,
      type: "qsharp",
      request: "launch",
      program: "${workspaceFolder}" + `${selfContainedName}`,
    });

    await waitUntilPaused([
      {
        id: 0,
        source: {
          name: selfContainedName,
          path: `vscode-test-web://mount/${selfContainedName}`,
          sourceReference: 0,
          adapterData: "qsharp-adapter-data",
        },
        line: 5,
        column: 1,
        name: "program ",
        endLine: 5,
        endColumn: 9,
      },
      { id: 0, line: 0, column: 0, name: "entry", source: undefined },
    ]);
  });

  test("Launch with launch.json configuration - file substitution", async () => {
    await vscode.window.showTextDocument(selfContainedUri);

    // ${file} will expand to the filesystem path of the currently opened file
    await vscode.debug.startDebugging(workspaceFolder, {
      name: `Launch ${selfContainedName}`,
      type: "qsharp",
      request: "launch",
      program: "${file}",
    });

    await waitUntilPaused([
      {
        id: 0,
        source: {
          name: selfContainedName,
          path: `vscode-test-web://mount/${selfContainedName}`,
          sourceReference: 0,
          adapterData: "qsharp-adapter-data",
        },
        line: 5,
        column: 1,
        name: "program ",
        endLine: 5,
        endColumn: 9,
      },
      { id: 0, line: 0, column: 0, name: "entry", source: undefined },
    ]);
  });

  test("Run until completion", async () => {
    // launch debugger
    await vscode.debug.startDebugging(workspaceFolder, {
      name: `Launch ${selfContainedName}`,
      type: "qsharp",
      request: "launch",
      program: "${workspaceFolder}" + `${selfContainedName}`,
      stopOnEntry: true,
    });

    // should hit the breakpoint we set above
    await waitUntilPaused([
      {
        id: 0,
        source: {
          name: selfContainedName,
          path: `vscode-test-web://mount/${selfContainedName}`,
          sourceReference: 0,
          adapterData: "qsharp-adapter-data",
        },
        line: 5,
        column: 1,
        name: "program ",
        endLine: 5,
        endColumn: 9,
      },
      { id: 0, line: 0, column: 0, name: "entry", source: undefined },
    ]);

    vscode.commands.executeCommand("workbench.action.debug.continue");

    // wait until there's no longer an active debug session
    await waitForCondition(
      () => !vscode.debug.activeDebugSession,
      vscode.debug.onDidChangeActiveDebugSession,
      2000,
      "timed out waiting for the debugger to be terminated",
    );
  });

  test("Set breakpoint in single file program", async () => {
    // Set a breakpoint on line 6 of foo.qs (5 when 0-indexed)
    await vscode.debug.addBreakpoints([
      new vscode.SourceBreakpoint(
        new vscode.Location(selfContainedUri, new vscode.Position(5, 0)),
      ),
    ]);

    // launch debugger
    await vscode.debug.startDebugging(workspaceFolder, {
      name: `Launch ${selfContainedName}`,
      type: "qsharp",
      request: "launch",
      program: "${workspaceFolder}" + `${selfContainedName}`,
      stopOnEntry: false,
    });

    // should hit the breakpoint we set above
    await waitUntilPaused([
      {
        id: 0,
        source: {
          name: selfContainedName,
          path: `vscode-test-web://mount/${selfContainedName}`,
          sourceReference: 0,
          adapterData: "qsharp-adapter-data",
        },
        line: 6,
        column: 7,
        name: "program ",
        endLine: 6,
        endColumn: 8,
      },
      { id: 0, line: 0, column: 0, name: "entry", source: undefined },
    ]);
  });

  test("Step into standard lib", async () => {
    // Set a breakpoint on line 8 of foo.qs (7 when 0-indexed)
    // This will be a call into stdlib
    await vscode.debug.addBreakpoints([
      new vscode.SourceBreakpoint(
        new vscode.Location(selfContainedUri, new vscode.Position(7, 0)),
      ),
    ]);

    // launch debugger
    await vscode.debug.startDebugging(workspaceFolder, {
      name: `Launch ${selfContainedName}`,
      type: "qsharp",
      request: "launch",
      program: "${workspaceFolder}" + `${selfContainedName}`,
      stopOnEntry: false,
    });

    // should hit the breakpoint we set above
    await waitUntilPaused([
      {
        id: 0,
        source: {
          name: selfContainedName,
          path: `vscode-test-web://mount/${selfContainedName}`,
          sourceReference: 0,
          adapterData: "qsharp-adapter-data",
        },
        line: 8,
        column: 1,
        name: "program ",
        endLine: 8,
        endColumn: 5,
      },
      { id: 0, line: 0, column: 0, name: "entry", source: undefined },
    ]);

    // step into call (will be a call into intrinsic.qs)
    await vscode.commands.executeCommand("workbench.action.debug.stepInto");

    await waitUntilPaused([
      {
        id: 1,
        source: {
          name: "Intrinsic.qs",
          path: "qsharp-library-source:Std/OpenQASM/Intrinsic.qs",
          sourceReference: 0,
          adapterData: "qsharp-adapter-data",
        },
        line: 107,
        column: 5,
        name: "h ",
        endLine: 107,
        endColumn: 28,
      },
      {
        id: 0,
        source: {
          name: selfContainedName,
          path: `vscode-test-web://mount/${selfContainedName}`,
          sourceReference: 0,
          adapterData: "qsharp-adapter-data",
        },
        line: 8,
        column: 1,
        name: "program ",
        endLine: 8,
        endColumn: 5,
      },
      { id: 0, line: 0, column: 0, name: "entry", source: undefined },
    ]);

    // text editor should now be open on intrinsic.qs
    await waitForTextEditorOn(
      vscode.Uri.parse("qsharp-library-source:Std/OpenQASM/Intrinsic.qs"),
    );
  });

  /**
   * Wait until the debugger has entered the paused state.
   *
   * @param expectedStackTrace assert that the stack trace matches this value
   */
  function waitUntilPaused(expectedStackTrace: DebugProtocol.StackFrame[]) {
    return tracker!.waitUntilPaused(expectedStackTrace);
  }
});

/**
 * Terminate the active debug session and wait for it to end.
 */
async function terminateSession() {
  vscode.commands.executeCommand("workbench.action.debug.stop");
  await waitForCondition(
    () => !vscode.debug.activeDebugSession,
    vscode.debug.onDidChangeActiveDebugSession,
    2000,
    "timed out waiting for the debugger to be terminated",
  );
}

/**
 * Wait for the active text editor to be open to the given document URI.
 */
async function waitForTextEditorOn(uri: vscode.Uri) {
  await waitForCondition(
    () =>
      vscode.window.activeTextEditor?.document.uri.toString() ===
      uri.toString(),
    vscode.window.onDidChangeActiveTextEditor,
    500,
    `timed out waiting for the text editor to open to ${uri}.\nactive text editor is ${vscode.window.activeTextEditor?.document.uri}`,
  );
}
