// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { assert } from "chai";
import { waitForCondition } from "../extensionUtils";
import { DebugProtocol } from "@vscode/debugprotocol";

/**
 * Set to true to log Debug Adapter Protocol messages to the console.
 * This is useful for debugging test failures.
 */
const logDebugAdapterActivity = false;

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
  private kind = "qsharp";
  private stoppedCount = 0;
  private stackTrace;
  private variables;
  private onVariablesResponse: ((e: any) => void) | undefined;

  constructor(kind: string = "qsharp") {
    this.kind = kind;
  }

  /**
   * Wait until the debugger has entered the paused state by waiting for the
   * appropriate sequence of messages in the debug adapter.
   */
  async waitUntilPaused() {
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

    const stepMs = performance.now() - start;
    if (stepMs > 700) {
      // Not much we can control here if the debugger is taking too long,
      // but log a warning so that we see it in the test log if we get
      // close to hitting test timeouts.
      // The default mocha test timeout is 2000ms.
      console.log(`${this.kind}-tests: debugger took ${stepMs}ms to stop`);
    }
    if (logDebugAdapterActivity) {
      console.log(`${this.kind}-tests: debugger paused`);
    }
  }

  /**
   * Reset the state of the tracker so that we can use waitUntilPaused() again.
   */
  resetState() {
    this.stoppedCount = 0;
    this.stackTrace = undefined;
    this.variables = undefined;
  }

  /**
   * Wait until the debugger has entered the paused state and then asserts
   * that the stack traces matches the `expectedStackTrace`.
   *
   * @param expectedStackTrace assert that the stack trace matches this value.
   */
  async assertStackTrace(expectedStackTrace: DebugProtocol.StackFrame[]) {
    await this.waitUntilPaused();

    assert.deepEqual(
      this.stackTrace,
      expectedStackTrace,
      // print copy-pastable stack trace
      `actual stack trace:\n${JSON.stringify(this.stackTrace)}\n`,
    );

    this.resetState();
  }

  /**
   * Wait until the debugger has entered the paused state and then asserts
   * that the local variables match the `expectedVariables`.
   *
   * @param expectedVariables assert that the tracker.variables trace matches this value.
   */
  async assertVariables(expectedVariables: DebugProtocol.Variable[]) {
    await this.waitUntilPaused();

    assert.deepEqual(
      this.variables,
      expectedVariables,
      // print copy-pastable variables
      `actual variables:\n${JSON.stringify(this.variables)}\n`,
    );

    this.resetState();
  }

  onWillReceiveMessage(message: any): void {
    if (logDebugAdapterActivity) {
      console.log(`${this.kind}-tests: ->  ${JSON.stringify(message)}`);
    }
  }

  onDidSendMessage(message: any): void {
    if (logDebugAdapterActivity) {
      if (message.type === "response") {
        console.log(`${this.kind}-tests:  <- ${JSON.stringify(message)}`);
      } else {
        // message.type === "event"
        console.log(`${this.kind}-tests: <-* ${JSON.stringify(message)}`);
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
      console.log(`${this.kind}-tests: starting debug session`);
    }
  }

  onWillStopSession(): void {
    if (logDebugAdapterActivity) {
      console.log(`${this.kind}-tests: stopping debug session`);
    }
  }

  onError(error: Error): void {
    console.log(`${this.kind}-tests: [error] error in debug session: ${error}`);
  }

  onExit(code: number, signal: string): void {
    if (logDebugAdapterActivity) {
      console.log(
        `${this.kind}-tests: debug session exited with code ${code} and signal ${signal}`,
      );
    }
  }
}
export { Tracker };
