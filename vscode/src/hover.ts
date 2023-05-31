/* eslint-disable @typescript-eslint/no-unused-vars */
import { type getCompiler } from "qsharp";
import * as vscode from "vscode";

type ICompiler = Awaited<ReturnType<typeof getCompiler>>;

export function createHoverProvider(
  compiler: ICompiler,
  output: vscode.OutputChannel
) {
  return new QSharpHoverProvider(compiler, output);
}

class QSharpHoverProvider implements vscode.HoverProvider {
  constructor(
    public compiler: ICompiler,
    public output: vscode.OutputChannel
  ) {}

  async provideHover(
    document: vscode.TextDocument,
    position: vscode.Position,
    token: vscode.CancellationToken
  ) {
    this.output.appendLine("requesting hover...");
    const hover = await this.compiler.getHover(
      document.getText(),
      document.offsetAt(position)
    );

    this.output.appendLine("hover:");
    this.output.appendLine(JSON.stringify(hover));
    return new vscode.Hover(
      new vscode.MarkdownString(hover.contents),
      new vscode.Range(
        document.positionAt(hover.span.start),
        document.positionAt(hover.span.end)
      )
    );
  }
}
