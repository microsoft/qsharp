// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService } from "qsharp-lang";
import * as vscode from "vscode";
import { loadDocument } from "./common";

export function createDefinitionProvider(languageService: ILanguageService) {
  return new QSharpDefinitionProvider(languageService);
}

class QSharpDefinitionProvider implements vscode.DefinitionProvider {
  constructor(public languageService: ILanguageService) {}

  async provideDefinition(
    document: vscode.TextDocument,
    position: vscode.Position,
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    token: vscode.CancellationToken,
  ) {
    const definition = await this.languageService.getDefinition(
      document.uri.toString(),
      document.offsetAt(position),
    );
    if (!definition) return null;
    const uri = vscode.Uri.parse(definition.source);
    const definitionDoc = await loadDocument(uri);
    const definitionRange = new vscode.Range(
      definitionDoc.positionAt(definition.span.start),
      definitionDoc.positionAt(definition.span.end),
    );
    return new vscode.Location(uri, definitionRange);
  }
}
