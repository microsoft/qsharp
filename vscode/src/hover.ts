/* eslint-disable @typescript-eslint/no-unused-vars */
import { type getCompiler } from "qsharp";
import * as vscode from "vscode";

type ICompiler = Awaited<ReturnType<typeof getCompiler>>;

export function createHoverProvider(compiler: ICompiler) {
  return new QSharpHoverProvider(compiler);
}

class QSharpHoverProvider implements vscode.HoverProvider {
  constructor(public compiler: ICompiler) {}

  async provideHover(
    document: vscode.TextDocument,
    position: vscode.Position,
    token: vscode.CancellationToken
  ) {
    const hover = await this.compiler.getHover(
      document.uri.toString(),
      document.getText(),
      document.offsetAt(position)
    );
    return (
      hover &&
      new vscode.Hover(
        new vscode.MarkdownString(hover.contents),
        new vscode.Range(
          document.positionAt(hover.span.start),
          document.positionAt(hover.span.end)
        )
      )
    );
  }
}
