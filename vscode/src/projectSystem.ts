// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "qsharp-lang";
import { Utils, URI } from "vscode-uri";
import * as vscode from "vscode";

// This flag is a hacky way of disabling the debugger when running in project mode.
// We will have debugger support for projects _very soon_, and this flag will get
// removed when that lands. To be clear, this is _not_ the ideal way to transmit
// data to the debugger, but it is a temporary workaround.
// It stores files that are a part of projects and disables the debugger on them.
let PROJECT_MODE = false;
export const getProjectMode = () => PROJECT_MODE;

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
    PROJECT_MODE = false;
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
    PROJECT_MODE = false;
    return null;
  }

  const manifestDirectory = Utils.dirname(manifestDocument.uri);

  PROJECT_MODE = true;
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
  // /home/foo/bar/document.qs
  const currentDocumentUri = URI.parse(currentDocumentUriString);

  // /home/foo/bar
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
      listing = await readFile(potentialManifestLocation);
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
export async function directoryListingCallback(
  directoryQuery: string,
): Promise<[string, number][]> {
  const uriToQuery = vscode.Uri.parse(directoryQuery);

  const fileSearchResult = await vscode.workspace.fs.readDirectory(uriToQuery);
  const mappedFiles: [string, vscode.FileType][] = fileSearchResult.map(
    ([name, type]) => [Utils.joinPath(uriToQuery, name).toString(), type],
  );

  return mappedFiles;
}

export async function readFileCallback(uri: string): Promise<string | null> {
  const file = await readFile(uri);
  return file?.content || null;
}

async function readFile(
  maybeUri: string | vscode.Uri,
): Promise<{ uri: vscode.Uri; content: string } | null> {
  const uri: vscode.Uri = (maybeUri as any).path
    ? (maybeUri as vscode.Uri)
    : vscode.Uri.parse(maybeUri as string);
  return await vscode.workspace.fs.readFile(uri).then((res) => {
    return {
      content: new TextDecoder().decode(res),
      uri: uri,
    };
  });
}
