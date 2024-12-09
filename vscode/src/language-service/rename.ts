// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService } from "qsharp-lang";
import * as vscode from "vscode";
import { toVscodeRange, toVscodeWorkspaceEdit } from "../common";

export function createRenameProvider(languageService: ILanguageService) {
  return new QSharpRenameProvider(languageService);
}

class QSharpRenameProvider implements vscode.RenameProvider {
  constructor(public languageService: ILanguageService) {}

  async provideRenameEdits(
    document: vscode.TextDocument,
    position: vscode.Position,
    newName: string,
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    token: vscode.CancellationToken,
  ) {
    const rename = await this.languageService.getRename(
      document.uri.toString(),
      position,
      newName,
    );
    if (!rename) return null;
    return toVscodeWorkspaceEdit(rename);
  }

  async prepareRename(
    document: vscode.TextDocument,
    position: vscode.Position,
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    token: vscode.CancellationToken,
  ) {
    const prepareRename = await this.languageService.prepareRename(
      document.uri.toString(),
      position,
    );
    if (prepareRename) {
      return {
        range: toVscodeRange(prepareRename.range),
        placeholder: prepareRename.newText,
      };
    } else {
      throw "Rename is unavailable at this location.";
    }
  }
}
