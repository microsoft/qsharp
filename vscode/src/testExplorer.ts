// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

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
import { getActiveQSharpDocumentUri } from "./programConfig";
import { IProgramConfig } from "../../npm/qsharp/lib/web/qsc_wasm";
import { getTarget } from "./config";
import { toVsCodeRange } from "./common";

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

async function getProgramConfig(): Promise<IProgramConfig | null> {
  if (!vscode.workspace.workspaceFolders) {
    log.info("No workspace detected; not starting test explorer");
    return null;
  }

  const docUri = getActiveQSharpDocumentUri();
  if (!docUri) {
    log.info("No active document detected; not starting test explorer");
    return null;
  }

  const projectConfig: IProjectConfig = await loadProject(docUri);
  if (!projectConfig) {
    log.info("No project detected; not starting test explorer");
    return null;
  }

  return {
    profile: getTarget(),
    ...projectConfig,
  };
}

/**
 * Constructs the handler to pass to the `TestController` that refreshes the discovered tests.
 * if `shouldDeleteOldTests` is `true`, then clear out previously discovered tests. If `false`, add new tests but don't dissolve old ones.
 *
 */
function mkRefreshHandler(
  ctrl: vscode.TestController,
  context: vscode.ExtensionContext,
  shouldDeleteOldTests: boolean = true,
) {
  return async () => {
    if (shouldDeleteOldTests) {
      for (const [id] of ctrl.items) {
        ctrl.items.delete(id);
      }
    }
    const programConfig = await getProgramConfig();
    if (!programConfig) {
      return;
    }
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

    const program = await getProgramConfig();
    if (!program) {
      return;
    }

    const queue = [];

    for (const testCase of request.include || []) {
      if (testCase.children.size > 0) {
        for (const childTestCase of testCase.children) {
          queue.push(async () =>
            runTestCase(ctrl, childTestCase[1], request, worker, program),
          );
        }
      } else {
        queue.push(async () =>
          runTestCase(ctrl, testCase, request, worker, program),
        );
      }
    }

    for (const func of queue) {
      await func();
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
    if (e.uri.scheme !== "file") {
      return;
    }

    if (!e.uri.path.endsWith(".qs")) {
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
    const refresher = mkRefreshHandler(controller, context, true);
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

    // findInitialFiles(controller, pattern);

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
  log.info("ran test", testCase.id);
}
