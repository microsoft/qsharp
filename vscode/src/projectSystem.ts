// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { Utils, URI } from "vscode-uri";
import * as vscode from "vscode";

import { getProjectLoader, log } from "qsharp-lang";

export async function isQsharpProject(uri: vscode.Uri): Promise<boolean> {
  // we look for a manifest file in the parent directory of the current file
  // we don't care if it is correct or well-formed, just that it exists
  return (await findManifestDocument(uri.toString())) == undefined;
}

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

  const manifestDirectory = Utils.dirname(manifestDocument.uri);

  return {
    excludeFiles: parsedManifest.excludeFiles || [],
    excludeRegexes: parsedManifest.excludeRegexes || [],
    manifestDirectory: manifestDirectory.toString(),
  };
}

/** Returns the manifest document if one is found
 * returns null otherwise
 */
async function findManifestDocument(
  currentDocumentUriString: string,
): Promise<{ uri: vscode.Uri; content: string } | null> {
  // file://home/foo/bar/document.qs
  // or, e.g. in vscode on a virtual file system,
  // vscode-vfs://github%2B7b2276223a312c22726566223a7b2274797065223a332c226964223a22383439227d7d/microsoft/qsharp/samples/shor.qs
  const currentDocumentUri = URI.parse(currentDocumentUriString);

  // just the parent
  // e.g.
  // file://home/foo/bar
  let uriToQuery = Utils.dirname(currentDocumentUri);

  let attempts = 100;

  while (attempts > 0) {
    attempts--;
    // we abort this check if we are going above the current VS Code
    // workspace. If the user is working in a multi-root workspace [1],
    // then we do not perform this check. This is because a multi-
    // root workspace could contain different roots at different
    // levels in each others' path ancestry.
    // [1]: https://code.visualstudio.com/docs/editor/workspaces#_multiroot-workspaces
    if (
      vscode.workspace.workspaceFolders?.length === 1 &&
      Utils.resolvePath(vscode.workspace.workspaceFolders[0].uri, "..") ===
        uriToQuery
    ) {
      log.debug("Aborting search for manifest file outside of workspace");
      return null;
    }
    const potentialManifestLocation = Utils.joinPath(uriToQuery, "qsharp.json");

    let listing;
    try {
      listing = await readFileUri(potentialManifestLocation);
    } catch (err) {
      log.error("Error thrown when reading file: ", err);
    }

    if (listing) {
      return listing;
    }

    const oldUriToQuery = uriToQuery;
    uriToQuery = Utils.resolvePath(uriToQuery, "..");
    if (oldUriToQuery === uriToQuery) {
      return null;
    }
  }
  return null;
}

// this function currently assumes that `directoryQuery` will be a relative path from
// the root of the workspace
export async function listDir(
  directoryQuery: string,
): Promise<[string, number][]> {
  const uriToQuery = vscode.Uri.parse(directoryQuery);

  const fileSearchResult = await vscode.workspace.fs.readDirectory(uriToQuery);
  const mappedFiles: [string, vscode.FileType][] = fileSearchResult.map(
    ([name, type]) => [Utils.joinPath(uriToQuery, name).toString(), type],
  );

  return mappedFiles;
}

export async function readFile(uri: string): Promise<string | null> {
  const file = await readFileUri(uri);
  return file?.content || null;
}

async function readFileUri(
  maybeUri: string | vscode.Uri,
): Promise<{ uri: vscode.Uri; content: string } | null> {
  const uri: vscode.Uri = (maybeUri as any).path
    ? (maybeUri as vscode.Uri)
    : vscode.Uri.parse(maybeUri as string);
  try {
    return await vscode.workspace.fs.readFile(uri).then((res) => {
      return {
        content: new TextDecoder().decode(res),
        uri: uri,
      };
    });
  } catch (_err) {
    // `readFile` returns `err` if the file didn't exist.
    return null;
  }
}

export async function loadProject(
  documentUri: vscode.Uri,
): Promise<[string, string][]> {
  // get the project using this.program
  const manifest = await getManifest(documentUri.toString());
  if (manifest === null) {
    // return just the one file if we are in single file mode
    const file = await vscode.workspace.openTextDocument(documentUri);

    return [[documentUri.toString(), file.getText()]];
  }

  const projectLoader = await getProjectLoader(readFile, listDir, getManifest);
  log.info("using project loader to debug");
  return await projectLoader.load_project(manifest);
}
