// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {
  ICodeLens,
  ILanguageService,
  qsharpLibraryUriScheme,
} from "qsharp-lang";
import * as vscode from "vscode";
import { toVscodeRange } from "../common";

export function createCodeLensProvider(languageService: ILanguageService) {
  return new QSharpCodeLensProvider(languageService);
}

class QSharpCodeLensProvider implements vscode.CodeLensProvider {
  constructor(public languageService: ILanguageService) {}
  // We could raise events when code lenses change,
  // but there's no need as the editor seems to query often enough.
  // onDidChangeCodeLenses?: vscode.Event<void> | undefined;
  async provideCodeLenses(
    document: vscode.TextDocument,
  ): Promise<vscode.CodeLens[]> {
    if (document.uri.scheme === qsharpLibraryUriScheme) {
      // Don't show any code lenses for library files, none of the actions
      // would work since compiling library files through the editor is unsupported.
      return [];
    }

    const codeLenses = await this.languageService.getCodeLenses(
      document.uri.toString(),
    );

    return codeLenses.map((cl) => mapCodeLens(cl));
  }
}

function mapCodeLens(cl: ICodeLens): vscode.CodeLens {
  let command;
  let title;
  let tooltip;
  let args = undefined;
  switch (cl.command) {
    case "histogram":
      title = "Histogram";
      command = "qsharp-vscode.showHistogram";
      tooltip = "Run and show histogram";
      break;
    case "estimate":
      title = "Estimate";
      command = "qsharp-vscode.showRe";
      tooltip = "Calculate resource estimates";
      break;
    case "debug":
      title = "Debug";
      command = "qsharp-vscode.debugEditorContents";
      tooltip = "Debug program";
      break;
    case "run":
      title = "Run";
      command = "qsharp-vscode.runEditorContents";
      tooltip = "Run program";
      break;
    case "circuit":
      title = "Circuit";
      command = "qsharp-vscode.showCircuit";
      tooltip = "Show circuit";
      if (cl.args) {
        args = [cl.args];
      }
      break;
  }

  return new vscode.CodeLens(toVscodeRange(cl.range), {
    title,
    command,
    arguments: args,
    tooltip,
  });
}
