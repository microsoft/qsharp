// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {
  ILanguageService,
  VSDiagnostic,
  qsharpLibraryUriScheme,
  IRange,
  ILocation,
  log,
  ICompilerWorker,
  ProgramConfig,
} from "qsharp-lang";
import * as vscode from "vscode";
import { getCommonCompilerWorker, qsharpLanguageId, toVsCodeDiagnostic , toVsCodeLocation, toVsCodeRange} from "../common";
import { getActiveProgram } from "../programConfig";
import { createDebugConsoleEventTarget } from "../debugger/output";

export function startLanguageServiceDiagnostics(
  languageService: ILanguageService,
): vscode.Disposable[] {
  const diagCollection =
    vscode.languages.createDiagnosticCollection(qsharpLanguageId);

  const testController: vscode.TestController = vscode.tests.createTestController(
    "qsharpTestController",
    "Q# Tests",
  );

  async function onDiagnostics(evt: {
    detail: {
      uri: string;
      version: number;
      diagnostics: VSDiagnostic[];
    };
  }) {
    const diagnostics = evt.detail;
    const uri = vscode.Uri.parse(diagnostics.uri);

    if (uri.scheme === qsharpLibraryUriScheme) {
      // Don't report diagnostics for library files.
      return;
    }

    diagCollection.set(
      uri,
      diagnostics.diagnostics.map((d) => toVsCodeDiagnostic(d)),
    );
  }

  // test explorer features

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

  async function onTestCallables(evt: {
    detail: {
      callables: [string, ILocation][];
    };
  }) {

    for (const [id, testItem] of testController.items) {
        testController.items.delete(id);
    }


    // break down the test callable into its parts, so we can construct
    // the namespace hierarchy in the test explorer
    for (const [ callableName, location ] of evt.detail.callables) {
      const vscRange = toVsCodeLocation(location);
      const parts = callableName.split(".");

      // for an individual test case, e.g. foo.bar.baz, create a hierarchy of items
      let rover = testController.items;
      for (let i = 0; i < parts.length; i++) {
        const part = parts[i];
        const id = i === parts.length - 1 ? callableName : part;
        if (!rover.get(part)) {
          const testItem = testController.createTestItem(id, part, vscRange.uri);
          testItem.range = vscRange.range;
          rover.add(testItem);
        }
        rover = rover.get(id)!.children;
      }
    }

  }

  languageService.addEventListener("diagnostics", onDiagnostics);
  languageService.addEventListener("testCallables", onTestCallables);

  return [
    {
      dispose: () => {
        languageService.removeEventListener("diagnostics", onDiagnostics);
        languageService.removeEventListener("testCallables", onTestCallables);
      },
    },
    diagCollection,
    testController,
  ];
}
