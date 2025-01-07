// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService } from "qsharp-lang";
import * as vscode from "vscode";
import { toVscodeLocation } from "../common";

export function createDefinitionProvider(languageService: ILanguageService) {
  return new QSharpDefinitionProvider(languageService);
}

class QSharpDefinitionProvider implements vscode.DefinitionProvider {
  constructor(public languageService: ILanguageService) {}

  async provideDefinition(
    document: vscode.TextDocument,
    position: vscode.Position,
  ) {
    const definition = await this.languageService.getDefinition(
      document.uri.toString(),
      position,
    );
    if (!definition) return null;
    return toVscodeLocation(definition);
  }
}
