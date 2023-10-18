// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService } from "qsharp-lang";
import * as vscode from "vscode";
import { qsharpDocumentFilter } from "./common";

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

    const definitionPosition = (await openDocument(uri)).positionAt(
      definition.offset
    );
    return new vscode.Location(uri, definitionPosition);
  }
}

async function openDocument(uri: vscode.Uri) {
  // Search both text documents and notebook cell documents
  const doc = vscode.workspace.textDocuments
    .concat(
      vscode.workspace.notebookDocuments.flatMap((doc) =>
        doc
          .getCells()
          .filter((cell) =>
            vscode.languages.match(qsharpDocumentFilter, cell.document)
          )
          .map((cell) => cell.document)
      )
    )
    .find((doc) => doc.uri.toString() === uri.toString());

  return doc || (await vscode.workspace.openTextDocument(uri));
}
