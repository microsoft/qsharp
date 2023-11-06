import { log } from "qsharp-lang";
import * as vscode from "vscode";
import * as path from "path";


export async function findManifest(document: vscode.TextDocument): string | null {
  // /home/foo/bar/document.qs
  const currentDocumentUri = document.uri;

  // /home/foo/bar
  let pathToQuery = path.dirname(currentDocumentUri.path);

  let attempts = 100;
  
  while (true) {
    attempts--;
    let pattern = new vscode.RelativePattern(pathToQuery, "qsharp.json");
    const listing = await vscode.workspace.findFiles(pattern);

    if (listing.length === 1) { return listing[0] }
    else if (listing.length > 1) { log.error("Found multiple manifest files in the same directory -- this shouldn't be possible."); return listing[0] }

    const oldPathToQuery = pathToQuery;
    pathToQuery = path.resolve(pathToQuery, "..");
    if (oldPathToQuery === pathToQuery) {
      log.trace("no qsharp manifest file found");
      return null;
    }

    if (attempts === 0) { return null; }
  }
}


// this function currently assumes that `directoryQuery` will be a relative path from
// the root of the workspace
export async function directoryListingCallback(document: vscode.TextDocument, directoryQuery: string): Promise<vscode.Uri[]> {
  log.debug("querying directory for project system", directoryQuery);
  let workspaceFolder: vscode.WorkspaceFolder | undefined = vscode.workspace.getWorkspaceFolder(document.uri);

  if (!workspaceFolder) {
    log.trace("no workspace found; no project will be loaded");
    return [];
  }

  let workspaceFolderPath: string = workspaceFolder.uri.path;

  const absoluteDirectoryQuery = path.normalize(workspaceFolderPath + '/' + directoryQuery);

  const pattern: vscode.RelativePattern = new vscode.RelativePattern(
    absoluteDirectoryQuery,
    '/**/*.qs');

  const fileSearchResult = await vscode.workspace.findFiles(pattern);

  return fileSearchResult;
}

export function fileLookupCallback(uri: string): [string, string] | null {
  const maybeDocument = vscode.workspace.textDocuments.filter((x) => x.fileName === uri)[0];

  return (maybeDocument && [maybeDocument.fileName, maybeDocument.getText()]) || null

}
