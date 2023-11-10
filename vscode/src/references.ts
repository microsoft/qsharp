// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService } from "qsharp-lang";
import * as vscode from "vscode";

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
      document.offsetAt(position),
      context.includeDeclaration,
    );
    if (!lsReferences) return [];
    const references = [];
    for (const reference of lsReferences) {
      const uri = vscode.Uri.parse(reference.source);
      const referenceDoc = await vscode.workspace.openTextDocument(uri);
      const referenceRange = new vscode.Range(
        referenceDoc.positionAt(reference.span.start),
        referenceDoc.positionAt(reference.span.end),
      );
      references.push(new vscode.Location(uri, referenceRange));
    }
    return references;
  }
}
