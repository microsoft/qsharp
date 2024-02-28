// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService } from "qsharp-lang";
import * as vscode from "vscode";
import { ICodeLens } from "../../npm/lib/web/qsc_wasm";
import { toVscodeRange } from "./common";

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
    default:
      throw new Error(`Unknown code lens command: ${cl.command}`);
  }

  return new vscode.CodeLens(toVscodeRange(cl.range), {
    title,
    command,
    arguments: cl.args,
    tooltip,
  });
}
