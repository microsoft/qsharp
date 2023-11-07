import { log } from "qsharp-lang";
import { Utils } from 'vscode-uri';
import * as vscode from "vscode";

/** Returns the path to the manifest if one is found
  * returns null otherwise
  */
export function findManifest(uri: vscode.Uri): string | null {
  // https://something.com/home/foo/bar/document.qs
  let uriToQuery= uri;


  let attempts = 100;

  while (attempts > 0) {
    // we can't use vscode.workspace.findFiles here because that is async
    // so we iterate through the workspace instead

    // if path.relative(foo/bar/, foo/bar/qsharp.json) === qsharp.json, then this directory contains a qsharp.json,
    const listing = vscode.workspace.textDocuments
      .filter((x) => x.uri.path.startsWith(uriToQuery.path))
      .filter((doc) => {
        return doc.uri.path.toString().replace(uriToQuery.toString(), '') === "qsharp.json";
      });

    if (listing.length === 1) {
      return listing[0].uri.path;
    } else if (listing.length > 1) {
      log.error(
        "Found multiple manifest files in the same directory -- this shouldn't be possible.",
      );
      return listing[0].uri.path;
    }

    const oldUriToQuery = uriToQuery;
    uriToQuery = Utils.resolvePath(uriToQuery, "..");
    if (oldUriToQuery === uriToQuery) {
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
export function directoryListingCallback(
  baseUri: vscode.Uri,
  directoryQuery: string,
): string[] {
  log.debug("querying directory for project system", directoryQuery);
  const workspaceFolder: vscode.WorkspaceFolder | undefined =
    vscode.workspace.getWorkspaceFolder(baseUri);

  if (!workspaceFolder) {
    log.trace("no workspace found; no project will be loaded");
    return [];
  }


  const absoluteDirectoryQuery = Utils.resolvePath(
    workspaceFolder.uri, "/" + directoryQuery
  );

  const filesInDir = vscode.workspace.textDocuments
    .filter((doc) => doc.uri.path.startsWith(absoluteDirectoryQuery.path))
    .map((doc) => doc.getText());

  return filesInDir;
}

export function readFileCallback(uri: string): string | null {
  const maybeDocument = vscode.workspace.textDocuments.filter(
    (x) => x.fileName === uri,
  )[0];

  return (
    (maybeDocument && maybeDocument.getText() || null)
  );
}
