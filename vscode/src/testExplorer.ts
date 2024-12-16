// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// This file uses the VS Code Test Explorer API (https://code.visualstudio.com/docs/editor/testing)

import * as vscode from "vscode";
import { loadProject } from "./projectSystem";
import {
  getCompilerWorker,
  ICompilerWorker,
  IProjectConfig,
  log,
  ProgramConfig,
  QscEventTarget,
} from "qsharp-lang";
import { getActiveProgram } from "./programConfig";
import { getTarget } from "./config";
import { isQsharpDocument, toVsCodeRange } from "./common";

function localGetCompilerWorker(
  context: vscode.ExtensionContext,
): ICompilerWorker {
  const compilerWorkerScriptPath = vscode.Uri.joinPath(
    context.extensionUri,
    "./out/compilerWorker.js",
  ).toString();
  const worker = getCompilerWorker(compilerWorkerScriptPath);
  return worker;
}

/**
 * Constructs the handler to pass to the `TestController` that refreshes the discovered tests.
 */
function mkRefreshHandler(
  ctrl: vscode.TestController,
  context: vscode.ExtensionContext,
) {
  return async () => {
    for (const [id] of ctrl.items) {
      ctrl.items.delete(id);
    }
    const program = await getActiveProgram();
    if (!program.success) {
      throw new Error(program.errorMsg);
    }

    const programConfig = program.programConfig;

    const worker = localGetCompilerWorker(context);

    const testCallables = await worker.collectTestCallables(programConfig);

    // break down the test callable into its parts, so we can construct
    // the namespace hierarchy in the test explorer
    for (const testCallable of testCallables) {
      const parts = testCallable.split(".");

      // for an individual test case, e.g. foo.bar.baz, create a hierarchy of items
      let rover = ctrl.items;
      for (let i = 0; i < parts.length; i++) {
        const part = parts[i];
        const id = i === parts.length - 1 ? testCallable : part;
        if (!rover.get(part)) {
          rover.add(ctrl.createTestItem(id, part));
        }
        rover = rover.get(id)!.children;
      }
    }
  };
}

export async function initTestExplorer(context: vscode.ExtensionContext) {
  const ctrl: vscode.TestController = vscode.tests.createTestController(
    "qsharpTestController",
    "Q# Tests",
  );
  context.subscriptions.push(ctrl);
  // construct the handler that runs when the user presses the refresh button in the test explorer
  const refreshHandler = mkRefreshHandler(ctrl, context);
  // initially populate tests
  await refreshHandler();

  ctrl.refreshHandler = refreshHandler;

  const runHandler = (request: vscode.TestRunRequest) => {
    if (!request.continuous) {
      return startTestRun(request);
    }
  };

  // runs an individual test run
  // or test group (a test run where there are child tests)
  const startTestRun = async (request: vscode.TestRunRequest) => {
    // use the compiler worker to run the test in the interpreter

    log.info("Starting test run, request was", JSON.stringify(request));
    const worker = localGetCompilerWorker(context);

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

  ctrl.resolveHandler = async (item) => {
    if (!item) {
      context.subscriptions.push(...startWatchingWorkspace(ctrl, context));
      return;
    }
  };

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
 * If there are no workspace folders, then we can't watch anything. In general, though, there is a workspace since this extension
 * is only activated when a .qs file is opened.
 **/

function getWorkspaceTestPatterns() {
  if (!vscode.workspace.workspaceFolders) {
    return [];
  }

  return vscode.workspace.workspaceFolders.map((workspaceFolder) => ({
    workspaceFolder,
    pattern: new vscode.RelativePattern(workspaceFolder, "**/*.qs"),
  }));
}

/**
 * Watches *.qs files and triggers the test discovery function on update/creation/deletion, ensuring we detect new tests without
 * the user having to manually refresh the test explorer.
 **/
function startWatchingWorkspace(
  controller: vscode.TestController,
  context: vscode.ExtensionContext,
) {
  return getWorkspaceTestPatterns().map(({ pattern }) => {
    const refresher = mkRefreshHandler(controller, context);
    const watcher = vscode.workspace.createFileSystemWatcher(pattern);
    watcher.onDidCreate(async () => {
      await refresher();
    });

    watcher.onDidChange(async () => {
      await refresher();
    });

    watcher.onDidDelete(async () => {
      await refresher();
    });

    return watcher;
  });
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
