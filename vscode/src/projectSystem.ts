import { log } from "qsharp-lang";
import * as vscode from "vscode";
import * as path from "path";


export function findManifest(uri: vscode.Uri): string | null {
  // /home/foo/bar/document.qs
  const currentDocumentUri = uri;

  // /home/foo/bar
  let pathToQuery = path.dirname(currentDocumentUri.path);

  let attempts = 100;

  while (attempts > 0) {
    // we can't use vscode.workspace.findFiles here because that is async
    // so we iterate through the workspace instead

    // if path.relative(foo/bar/, foo/bar/qsharp.json) === qsharp.json, then this directory contains a qsharp.json, 
    const listing = vscode.workspace.textDocuments
      .filter(x => x.uri.path.startsWith(pathToQuery))
      .filter(doc => {
        const thisFilePath = doc.uri.path;
        return path.relative(pathToQuery, thisFilePath) === "qsharp.json"
      });

    if (listing.length === 1) { return listing[0].uri.path }
    else if (listing.length > 1) { log.error("Found multiple manifest files in the same directory -- this shouldn't be possible."); return listing[0].uri.path }

    const oldPathToQuery = pathToQuery;
    pathToQuery = path.resolve(pathToQuery, "..");
    if (oldPathToQuery === pathToQuery) {
      log.trace("no qsharp manifest file found");
      return null;
    }

    // just in case there are weird FS edge cases involving infinite `..` never terminating
    attempts--;
  }
  return null;
}


// this function currently assumes that `directoryQuery` will be a relative path from
// the root of the workspace
export function directoryListingCallback(baseUri: vscode.Uri, directoryQuery: string): string[] {
  log.debug("querying directory for project system", directoryQuery);
  const workspaceFolder: vscode.WorkspaceFolder | undefined = vscode.workspace.getWorkspaceFolder(baseUri);

  if (!workspaceFolder) {
    log.trace("no workspace found; no project will be loaded");
    return [];
  }

  const workspaceFolderPath: string = workspaceFolder.uri.path;

  const absoluteDirectoryQuery = path.normalize(workspaceFolderPath + '/' + directoryQuery);

  const filesInDir = vscode.workspace.textDocuments.filter(doc => doc.uri.path.startsWith(absoluteDirectoryQuery)).map(doc => doc.getText());

  return filesInDir;

}

export function readFileCallback(uri: string): [string, string] | null {
  const maybeDocument = vscode.workspace.textDocuments.filter((x) => x.fileName === uri)[0];

  return (maybeDocument && [maybeDocument.fileName, maybeDocument.getText()]) || null

}
