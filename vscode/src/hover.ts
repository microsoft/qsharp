// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService } from "qsharp-lang";
import * as vscode from "vscode";
import { toVscodeRange } from "./common";

export function createHoverProvider(languageService: ILanguageService) {
  return new QSharpHoverProvider(languageService);
}

class QSharpHoverProvider implements vscode.HoverProvider {
  constructor(public languageService: ILanguageService) {}

  async provideHover(document: vscode.TextDocument, position: vscode.Position) {
    const hover = await this.languageService.getHover(
      document.uri.toString(),
      position,
    );
    return (
      hover &&
      new vscode.Hover(
        new vscode.MarkdownString(hover.contents),
        toVscodeRange(hover.span),
      )
    );
  }
}
