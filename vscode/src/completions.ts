/* eslint-disable @typescript-eslint/no-unused-vars */
import { type getCompiler } from "qsharp";
import * as vscode from "vscode";
import { CompletionItem } from "vscode";

type ICompiler = Awaited<ReturnType<typeof getCompiler>>;

export function createCompletionItemProvider(
  compiler: ICompiler,
  output: vscode.OutputChannel
) {
  return new QSharpCompletionItemProvider(compiler, output);
}

class QSharpCompletionItemProvider implements vscode.CompletionItemProvider {
  constructor(
    public compiler: ICompiler,
    public output: vscode.OutputChannel
  ) {}

  async provideCompletionItems(
    document: vscode.TextDocument,
    position: vscode.Position,
    token: vscode.CancellationToken,
    context: vscode.CompletionContext
  ) {
    this.output.appendLine("getting completions");
    let compiler;
    try {
      compiler = await this.compiler;
    } catch (e: unknown) {
      this.output.appendLine("failed to get compiler");
      this.output.appendLine(JSON.stringify(e));
    }
    if (compiler) {
      try {
        const completions = await compiler.getCompletions(document.getText());
        this.output.appendLine(`got ${completions.items.length} completions`);
        return completions.items.map((c) => new CompletionItem(c.label));
      } catch (e: unknown) {
        this.output.appendLine("failed to get completions");
        this.output.appendLine(JSON.stringify(e));
      }
    }
    return [new CompletionItem("DUMMY")];
  }
}
