// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService } from "qsharp-lang";
import * as vscode from "vscode";
import { toVscodeRange } from "./common";

export function createFormatProvider(languageService: ILanguageService) {
  return new QSharpFormatProvider(languageService);
}

class QSharpFormatProvider implements vscode.DocumentFormattingEditProvider {
  constructor(public languageService: ILanguageService) {}

  async provideDocumentFormattingEdits(document: vscode.TextDocument) {
    const lsEdits = await this.languageService.getFormatChanges(
      document.uri.toString(),
    );

    if (!lsEdits) return [];
    const edits = [];
    for (const edit of lsEdits) {
      const referenceRange = toVscodeRange(edit.range);
      edits.push(new vscode.TextEdit(referenceRange, edit.newText));
    }
    return edits;
  }
}
