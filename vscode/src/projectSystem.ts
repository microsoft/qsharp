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
  // /home/foo/bar/document.qs
  const currentDocumentUri = URI.parse(currentDocumentUriString);

  // /home/foo/bar
  let uriToQuery = Utils.dirname(currentDocumentUri);

  let attempts = 100;

  while (true) {
    attempts--;
    const potentialManifestLocation = Utils.joinPath(uriToQuery, "qsharp.json");

    let listing;
    try {
      listing = await readFile(potentialManifestLocation);
    } catch (err) {}

    if (listing) {
      return listing;
    }

    const oldUriToQuery = uriToQuery;
    uriToQuery = Utils.resolvePath(uriToQuery, "..");
    if (oldUriToQuery === uriToQuery) {
      return null;
    }

    if (attempts === 0) {
      return null;
    }
  }
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
