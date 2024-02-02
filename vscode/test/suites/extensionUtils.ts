// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { type ExtensionApi } from "../../src/extension";

// Q# extension log level. Increase this for debugging.
const extensionLogLevel = "warn";

export async function activateExtension() {
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const ext = vscode.extensions.getExtension("quantum.qsharp-lang-vscode-dev")!;
  if (ext.isActive) {
    return;
  }

  const start = performance.now();
  const extensionApi: ExtensionApi = await ext.activate();
  const logForwarder = extensionApi.logging;
  if (!logForwarder) {
    throw new Error(`qsharp-tests: extension did not return a log forwarder`);
  }

  logForwarder.setLevel(extensionLogLevel);
  logForwarder.setListener((level, ...args) => {
    // Write extension logs to the console.
    console.log(`qsharp: [${level}] ${args.join(" ")}`);
  });

  console.log(
    `qsharp-tests: activate() completed in ${performance.now() - start}ms`,
  );
}

/**
 * Waits until a condition is met, with a timeout.
 *
 * @param condition A function that returns true when the condition is met
 * @param wakeUpOn The event that we want to wake up to check the condition
 * @param timeoutMs If the condition is not met by this timeout, this function will throw
 * @param timeoutErrorMsg The custom error message to throw if the condition is not met
 */
export async function waitForCondition(
  condition: (...args: any[]) => boolean,
  wakeUpOn: vscode.Event<any>,
  timeoutMs: number,
  timeoutErrorMsg: string,
) {
  let disposable: vscode.Disposable | undefined;
  await new Promise<void>((resolve, reject) => {
    let done = false;
    setTimeout(() => {
      if (!done) {
        reject(new Error(timeoutErrorMsg));
      }
    }, timeoutMs);

    disposable = wakeUpOn(() => {
      if (!done && condition()) {
        done = true;
        resolve();
      }
    });

    // Resolve immediately if condition is already met
    if (condition()) {
      done = true;
      resolve();
    }
  });
  disposable?.dispose();
}

/**
 * Convenience method to add a time delay.
 *
 * @deprecated While this function is convenient for local development,
 * please do NOT use it in actual tests. Instead, use `waitForCondition`
 * with an appropriate condition and wake up event.
 *
 * Adding an unconditional delay to tests is guaranteed to slow tests down
 * unnecessarily. It also reduces reliability when we're not clear about
 * exactly what we're waiting on.
 */
export async function delay(timeoutMs: number) {
  try {
    await waitForCondition(
      () => false,
      () => ({ dispose() {} }),
      timeoutMs,
      "hit the expected timeout",
    );
  } catch (e) {
    // expected
  }
}
