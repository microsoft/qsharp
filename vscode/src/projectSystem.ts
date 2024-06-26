// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {
  IProjectConfig,
  getProjectLoader,
  log,
  ProjectLoader,
} from "qsharp-lang";
import * as vscode from "vscode";
import { URI, Utils } from "vscode-uri";
import { updateQSharpJsonDiagnostics } from "./diagnostics";

/** Returns the manifest document if one is found
 * returns null otherwise
 */
async function findManifestDocument(
  currentDocumentUriString: string,
): Promise<{ directory: URI; manifest: URI } | null> {
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
      let qsharpJsonExists = false;
      const potentialManifestUri = Utils.joinPath(uriToQuery, "qsharp.json");

      try {
        qsharpJsonExists =
          (await vscode.workspace.fs.stat(potentialManifestUri)).type ===
          vscode.FileType.File;
      } catch (err) {
        // qsharp.json doesn't exist or is inaccessible, move on
      }

      if (qsharpJsonExists) {
        return { directory: uriToQuery, manifest: potentialManifestUri };
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

export async function findManifestDirectory(uri: string) {
  const result = await findManifestDocument(uri);
  if (result) {
    return result.directory.toString();
  }
  return null;
}

// this function currently assumes that `directoryQuery` will be a relative path from
// the root of the workspace
export async function listDirectory(
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

let projectLoader: ProjectLoader | undefined = undefined;

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
): Promise<IProjectConfig> {
  const manifestDocument = await findManifestDocument(documentUri.toString());

  if (!manifestDocument) {
    // return just the one file if we are in single file mode
    return await singleFileProject(documentUri);
  }

  if (!projectLoader) {
    projectLoader = await getProjectLoader({
      findManifestDirectory,
      readFile,
      listDirectory,
      resolvePath: async (a, b) => resolvePath(a, b),
    });
  }

  // Clear diagnostics for this project
  updateQSharpJsonDiagnostics(manifestDocument.manifest);

  let project;
  try {
    project = await projectLoader.load_project(
      manifestDocument.directory.toString(),
    );
  } catch (e: any) {
    updateQSharpJsonDiagnostics(
      manifestDocument.manifest,
      e.message ||
        "Failed to parse Q# manifest. For a minimal Q# project manifest, try: {}",
    );

    throw e;
  }

  return project;
}

async function singleFileProject(
  documentUri: vscode.Uri,
): Promise<IProjectConfig> {
  const file = await vscode.workspace.openTextDocument(documentUri);

  return {
    projectName: Utils.basename(documentUri),
    projectUri: documentUri.toString(),
    sources: [[documentUri.toString(), file.getText()]] as [string, string][],
    languageFeatures: [],
    lints: [],
  };
}

export function resolvePath(base: string, relative: string): string | null {
  try {
    return Utils.resolvePath(URI.parse(base, true), relative).toString();
  } catch (e) {
    log.warn(`Failed to resolve path ${base} and ${relative}: ${e}`);
    return null;
  }
}
