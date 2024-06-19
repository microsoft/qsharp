// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { getProjectLoader, log } from "qsharp-lang";
import * as vscode from "vscode";
import { URI, Utils } from "vscode-uri";
import { updateQSharpJsonDiagnostics } from "./diagnostics";

/**
 * Finds and parses a manifest. Returns `null` if no manifest was found for the given uri, or if the manifest
 * was malformed.
 */
export async function getManifest(uri: string): Promise<{
  manifestDirectory: string;
  languageFeatures: string[] | undefined;
  lints: { lint: string; level: string }[];
} | null> {
  const manifestDocument = await findManifestDocument(uri);
  if (manifestDocument === null) {
    return null;
  }
  let result;
  try {
    result = await getManifestThrowsOnParseFailure(uri);
  } catch (e) {
    log.warn(
      `failed to parse manifest at ${manifestDocument.uri.toString()}`,
      e,
    );
    updateQSharpJsonDiagnostics(
      manifestDocument.uri,
      "Failed to parse Q# manifest. For a minimal Q# project manifest, try: {}",
    );
    return null;
  }
  return result;
}

/** Returns the manifest document if one is found
 * returns null otherwise
 */
async function findManifestDocument(
  currentDocumentUriString: string,
): Promise<{ uri: vscode.Uri; content: string } | null> {
  // file://home/foo/bar/src/document.qs
  // or, e.g. in vscode on a virtual file system,
  // vscode-vfs://github%2B7b2276223a312c22726566223a7b2274797065223a332c226964223a22383439227d7d/microsoft/qsharp/samples/shor.qs
  const currentDocumentUri = URI.parse(currentDocumentUriString);

  // Untitled documents don't have a file location, thus can't have a manifest
  if (currentDocumentUri.scheme === "untitled") return null;

  // just the parent
  // e.g.
  // file://home/foo/bar/src
  let uriToQuery = Utils.dirname(currentDocumentUri);

  let attempts = 100;

  let seenSrcDir = false;

  while (attempts > 0) {
    attempts--;

    // Make sure that the path doesn't go above one of the open workspaces.
    if (
      !vscode.workspace.workspaceFolders?.some((root) =>
        uriToQuery.toString().startsWith(root.uri.toString()),
      )
    ) {
      log.debug("Aborting search for manifest file outside of workspace");
      return null;
    }

    if (seenSrcDir) {
      const potentialManifestLocation = Utils.joinPath(
        uriToQuery,
        "qsharp.json",
      );

      let listing;
      try {
        listing = await readFileUri(potentialManifestLocation);
      } catch (err) {
        log.error("Error thrown when reading file: ", err);
      }

      if (listing) {
        return listing;
      }
    }
    if (uriToQuery.toString().endsWith("src")) {
      seenSrcDir = true;
    } else {
      seenSrcDir = false;
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

  // If any open documents match this uri, return their contents instead of from disk
  const opendoc = vscode.workspace.textDocuments.find(
    (opendoc) => opendoc.uri.toString() === uri.toString(),
  );

  if (opendoc) {
    return {
      content: opendoc.getText(),
      uri,
    };
  }

  try {
    return await vscode.workspace.fs.readFile(uri).then((res) => {
      return {
        content: new TextDecoder().decode(res),
        uri: uri,
      };
    });
  } catch (_err) {
    // `readFile` should throw the below if the file is not found
    if (
      !(_err instanceof vscode.FileSystemError && _err.code === "FileNotFound")
    ) {
      log.error("Unexpected error trying to read file", _err);
    }
    return null;
  }
}

async function getManifestThrowsOnParseFailure(uri: string): Promise<{
  manifestDirectory: string;
  languageFeatures: string[] | undefined;
  lints: { lint: string; level: string }[];
} | null> {
  const manifestDocument = await findManifestDocument(uri);
  let parsedManifest: {
    languageFeatures: string[];
    lints: { lint: string; level: string }[] | undefined;
  } | null = null;

  if (manifestDocument) {
    try {
      parsedManifest = JSON.parse(manifestDocument.content); // will throw if invalid
    } catch (e: any) {
      updateQSharpJsonDiagnostics(
        manifestDocument.uri,
        "Failed to parse Q# manifest. For a minimal Q# project manifest, try: {}",
      );
      throw new Error(
        "Failed to parse qsharp.json. For a minimal Q# project manifest, try: {}",
      );
    }

    updateQSharpJsonDiagnostics(manifestDocument.uri);

    const manifestDirectory = Utils.dirname(manifestDocument.uri);

    return {
      manifestDirectory: manifestDirectory.toString(),
      languageFeatures: parsedManifest?.languageFeatures,
      lints: parsedManifest?.lints || [],
    };
  }
  return null;
}

let projectLoader: any | undefined = undefined;

export type ProjectConfig = {
  /**
   * Friendly name for the project, based on the name of the Q# document or project directory
   */
  projectName: string;
  sources: [string, string][];
  languageFeatures: string[];
  lints: {
    lint: string;
    level: string;
  }[];
};

/**
 * Given a Q# Document URI, returns the configuration and list of complete source files
 * associated with that document.
 *
 * If there is a qsharp.json manifest for this document, the settings from that are used.
 *
 * If a manifest is not found, the returned project contains the single input file and the default settings.
 *
 * @param documentUri A Q# document.
 * @returns The project configuration for that document.
 * @throws Error if the qsharp.json cannot be parsed.
 */
export async function loadProject(
  documentUri: vscode.Uri,
): Promise<ProjectConfig> {
  // get the project using this.program
  const manifest = await getManifestThrowsOnParseFailure(
    documentUri.toString(),
  );
  if (manifest === null) {
    // return just the one file if we are in single file mode
    const file = await vscode.workspace.openTextDocument(documentUri);

    return {
      projectName: Utils.basename(documentUri),
      sources: [[documentUri.toString(), file.getText()]],
      languageFeatures: [],
      lints: [],
    };
  }

  if (!projectLoader) {
    projectLoader = await getProjectLoader(readFile, listDir, getManifest);
  }
  const project: [string, string][] =
    await projectLoader.load_project(manifest);
  return {
    projectName:
      Utils.basename(URI.parse(manifest.manifestDirectory)) || "Q# Project",
    sources: project,
    languageFeatures: manifest.languageFeatures || [],
    lints: manifest.lints,
  };
}
