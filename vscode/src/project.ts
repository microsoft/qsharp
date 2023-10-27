import * as vscode from "vscode";
import { log } from "qsharp-lang";

export function readFile(path: string) {
  log.info("reading file from ts", path);
  for (const doc of vscode.workspace.textDocuments) {
    log.info("doc: ", doc.uri.toString());
  }

  vscode.workspace.textDocuments.find((doc) => doc.uri.toString() == path);
}
