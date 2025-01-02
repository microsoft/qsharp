// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {
  ILanguageService,
  VSDiagnostic,
  qsharpLibraryUriScheme,
  IRange,
  ILocation,
  log,
} from "qsharp-lang";
import * as vscode from "vscode";
import { getCommonCompilerWorker, qsharpLanguageId, toVsCodeDiagnostic , toVsCodeLocation, toVsCodeRange} from "../common";
import { getActiveProgram } from "../programConfig";

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
