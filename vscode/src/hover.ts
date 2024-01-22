// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService } from "qsharp-lang";
import * as vscode from "vscode";

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
        new vscode.Range(
          new vscode.Position(
            hover.span.start.line,
            hover.span.start.character,
          ),
          new vscode.Position(hover.span.end.line, hover.span.end.character),
        ),
      )
    );
  }
}
