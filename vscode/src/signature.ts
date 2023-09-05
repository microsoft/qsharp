// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService, log } from "qsharp";
import * as vscode from "vscode";

export function createSignatureHelpProvider(languageService: ILanguageService) {
  return new QSharpSignatureHelpProvider(languageService);
}

class QSharpSignatureHelpProvider implements vscode.SignatureHelpProvider {
  constructor(public languageService: ILanguageService) {}

  public provideSignatureHelp(
    document: vscode.TextDocument,
    position: vscode.Position,
    token: vscode.CancellationToken
  ) {
    log.info("providing signature help:");
    const params = [
      new vscode.ParameterInformation("a : Int", "The parameter `a`"),
      new vscode.ParameterInformation("b : Double", "The parameter `b`"),
      new vscode.ParameterInformation("c : String", "The parameter `c`"),
    ];
    const sig = new vscode.SignatureInformation(
      "operation Foo(a: Int, b: Double, c: String) : Unit"
    );
    sig.parameters = params;
    const sigHelp = new vscode.SignatureHelp();
    sigHelp.signatures = [sig];
    sigHelp.activeSignature = 0;
    sigHelp.activeParameter = 0;
    return null;
  }

  // async provideDefinition(
  //   document: vscode.TextDocument,
  //   position: vscode.Position,
  //   // eslint-disable-next-line @typescript-eslint/no-unused-vars
  //   token: vscode.CancellationToken
  // ) {
  //   const definition = await this.languageService.getDefinition(
  //     document.uri.toString(),
  //     document.offsetAt(position)
  //   );
  //   if (!definition) return null;
  //   const uri = vscode.Uri.parse(definition.source);
  //   // We have to do this to map the position :(
  //   const definitionPosition = (
  //     await vscode.workspace.openTextDocument(uri)
  //   ).positionAt(definition.offset);
  //   return new vscode.Location(uri, definitionPosition);
  // }
}
