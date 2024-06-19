// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { IProjectConfig, getProjectLoader, log } from "qsharp-lang";
import * as vscode from "vscode";
import { Uri } from "vscode";
import { URI, Utils } from "vscode-uri";
import { updateQSharpJsonDiagnostics } from "./diagnostics";
import { ProjectLoader } from "../../npm/qsharp/lib/web/qsc_wasm";

/** Returns the manifest document if one is found
 * returns null otherwise
 */
async function findManifestDocument(
  currentDocumentUriString: string,
): Promise<{ directory: vscode.Uri; uri: vscode.Uri; content: string } | null> {
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
      const listing: { uri: vscode.Uri; content: string } | null =
        await tryReadManifestInDir(uriToQuery);

      if (listing) {
        return { directory: Utils.dirname(listing.uri), ...listing };
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

async function tryReadManifestInDir(
  uriToQuery: URI,
): Promise<{ uri: vscode.Uri; content: string } | null> {
  const potentialManifestLocation = Utils.joinPath(uriToQuery, "qsharp.json");

  let listing: { uri: vscode.Uri; content: string } | null = null;
  try {
    listing = await readFileUri(potentialManifestLocation);
  } catch (err) {
    log.error("Error thrown when reading file: ", err);
  }
  return listing;
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

  // Shouldn't return null because we already passed in content
  return (await loadProjectInner(manifestDocument))!;
}

export async function loadProjectNoSingleFile(
  documentUri: vscode.Uri,
): Promise<IProjectConfig | null> {
  // TODO: this is a perf fix.... sad
  await new Promise((r) => setTimeout(r, 0));

  const manifestDocument = await findManifestDocument(documentUri.toString());

  if (!manifestDocument) {
    return null;
  }

  return loadProjectInner(manifestDocument);
}

export async function setFetchHook(
  fetchHook: (url: string) => Promise<string>,
) {
  projectLoader = await getProjectLoader(
    readFile,
    listDir,
    async (a, b) => resolvePath(a, b) || "",
    fetchHook,
  );
}

export async function loadProjectInner(manifestDocument: {
  directory: vscode.Uri;
  uri: vscode.Uri;
  content?: string;
}): Promise<IProjectConfig | null> {
  if (!projectLoader) {
    projectLoader = await getProjectLoader(
      readFile,
      listDir,
      async (a, b) => resolvePath(a, b) || "",
      fetchGithubRaw,
    );
  }

  const project = await projectLoader.load_project_with_deps(
    manifestDocument.directory.toString(),
  );

  if (project.errors.length > 0) {
    for (const error of project.errors) {
      updateQSharpJsonDiagnostics(manifestDocument.uri, error);
    }
  }
  project.projectUri = manifestDocument.uri.toString();
  project.projectName =
    Utils.basename(manifestDocument.directory) || "Q# Project";
  return project;
}

async function singleFileProject(
  documentUri: vscode.Uri,
): Promise<IProjectConfig> {
  const file = await vscode.workspace.openTextDocument(documentUri);

  return {
    projectName: Utils.basename(documentUri),
    projectUri: documentUri.toString(),
    packageGraphSources: {
      root: {
        sources: [[documentUri.toString(), file.getText()]] as [
          string,
          string,
        ][],
        languageFeatures: [],
        dependencies: {},
      },
      packages: {},
    },
    lints: [],
    errors: [],
  };
}

// TODO: need to actually use this global cache :(
// const globalCache: Record<
//   PackageKey,
//   | {
//       manifest: QSharpJsonManifest;
//       packageInfo: IPackageInfo;
//     }
//   | {
//       error: string;
//     }
// > = {};

function resolvePath(base: string, relative: string): string | null {
  try {
    return Utils.resolvePath(Uri.parse(base, true), relative).toString();
  } catch (e) {
    log.warn(`Failed to resolve path ${base} and ${relative}: ${e}`);
    return null;
  }
}

async function fetchGithubRaw(
  owner: string,
  repo: string,
  ref: string,
  path: string,
): Promise<string | null> {
  const pathNoLeadingSlash = path.startsWith("/") ? path.slice(1) : path;
  const uri = `https://raw.githubusercontent.com/${owner}/${repo}/${ref}/${pathNoLeadingSlash}`;
  log.debug(`making request to ${uri}`);
  const response = await fetch(uri);
  if (!response.ok) {
    log.warn(
      `fetchGithubRaw: ${owner}/${repo}/${ref}/${path} -> ${response.status} ${response.statusText}`,
    );
    return null;
  }

  // TODO: catch exceptions
  return response.text();
}
