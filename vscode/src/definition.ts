/* eslint-disable @typescript-eslint/no-unused-vars */
import { type getCompiler } from "qsharp";
import * as vscode from "vscode";

type ICompiler = Awaited<ReturnType<typeof getCompiler>>;

export function createDefinitionProvider(compiler: ICompiler) {
  return new QSharpDefinitionProvider(compiler);
}

class QSharpDefinitionProvider implements vscode.DefinitionProvider {
  constructor(public compiler: ICompiler) {}

  async provideDefinition(
    document: vscode.TextDocument,
    position: vscode.Position,
    token: vscode.CancellationToken
  ) {
    const definition = await this.compiler.getDefinition(
      document.uri.toString(),
      document.getText(),
      document.offsetAt(position)
    );
    if (!definition) return null;
    const uri = vscode.Uri.parse(definition.source);
    // We have to do this to map the position :(
    const definitionPosition = (
      await vscode.workspace.openTextDocument(uri)
    ).positionAt(definition.offset);
    return new vscode.Location(uri, definitionPosition);
  }
}
