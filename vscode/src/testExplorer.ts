// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// This file uses the VS Code Test Explorer API (https://code.visualstudio.com/docs/editor/testing)

import * as vscode from "vscode";
import {
  ICompilerWorker,
  log,
  ProgramConfig,
  QscEventTarget,
} from "qsharp-lang";
import { getActiveProgram } from "./programConfig";
import {
  getCommonCompilerWorker,
  isQsharpDocument,
  toVscodeLocation,
  toVsCodeRange,
} from "./common";

/**
 * Constructs the handler to pass to the `TestController` that refreshes the discovered tests.
 */
function mkRefreshHandler(
  ctrl: vscode.TestController,
  context: vscode.ExtensionContext,
) {
  /// if `uri` is null, then we are performing a full refresh and scanning the entire program
  return async (uri: vscode.Uri  | null = null) => {
    log.info("Refreshing tests for uri", uri);
    // clear out old tests
    for (const [id, testItem] of ctrl.items) {
      // if the uri is null, delete all test items, as we are going to repopulate
      // all tests.
      // if the uri is some value, and the test item is from this same URI,
      //  delete it because we are about to repopulate tests from that document.
      if (uri === null || testItem.uri?.toString() == uri.toString()) {
        ctrl.items.delete(id);
      }
    }

    const program = await getActiveProgram();
    if (!program.success) {
      throw new Error(program.errorMsg);
    }

    const programConfig = program.programConfig;
    const worker = getCommonCompilerWorker(context);
    const allTestCallables = await worker.collectTestCallables(programConfig);

    // only update test callables from this Uri
    const scopedTestCallables = uri === null ? allTestCallables : allTestCallables.filter(({callableName, location}) => {
      const vscLocation = toVscodeLocation(location);
      return vscLocation.uri.toString() === uri.toString();
    });

    // break down the test callable into its parts, so we can construct
    // the namespace hierarchy in the test explorer
    for (const { callableName, location } of scopedTestCallables) {
      const vscLocation = toVscodeLocation(location);
      const parts = callableName.split(".");

      // for an individual test case, e.g. foo.bar.baz, create a hierarchy of items
      let rover = ctrl.items;
      for (let i = 0; i < parts.length; i++) {
        const part = parts[i];
        const id = i === parts.length - 1 ? callableName : part;
        if (!rover.get(part)) {
          let testItem = ctrl.createTestItem(id, part, vscLocation.uri);
          testItem.range = vscLocation.range;
          rover.add(testItem);
        }
        rover = rover.get(id)!.children;
      }
    }
  };
}

/** 
 * Initializes the test explorer with the Q# tests in the active document.
 **/
export async function initTestExplorer(
  context: vscode.ExtensionContext,
  updateDocumentEvent: vscode.Event<vscode.Uri>,
) {
  const ctrl: vscode.TestController = vscode.tests.createTestController(
    "qsharpTestController",
    "Q# Tests",
  );
  context.subscriptions.push(ctrl);

  const refreshHandler = mkRefreshHandler(ctrl, context);
  // initially populate tests
  await refreshHandler(null);

  // when the refresh button is pressed, refresh all tests by passing in a null uri
  ctrl.refreshHandler = () => refreshHandler(null);

  // when the language service detects an updateDocument, this event fires. 
  // we call the test refresher when that happens
  updateDocumentEvent(refreshHandler);

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
    const worker = getCommonCompilerWorker(context);

    const programResult = await getActiveProgram();
    if (!programResult.success) {
      throw new Error(programResult.errorMsg);
    }

    const program = programResult.programConfig;

    for (const testCase of request.include || []) {
      await runTestCase(ctrl, testCase, request, worker, program);
    }
  };

  ctrl.createRunProfile(
    "Run Tests",
    vscode.TestRunProfileKind.Run,
    runHandler,
    true,
    undefined,
    false,
  );

  function updateNodeForDocument(e: vscode.TextDocument) {
    if (!isQsharpDocument(e)) {
      return;
    }
  }

  for (const document of vscode.workspace.textDocuments) {
    updateNodeForDocument(document);
  }

  context.subscriptions.push(
    vscode.workspace.onDidOpenTextDocument(updateNodeForDocument),
    vscode.workspace.onDidChangeTextDocument((e) =>
      updateNodeForDocument(e.document),
    ),
  );
}

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
  if (testCase.children.size > 0) {
    for (const childTestCase of testCase.children) {
      await runTestCase(ctrl, childTestCase[1], request, worker, program);
    }
    return;
  }
  const run = ctrl.createTestRun(request);
  const evtTarget = new QscEventTarget(false);
  evtTarget.addEventListener("Message", (msg) => {
    run.appendOutput(`Test ${testCase.id}: ${msg.detail}\r\n`);
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
