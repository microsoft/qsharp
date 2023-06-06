/* eslint-disable @typescript-eslint/no-unused-vars */
import { type getCompiler } from "qsharp";
import * as vscode from "vscode";
import { CompletionItem } from "vscode";

type ICompiler = Awaited<ReturnType<typeof getCompiler>>;

export function createCompletionItemProvider(compiler: ICompiler) {
  return new QSharpCompletionItemProvider(compiler);
}

class QSharpCompletionItemProvider implements vscode.CompletionItemProvider {
  constructor(public compiler: ICompiler) {}

  async provideCompletionItems(
    document: vscode.TextDocument,
    position: vscode.Position,
    token: vscode.CancellationToken,
    context: vscode.CompletionContext
  ) {
    const completions = await this.compiler.getCompletions(
      document.uri.toString(),
      document.getText(),
      document.offsetAt(position)
    );
    return completions.items.map((c) => {
      let kind;
      switch (c.kind) {
        case "function":
          kind = vscode.CompletionItemKind.Function;
          break;
        case "module":
          kind = vscode.CompletionItemKind.Module;
          break;
        case "keyword":
          kind = vscode.CompletionItemKind.Keyword;
          break;
        case "issue":
          kind = vscode.CompletionItemKind.Issue;
          break;
      }
      return new CompletionItem(c.label, kind);
    });
  }
}
