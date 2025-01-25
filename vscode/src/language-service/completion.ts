// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService, samples } from "qsharp-lang";
import * as vscode from "vscode";
import { CompletionItem } from "vscode";
import { toVscodeRange } from "../common";
import { EventType, sendTelemetryEvent } from "../telemetry";

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
        case "field":
          kind = vscode.CompletionItemKind.Field;
          break;
        case "class":
          kind = vscode.CompletionItemKind.Class;
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

    // Include the samples in contexts that are syntactically appropriate.
    // The presence of the "operation" keyword in the completion list is a
    // hint that the cursor is at a point we can insert the sample code.

    const shouldIncludeSamples =
      results.findIndex(
        (i) =>
          i.kind === vscode.CompletionItemKind.Keyword &&
          i.label === "operation",
      ) !== -1;

    return !shouldIncludeSamples ? results : results.concat(this.samples);
  }
}
