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
      document.uri.fsPath,
      document.getText(),
      document.offsetAt(position)
    );
    return completions.items.map((c) => new CompletionItem(c.label, c.kind));
  }
}
