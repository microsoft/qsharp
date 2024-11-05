// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { assert } from "chai";
import { activateExtension, waitForCondition } from "../extensionUtils";
import { DebugProtocol } from "@vscode/debugprotocol";

/**
 * Set to true to log Debug Adapter Protocol messages to the console.
 * This is useful for debugging test failures.
 */
const logDebugAdapterActivity = false;

suite("Q# Debugger Tests", function suite() {
  const workspaceFolder =
    vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders[0];
  assert(workspaceFolder, "Expecting an open folder");
  const fooUri = vscode.Uri.joinPath(workspaceFolder.uri, "src", "foo.qs");
  const barUri = vscode.Uri.joinPath(workspaceFolder.uri, "src", "bar.qs");

  let tracker: Tracker | undefined;
  let disposable;

  this.beforeAll(async () => {
    await activateExtension();
  });

  this.beforeEach(async () => {
    tracker = new Tracker();
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

  test("Launch with launch.json configuration - workspaceFolder substitution", async () => {
    // The DebugConfiguration object is what would go in launch.json,
    // pass it in directly here
    await vscode.debug.startDebugging(workspaceFolder, {
      name: "Launch foo.qs",
      type: "qsharp",
      request: "launch",
      program: "${workspaceFolder}src/foo.qs",
    });

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

  test("Launch with launch.json configuration - file substitution", async () => {
    await vscode.window.showTextDocument(fooUri);

    // ${file} will expand to the filesystem path of the currently opened file
    await vscode.debug.startDebugging(workspaceFolder, {
      name: "Launch foo.qs",
      type: "qsharp",
      request: "launch",
      program: "${file}",
    });

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

  test("Run until completion", async () => {
    // launch debugger
    await vscode.debug.startDebugging(workspaceFolder, {
      name: "Launch foo.qs",
      type: "qsharp",
      request: "launch",
      program: "${workspaceFolder}src/foo.qs",
      stopOnEntry: true,
    });

    // should hit the breakpoint we set above
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

    vscode.commands.executeCommand("workbench.action.debug.continue");

    // wait until there's no longer an active debug session
    await waitForCondition(
      () => !vscode.debug.activeDebugSession,
      vscode.debug.onDidChangeActiveDebugSession,
      2000,
      "timed out waiting for the debugger to be terminated",
    );
  });

  test("Set breakpoint in main file", async () => {
    // Set a breakpoint on line 6 of foo.qs (5 when 0-indexed)
    await vscode.debug.addBreakpoints([
      new vscode.SourceBreakpoint(
        new vscode.Location(fooUri, new vscode.Position(5, 0)),
      ),
    ]);

    // launch debugger
    await vscode.debug.startDebugging(workspaceFolder, {
      name: "Launch foo.qs",
      type: "qsharp",
      request: "launch",
      program: "${workspaceFolder}src/foo.qs",
      stopOnEntry: false,
    });

    // should hit the breakpoint we set above
    await waitUntilPaused([
      {
        id: 0,
        source: {
          name: "foo.qs",
          path: "vscode-test-web://mount/src/foo.qs",
          sourceReference: 0,
          adapterData: "qsharp-adapter-data",
        },
        line: 6,
        column: 9,
        name: "Foo ",
        endLine: 6,
        endColumn: 15,
      },
      { id: 0, line: 0, column: 0, name: "entry", source: undefined },
    ]);
  });

  test("Set breakpoint in other file", async () => {
    // Set a breakpoint on line 3 of bar.qs (2 when 0-indexed)
    await vscode.debug.addBreakpoints([
      new vscode.SourceBreakpoint(
        new vscode.Location(barUri, new vscode.Position(2, 0)),
      ),
    ]);

    // launch debugger
    await vscode.debug.startDebugging(workspaceFolder, {
      name: "Launch foo.qs",
      type: "qsharp",
      request: "launch",
      program: "${workspaceFolder}src/foo.qs",
      stopOnEntry: false,
    });

    // should hit the breakpoint we set above
    await waitUntilPaused([
      {
        id: 1,
        source: {
          name: "bar.qs",
          path: "vscode-test-web://mount/src/bar.qs",
          sourceReference: 0,
          adapterData: "qsharp-adapter-data",
        },
        line: 3,
        column: 9,
        name: "Bar ",
        endLine: 3,
        endColumn: 26,
      },
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
        endColumn: 14,
      },
      { id: 0, line: 0, column: 0, name: "entry", source: undefined },
    ]);

    // text editor should now be open on bar.qs
    await waitForTextEditorOn(barUri);
  });

  test("Step into other file", async () => {
    // launch debugger
    await vscode.debug.startDebugging(workspaceFolder, {
      name: "Launch foo.qs",
      type: "qsharp",
      request: "launch",
      program: "${workspaceFolder}src/foo.qs",
      stopOnEntry: true,
    });

    // should break on entry (per debug config above)
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

    // step into call (will be a call into bar.qs)
    await vscode.commands.executeCommand("workbench.action.debug.stepInto");

    await waitUntilPaused([
      {
        id: 1,
        source: {
          name: "bar.qs",
          path: "vscode-test-web://mount/src/bar.qs",
          sourceReference: 0,
          adapterData: "qsharp-adapter-data",
        },
        line: 3,
        column: 9,
        name: "Bar ",
        endLine: 3,
        endColumn: 26,
      },
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
        endColumn: 14,
      },
      { id: 0, line: 0, column: 0, name: "entry", source: undefined },
    ]);

    // text editor should now be open on bar.qs
    await waitForTextEditorOn(barUri);
  });

  test("Step into standard lib", async () => {
    // Set a breakpoint on line 8 of foo.qs (7 when 0-indexed)
    // This will be a call into stdlib
    await vscode.debug.addBreakpoints([
      new vscode.SourceBreakpoint(
        new vscode.Location(fooUri, new vscode.Position(7, 0)),
      ),
    ]);

    // launch debugger
    await vscode.debug.startDebugging(workspaceFolder, {
      name: "Launch foo.qs",
      type: "qsharp",
      request: "launch",
      program: "${workspaceFolder}src/foo.qs",
      stopOnEntry: false,
    });

    // should hit the breakpoint we set above
    await waitUntilPaused([
      {
        id: 0,
        source: {
          name: "foo.qs",
          path: "vscode-test-web://mount/src/foo.qs",
          sourceReference: 0,
          adapterData: "qsharp-adapter-data",
        },
        line: 8,
        column: 9,
        name: "Foo ",
        endLine: 8,
        endColumn: 14,
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
          path: "qsharp-library-source:Std/Intrinsic.qs",
          sourceReference: 0,
          adapterData: "qsharp-adapter-data",
        },
        line: 206,
        column: 9,
        name: "H ",
        endLine: 206,
        endColumn: 40,
      },
      {
        id: 0,
        source: {
          name: "foo.qs",
          path: "vscode-test-web://mount/src/foo.qs",
          sourceReference: 0,
          adapterData: "qsharp-adapter-data",
        },
        line: 8,
        column: 9,
        name: "Foo ",
        endLine: 8,
        endColumn: 13,
      },
      { id: 0, line: 0, column: 0, name: "entry", source: undefined },
    ]);

    // text editor should now be open on intrinsic.qs
    await waitForTextEditorOn(
      vscode.Uri.parse("qsharp-library-source:Std/Intrinsic.qs"),
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

/**
 * This class will listen to the communication between VS Code and the debug adapter (our code).
 *
 * VS Code does not provide an easy way to hook into debug session state for our tests. But there
 * is a predictable pattern of Debug Adapter Protocol messages we can listen to,
 * to figure out when the debugger has entered the paused state (as a result of a breakpoint, step, breaking on entry, etc.).
 *
 * 1. a "stopped" event coming from the debug adapter.
 * 2. a response to a "stackTrace" request.
 * 3. a response to a "variables" request.
 *
 * The "variables" request is the last thing VS Code sends to the debug adapter, and thus we can
 * use that event to reasonably determine we're ready to move on to the next test command.
 *
 * This pattern is based on the debug tests in the VS Code repo:
 * https://github.com/microsoft/vscode/blob/13e49a698cf441f82984b357f09ed095779751b8/extensions/vscode-api-tests/src/singlefolder-tests/debug.test.ts#L52
 */
class Tracker implements vscode.DebugAdapterTracker {
  private stoppedCount = 0;
  private stackTrace;
  private variables;
  private onVariablesResponse: ((e: any) => void) | undefined;

  /**
   * Wait until the debugger has entered the paused state by waiting for the
   * appropriate sequence of messages in the debug adapter.
   *
   * @param expectedStackTrace assert that the stack trace matches this value
   */
  async waitUntilPaused(expectedStackTrace: DebugProtocol.StackFrame[]) {
    const start = performance.now();

    await waitForCondition(
      () => this.stoppedCount === 1 && this.stackTrace && this.variables,
      (listener: (e: any) => void) => {
        this.onVariablesResponse = listener;
        return {
          dispose() {
            this.onVariablesResponse = undefined;
          },
        };
      },
      1800,
      "timed out waiting for the debugger to stop",
    );

    assert.deepEqual(
      this.stackTrace,
      expectedStackTrace,
      // print copy-pastable stack trace
      `actual stack trace:\n${JSON.stringify(this.stackTrace)}\n`,
    );

    this.stoppedCount = 0;
    this.stackTrace = undefined;
    this.variables = undefined;

    const stepMs = performance.now() - start;
    if (stepMs > 700) {
      // Not much we can control here if the debugger is taking too long,
      // but log a warning so that we see it in the test log if we get
      // close to hitting test timeouts.
      // The default mocha test timeout is 2000ms.
      console.log(`qsharp-tests: debugger took ${stepMs}ms to stop`);
    }
    if (logDebugAdapterActivity) {
      console.log(`qsharp-tests: debugger paused`);
    }
  }

  onWillReceiveMessage(message: any): void {
    if (logDebugAdapterActivity) {
      console.log(`qsharp-tests: ->  ${JSON.stringify(message)}`);
    }
  }

  onDidSendMessage(message: any): void {
    if (logDebugAdapterActivity) {
      if (message.type === "response") {
        console.log(`qsharp-tests:  <- ${JSON.stringify(message)}`);
      } else {
        // message.type === "event"
        console.log(`qsharp-tests: <-* ${JSON.stringify(message)}`);
      }
    }

    if (message.type === "event") {
      if (message.event === "stopped") {
        this.stoppedCount++;
      }
    } else if (message.type === "response") {
      if (message.command === "variables") {
        this.variables = message.body.variables;
        this.onVariablesResponse?.(undefined);
      } else if (message.command === "stackTrace") {
        this.stackTrace = message.body.stackFrames;
      }
    }
  }

  onWillStartSession(): void {
    if (logDebugAdapterActivity) {
      console.log(`qsharp-tests: starting debug session`);
    }
  }

  onWillStopSession(): void {
    if (logDebugAdapterActivity) {
      console.log(`qsharp-tests: stopping debug session`);
    }
  }

  onError(error: Error): void {
    console.log(`qsharp-tests: [error] error in debug session: ${error}`);
  }

  onExit(code: number, signal: string): void {
    if (logDebugAdapterActivity) {
      console.log(
        `qsharp-tests: debug session exited with code ${code} and signal ${signal}`,
      );
    }
  }
}
