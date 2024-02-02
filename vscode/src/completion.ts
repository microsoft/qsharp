// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService, samples } from "qsharp-lang";
import * as vscode from "vscode";
import { CompletionItem } from "vscode";
import { EventType, sendTelemetryEvent } from "./telemetry";
import { toVscodeRange } from "./common";

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
    const start = performance.now();
    const completions = await this.languageService.getCompletions(
      document.uri.toString(),
      position,
    );
    const end = performance.now();
    sendTelemetryEvent(
      EventType.ReturnCompletionList,
      {},
      {
        timeToCompletionMs: end - start,
        completionListLength: completions.items.length,
      },
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
        case "variable":
          kind = vscode.CompletionItemKind.Variable;
          break;
        case "typeParameter":
          kind = vscode.CompletionItemKind.TypeParameter;
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
        return new vscode.TextEdit(toVscodeRange(edit.range), edit.newText);
      });
      return item;
    });
    return results.concat(this.samples);
  }
}
