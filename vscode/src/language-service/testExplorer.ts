// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {
  ICompilerWorker,
  ILanguageService,
  ITestDescriptor,
  log,
  ProgramConfig,
} from "qsharp-lang";
import * as vscode from "vscode";
import { loadCompilerWorker, toVsCodeLocation, toVsCodeRange } from "../common";
import { getActiveProgram } from "../programConfig";
import { createDebugConsoleEventTarget } from "../debugger/output";

let worker: ICompilerWorker | null = null;
/**
 * Returns a singleton instance of the compiler worker.
 * @param context The extension context.
 * @returns The compiler worker.
 **/
function getLocalCompilerWorker(extensionUri: string): ICompilerWorker {
  if (worker !== null) {
    return worker;
  }

  worker = loadCompilerWorker(extensionUri);

  return worker;
}

export function startTestDiscovery(
  languageService: ILanguageService,
  context: vscode.ExtensionContext,
): vscode.Disposable[] {
  // test explorer features
  const testController: vscode.TestController =
    vscode.tests.createTestController("qsharpTestController", "Q# Tests");
  const runHandler = (request: vscode.TestRunRequest) => {
    if (!request.continuous) {
      return startTestRun(request);
    }
  };

  // runs an individual test run
  // or test group (a test run where there are child tests)
  const startTestRun = async (request: vscode.TestRunRequest) => {
    // use the compiler worker to run the test in the interpreter

    log.trace("Starting test run, request was", JSON.stringify(request));
    const worker = getLocalCompilerWorker(context);

    const programResult = await getActiveProgram();
    if (!programResult.success) {
      throw new Error(programResult.errorMsg);
    }

    const program = programResult.programConfig;

    for (const testCase of request.include || []) {
      await runTestCase(testController, testCase, request, worker, program);
    }
  };

  /**
   * Given a single test case, run it in the worker (which runs the interpreter) and report results back to the
   * `TestController` as a side effect.
   *
   * This function manages its own event target for the results of the test run and uses the controller to render the output in the VS Code UI.
   **/
  async function runTestCase(
    ctrl: vscode.TestController,
    testCase: vscode.TestItem,
    request: vscode.TestRunRequest,
    worker: ICompilerWorker,
    program: ProgramConfig,
  ): Promise<void> {
    log.trace("Running Q# test: ", testCase.id);
    if (testCase.children.size > 0) {
      for (const childTestCase of testCase.children) {
        await runTestCase(ctrl, childTestCase[1], request, worker, program);
      }
      return;
    }
    const run = ctrl.createTestRun(request);
    const evtTarget = createDebugConsoleEventTarget((msg) => {
      run.appendOutput(`${msg}\n`);
    });
    evtTarget.addEventListener("Result", (msg) => {
      if (msg.detail.success) {
        run.passed(testCase);
      } else {
        const message: vscode.TestMessage = {
          message: msg.detail.value.message,
          location: {
            range: toVsCodeRange(msg.detail.value.range),
            uri: vscode.Uri.parse(msg.detail.value.uri || ""),
          },
        };
        run.failed(testCase, message);
      }
      run.end();
    });

    const callableExpr = `${testCase.id}()`;

    try {
      await worker.run(program, callableExpr, 1, evtTarget);
    } catch (error) {
      log.error(`Error running test ${testCase.id}:`, error);
      run.appendOutput(`Error running test ${testCase.id}: ${error}\r\n`);
    }
    log.trace("ran test:", testCase.id);
  }

  testController.createRunProfile(
    "Interpreter",
    vscode.TestRunProfileKind.Run,
    runHandler,
    true,
    undefined,
    false,
  );

  const testMetadata = new WeakMap<vscode.TestItem, number>();
  async function onTestCallables(evt: {
    detail: {
      callables: ITestDescriptor[];
    };
  }) {
    let currentVersion = 0;
    for (const [, testItem] of testController.items) {
      currentVersion = (testMetadata.get(testItem) || 0) + 1;
      break;
    }

    for (const { callableName, location } of evt.detail.callables) {
      const vscLocation = toVsCodeLocation(location);
      const parts = callableName.split(".");

      let rover = testController.items;
      for (let i = 0; i < parts.length; i++) {
        const part = parts[i];
        const id =
          i === parts.length - 1
            ? callableName
            : parts.slice(0, i + 1).join(".");
        let testItem = rover.get(id);
        if (!testItem) {
          testItem = testController.createTestItem(id, part, vscLocation.uri);
          testItem.range = vscLocation.range;
          rover.add(testItem);
        }
        testMetadata.set(testItem, currentVersion);
        rover = testItem.children;
      }
    }

    // delete old items from previous versions that were not updated
    deleteItemsNotOfVersion(
      currentVersion,
      testController.items,
      testController,
    );
  }

  function deleteItemsNotOfVersion(
    version: number,
    items: vscode.TestItemCollection,
    testController: vscode.TestController,
  ) {
    for (const [id, testItem] of items) {
      deleteItemsNotOfVersion(version, testItem.children, testController);
      if (testMetadata.get(testItem) !== version) {
        items.delete(id);
      }
    }
  }

  languageService.addEventListener("testCallables", onTestCallables);

  return [
    {
      dispose: () => {
        languageService.removeEventListener("testCallables", onTestCallables);
      },
    },
    testController,
  ];
}
