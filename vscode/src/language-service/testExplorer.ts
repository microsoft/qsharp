// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {
  ICompilerWorker,
  ILanguageService,
  ITestDescriptor,
  log,
} from "qsharp-lang";
import * as vscode from "vscode";
import { loadCompilerWorker, toVsCodeLocation, toVsCodeRange } from "../common";
import { getProgramForDocument } from "../programConfig";
import { createDebugConsoleEventTarget } from "../debugger/output";

let worker: ICompilerWorker | null = null;
/**
 * Returns a singleton instance of the compiler worker.
 * @param context The extension context.
 * @returns The compiler worker.
 **/
function getLocalCompilerWorker(extensionUri: vscode.Uri): ICompilerWorker {
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

    const worker = getLocalCompilerWorker(context.extensionUri);

    // request.include is an array of test cases to run, and it is only provided if a specific set of tests were selected.
    if (request.include !== undefined) {
      for (const testCase of request.include || []) {
        await runTestCase(testController, testCase, request, worker);
      }
    } else {
      // alternatively, if there is no include specified, we run all tests that are not in the exclude list
      for (const [, testCase] of testController.items) {
        if (request.exclude && request.exclude.includes(testCase)) {
          continue;
        }
        await runTestCase(testController, testCase, request, worker);
      }
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
  ): Promise<void> {
    log.trace("Running Q# test: ", testCase.id);
    if (testCase.children.size > 0) {
      for (const childTestCase of testCase.children) {
        await runTestCase(ctrl, childTestCase[1], request, worker);
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
        const failureLocation =
          msg.detail?.value?.uri ||
          (msg.detail?.value?.related &&
            msg.detail.value.related[0].location?.source) ||
          null;

        const message: vscode.TestMessage = {
          message: msg.detail.value.message,
          location:
            failureLocation === null
              ? undefined
              : {
                  range: toVsCodeRange(msg.detail.value.range),
                  uri: vscode.Uri.parse(failureLocation),
                },
        };
        run.failed(testCase, message);
      }
      run.end();
    });

    const callableExpr = `${testCase.id}()`;
    const uri = testCase.uri;
    if (!uri) {
      log.error(`No compilation URI for test ${testCase.id}`);
      run.appendOutput(`No compilation URI for test ${testCase.id}\r\n`);
      return;
    }
    const programResult = await getProgramForDocument(uri);

    if (!programResult.success) {
      throw new Error(programResult.errorMsg);
    }

    const program = programResult.programConfig;

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

  const testVersions = new WeakMap<vscode.TestItem, number>();
  async function onTestCallables(evt: {
    detail: {
      callables: ITestDescriptor[];
    };
  }) {
    let currentVersion = 0;
    for (const [, testItem] of testController.items) {
      currentVersion = (testVersions.get(testItem) || 0) + 1;
      break;
    }

    for (const { callableName, location, friendlyName } of evt.detail
      .callables) {
      const vscLocation = toVsCodeLocation(location);
      const parts = [friendlyName, ...callableName.split(".")];

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
          // if this is the actual test item, give it a range and a compilation uri
          if (i === parts.length - 1) {
            testItem.range = vscLocation.range;
          }
          rover.add(testItem);
        }
        testVersions.set(testItem, currentVersion);
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
      if (testVersions.get(testItem) !== version) {
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
