// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService } from "qsharp-lang";
import * as vscode from "vscode";
import { toVscodeLocation } from "./common";

export function createReferenceProvider(languageService: ILanguageService) {
  return new QSharpReferenceProvider(languageService);
}

class QSharpReferenceProvider implements vscode.ReferenceProvider {
  constructor(public languageService: ILanguageService) {}
  async provideReferences(
    document: vscode.TextDocument,
    position: vscode.Position,
    context: vscode.ReferenceContext,
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    token: vscode.CancellationToken,
  ) {
    const lsReferences = await this.languageService.getReferences(
      document.uri.toString(),
      position,
      context.includeDeclaration,
    );
    if (!lsReferences) return [];
    return lsReferences.map(toVscodeLocation);
  }
}
