// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {
  ILanguageService,
  VSDiagnostic,
  qsharpLibraryUriScheme,
  IRange,
  ILocation,
} from "qsharp-lang";
import * as vscode from "vscode";
import { qsharpLanguageId, toVsCodeDiagnostic , toVsCodeLocation, toVsCodeRange} from "../common";

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
