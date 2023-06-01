/* eslint-disable @typescript-eslint/no-unused-vars */
import { type getCompiler } from "qsharp";
import * as vscode from "vscode";

type ICompiler = Awaited<ReturnType<typeof getCompiler>>;

export function createDefinitionProvider(
  compiler: ICompiler,
  output: vscode.OutputChannel
) {
  return new QSharpDefinitionProvider(compiler, output);
}

class QSharpDefinitionProvider implements vscode.DefinitionProvider {
  constructor(
    public compiler: ICompiler,
    public output: vscode.OutputChannel
  ) {}

  async provideDefinition(
    document: vscode.TextDocument,
    position: vscode.Position,
    token: vscode.CancellationToken
  ) {
    this.output.appendLine("requesting definition...");
    const definition = await this.compiler.getDefinition(
      document.uri.fsPath,
      document.getText(),
      document.offsetAt(position)
    );

    const uri = vscode.Uri.file(definition.source);
    // We have to do this to map the position :(
    const definitionPosition = (
      await vscode.workspace.openTextDocument(uri)
    ).positionAt(definition.offset);
    return new vscode.Location(uri, definitionPosition);
  }
}
