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
  log.info("looking for manifest");

  const manifestDocument = await findManifestDocument(uri);
  log.info("1");

  if (manifestDocument === null) {
    log.info("did not find");
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
  log.info("2");

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
  log.info("in findManifestDocument");
  // /home/foo/bar/document.qs
  const currentDocumentUri = URI.parse(currentDocumentUriString);
  log.info("a");

  // /home/foo/bar
  let uriToQuery = Utils.dirname(currentDocumentUri);

  let attempts = 100;

  while (true) {
    log.info("b");
    attempts--;
    const potentialManifestLocation = Utils.joinPath(uriToQuery, "qsharp.json");
    log.info("looking for ", potentialManifestLocation);

    let listing;
    try {
      listing = await readFile(potentialManifestLocation);
    } catch (err) {
    }

    if (listing) {
      log.info("found manifest at ", potentialManifestLocation)
      return listing;
    }


    log.info("f");
    const oldUriToQuery = uriToQuery;
    uriToQuery = Utils.resolvePath(uriToQuery, "..");
    if (oldUriToQuery === uriToQuery) {
      log.info("no qsharp manifest file found");
      return null;
    }

    log.info("g");
    if (attempts === 0) {
      log.info("returned null");
      return null;
    }
    log.info("h");
  }
}

// this function currently assumes that `directoryQuery` will be a relative path from
// the root of the workspace
export async function directoryListingCallback(
  baseUri: vscode.Uri,
  directoryQuery: string,
): Promise<string[]> {
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
  const pattern: vscode.RelativePattern = new vscode.RelativePattern(
    absoluteDirectoryQuery,
    "/**/*.qs",
  );

  const fileSearchResult = await vscode.workspace.findFiles(pattern);

  return fileSearchResult.map((x) => x.toString());
}

export async function readFileCallback(uri: string): Promise<string | null> {
  const file = await readFile(uri);
  return file?.content || null;
}

async function readFile(
  maybeUri: string | vscode.Uri,
): Promise<{ uri: vscode.Uri; content: string } | null> {
  log.info("reading file");
  const uri: vscode.Uri = (maybeUri as any).path
    ? (maybeUri as vscode.Uri)
    : vscode.Uri.parse(maybeUri as string);
  log.info("reading file 1");
  return await vscode.workspace.fs.readFile(uri).then((res) => {
    return {
      content: new TextDecoder().decode(res),
      uri: uri,
    };
  });
}
