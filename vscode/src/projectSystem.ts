import { log } from "qsharp-lang";
import { Utils } from "vscode-uri";
import * as vscode from "vscode";

export function findManifest(uri: string): {
  excludeFiles: string[];
  excludeRegexes: string[];
  manifestDirectory: string;
} | null {
  const manifestDocument = findManifestDocument(uri);
  if (manifestDocument === null) {
    return null;
  }

  let parsedManifest;
  try {
    parsedManifest = JSON.parse(manifestDocument.getText());
  } catch (e) {
    log.error(
      "Found manifest document, but the Q# manifest was not valid JSON",
      e,
    );
    return null;
  }

  return {
    excludeFiles: parsedManifest.excludeFiles || [],
    excludeRegexes: parsedManifest.excludeRegexes || [],
    manifestDirectory: manifestDocument.uri.path,
  };
}

/** Returns the manifest document if one is found
 * returns null otherwise
 */
function findManifestDocument(uri: string): vscode.TextDocument | null {
  let openedFile = readFile(uri);
  if (openedFile === null) {
    return null;
  }
  // https://something.com/home/foo/bar/document.qs

  let uriToQuery = openedFile.uri;

  let attempts = 100;

  while (attempts > 0) {
    // we can't use vscode.workspace.findFiles here because that is async
    // so we iterate through the workspace instead

    // if path.relative(foo/bar/, foo/bar/qsharp.json) === qsharp.json, then this directory contains a qsharp.json,
    const listing = vscode.workspace.textDocuments
      .filter((x) => x.uri.path.startsWith(uriToQuery.path))
      .filter((doc) => {
        return (
          doc.uri.path.toString().replace(uriToQuery.toString(), "") ===
          "qsharp.json"
        );
      });

    if (listing.length === 1) {
      return listing[0];
    } else if (listing.length > 1) {
      log.error(
        "Found multiple manifest files in the same directory -- this shouldn't be possible.",
      );
      return listing[0];
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
    workspaceFolder.uri,
    "/" + directoryQuery,
  );

  const filesInDir = vscode.workspace.textDocuments
    .filter((doc) => doc.uri.path.startsWith(absoluteDirectoryQuery.path))
    .map((doc) => doc.getText());

  return filesInDir;
}

export function readFileCallback(uri: string): string | null {
  const file = readFile(uri);
  return (file && file.getText()) || null;
}

function readFile(uri: string): vscode.TextDocument | null {
  return (
    vscode.workspace.textDocuments.filter((x) => x.fileName === uri)[0] || null
  );
}
