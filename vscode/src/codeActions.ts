// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService } from "qsharp-lang";
import * as vscode from "vscode";

export function createCodeActionsProvider(languageService: ILanguageService) {
  return new QSharpCodeActionProvider(languageService);
}

class QSharpCodeActionProvider implements vscode.CodeActionProvider {
  constructor(public languageService: ILanguageService) {}
  provideCodeActions(
    document: vscode.TextDocument,
    range: vscode.Range | vscode.Selection,
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    context: vscode.CodeActionContext,
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    token: vscode.CancellationToken,
  ): vscode.ProviderResult<vscode.CodeAction[]> {
    const codeAction = new vscode.CodeAction(
      "Demo",
      vscode.CodeActionKind.QuickFix,
    );
    const edit = new vscode.WorkspaceEdit();
    edit.replace(document.uri, range, "HelloQuickFixes");
    codeAction.edit = edit;
    // return [codeAction];
    const codeActions = [];

    for (const diagnostic of context.diagnostics) {
      if (diagnostic.range.intersection(range)) {
        const codeAction = new vscode.CodeAction(
          "Demo",
          vscode.CodeActionKind.QuickFix,
        );
        const edit = new vscode.WorkspaceEdit();
        edit.replace(document.uri, range, "HelloQuickFixes");
        codeAction.edit = edit;
        codeActions.push(codeAction);
      }
    }

    return codeActions;
  }

  //   resolveCodeAction?(
  //     codeAction: vscode.CodeAction,
  //     token: vscode.CancellationToken,
  //   ): vscode.ProviderResult<vscode.CodeAction> {
  //     throw new Error("Method not implemented.");
  //   }
}
