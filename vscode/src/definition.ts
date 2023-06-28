// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService } from "qsharp";
import * as vscode from "vscode";

export function createDefinitionProvider(languageService: ILanguageService) {
  return new QSharpDefinitionProvider(languageService);
}

class QSharpDefinitionProvider implements vscode.DefinitionProvider {
  constructor(public languageService: ILanguageService) {}

  async provideDefinition(
    document: vscode.TextDocument,
    position: vscode.Position,
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    token: vscode.CancellationToken
  ) {
    const definition = await this.languageService.getDefinition(
      document.uri.toString(),
      document.offsetAt(position)
    );
    if (!definition) return null;
    const uri = vscode.Uri.parse(definition.source);
    // We have to do this to map the position :(
    const definitionPosition = (
      await vscode.workspace.openTextDocument(uri)
    ).positionAt(definition.offset);
    return new vscode.Location(uri, definitionPosition);
  }
}
