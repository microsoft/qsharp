// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "qsharp-lang";
import { Utils, URI } from "vscode-uri";
import * as vscode from "vscode";



/**
 * Finds and parses a manifest. Returns `null` if no manifest was found for the given uri, or if the manifest
 * was malformed.
 */
export async function getManifest(uri: string): Promise<{
  excludeFiles: string[];
  excludeRegexes: string[];
  manifestDirectory: string;
} | null> {
  const manifestDocument = await findManifestDocument(uri);
  if (manifestDocument === null) {
    return null;
  }

  let parsedManifest;
  try {
    parsedManifest = JSON.parse(manifestDocument.content);
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
    manifestDirectory: manifestDocument.uri.toString(),
  };
}

/** Returns the manifest document if one is found
 * returns null otherwise
 */
async function findManifestDocument(
  currentDocumentUriString: string,
): Promise<{ uri: vscode.Uri; content: string } | null> {
  // /home/foo/bar/document.qs
  const currentDocumentUri = URI.parse(currentDocumentUriString);

  // /home/foo/bar
  let uriToQuery = Utils.dirname(currentDocumentUri);

  let attempts = 100;

  while (true) {
    attempts--;
    let pattern = new vscode.RelativePattern(uriToQuery, "qsharp.json");
    const listing = await vscode.workspace.findFiles(pattern);

    if (listing.length > 1) {
      log.error(
        "Found multiple manifest files in the same directory -- this shouldn't be possible.",
      );
    }

    if (listing.length > 0) {
      return await readFile(listing[0])
    }

    const oldUriToQuery = uriToQuery;
    uriToQuery = Utils.resolvePath(uriToQuery, "..");
    if (oldUriToQuery === uriToQuery) {
      log.trace("no qsharp manifest file found");
      return null;
    }

    if (attempts === 0) {
      return null;
    }
  }
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

export async function readFileCallback(uri: string): Promise<string | null >{
  const file = await readFile(uri);
  return file?.content || null
}

async function readFile(maybeUri: string | vscode.Uri): Promise<{ uri: vscode.Uri, content: string } | null> {
  const uri: vscode.Uri = (maybeUri as any).path ? maybeUri as vscode.Uri : vscode.Uri.parse(maybeUri as string);
  return await vscode.workspace.fs.readFile(uri).then((res) => {
    return {
      content: new TextDecoder().decode(res),
      uri: uri,
    };
  })
}
