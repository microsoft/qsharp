// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService, ICodeAction } from "qsharp-lang";
import * as vscode from "vscode";
import { toVscodeWorkspaceEdit } from "../common";

export function createCodeActionsProvider(languageService: ILanguageService) {
  return new QSharpCodeActionProvider(languageService);
}

class QSharpCodeActionProvider implements vscode.CodeActionProvider {
  constructor(public languageService: ILanguageService) {}
  async provideCodeActions(
    document: vscode.TextDocument,
    range: vscode.Range | vscode.Selection,
  ) {
    const iCodeActions = await this.languageService.getCodeActions(
      document.uri.toString(),
      range,
    );

    // Convert language-service type to vscode type
    return iCodeActions.map(toCodeAction);
  }
}

function toCodeAction(iCodeAction: ICodeAction): vscode.CodeAction {
  const codeAction = new vscode.CodeAction(
    iCodeAction.title,
    toCodeActionKind(iCodeAction.kind),
  );
  if (iCodeAction.edit) {
    codeAction.edit = toVscodeWorkspaceEdit(iCodeAction.edit);
  }
  codeAction.isPreferred = iCodeAction.isPreferred;
  return codeAction;
}

function toCodeActionKind(
  codeActionKind?: string,
): vscode.CodeActionKind | undefined {
  switch (codeActionKind) {
    case "Empty":
      return vscode.CodeActionKind.Empty;
    case "QuickFix":
      return vscode.CodeActionKind.QuickFix;
    case "Refactor":
      return vscode.CodeActionKind.Refactor;
    case "RefactorExtract":
      return vscode.CodeActionKind.RefactorExtract;
    case "RefactorInline":
      return vscode.CodeActionKind.RefactorInline;
    case "RefactorMove":
      return vscode.CodeActionKind.RefactorMove;
    case "RefactorRewrite":
      return vscode.CodeActionKind.RefactorRewrite;
    case "Source":
      return vscode.CodeActionKind.Source;
    case "SourceOrganizeImports":
      return vscode.CodeActionKind.SourceOrganizeImports;
    case "SourceFixAll":
      return vscode.CodeActionKind.SourceFixAll;
    case "Notebook":
      return vscode.CodeActionKind.Notebook;
  }
}
