// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService } from "qsharp";
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
    token: vscode.CancellationToken
  ) {
    const sigHelpLs = await this.languageService.getSignatureHelp(
      document.uri.toString(),
      document.offsetAt(position)
    );
    if (!sigHelpLs) return null;

    const sigHelp = new vscode.SignatureHelp();
    sigHelp.signatures = sigHelpLs.signatures.map((sig) => {
      const info = new vscode.SignatureInformation(
        sig.label,
        sig.documentation
      );
      info.parameters = sig.parameters.map(
        (param) =>
          new vscode.ParameterInformation(
            [param.label.start, param.label.end],
            param.documentation
          )
      );
      return info;
    });
    sigHelp.activeSignature = sigHelpLs.active_signature;
    sigHelp.activeParameter = sigHelpLs.active_parameter;

    return sigHelp;
  }
}
