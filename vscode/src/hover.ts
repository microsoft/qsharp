// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService } from "qsharp-lang";
import * as vscode from "vscode";

export function createHoverProvider(languageService: ILanguageService) {
  return new QSharpHoverProvider(languageService);
}

class QSharpHoverProvider implements vscode.HoverProvider {
  constructor(public languageService: ILanguageService) {}

  async provideHover(
    document: vscode.TextDocument,
    position: vscode.Position,
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    token: vscode.CancellationToken,
  ) {
    const hover = await this.languageService.getHover(
      document.uri.toString(),
      document.offsetAt(position),
    );
    return (
      hover &&
      new vscode.Hover(
        new vscode.MarkdownString(hover.contents),
        new vscode.Range(
          document.positionAt(hover.span.start),
          document.positionAt(hover.span.end),
        ),
      )
    );
  }
}
