// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService, samples } from "qsharp-lang";
import * as vscode from "vscode";
import { CompletionItem } from "vscode";

export function createCompletionItemProvider(
  languageService: ILanguageService,
) {
  return new QSharpCompletionItemProvider(languageService);
}

class QSharpCompletionItemProvider implements vscode.CompletionItemProvider {
  private samples: vscode.CompletionItem[] = [];

  constructor(public languageService: ILanguageService) {
    this.samples = samples.map((s) => {
      const item = new CompletionItem(
        s.title + " sample",
        vscode.CompletionItemKind.Snippet,
      );
      item.insertText = s.code;
      return item;
    });
  }

  async provideCompletionItems(
    document: vscode.TextDocument,
    position: vscode.Position,
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    token: vscode.CancellationToken,
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    context: vscode.CompletionContext,
  ) {
    const completions = await this.languageService.getCompletions(
      document.uri.toString(),
      document.offsetAt(position),
    );
    const results = completions.items.map((c) => {
      let kind;
      switch (c.kind) {
        case "function":
          kind = vscode.CompletionItemKind.Function;
          break;
        case "interface":
          kind = vscode.CompletionItemKind.Interface;
          break;
        case "keyword":
          kind = vscode.CompletionItemKind.Keyword;
          break;
        case "module":
          kind = vscode.CompletionItemKind.Module;
          break;
        case "property":
          kind = vscode.CompletionItemKind.Property;
          break;
      }
      const item = new CompletionItem(c.label, kind);
      item.sortText = c.sortText;
      item.detail = c.detail;
      item.additionalTextEdits = c.additionalTextEdits?.map((edit) => {
        return new vscode.TextEdit(
          new vscode.Range(
            document.positionAt(edit.range.start),
            document.positionAt(edit.range.end),
          ),
          edit.newText,
        );
      });
      return item;
    });
    return results.concat(this.samples);
  }
}
