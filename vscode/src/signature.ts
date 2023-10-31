// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService } from "qsharp-lang";
import * as vscode from "vscode";

export function createSignatureHelpProvider(languageService: ILanguageService) {
  return new QSharpSignatureHelpProvider(languageService);
}

class QSharpSignatureHelpProvider implements vscode.SignatureHelpProvider {
  constructor(public languageService: ILanguageService) {}

  async provideSignatureHelp(
    document: vscode.TextDocument,
    position: vscode.Position,
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    token: vscode.CancellationToken,
  ) {
    const sigHelpLs = await this.languageService.getSignatureHelp(
      document.uri.toString(),
      document.offsetAt(position),
    );
    if (!sigHelpLs) return null;

    const sigHelp = new vscode.SignatureHelp();
    sigHelp.signatures = sigHelpLs.signatures.map((sig) => {
      const documentation = sig.documentation
        ? new vscode.MarkdownString(sig.documentation)
        : undefined;
      const info = new vscode.SignatureInformation(sig.label, documentation);
      info.parameters = sig.parameters.map((param) => {
        const documentation = param.documentation
          ? new vscode.MarkdownString(param.documentation)
          : undefined;
        return new vscode.ParameterInformation(
          [param.label.start, param.label.end],
          documentation,
        );
      });
      return info;
    });
    sigHelp.activeSignature = sigHelpLs.activeSignature;
    sigHelp.activeParameter = sigHelpLs.activeParameter;

    return sigHelp;
  }
}
